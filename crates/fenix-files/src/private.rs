//! Private storage and portable advisory locks. Never silently weaken permissions.
use anyhow::{Context, Result, ensure};
use cap_fs_ext::{DirExt, FollowSymlinks, OpenOptionsFollowExt};
use cap_std::fs::{Dir, OpenOptions};
use fs2::FileExt;
use std::{
    fs::{File, Permissions},
    io::{Read, Write},
    path::{Component, Path, PathBuf},
};

pub fn secure(path: &Path) -> Result<()> {
    let metadata = std::fs::symlink_metadata(path)?;
    ensure!(
        !metadata.file_type().is_symlink(),
        "Private storage cannot be a link"
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(
            path,
            Permissions::from_mode(if metadata.is_dir() { 0o700 } else { 0o600 }),
        )?;
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::{MetadataExt, OpenOptionsExt};
        use windows_permissions::{
            LocalBox, SecurityDescriptor,
            constants::{SeObjectType, SecurityInformation},
            wrappers::SetSecurityInfo,
        };
        ensure!(
            metadata.file_attributes() & 0x400 == 0,
            "Private storage cannot be a reparse point"
        );
        let sid = windows_permissions::utilities::current_process_sid()?;
        let sddl = format!("D:P(A;OICI;FA;;;{sid})");
        let descriptor: LocalBox<SecurityDescriptor> = sddl.parse()?;
        let mut file = std::fs::OpenOptions::new()
            .access_mode(0x00040000 | 0x00020000)
            .custom_flags(0x02000000 | 0x00200000)
            .open(path)?;
        SetSecurityInfo(
            &mut file,
            SeObjectType::SE_FILE_OBJECT,
            SecurityInformation::Dacl | SecurityInformation::ProtectedDacl,
            None,
            None,
            descriptor.dacl(),
            None,
        )?;
    }
    Ok(())
}
pub fn create_private_dir(path: &Path) -> Result<PathBuf> {
    std::fs::create_dir_all(path)?;
    secure(path)?;
    Ok(path.canonicalize()?)
}
pub fn sync_directory(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        crate::directory(path)?.sync_all()?;
    }
    #[cfg(windows)]
    {
        let _ = path;
    }
    Ok(())
}
/// Stable lock file rather than directory flock (unsupported on Windows).
pub struct Lock(File);
impl Lock {
    pub fn try_exclusive(path: &Path) -> Result<Self> {
        let directory = PrivateDirectory::open(path.parent().context("Lock parent required")?)?;
        let name = path.file_name().context("Lock name required")?;
        let mut options = nofollow();
        options.read(true).write(true).create(true);
        #[cfg(unix)]
        {
            use cap_std::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let file = directory.dir.open_with(name, &options)?.into_std();
        validate(&file)?;
        file.try_lock_exclusive()?;
        Ok(Self(file))
    }
    pub fn exclusive(directory: &PrivateDirectory) -> Result<Self> {
        let mut options = nofollow();
        options.read(true).write(true).create(true);
        #[cfg(unix)]
        {
            use cap_std::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let file = directory
            .dir
            .open_with(".skills.lock", &options)?
            .into_std();
        validate(&file)?;
        file.lock_exclusive()?;
        Ok(Self(file))
    }
}
impl Drop for Lock {
    fn drop(&mut self) {
        let _ = FileExt::unlock(&self.0);
    }
}
fn validate(file: &File) -> Result<()> {
    ensure!(
        file.metadata()?.is_file(),
        "Private storage requires regular files"
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        ensure!(
            file.metadata()?.nlink() == 1,
            "Private files cannot have hard links"
        );
    }
    #[cfg(windows)]
    {
        ensure!(
            winapi_util::file::information(file)?.number_of_links() == 1,
            "Private files cannot have hard links"
        );
    }
    Ok(())
}
fn nofollow() -> OpenOptions {
    let mut options = OpenOptions::new();
    options.follow(FollowSymlinks::No);
    #[cfg(unix)]
    {
        use cap_std::fs::OpenOptionsExt;
        options.custom_flags(nix::libc::O_NONBLOCK);
    }
    options
}
fn relative(path: &Path) -> Result<()> {
    ensure!(
        !path.as_os_str().is_empty()
            && path.components().all(|c| matches!(c, Component::Normal(_))),
        "Private paths must be relative without traversal"
    );
    Ok(())
}
pub struct PrivateDirectory {
    dir: Dir,
}
impl PrivateDirectory {
    pub fn open(path: &Path) -> Result<Self> {
        Ok(Self {
            dir: Dir::from_std_file(crate::directory(path)?),
        })
    }
    pub fn subdir(&self, path: &Path, create: bool) -> Result<Self> {
        relative(path)?;
        let mut dir = self.dir.try_clone()?;
        for component in path.components() {
            if create {
                let builder = cap_std::fs::DirBuilder::new();
                #[cfg(unix)]
                let builder = {
                    use cap_std::fs::DirBuilderExt;
                    let mut builder = builder;
                    builder.mode(0o700);
                    builder
                };
                match dir.create_dir_with(component.as_os_str(), &builder) {
                    Ok(()) => {}
                    Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
                    Err(e) => return Err(e.into()),
                }
            }
            dir = dir.open_dir_nofollow(component.as_os_str())?;
        }
        Ok(Self { dir })
    }
    fn parent(&self, path: &Path, create: bool) -> Result<(Self, std::ffi::OsString)> {
        relative(path)?;
        let parent = match path.parent().filter(|p| !p.as_os_str().is_empty()) {
            Some(parent) => self.subdir(parent, create)?,
            None => Self {
                dir: self.dir.try_clone()?,
            },
        };
        Ok((
            parent,
            path.file_name().context("File name required")?.to_owned(),
        ))
    }
    pub fn open_file(&self, path: &Path) -> Result<File> {
        let (parent, name) = self.parent(path, false)?;
        let file = parent
            .dir
            .open_with(name, nofollow().read(true))?
            .into_std();
        validate(&file)?;
        Ok(file)
    }
    pub fn read(&self, path: &Path, limit: u64) -> Result<Vec<u8>> {
        let file = self.open_file(path)?;
        ensure!(
            file.metadata()?.len() <= limit,
            "Private file exceeds its size limit"
        );
        let mut bytes = Vec::new();
        file.take(limit.saturating_add(1)).read_to_end(&mut bytes)?;
        ensure!(
            bytes.len() as u64 <= limit,
            "Private file grew beyond its size limit"
        );
        Ok(bytes)
    }
    pub fn entries(&self) -> Result<cap_std::fs::ReadDir> {
        Ok(self.dir.entries()?)
    }
    pub fn create(&self, path: &Path, bytes: &[u8], executable: bool) -> Result<()> {
        let (parent, name) = self.parent(path, true)?;
        let mut options = nofollow();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use cap_std::fs::OpenOptionsExt;
            options.mode(if executable { 0o700 } else { 0o600 });
        }
        #[cfg(windows)]
        {
            let _ = executable;
        }
        let mut file = parent.dir.open_with(name, &options)?.into_std();
        file.write_all(bytes)?;
        file.sync_all()?;
        parent.sync()?;
        Ok(())
    }
    pub fn replace(&self, path: &Path, expected: &[u8], bytes: &[u8]) -> Result<()> {
        let (parent, name) = self.parent(path, false)?;
        let name = Path::new(&name);
        ensure!(
            parent.read(name, expected.len() as u64)? == expected,
            "Private file changed before editing"
        );
        let permissions = parent.open_file(name)?.metadata()?.permissions();
        #[cfg(windows)]
        ensure!(!permissions.readonly(), "Private file is read-only");
        let temporary = format!(".write-{}.tmp", uuid::Uuid::new_v4());
        let result = (|| -> Result<()> {
            parent.create(Path::new(&temporary), bytes, false)?;
            let mut options = nofollow();
            options.write(true);
            let output = parent.dir.open_with(&temporary, &options)?.into_std();
            output.set_permissions(permissions)?;
            output.sync_all()?;
            drop(output);
            ensure!(
                parent.read(name, expected.len() as u64)? == expected,
                "Private file changed before editing"
            );
            parent.dir.rename(&temporary, &parent.dir, name)?;
            parent.sync()?;
            Ok(())
        })();
        if result.is_err() {
            let _ = parent.dir.remove_file(&temporary);
        }
        result
    }
    pub fn rename(&self, source: &Path, destination: &Path) -> Result<()> {
        relative(source)?;
        relative(destination)?;
        self.dir.rename(source, &self.dir, destination)?;
        self.sync()
    }
    pub fn rename_noreplace(&self, source: &Path, destination: &Path) -> Result<()> {
        relative(source)?;
        relative(destination)?;
        #[cfg(unix)]
        rustix::fs::renameat_with(
            &self.dir,
            source,
            &self.dir,
            destination,
            rustix::fs::RenameFlags::NOREPLACE,
        )?;
        #[cfg(windows)]
        {
            ensure!(!self.exists(destination)?, "Destination exists");
            self.dir.rename(source, &self.dir, destination)?;
        }
        self.sync()
    }
    pub fn exists(&self, path: &Path) -> Result<bool> {
        relative(path)?;
        match self.dir.symlink_metadata(path) {
            Ok(_) => Ok(true),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(e) => Err(e.into()),
        }
    }
    pub fn remove_tree(&self, path: &Path) -> Result<()> {
        relative(path)?;
        // Refuse to traverse a replaced root link. Descendant links are removed, never followed.
        drop(self.subdir(path, false)?);
        self.dir.remove_dir_all(path)?;
        self.sync()
    }
    pub fn sync(&self) -> Result<()> {
        #[cfg(unix)]
        {
            let fd = rustix::fs::openat(
                &self.dir,
                ".",
                rustix::fs::OFlags::RDONLY
                    | rustix::fs::OFlags::DIRECTORY
                    | rustix::fs::OFlags::CLOEXEC,
                rustix::fs::Mode::empty(),
            )?;
            File::from(fd).sync_all()?;
        }
        Ok(())
    }
}
pub fn executable(metadata: &std::fs::Metadata) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        metadata.permissions().mode() & 0o111 != 0
    }
    #[cfg(windows)]
    {
        let _ = metadata;
        false
    }
}
pub fn permissions_from_mode(mode: u32) -> Option<Permissions> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        Some(Permissions::from_mode(mode))
    }
    #[cfg(windows)]
    {
        let _ = mode;
        None
    }
}
pub fn permission_mode(metadata: &std::fs::Metadata) -> Option<u32> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        Some(metadata.permissions().mode())
    }
    #[cfg(windows)]
    {
        let _ = metadata;
        None
    }
}
