use super::*;
#[tokio::test]
async fn versions_search_and_scope_are_durable() {
    let dir = tempfile::tempdir().unwrap();
    let rt = Runtime::open(dir.path(), Some(&dir.path().join("skills")))
        .await
        .unwrap();
    let worker = tokio::spawn(rt.clone().library_worker());
    let d = rt
        .library_import(
            "contract.md",
            b"# Contract\n\nCancellation requires thirty days notice.\n",
            ImportOptions::default(),
        )
        .await
        .unwrap();
    for _ in 0..100 {
        if rt.store.library_document(&d.id, None).unwrap().status == "ready" {
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    assert_eq!(
        rt.store.library_document(&d.id, None).unwrap().status,
        "ready"
    );
    let result = rt
        .library_search(Search {
            query: "cancellation".into(),
            semantic: Some(false),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(result["results"].as_array().unwrap().len(), 1);
    assert!(
        result["results"][0]["citation"]
            .as_str()
            .unwrap()
            .contains("revision=1")
    );
    let d2 = rt
        .library_import(
            "contract.md",
            b"# Contract\n\nCancellation requires sixty days notice.",
            ImportOptions {
                id: Some(d.id.clone()),
                expected_revision: Some(1),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert_eq!(d2.revision, 2);
    assert_eq!(
        rt.library_original(&d).unwrap(),
        b"# Contract\n\nCancellation requires thirty days notice.\n"
    );
    assert!(
        rt.library_import(
            "contract.md",
            b"stale",
            ImportOptions {
                id: Some(d.id.clone()),
                expected_revision: Some(1),
                ..Default::default()
            }
        )
        .await
        .is_err()
    );
    rt.stop.cancel();
    worker.await.unwrap();
    rt.library_delete(&d.id, 2).await.unwrap();
    assert!(rt.store.library_document(&d.id, None).is_err());
}
#[test]
fn embedding_responses_are_ordered_and_validated() {
    let a = vec![1.0; embeddings::DIMENSIONS];
    let b = vec![2.0; embeddings::DIMENSIONS];
    let value = json!({"data":[{"index":1,"embedding":b},{"index":0,"embedding":a}],"usage":{"total_tokens":7}});
    let (v, t) = embeddings::decode(&value, 2).unwrap();
    assert_eq!(v[0][0], 1.0);
    assert_eq!(t, 7);
    let mut duplicate = value;
    duplicate["data"][0]["index"] = json!(0);
    assert!(embeddings::decode(&duplicate, 2).is_err());
}

#[tokio::test]
async fn interrupted_extraction_resumes_after_reopening_the_library() {
    let root = tempfile::tempdir().unwrap();
    let skills = root.path().join("skills");
    let id = {
        let rt = Runtime::open(root.path(), Some(&skills)).await.unwrap();
        let mut doc = rt
            .library_import(
                "recovery.txt",
                b"Durable recovery evidence.",
                ImportOptions::default(),
            )
            .await
            .unwrap();
        doc.status = "processing".into();
        rt.store.library_state(&doc).unwrap();
        doc.id
    };
    let rt = Runtime::open(root.path(), Some(&skills)).await.unwrap();
    let worker = tokio::spawn(rt.clone().library_worker());
    for _ in 0..100 {
        if rt.store.library_document(&id, None).unwrap().status == "ready" {
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    let doc = rt.store.library_document(&id, None).unwrap();
    rt.stop.cancel();
    worker.await.unwrap();
    assert_eq!(doc.status, "ready");
    assert_eq!(
        rt.library_original(&doc).unwrap(),
        b"Durable recovery evidence."
    );
    assert_eq!(
        rt.store
            .library_search("recovery", None, &[], None, None, 8)
            .unwrap()
            .len(),
        1
    );
}

#[tokio::test]
async fn reextracting_unchanged_evidence_preserves_citations_and_embeddings() {
    let root = tempfile::tempdir().unwrap();
    let rt = Runtime::open(root.path(), Some(&root.path().join("skills")))
        .await
        .unwrap();
    let mut doc = rt
        .library_import(
            "reference.txt",
            b"A stable source passage.",
            ImportOptions::default(),
        )
        .await
        .unwrap();
    let (first, _) = rt.library_extract(&doc).await.unwrap();
    rt.store.library_store_chunks(&doc, &first).unwrap();
    rt.store
        .library_embeddings(
            &first,
            embeddings::IDENTITY,
            &[vec![0.5; embeddings::DIMENSIONS]],
            5,
        )
        .unwrap();
    doc.status = "partial".into();
    rt.store.library_state(&doc).unwrap();
    rt.library_retry(&doc.id, 1).await.unwrap();
    let (second, _) = rt.library_extract(&doc).await.unwrap();
    assert_ne!(first[0].id, second[0].id);
    rt.store.library_store_chunks(&doc, &second).unwrap();
    let preserved = rt.store.library_chunk(&first[0].id).unwrap();
    assert_eq!(preserved.text, first[0].text);
    assert_eq!(preserved.location, first[0].location);
    assert_eq!(rt.store.library_chunks(&doc.id, 1, 0, 20).unwrap().len(), 1);
    assert!(
        rt.store
            .library_pending_embeddings(&doc.id, 1, embeddings::IDENTITY)
            .unwrap()
            .is_empty()
    );
    doc.status = "ready".into();
    rt.store.library_state(&doc).unwrap();
    let hits = rt
        .store
        .library_search("stable", None, &[], None, None, 8)
        .unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].chunk.id, first[0].id);
    // A changed source location must not inherit a citation to different evidence.
    let mut changed = second;
    changed[0].location["line_start"] = json!(42);
    rt.store.library_store_chunks(&doc, &changed).unwrap();
    assert!(rt.store.library_chunk(&first[0].id).is_err());
}
#[test]
fn chunks_preserve_unicode_and_source_locations() {
    let blocks =
        extract::text_blocks("# Résumé\n\n日本語 investigación\n".as_bytes(), "md").unwrap();
    assert!(blocks.iter().any(|b| b.1["section"] == "Résumé"));
}

#[tokio::test]
async fn hybrid_search_filters_before_ranking_and_reuses_revision_embeddings() {
    let root = tempfile::tempdir().unwrap();
    let rt = Runtime::open(root.path(), Some(&root.path().join("skills")))
        .await
        .unwrap();
    let project = rt
        .store
        .save_project(
            None,
            &fenix_storage::ProjectInput {
                name: "Private study".into(),
                ..Default::default()
            },
        )
        .unwrap();
    let text = b"Cancellation requires notice.";
    let general = rt
        .library_import("terms.md", text, ImportOptions::default())
        .await
        .unwrap();
    let duplicate = rt
        .library_import("renamed.md", text, ImportOptions::default())
        .await
        .unwrap();
    assert_eq!(general.id, duplicate.id);
    let private = rt
        .library_import(
            "terms.md",
            text,
            ImportOptions {
                project_id: Some(project.id.clone()),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert_ne!(general.id, private.id);
    let vector = vec![1.0; embeddings::DIMENSIONS];
    for mut doc in [general.clone(), private.clone()] {
        let (chunks, _) = rt.library_extract(&doc).await.unwrap();
        rt.store.library_store_chunks(&doc, &chunks).unwrap();
        rt.store
            .library_embeddings(
                &chunks,
                embeddings::IDENTITY,
                &vec![vector.clone(); chunks.len()],
                10,
            )
            .unwrap();
        // Retrying extraction of the same version must not repay unchanged vectors.
        rt.store.library_store_chunks(&doc, &chunks).unwrap();
        assert!(
            rt.store
                .library_pending_embeddings(&doc.id, doc.revision, embeddings::IDENTITY)
                .unwrap()
                .is_empty()
        );
        doc.status = "ready".into();
        doc.embedding_status = "ready".into();
        rt.store.library_state(&doc).unwrap();
    }
    let hits = rt
        .store
        .library_search(
            "cancellation",
            None,
            &[],
            None,
            Some((embeddings::IDENTITY, &vector)),
            8,
        )
        .unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].chunk.document_id, general.id);
    assert_eq!(hits[0].branches, vec!["lexical", "semantic"]);
    assert!(
        rt.library_search(Search {
            query: "terms".into(),
            document_ids: vec![private.id.clone()],
            semantic: Some(false),
            ..Default::default()
        })
        .await
        .is_err()
    );
    let selected = rt
        .store
        .library_search(
            "cancellation",
            Some(&project.id),
            &[general.id.clone()],
            None,
            Some((embeddings::IDENTITY, &vector)),
            8,
        )
        .unwrap();
    assert!(selected.is_empty());
    let other_root = tempfile::tempdir().unwrap();
    let other = Runtime::open(other_root.path(), Some(&other_root.path().join("skills")))
        .await
        .unwrap();
    assert!(other.library_read(&private.id, None, 0).is_err());
    let mut new = rt
        .library_import(
            "terms.md",
            b"Cancellation requires notice.\n\nA new condition.",
            ImportOptions {
                id: Some(general.id.clone()),
                expected_revision: Some(1),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    let (chunks, _) = rt.library_extract(&new).await.unwrap();
    rt.store.library_store_chunks(&new, &chunks).unwrap();
    new.status = "ready".into();
    rt.store.library_state(&new).unwrap();
    assert_eq!(
        rt.store
            .library_pending_embeddings(&new.id, 2, embeddings::IDENTITY)
            .unwrap()
            .len(),
        1
    );
    let hits = rt
        .store
        .library_search("cancellation", None, &[], None, None, 8)
        .unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].chunk.revision, 2);
    assert!(
        !rt.store
            .library_chunks(&new.id, 1, 0, 20)
            .unwrap()
            .is_empty()
    );
    rt.store
        .library_metadata(&new.id, 2, "Terms".into(), "".into(), vec![], false)
        .unwrap();
    assert!(
        rt.store
            .library_search(
                "cancellation",
                None,
                &[],
                None,
                Some((embeddings::IDENTITY, &vector)),
                8
            )
            .unwrap()
            .is_empty()
    );
    rt.library_delete(&new.id, 2).await.unwrap();
    assert!(rt.store.library_chunk(&chunks[0].id).is_err());
    assert!(!rt.library_path(&general).unwrap().exists());
    assert!(!rt.library_path(&new).unwrap().exists());
}

#[tokio::test]
async fn embedding_http_contract_retries_transient_failures_without_leaking_keys() {
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{body_partial_json, header, method, path},
    };
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/embeddings"))
        .respond_with(ResponseTemplate::new(503))
        .up_to_n_times(1)
        .expect(1)
        .with_priority(1)
        .mount(&server)
        .await;
    Mock::given(method("POST")).and(path("/embeddings"))
        .and(header("authorization","Bearer test-only-key"))
        .and(header("openai-project","project-test"))
        .and(body_partial_json(json!({"model":"text-embedding-3-small","dimensions":1536,"input":["short source"],"encoding_format":"float"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data":[{"index":0,"embedding":vec![0.5;1536]}],"usage":{"total_tokens":3}})))
        .expect(1).with_priority(2).mount(&server).await;
    let (_, tokens) = embeddings::request(
        &format!("{}/embeddings", server.uri()),
        "test-only-key",
        None,
        Some("project-test"),
        &["short source".into()],
    )
    .await
    .unwrap();
    assert_eq!(tokens, 3);
    Mock::given(path("/denied"))
        .respond_with(ResponseTemplate::new(401))
        .expect(1)
        .mount(&server)
        .await;
    let error = embeddings::request(
        &format!("{}/denied", server.uri()),
        "private-test-value",
        None,
        None,
        &["source".into()],
    )
    .await
    .unwrap_err();
    assert!(!error.to_string().contains("private-test-value"));
}

#[test]
fn csv_preserves_quoted_multiline_rows_without_inventing_headers() {
    let blocks =
        extract::text_blocks(b"person,notes\r\nAlice,\"first, second\nthird\"\r\n", "csv").unwrap();
    assert_eq!(blocks.len(), 2);
    assert_eq!(blocks[1].1["row"], 2);
    assert!(blocks[1].0.contains("Column 2: first, second\nthird"));
}

#[test]
fn spreadsheet_extraction_retains_neighboring_cells_and_formula_provenance() {
    let bytes=fenix_documents::create(serde_json::from_value(json!({"format":"xlsx","sheets":[{"name":"Sales","rows":[["Month","Total"],["April",125],["May",{"formula":"B2*2"}]]}]})).unwrap()).unwrap();
    let rows = extract::text_blocks(&bytes, "xlsx").unwrap();
    assert_eq!(rows.len(), 3);
    assert!(rows[1].0.contains("A2: April"));
    assert!(rows[1].0.contains("B2: 125"));
    assert_eq!(rows[2].1["cells"][1]["formula"], "B2*2");
    assert_eq!(rows[2].1["cells"][1]["cached_value_only"], true);
    assert!(rows[2].0.contains("not recalculated"));
}
