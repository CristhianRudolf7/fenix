//! Desktop browser discovery and native process memory measurements.
use std::path::{Path, PathBuf};

pub(crate) fn executable(configured: &str) -> anyhow::Result<String> {
    if let Some(value) = std::env::var_os("FENIX_BROWSER_EXECUTABLE") {
        return resolve(&PathBuf::from(value))
            .map(|p| p.to_string_lossy().into_owned())
            .ok_or_else(|| {
                anyhow::anyhow!("FENIX_BROWSER_EXECUTABLE does not identify an installed browser")
            });
    }
    let legacy = configured == "/usr/bin/chromium" && !cfg!(target_os = "linux");
    if configured != "auto" && !configured.is_empty() && !legacy {
        return Ok(resolve(Path::new(configured))
            .unwrap_or_else(|| configured.into())
            .to_string_lossy()
            .into_owned());
    }
    Ok(candidates()
        .into_iter()
        .find_map(|p| resolve(&p))
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default())
}
fn resolve(path: &Path) -> Option<PathBuf> {
    if path.components().count() > 1 || path.is_absolute() {
        return path.is_file().then(|| path.to_path_buf());
    }
    std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default())
        .filter(|p| p.is_absolute())
        .map(|p| p.join(path))
        .find(|p| p.is_file())
}
fn candidates() -> Vec<PathBuf> {
    if cfg!(windows) {
        let mut paths = Vec::new();
        for variable in ["LOCALAPPDATA", "ProgramFiles", "ProgramFiles(x86)"] {
            if let Some(base) = std::env::var_os(variable) {
                let base = PathBuf::from(base);
                for suffix in [
                    "Chromium/Application/chrome.exe",
                    "Google/Chrome/Application/chrome.exe",
                    "Microsoft/Edge/Application/msedge.exe",
                ] {
                    paths.push(base.join(suffix));
                }
            }
        }
        paths.extend([
            "chrome.exe".into(),
            "msedge.exe".into(),
            "chromium.exe".into(),
        ]);
        paths
    } else if cfg!(target_os = "macos") {
        let mut paths = Vec::new();
        let mut roots = vec![PathBuf::from("/Applications")];
        if let Some(home) = std::env::var_os("HOME") {
            roots.push(PathBuf::from(home).join("Applications"));
        }
        for root in roots {
            for suffix in [
                "Chromium.app/Contents/MacOS/Chromium",
                "Google Chrome.app/Contents/MacOS/Google Chrome",
                "Microsoft Edge.app/Contents/MacOS/Microsoft Edge",
            ] {
                paths.push(root.join(suffix));
            }
        }
        paths.extend([
            "/opt/homebrew/bin/chromium".into(),
            "/usr/local/bin/chromium".into(),
            "chromium".into(),
        ]);
        paths
    } else {
        [
            "chromium",
            "chromium-browser",
            "google-chrome",
            "google-chrome-stable",
            "/usr/bin/chromium",
        ]
        .into_iter()
        .map(PathBuf::from)
        .collect()
    }
}
#[cfg(any(target_os = "linux", target_os = "macos", windows))]
pub(crate) fn memory_mib(root: u32) -> Option<u64> {
    use sysinfo::{Pid, ProcessesToUpdate, System};
    if root == 0 {
        return None;
    }
    let mut system = System::new();
    system.refresh_processes(ProcessesToUpdate::All, true);
    let root = Pid::from_u32(root);
    system.process(root)?;
    let mut ids = std::collections::HashSet::from([root]);
    loop {
        let children: Vec<_> = system
            .processes()
            .iter()
            .filter(|(id, p)| {
                !ids.contains(id) && p.parent().is_some_and(|parent| ids.contains(&parent))
            })
            .map(|(id, _)| *id)
            .collect();
        if children.is_empty() {
            break;
        }
        ids.extend(children);
    }
    Some(
        ids.into_iter()
            .filter_map(|id| system.process(id))
            .map(|p| p.memory())
            .sum::<u64>()
            / 1024
            / 1024,
    )
}
#[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
pub(crate) fn memory_mib(_root: u32) -> Option<u64> {
    None
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn explicit_paths_with_spaces_are_preserved_and_missing_paths_are_not_invented() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("Browser with spaces");
        std::fs::write(&file, b"fixture").unwrap();
        assert_eq!(resolve(&file), Some(file));
        assert!(resolve(&dir.path().join("missing")).is_none());
    }
    #[test]
    fn candidates_are_not_empty() {
        assert!(!candidates().is_empty());
    }
    #[cfg(any(target_os = "linux", target_os = "macos", windows))]
    #[test]
    fn memory_observes_the_current_process_and_missing_process_is_unknown() {
        assert!(memory_mib(std::process::id()).is_some());
        assert_eq!(memory_mib(0), None);
    }
}
