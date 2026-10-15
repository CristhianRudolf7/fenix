use fenix_files as files;
use std::path::PathBuf;

fn root() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    (dir, root)
}

#[test]
fn unicode_external_paths_and_missing_creation_tails() {
    let (_dir, root) = root();
    let workspace = root.join("workspace");
    std::fs::create_dir(&workspace).unwrap();
    let path = files::resolve(
        &workspace,
        std::path::Path::new("../informes/área/文档.txt"),
        true,
    )
    .unwrap();
    files::create_workspace(&path, "línea 1\r\nlínea 2\r\n".as_bytes(), None).unwrap();
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        "línea 1\r\nlínea 2\r\n"
    );
    assert_eq!(
        files::resolve(&workspace, &path, false).unwrap(),
        path.canonicalize().unwrap()
    );
    assert!(files::resolve(&workspace, std::path::Path::new("absent"), false).is_err());
    assert_eq!(
        files::resolve(&workspace, std::path::Path::new("absent/../next"), true).unwrap(),
        workspace.join("next")
    );
}

#[test]
fn optimistic_replace_delete_and_no_overwrite() {
    let (_dir, root) = root();
    let path = root.join("report.txt");
    files::create(&path, b"before").unwrap();
    assert!(files::create(&path, b"overwrite").is_err());
    assert!(files::replace(&path, b"stale", b"wrong").is_err());
    assert_eq!(files::read(&path, 100).unwrap(), b"before");
    files::replace(&path, b"before", b"after").unwrap();
    assert_eq!(files::read(&path, 100).unwrap(), b"after");
    assert!(files::read(&path, 1).is_err());
    assert!(files::remove(&path, b"stale").is_err());
    files::remove(&path, b"after").unwrap();
    assert!(!path.exists());
    assert!(!std::fs::read_dir(root).unwrap().any(|e| {
        e.unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".fenix-")
    }));
}

#[test]
fn personal_hard_links_do_not_relax_private_file_checks() {
    let (_dir, root) = root();
    let original = root.join("original.txt");
    let alias = root.join("alias.txt");
    files::create(&original, b"before").unwrap();
    std::fs::hard_link(&original, &alias).unwrap();
    assert!(files::open(&alias).is_err());
    assert!(files::replace(&alias, b"before", b"bad").is_err());
    assert!(files::open_user_file(&alias).is_ok());
    files::replace_user_file(&alias, b"before", b"after").unwrap();
    assert_eq!(std::fs::read(&alias).unwrap(), b"after");
    assert_eq!(std::fs::read(&original).unwrap(), b"before");
    files::remove_user_file(&original, b"before").unwrap();
}

#[test]
fn pinned_directory_listing_and_root_listing() {
    let (_dir, root) = root();
    std::fs::create_dir(root.join("folder")).unwrap();
    std::fs::write(root.join("folder/entry.txt"), "entry").unwrap();
    let directory = files::directory(&root.join("folder")).unwrap();
    let names = files::entries(&directory)
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect::<Vec<_>>();
    assert_eq!(names, vec![std::ffi::OsString::from("entry.txt")]);
    let anchor = root.ancestors().last().unwrap();
    assert!(files::entries(&files::directory(anchor).unwrap()).is_ok());
}

#[cfg(unix)]
#[test]
fn symlink_resolution_retains_filesystem_parent_semantics() {
    let (_dir, root) = root();
    std::fs::create_dir_all(root.join("nested/target")).unwrap();
    std::fs::create_dir(root.join("workspace")).unwrap();
    std::os::unix::fs::symlink(root.join("nested/target"), root.join("workspace/link")).unwrap();
    let result = files::resolve(
        &root.join("workspace"),
        std::path::Path::new("link/../new.txt"),
        true,
    )
    .unwrap();
    assert_eq!(result, root.join("nested/new.txt"));
}

#[cfg(windows)]
#[test]
fn windows_absolute_drive_paths_and_readonly_files() {
    let (_dir, root) = root();
    assert!(files::resolve(&root, std::path::Path::new("C:relative.txt"), true).is_err());
    let path = root.join("locked.txt");
    files::create(&path, b"original").unwrap();
    let mut permissions = std::fs::metadata(&path).unwrap().permissions();
    permissions.set_readonly(true);
    std::fs::set_permissions(&path, permissions).unwrap();
    assert!(files::replace(&path, b"original", b"wrong").is_err());
    assert_eq!(std::fs::read(&path).unwrap(), b"original");
    let mut permissions = std::fs::metadata(&path).unwrap().permissions();
    permissions.set_readonly(false);
    std::fs::set_permissions(&path, permissions).unwrap();
    files::replace(&path, b"original", b"updated").unwrap();
}

#[cfg(windows)]
#[test]
fn windows_sharing_violation_preserves_original_and_cleans_temporary() {
    use std::os::windows::fs::OpenOptionsExt;
    let (_dir, root) = root();
    let path = root.join("held.txt");
    files::create(&path, b"original").unwrap();
    let held = std::fs::OpenOptions::new()
        .read(true)
        .share_mode(1)
        .open(&path)
        .unwrap();
    assert!(files::replace(&path, b"original", b"wrong").is_err());
    drop(held);
    assert_eq!(std::fs::read(&path).unwrap(), b"original");
    assert_eq!(std::fs::read_dir(&root).unwrap().count(), 1);
}

#[cfg(unix)]
#[test]
fn replacements_preserve_modes_and_listing_survives_directory_rename() {
    use std::os::unix::fs::PermissionsExt;
    let (_dir, root) = root();
    let path = root.join("script");
    files::create(&path, b"before").unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o750)).unwrap();
    files::replace(&path, b"before", b"after").unwrap();
    assert_eq!(
        std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
        0o750
    );
    let original = root.join("folder");
    std::fs::create_dir(&original).unwrap();
    std::fs::write(original.join("old.txt"), "old").unwrap();
    let directory = files::directory(&original).unwrap();
    std::fs::rename(&original, root.join("moved")).unwrap();
    std::fs::create_dir(&original).unwrap();
    std::fs::write(original.join("new.txt"), "new").unwrap();
    let names = files::entries(&directory)
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect::<Vec<_>>();
    assert_eq!(names, vec![std::ffi::OsString::from("old.txt")]);
}
