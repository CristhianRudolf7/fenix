//! Descriptor-relative file access. Ancestor symlinks and hard-linked files are
//! rejected; an attacker cannot redirect a checked path before open/rename.
use anyhow::{Context, Result, bail, ensure};
use nix::{
    fcntl::{OFlag, openat, renameat},
    sys::stat::{Mode, mkdirat},
    unistd::{UnlinkatFlags, unlinkat},
};
use std::{
    ffi::OsString,
    fs::File,
    io::{Read, Write},
    os::{fd::OwnedFd, unix::fs::MetadataExt},
    path::{Component, Path, PathBuf},
};

/// `create` gives the mode for missing directories (before the process umask).
fn parent(path: &Path, create: Option<Mode>) -> Result<(OwnedFd, OsString)> {
    let path: PathBuf = if path.is_absolute() {
        path.into()
    } else {
        std::env::current_dir()?.join(path)
    };
    let name = path.file_name().context("File name required")?.to_owned();
    let mut fd = nix::fcntl::open(
        Path::new("/"),
        OFlag::O_RDONLY | OFlag::O_DIRECTORY | OFlag::O_CLOEXEC,
        Mode::empty(),
    )?;
    for part in path.parent().context("Parent required")?.components() {
        match part {
            Component::RootDir | Component::CurDir => continue,
            Component::Normal(part) => {
                if let Some(mode) = create {
                    match mkdirat(&fd, Path::new(part), mode) {
                        Ok(()) | Err(nix::errno::Errno::EEXIST) => {}
                        Err(error) => return Err(error.into()),
                    }
                }
                fd = openat(
                    &fd,
                    Path::new(part),
                    OFlag::O_RDONLY | OFlag::O_DIRECTORY | OFlag::O_NOFOLLOW | OFlag::O_CLOEXEC,
                    Mode::empty(),
                )?;
            }
            _ => bail!("Parent traversal is not allowed"),
        }
    }
    Ok((fd, name))
}

fn regular(file: &File) -> Result<()> {
    let metadata = file.metadata()?;
    ensure!(
        metadata.is_file() && metadata.nlink() == 1,
        "Only private regular files without hard links are supported"
    );
    Ok(())
}

pub fn open(path: &Path) -> Result<File> {
    let (parent, name) = parent(path, None)?;
    let file = File::from(openat(
        &parent,
        Path::new(&name),
        OFlag::O_RDONLY | OFlag::O_NOFOLLOW | OFlag::O_NONBLOCK | OFlag::O_CLOEXEC,
        Mode::empty(),
    )?);
    regular(&file)?;
    Ok(file)
}

pub fn read(path: &Path, limit: u64) -> Result<Vec<u8>> {
    let file = open(path)?;
    ensure!(
        file.metadata()?.len() <= limit,
        "File exceeds its size limit"
    );
    let mut bytes = vec![];
    file.take(limit + 1).read_to_end(&mut bytes)?;
    ensure!(
        bytes.len() as u64 <= limit,
        "File grew beyond its size limit"
    );
    Ok(bytes)
}

pub fn write(path: &Path, bytes: &[u8]) -> Result<()> {
    if path.exists() {
        let before = read(path, 128 * 1024 * 1024)?;
        replace(path, &before, bytes)
    } else {
        create(path, bytes)
    }
}

pub fn remove(path: &Path, expected: &[u8]) -> Result<()> {
    let (parent, name) = parent(path, None)?;
    let file = File::from(openat(
        &parent,
        Path::new(&name),
        OFlag::O_RDONLY | OFlag::O_NOFOLLOW | OFlag::O_NONBLOCK | OFlag::O_CLOEXEC,
        Mode::empty(),
    )?);
    regular(&file)?;
    let mut actual = vec![];
    file.take(expected.len() as u64 + 1)
        .read_to_end(&mut actual)?;
    ensure!(actual == expected, "File changed before deletion");
    unlinkat(&parent, Path::new(&name), UnlinkatFlags::NoRemoveDir)?;
    File::from(parent).sync_all()?;
    Ok(())
}

pub fn directory(path: &Path) -> Result<File> {
    let (parent, name) = parent(path, None)?;
    Ok(File::from(openat(
        &parent,
        Path::new(&name),
        OFlag::O_RDONLY | OFlag::O_NOFOLLOW | OFlag::O_DIRECTORY | OFlag::O_CLOEXEC,
        Mode::empty(),
    )?))
}

pub fn create(path: &Path, bytes: &[u8]) -> Result<()> {
    create_with_permissions(path, bytes, None)
}

pub fn create_with_permissions(
    path: &Path,
    bytes: &[u8],
    permissions: Option<std::fs::Permissions>,
) -> Result<()> {
    create_mode(path, bytes, permissions, 0o700, 0o600)
}

/// Workspace files follow the process umask like ordinary tools, so other local
/// users and services (for example a verifier or web server) can read them when
/// the umask allows. Private runtime files keep using `create`.
pub fn create_workspace(
    path: &Path,
    bytes: &[u8],
    permissions: Option<std::fs::Permissions>,
) -> Result<()> {
    create_mode(path, bytes, permissions, 0o777, 0o666)
}

fn create_mode(
    path: &Path,
    bytes: &[u8],
    permissions: Option<std::fs::Permissions>,
    directory_mode: u32,
    file_mode: u32,
) -> Result<()> {
    let (parent, name) = parent(path, Some(Mode::from_bits_truncate(directory_mode)))?;
    let mut file = File::from(openat(
        &parent,
        Path::new(&name),
        OFlag::O_WRONLY | OFlag::O_CREAT | OFlag::O_EXCL | OFlag::O_NOFOLLOW | OFlag::O_CLOEXEC,
        Mode::from_bits_truncate(file_mode),
    )?);
    file.write_all(bytes)?;
    if let Some(permissions) = permissions {
        file.set_permissions(permissions)?;
    }
    file.sync_all()?;
    File::from(parent).sync_all()?;
    Ok(())
}

pub fn replace(path: &Path, expected: &[u8], bytes: &[u8]) -> Result<()> {
    let (parent, name) = parent(path, None)?;
    let file = File::from(openat(
        &parent,
        Path::new(&name),
        OFlag::O_RDONLY | OFlag::O_NOFOLLOW | OFlag::O_NONBLOCK | OFlag::O_CLOEXEC,
        Mode::empty(),
    )?);
    regular(&file)?;
    let permissions = file.metadata()?.permissions();
    let mut actual = Vec::new();
    file.take(expected.len() as u64 + 1)
        .read_to_end(&mut actual)?;
    ensure!(
        actual == expected,
        "File changed before editing; inspect it again"
    );
    let temporary = format!(".fenix-{}.tmp", uuid::Uuid::new_v4());
    let result = (|| -> Result<()> {
        let mut output = File::from(openat(
            &parent,
            temporary.as_str(),
            OFlag::O_WRONLY | OFlag::O_CREAT | OFlag::O_EXCL | OFlag::O_NOFOLLOW | OFlag::O_CLOEXEC,
            Mode::from_bits_truncate(0o600),
        )?);
        output.write_all(bytes)?;
        output.set_permissions(permissions)?;
        output.sync_all()?;
        renameat(&parent, temporary.as_str(), &parent, Path::new(&name))?;
        Ok(())
    })();
    if result.is_err() {
        let _ = unlinkat(&parent, temporary.as_str(), UnlinkatFlags::NoRemoveDir);
    }
    result?;
    File::from(parent).sync_all()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn links_and_non_regular_inputs_cannot_cross_the_boundary() {
        let dir = tempfile::tempdir().unwrap();
        let secret = dir.path().join("secret");
        create(&secret, b"private").unwrap();
        let link = dir.path().join("link");
        std::os::unix::fs::symlink(&secret, &link).unwrap();
        assert!(open(&link).is_err());
        assert!(replace(&link, b"private", b"changed").is_err());
        let hard = dir.path().join("hard");
        std::fs::hard_link(&secret, &hard).unwrap();
        assert!(open(&hard).is_err());
        assert!(replace(&hard, b"private", b"changed").is_err());
        assert_eq!(std::fs::read(&secret).unwrap(), b"private");
        let escape = dir.path().join("escape");
        std::os::unix::fs::symlink(dir.path(), &escape).unwrap();
        assert!(create(&escape.join("new"), b"bad").is_err());
        assert!(!dir.path().join("new").exists());
    }
    #[test]
    fn workspace_files_follow_umask_but_private_files_stay_owner_only() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let previous = nix::sys::stat::umask(Mode::from_bits_truncate(0o022));
        let result = create_workspace(&dir.path().join("shared/module.py"), b"x", None)
            .and_then(|()| create(&dir.path().join("private/state"), b"y"));
        nix::sys::stat::umask(previous);
        result.unwrap();
        let mode = |path: &str| {
            std::fs::metadata(dir.path().join(path))
                .unwrap()
                .permissions()
                .mode()
                & 0o777
        };
        assert_eq!(mode("shared/module.py"), 0o644);
        assert_eq!(mode("shared"), 0o755);
        assert_eq!(mode("private/state"), 0o600);
        assert_eq!(mode("private"), 0o700);
    }
    #[test]
    fn create_and_compare_before_replace_work_without_overwriting_new_files() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nested/example");
        create(&path, b"before").unwrap();
        assert!(create(&path, b"overwrite").is_err());
        assert!(replace(&path, b"stale", b"after").is_err());
        replace(&path, b"before", b"after").unwrap();
        assert_eq!(std::fs::read(path).unwrap(), b"after");
    }
}
