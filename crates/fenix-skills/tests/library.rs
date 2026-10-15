use fenix_skills::{Bundle, Library};
use std::collections::{BTreeMap, BTreeSet};
#[cfg(unix)]
use std::path::Path;
fn bundle() -> Bundle {
    Bundle {
        name: "example".into(),
        files: BTreeMap::from([
            (
                "SKILL.md".into(),
                "---\ndescription: Example\n---\n\nRead references/guide.md.\n".into(),
            ),
            (
                "SKILL.es.md".into(),
                "description: Ejemplo\n\nTexto español.\n".into(),
            ),
            ("references/guide.md".into(), "Guide\r\n".into()),
            ("scripts/check.py".into(), "print('check')\n".into()),
        ]),
        executable: BTreeSet::from(["scripts/check.py".into()]),
    }
}
#[test]
fn install_export_edit_and_delete_preserve_resources_and_languages() {
    let temporary = tempfile::tempdir().unwrap();
    let root = fenix_files::private::create_private_dir(&temporary.path().join("skills")).unwrap();
    let library = Library::open(&root).unwrap();
    let source = bundle();
    library.install(&source, "example").unwrap();
    assert_eq!(library.export("example").unwrap().files, source.files);
    assert_eq!(
        library.export("example").unwrap().executable,
        source.executable
    );
    assert!(library.install(&source, "example").is_err());
    let (content, revision) = library.read_skill("example").unwrap();
    let updated = content.replace("Example", "Updated");
    {
        let _lock = library.lock().unwrap();
        assert!(library.replace_skill("example", "stale", &updated).is_err());
        library
            .replace_skill("example", &revision, &updated)
            .unwrap();
    }
    let exported = library.export("example").unwrap();
    assert_eq!(exported.files["SKILL.es.md"], source.files["SKILL.es.md"]);
    assert_eq!(exported.files["references/guide.md"], "Guide\r\n");
    assert_eq!(exported.executable, source.executable);
    let (_, revision) = library.read_skill("example").unwrap();
    let _lock = library.lock().unwrap();
    assert!(library.detach("example", "stale").is_err());
    let removed = library.detach("example", &revision).unwrap();
    assert!(!root.join("example").exists());
    library.remove_detached(&removed).unwrap();
    assert!(!root.join(removed).exists());
}
#[test]
fn concurrent_installations_never_overwrite_each_other() {
    let temporary = tempfile::tempdir().unwrap();
    let root = fenix_files::private::create_private_dir(&temporary.path().join("skills")).unwrap();
    let workers = (0..2)
        .map(|i| {
            let root = root.clone();
            std::thread::spawn(move || {
                let mut bundle = bundle();
                bundle
                    .files
                    .insert("references/owner.md".into(), i.to_string());
                Library::open(&root)
                    .unwrap()
                    .install(&bundle, "example")
                    .is_ok()
            })
        })
        .collect::<Vec<_>>();
    assert_eq!(
        workers
            .into_iter()
            .map(|w| w.join().unwrap() as usize)
            .sum::<usize>(),
        1
    );
    assert!(Library::open(&root).unwrap().export("example").is_ok());
    assert!(!std::fs::read_dir(root).unwrap().any(|e| {
        e.unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".import-")
    }));
}
#[test]
fn invalid_names_paths_sizes_and_binary_content_are_rejected() {
    for name in ["../outside", "", "a/b", "a\\b"] {
        let mut source = bundle();
        source.name = name.into();
        assert!(source.validate().is_err());
    }
    for path in [
        "../outside",
        "/absolute",
        ".secret",
        "a\\b",
        "C:drive",
        "a//b",
        "a/./b",
        "a/../b",
        "NUL.txt",
        "references/COM1.md",
        "trailing.",
        "space ",
    ] {
        let mut source = bundle();
        source.files.insert(path.into(), "x".into());
        assert!(source.validate().is_err(), "{path}");
    }
    let mut source = bundle();
    source.files.insert("binary".into(), "\0".into());
    assert!(source.validate().is_err());
    let mut source = bundle();
    source.files.insert("large".into(), "x".repeat(128 * 1024));
    assert!(source.validate().is_err());
    assert!(fenix_skills::metadata("---\ndescription: Unclosed").is_err());
}
#[cfg(unix)]
#[test]
fn links_in_skill_resources_are_rejected() {
    let temporary = tempfile::tempdir().unwrap();
    let root = fenix_files::private::create_private_dir(&temporary.path().join("skills")).unwrap();
    let library = Library::open(&root).unwrap();
    library.install(&bundle(), "example").unwrap();
    std::os::unix::fs::symlink(temporary.path(), root.join("example/escape")).unwrap();
    assert!(library.export("example").is_err());
    std::fs::remove_file(root.join("example/escape")).unwrap();
    std::fs::hard_link(root.join("example/SKILL.md"), root.join("example/alias.md")).unwrap();
    assert!(library.export("example").is_err());
    assert!(library.read_skill("example").is_err());
    let directory = fenix_files::private::PrivateDirectory::open(&root).unwrap();
    assert!(
        directory
            .open_file(Path::new("example/../outside"))
            .is_err()
    );
}
