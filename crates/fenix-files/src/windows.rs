//! Handle-relative Windows access. Never truncate an existing file in place.
use anyhow::{Context, Result, bail, ensure};
use cap_fs_ext::{DirExt, FollowSymlinks, OpenOptionsFollowExt};
use cap_std::{
    ambient_authority,
    fs::{Dir, OpenOptions},
};
use std::{
    ffi::OsString,
    fs::{File, Permissions},
    io::{Read, Write},
    path::{Component, Path, PathBuf},
};

fn parent(path: &Path, create: bool) -> Result<(Dir, OsString)> {
    let path = if path.is_absolute() {
        path.to_owned()
    } else {
        std::env::current_dir()?.join(path)
    };
    let name = path.file_name().context("File name required")?.to_owned();
    let parent = path.parent().context("Parent required")?;
    let mut anchor = PathBuf::new();
    let mut names = Vec::new();
    for component in parent.components() {
        match component {
            Component::Prefix(_) | Component::RootDir => anchor.push(component.as_os_str()),
            Component::Normal(name) => names.push(name),
            Component::CurDir => {}
            Component::ParentDir => bail!("Resolve parent traversal before opening"),
        }
    }
    let mut dir = Dir::open_ambient_dir(anchor, ambient_authority())?;
    for name in names {
        if create {
            match dir.create_dir(name) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(e) => return Err(e.into()),
            }
        }
        dir = dir.open_dir_nofollow(name)?;
    }
    Ok((dir, name))
}
fn options() -> OpenOptions {
    let mut options = OpenOptions::new();
    options.follow(FollowSymlinks::No);
    options
}
fn validate(file: &File, user: bool) -> Result<()> {
    ensure!(
        file.metadata()?.is_file(),
        "Only regular files are supported"
    );
    if !user {
        ensure!(
            winapi_util::file::information(file)?.number_of_links() == 1,
            "Private files must not have hard links"
        );
    }
    Ok(())
}
fn open_at(dir: &Dir, name: &OsString, user: bool) -> Result<File> {
    let file = dir.open_with(name, options().read(true))?.into_std();
    validate(&file, user)?;
    Ok(file)
}
pub fn open(path: &Path) -> Result<File> {
    let (dir, name) = parent(path, false)?;
    open_at(&dir, &name, false)
}
pub fn open_user_file(path: &Path) -> Result<File> {
    let (dir, name) = parent(path, false)?;
    open_at(&dir, &name, true)
}
pub fn directory(path: &Path) -> Result<File> {
    if path.parent().is_none() {
        return Ok(Dir::open_ambient_dir(path, ambient_authority())?.into_std_file());
    }
    let (dir, name) = parent(path, false)?;
    Ok(dir.open_dir_nofollow(name)?.into_std_file())
}
pub fn read(path: &Path, limit: u64) -> Result<Vec<u8>> {
    let file = open(path)?;
    ensure!(
        file.metadata()?.len() <= limit,
        "File exceeds its size limit"
    );
    let mut bytes = Vec::new();
    file.take(limit.saturating_add(1)).read_to_end(&mut bytes)?;
    ensure!(
        bytes.len() as u64 <= limit,
        "File grew beyond its size limit"
    );
    Ok(bytes)
}
pub fn create(path: &Path, bytes: &[u8]) -> Result<()> {
    create_with_permissions(path, bytes, None)
}
pub fn create_workspace(path: &Path, bytes: &[u8], permissions: Option<Permissions>) -> Result<()> {
    create_with_permissions(path, bytes, permissions)
}
pub fn create_with_permissions(
    path: &Path,
    bytes: &[u8],
    permissions: Option<Permissions>,
) -> Result<()> {
    let (dir, name) = parent(path, true)?;
    let mut file = dir
        .open_with(name, options().write(true).create_new(true))?
        .into_std();
    file.write_all(bytes)?;
    if let Some(permissions) = permissions {
        file.set_permissions(permissions)?;
    }
    file.sync_all()?;
    // New files inherit the user's directory ACL. Unix mode bits do not apply.
    Ok(())
}
pub fn write(path: &Path, bytes: &[u8]) -> Result<()> {
    if path.try_exists()? {
        replace(path, &read(path, 128 * 1024 * 1024)?, bytes)
    } else {
        create(path, bytes)
    }
}
fn check(file: &mut File, expected: &[u8]) -> Result<()> {
    let mut before = Vec::new();
    file.take(expected.len() as u64 + 1)
        .read_to_end(&mut before)?;
    ensure!(before == expected, "File changed; inspect it again");
    Ok(())
}
pub fn replace(path: &Path, expected: &[u8], bytes: &[u8]) -> Result<()> {
    replace_file(path, expected, bytes, false)
}
pub fn replace_user_file(path: &Path, expected: &[u8], bytes: &[u8]) -> Result<()> {
    replace_file(path, expected, bytes, true)
}
fn replace_file(path: &Path, expected: &[u8], bytes: &[u8], user: bool) -> Result<()> {
    let (dir, name) = parent(path, false)?;
    let mut file = open_at(&dir, &name, user)?;
    check(&mut file, expected)?;
    let permissions = file.metadata()?.permissions();
    ensure!(!permissions.readonly(), "File is read-only");
    // Keep the temporary in the same directory/volume. Windows rename reports
    // sharing violations; never delete the original to force a replacement.
    let temporary = OsString::from(format!(".fenix-{}.tmp", uuid::Uuid::new_v4()));
    let result = (|| -> Result<()> {
        let mut output = dir
            .open_with(&temporary, options().write(true).create_new(true))?
            .into_std();
        output.write_all(bytes)?;
        output.set_permissions(permissions)?;
        output.sync_all()?;
        drop(output);
        let mut latest = open_at(&dir, &name, user)?;
        check(&mut latest, expected)?;
        drop(latest);
        drop(file);
        dir.rename(&temporary, &dir, &name)?;
        Ok(())
    })();
    if result.is_err() {
        let _ = dir.remove_file(&temporary);
    }
    result
}
pub fn remove(path: &Path, expected: &[u8]) -> Result<()> {
    remove_file(path, expected, false)
}
pub fn remove_user_file(path: &Path, expected: &[u8]) -> Result<()> {
    remove_file(path, expected, true)
}
fn remove_file(path: &Path, expected: &[u8], user: bool) -> Result<()> {
    let (dir, name) = parent(path, false)?;
    let mut file = open_at(&dir, &name, user)?;
    check(&mut file, expected)?;
    drop(file);
    dir.remove_file(name)?;
    Ok(())
}
