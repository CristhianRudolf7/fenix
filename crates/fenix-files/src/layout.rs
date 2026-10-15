//! Platform defaults with explicit legacy detection; no automatic data relocation.
use anyhow::{Context, Result};
use std::path::PathBuf;

pub fn default_skills_dir() -> Result<PathBuf> {
    #[cfg(target_os = "ios")]
    return Ok(default_data_dir()?.join("skills"));
    #[cfg(not(target_os = "ios"))]
    Ok(std::env::home_dir()
        .filter(|p| p.is_absolute())
        .context("Cannot resolve home; configure --skills-dir explicitly")?
        .join(".fenix/skills"))
}
pub fn default_data_dir() -> Result<PathBuf> {
    #[cfg(target_os = "ios")]
    {
        let home = std::env::var_os("FENIX_APP_HOME")
            .context("The iOS host must supply its application container through FENIX_APP_HOME or an explicit data directory")?;
        return apple_mobile_data_dir(std::path::Path::new(&home));
    }
    #[cfg(not(target_os = "ios"))]
    {
        let cwd = std::env::current_dir()?;
        let legacy = cwd.join(".fenix");
        if legacy.try_exists()? {
            return Ok(legacy);
        }
        let home = std::env::home_dir()
            .filter(|p| p.is_absolute())
            .context("Cannot resolve home; configure --data-dir explicitly")?;
        #[cfg(windows)]
        let base = std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .filter(|p| p.is_absolute())
            .unwrap_or_else(|| home.join("AppData/Local"));
        #[cfg(target_os = "macos")]
        let base = home.join("Library/Application Support");
        #[cfg(all(unix, not(target_os = "macos")))]
        let base = std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .filter(|p| p.is_absolute())
            .unwrap_or_else(|| home.join(".local/share"));
        Ok(base.join("fenix"))
    }
}

/// The native host supplies NSHomeDirectory, never the desktop user's home.
pub fn apple_mobile_data_dir(app_home: &std::path::Path) -> Result<PathBuf> {
    anyhow::ensure!(
        app_home.is_absolute(),
        "The application container must be an absolute path"
    );
    Ok(app_home.join("Library/Application Support/fenix"))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ios_layout_keeps_state_and_skills_in_the_application_container() {
        let home = std::env::temp_dir().join("fenix-ios-container");
        assert_eq!(
            apple_mobile_data_dir(&home).unwrap(),
            home.join("Library/Application Support/fenix")
        );
        assert!(apple_mobile_data_dir(std::path::Path::new("relative-home")).is_err());
    }
}
