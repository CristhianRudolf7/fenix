//! Native file operations. Workspace paths organize work; OS permissions govern access.
use anyhow::{Context, Result, ensure};
use std::{
    fs::File,
    path::{Component, Path, PathBuf},
};

#[cfg(unix)]
mod unix;
#[cfg(unix)]
pub use unix::*;
#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::*;

/// Resolve symlinks/junctions before opening; normalize missing creation tails.
/// Windows drive-relative paths are rejected rather than resolved against a hidden cwd.
pub fn resolve(workspace: &Path, requested: &Path, create: bool) -> Result<PathBuf> {
    #[cfg(windows)]
    ensure!(
        !requested
            .components()
            .any(|c| matches!(c, Component::Prefix(_)))
            || requested.is_absolute(),
        "Use an absolute drive path such as C:\\folder\\file, not a drive-relative path"
    );
    let joined = if requested.is_absolute() {
        requested.to_owned()
    } else {
        workspace.join(requested)
    };
    let mut resolved = PathBuf::new();
    for component in joined.components() {
        match component {
            Component::ParentDir => {
                resolved.pop();
            }
            Component::CurDir => {}
            other => resolved.push(other.as_os_str()),
        }
        // Prefix alone (C: or a UNC share) is not a complete traversal step.
        #[cfg(windows)]
        if matches!(component, Component::Prefix(_)) {
            continue;
        }
        if resolved.try_exists()? {
            resolved = std::fs::canonicalize(&resolved)?;
        } else if !create {
            return Err(std::io::Error::from(std::io::ErrorKind::NotFound).into());
        }
    }
    ensure!(
        resolved.is_absolute(),
        "File path must resolve to an absolute location"
    );
    Ok(resolved)
}

/// Enumerate through a pinned directory handle on every supported OS.
/// Returned entries remain relative to the pinned directory, without /proc paths.
pub fn entries(directory: &File) -> Result<cap_std::fs::ReadDir> {
    cap_std::fs::Dir::from_std_file(directory.try_clone().context("Clone directory handle")?)
        .entries()
        .map_err(Into::into)
}

pub mod private;

pub mod layout;

/// Read a bounded personal file, allowing ordinary hard links without relaxing private storage.
pub fn read_user_file(path: &Path, limit: u64) -> Result<Vec<u8>> {
    use std::io::Read;
    let file = open_user_file(path)?;
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
