//! Compatibility facade for personal native execution; no remote service or socket protocol.
pub mod fs;
use anyhow::{Result, ensure};
pub use fenix_process::local_environment;

/// Reject retired deployment settings rather than accidentally running old tenant jobs locally.
pub fn validate_local_environment() -> Result<()> {
    ensure!(
        std::env::var("FENIX_CONTAINER").as_deref() != Ok("1")
            && std::env::var("FENIX_EXECUTION_REQUIRED").as_deref() != Ok("1")
            && std::env::var_os("FENIX_EXECUTION_SOCKET").is_none(),
        "Personal Fenix uses native local execution. Remove the retired sandbox deployment settings."
    );
    Ok(())
}
#[cfg(test)]
mod tests {
    #[test]
    fn inherited_environment_does_not_include_provider_secrets_or_startup_hooks() {
        let env = super::local_environment(std::path::Path::new("home"));
        assert!(env.contains_key("PATH"));
        for key in [
            "TOKEN",
            "AWS_SECRET_ACCESS_KEY",
            "BASH_ENV",
            "LD_PRELOAD",
            "PYTHONPATH",
            "HTTP_PROXY",
        ] {
            assert!(!env.contains_key(key));
        }
    }
    #[test]
    fn legacy_settings_are_rejected() {
        if std::env::var_os("FENIX_TEST_LEGACY_SANDBOX").is_some() {
            assert!(super::validate_local_environment().is_err());
            return;
        }
        for (name, value) in [
            ("FENIX_CONTAINER", "1"),
            ("FENIX_EXECUTION_REQUIRED", "1"),
            ("FENIX_EXECUTION_SOCKET", "retired.sock"),
        ] {
            assert!(
                std::process::Command::new(std::env::current_exe().unwrap())
                    .args(["--exact", "tests::legacy_settings_are_rejected"])
                    .env("FENIX_TEST_LEGACY_SANDBOX", "1")
                    .env(name, value)
                    .status()
                    .unwrap()
                    .success()
            );
        }
    }
}
