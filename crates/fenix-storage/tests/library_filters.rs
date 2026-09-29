use fenix_storage::{LibraryDocument, ProjectInput, Store};

#[test]
fn name_and_format_filters_apply_before_pagination_with_literal_matching() {
    let directory = tempfile::tempdir().unwrap();
    let store = Store::open(&directory.path().join("library.db")).unwrap();
    let project = store
        .save_project(
            None,
            &ProjectInput {
                name: "Office".into(),
                ..Default::default()
            },
        )
        .unwrap();
    for index in 0..62 {
        let document = LibraryDocument {
            id: format!("document-{index:03}"),
            project_id: (index == 61).then(|| project.id.clone()),
            title: if index >= 60 {
                "Quarterly Report".into()
            } else {
                format!("Note {index}")
            },
            collection: String::new(),
            tags: vec![],
            enabled: true,
            revision: 1,
            filename: if index >= 60 {
                "Report_100%.md".into()
            } else {
                format!("note-{index}.txt")
            },
            sha256: format!("hash-{index}"),
            bytes: 40,
            format: if index >= 60 {
                "md".into()
            } else {
                "txt".into()
            },
            status: "ready".into(),
            embedding_status: "pending".into(),
            error: None,
            warnings: vec![],
            created_at: "2026-09-28T00:00:00Z".into(),
            updated_at: "2026-09-28T00:00:00Z".into(),
        };
        store.library_save(&document, None).unwrap();
    }
    let all = store
        .library_documents_filtered(None, true, 0, 50, "REPORT", "md")
        .unwrap();
    assert_eq!(
        all.len(),
        2,
        "Matching documents beyond the first unfiltered page remain discoverable"
    );
    assert_eq!(
        store
            .library_documents_filtered(None, false, 0, 50, "report", "md")
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        store
            .library_documents_filtered(Some(&project.id), false, 0, 50, "report", "md")
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        store
            .library_documents_filtered(None, true, 1, 1, "report", "md")
            .unwrap()[0]
            .id,
        all[1].id
    );
    assert_eq!(
        store
            .library_documents_filtered(None, true, 0, 50, "100%", "")
            .unwrap()
            .len(),
        2
    );
    assert!(
        store
            .library_documents_filtered(None, true, 0, 50, "report", "pdf")
            .unwrap()
            .is_empty()
    );
    let other = Store::open(&directory.path().join("other.db")).unwrap();
    assert!(
        other
            .library_documents_filtered(None, true, 0, 50, "report", "md")
            .unwrap()
            .is_empty()
    );
}
