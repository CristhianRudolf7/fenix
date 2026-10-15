use fenix_files::private::{Lock, PrivateDirectory, create_private_dir};
use std::path::Path;
#[test]
fn stable_instance_lock_releases_when_owner_stops() {
    let temporary = tempfile::tempdir().unwrap();
    let root = create_private_dir(&temporary.path().join("data")).unwrap();
    let path = root.join("instance.lock");
    let first = Lock::try_exclusive(&path).unwrap();
    assert!(Lock::try_exclusive(&path).is_err());
    drop(first);
    assert!(Lock::try_exclusive(&path).is_ok());
}
#[test]
fn anchored_private_writes_no_replace_and_stale_content_checks() {
    let temporary = tempfile::tempdir().unwrap();
    let root = create_private_dir(&temporary.path().join("data")).unwrap();
    let directory = PrivateDirectory::open(&root).unwrap();
    directory
        .create(Path::new("nested/config.json"), b"first", false)
        .unwrap();
    assert!(
        directory
            .create(Path::new("nested/config.json"), b"bad", false)
            .is_err()
    );
    assert!(
        directory
            .replace(Path::new("nested/config.json"), b"stale", b"bad")
            .is_err()
    );
    directory
        .replace(Path::new("nested/config.json"), b"first", b"second")
        .unwrap();
    assert_eq!(
        directory
            .read(Path::new("nested/config.json"), 100)
            .unwrap(),
        b"second"
    );
    assert!(directory.read(Path::new("../outside"), 100).is_err());
    directory.subdir(Path::new("existing"), true).unwrap();
    assert!(
        directory
            .rename_noreplace(Path::new("nested"), Path::new("existing"))
            .is_err()
    );
    assert_eq!(
        directory
            .read(Path::new("nested/config.json"), 100)
            .unwrap(),
        b"second"
    );
}
#[cfg(windows)]
#[test]
fn private_directory_acl_is_protected_and_only_grants_current_user() {
    use windows_permissions::{WindowsSecure, constants::SecurityInformation};
    let temporary = tempfile::tempdir().unwrap();
    let root = create_private_dir(&temporary.path().join("data")).unwrap();
    let descriptor = root
        .as_os_str()
        .security_descriptor(SecurityInformation::Dacl)
        .unwrap();
    let sddl = windows_permissions::wrappers::ConvertSecurityDescriptorToStringSecurityDescriptor(
        &descriptor,
        SecurityInformation::Dacl,
    )
    .unwrap();
    let text = sddl.to_string_lossy();
    let sid = windows_permissions::utilities::current_process_sid()
        .unwrap()
        .to_string();
    assert!(text.contains("D:P"), "{text}");
    assert!(text.contains(&sid), "{text}");
    assert_eq!(text.matches("(A;").count(), 1, "{text}");
}
