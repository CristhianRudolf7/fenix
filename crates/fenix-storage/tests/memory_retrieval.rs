use fenix_storage::{
    EntityInput, MemoryMetadata, MemorySearchOptions, Store, memory_eval::retrieval_metrics,
};
use serde_json::json;
#[path = "support/memory_corpus.rs"]
mod support;
fn database() -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().unwrap();
    let s = Store::open(&dir.path().join("test.db")).unwrap();
    (dir, s)
}
#[test]
fn relational_search_retains_paths_without_crossing_scope_or_using_substrings() {
    let (_dir, s) = database();
    let c = support::corpus(&s);
    let report = s
        .search_memory_evidence(
            "Estudio Aurora",
            Some(&c.project),
            &Default::default(),
            vec![],
        )
        .unwrap();
    let hit = report
        .results
        .iter()
        .find(|h| h.memory.id == c.facts["owner"].id)
        .unwrap();
    assert!(
        hit.evidence
            .iter()
            .any(|e| e["path"].as_array().is_some_and(|p| p.len() == 2))
    );
    assert!(
        report
            .results
            .iter()
            .all(|h| h.memory.project_id.as_deref() == Some(&c.project))
    );
    assert!(
        s.search_memory_evidence("Auroralight", Some(&c.project), &Default::default(), vec![])
            .unwrap()
            .results
            .is_empty()
    );
    assert!(
        s.search_memory_evidence(
            "Aurora",
            Some(&c.other_project),
            &Default::default(),
            vec![]
        )
        .unwrap()
        .results
        .is_empty()
    );
    assert_eq!(
        s.memory_neighbors(&c.anchor, Some(&c.project), 2, 30)
            .unwrap()["neighbors"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    let pack = s
        .memory_context_pack("Estudio Aurora", Some(&c.project), 3000)
        .unwrap();
    assert!(
        pack["text"]
            .as_str()
            .unwrap()
            .contains(&c.facts["owner"].id)
    );
    assert!(pack["text"].as_str().unwrap().contains("relation_path"));
}
#[test]
fn changed_or_withdrawn_evidence_disables_relational_hits() {
    let (_dir, s) = database();
    let c = support::corpus(&s);
    s.delete_memory_scoped(&c.facts["owner"].id, Some(&c.project))
        .unwrap();
    let report = s
        .search_memory_evidence("Aurora", Some(&c.project), &Default::default(), vec![])
        .unwrap();
    assert!(
        !report
            .results
            .iter()
            .any(|h| h.memory.id == c.facts["owner"].id)
    );
    assert_eq!(
        s.memory_diagnostics(Some(&c.project), None).unwrap()["inactive_evidence_or_schema_edges"],
        1
    );
}
#[test]
fn editing_active_relation_evidence_requires_explicit_revalidation() {
    let (_dir, s) = database();
    let c = support::corpus(&s);
    let m = &c.facts["owner"];
    s.save_memory(
        Some(&m.id),
        "Ownership",
        "Marta no longer coordinates Helios",
        "owner",
    )
    .unwrap();
    assert!(
        s.memory_neighbors(&c.anchor, Some(&c.project), 2, 30)
            .unwrap()["neighbors"]
            .as_array()
            .unwrap()
            .iter()
            .all(|r| r["edge"]["evidence_memory_id"] != m.id)
    );
    assert_eq!(
        s.memory_diagnostics(Some(&c.project), None).unwrap()["inactive_evidence_or_schema_edges"],
        1
    );
}
#[test]
fn historical_queries_do_not_mix_current_graph_and_vectors() {
    let (_dir, s) = database();
    let c = support::corpus(&s);
    let opts = MemorySearchOptions {
        as_of: Some(chrono::Utc::now().to_rfc3339()),
        ..Default::default()
    };
    let report = s
        .search_memory_evidence("Aurora", Some(&c.project), &opts, vec![])
        .unwrap();
    assert!(
        report
            .warnings
            .iter()
            .any(|w| w == "historical_graph_disabled")
    );
    assert!(
        report
            .results
            .iter()
            .all(|h| !h.branches.contains(&"graph".into()))
    );
    assert!(
        s.search_memory_evidence(
            "Aurora",
            Some(&c.project),
            &opts,
            vec![(c.facts["owner"].clone(), 0.9)]
        )
        .is_err()
    );
}
#[test]
fn exact_content_is_not_the_same_as_title_or_semantic_similarity() {
    let (_dir, s) = database();
    let c = support::corpus(&s);
    let m = &c.facts["instrument"];
    let report = s
        .search_memory_evidence(&m.content, Some(&c.project), &Default::default(), vec![])
        .unwrap();
    assert_eq!(report.results[0].create_safety, "exists");
    let report = s
        .search_memory_evidence(&m.title, Some(&c.project), &Default::default(), vec![])
        .unwrap();
    assert_eq!(report.results[0].create_safety, "possible_match");
    let report = s
        .search_memory_evidence(
            "optical device",
            Some(&c.project),
            &Default::default(),
            vec![(m.clone(), 0.99), (c.facts["private"].clone(), 1.0)],
        )
        .unwrap();
    assert_eq!(report.results.len(), 1);
    assert_eq!(report.results[0].create_safety, "unknown");
}
#[test]
fn recency_is_kind_sensitive_and_never_erases_old_facts() {
    let (_dir, s) = database();
    let old = (chrono::Utc::now() - chrono::Duration::days(400)).to_rfc3339();
    for (content, kind) in [
        ("Service status was pending", "fact"),
        ("Service status is ready", "fact"),
        ("Service language is Rust", "preference"),
    ] {
        s.save_memory_record(
            None,
            "Service",
            content,
            "owner",
            None,
            None,
            Some(MemoryMetadata {
                kind: kind.into(),
                valid_from: content.contains("was").then(|| old.clone()),
                ..Default::default()
            }),
        )
        .unwrap();
    }
    let r = s
        .search_memory_evidence("current service", None, &Default::default(), vec![])
        .unwrap();
    assert_eq!(r.results.len(), 3);
    assert_eq!(r.diagnostics["recency_applied"], true);
    assert_eq!(
        r.results
            .iter()
            .find(|h| h.memory.metadata.kind == "preference")
            .unwrap()
            .freshness["ranking_multiplier"],
        1.0
    );
    assert_eq!(
        r.results
            .iter()
            .find(|h| h.memory.content.contains("was"))
            .unwrap()
            .freshness["needs_revalidation"],
        true
    );
    let r = s
        .search_memory_evidence("service", None, &Default::default(), vec![])
        .unwrap();
    assert_eq!(r.diagnostics["recency_applied"], false);
}
fn pair(s: &Store) -> serde_json::Value {
    for content in [
        "The service permits external uploads",
        "The service forbids external uploads",
    ] {
        s.save_memory(None, "Upload policy", content, "owner")
            .unwrap();
    }
    s.review_memory_overlaps().unwrap();
    s.memory_review_pairs(None).unwrap().remove(0)
}
fn assessment(p: &serde_json::Value) -> serde_json::Value {
    json!({"classification":"ambiguous","reason":"The sources do not specify the same time interval.","question":"Which upload policy is currently valid?","first_quote":p["first"]["content"],"second_quote":p["second"]["content"]})
}
#[test]
fn review_proposals_preserve_facts_validate_quotes_and_expire_on_revision() {
    let (_dir, s) = database();
    let p = pair(&s);
    let claimed = s.claim_memory_review().unwrap().unwrap();
    assert!(
        s.save_memory_review_assessment(&claimed, &assessment(&claimed))
            .unwrap()
    );
    assert!(s.claim_memory_review().unwrap().is_none());
    assert!(
        s.memories("", 10)
            .unwrap()
            .iter()
            .all(|m| m.metadata.status == "confirmed")
    );
    assert!(
        s.memory_context_pack("Upload policy", None, 3000).unwrap()["text"]
            .as_str()
            .unwrap()
            .contains("Which upload policy")
    );
    let mut bad = assessment(&p);
    bad["first_quote"] = json!("invented quote");
    assert!(s.save_memory_review_assessment(&p, &bad).is_err());
    s.save_memory(
        Some(p["first_id"].as_str().unwrap()),
        "Upload policy",
        "The service permits internal uploads only",
        "owner",
    )
    .unwrap();
    assert!(
        !s.save_memory_review_assessment(&p, &assessment(&p))
            .unwrap()
    );
    assert!(s.memory_review_pairs(None).unwrap().is_empty());
    s.review_memory_overlaps().unwrap();
    assert!(
        s.resolve_memory_pair_at(
            None,
            p["first_id"].as_str().unwrap(),
            p["second_id"].as_str().unwrap(),
            "dismissed",
            p["first_revision"].as_i64().unwrap(),
            p["second_revision"].as_i64().unwrap()
        )
        .is_err()
    );
    assert!(s.claim_memory_review().unwrap().is_some());
}
#[test]
fn ambiguous_review_attempts_are_bounded_and_survive_reopening() {
    let (dir, s) = database();
    let _p = pair(&s);
    assert!(s.claim_memory_review().unwrap().is_some());
    assert!(
        s.claim_memory_review().unwrap().is_none(),
        "lease excludes concurrent reviewers"
    );
    let db = rusqlite::Connection::open(dir.path().join("test.db")).unwrap();
    db.execute(
        "UPDATE memory_review_pairs SET review_after='2020-01-01T00:00:00Z'",
        [],
    )
    .unwrap();
    assert!(s.claim_memory_review().unwrap().is_some());
    assert!(s.claim_memory_review().unwrap().is_none());
    drop(s);
    let s = Store::open(&dir.path().join("test.db")).unwrap();
    assert!(s.claim_memory_review().unwrap().is_none());
    assert_eq!(
        s.memory_diagnostics(None, None).unwrap()["review_exhausted"],
        1
    );
}
#[test]
fn disjoint_explicit_validity_needs_no_model() {
    let (_dir, s) = database();
    for (content, from, until) in [
        (
            "Ana works at Company A",
            "2020-01-01T00:00:00Z",
            "2021-01-01T00:00:00Z",
        ),
        (
            "Ana works at Company B",
            "2021-01-01T00:00:00Z",
            "2022-01-01T00:00:00Z",
        ),
    ] {
        s.save_memory_record(
            None,
            "Employer",
            content,
            "owner",
            None,
            None,
            Some(MemoryMetadata {
                status: "candidate".into(),
                valid_from: Some(from.into()),
                valid_until: Some(until.into()),
                ..Default::default()
            }),
        )
        .unwrap();
    }
    s.review_memory_overlaps().unwrap();
    assert_eq!(
        s.memory_review_pairs(None).unwrap()[0]["assessment"]["classification"],
        "temporal_change"
    );
    assert!(s.claim_memory_review().unwrap().is_none());
}
#[test]
fn health_repairs_only_derived_index_and_reports_orphans() {
    let (dir, s) = database();
    let c = support::corpus(&s);
    s.save_entity(
        None,
        Some(&c.project),
        EntityInput {
            name: "Unlinked note".into(),
            kind: "concept".into(),
            aliases: vec![],
            description: String::new(),
        },
    )
    .unwrap();
    let external = rusqlite::Connection::open(dir.path().join("test.db")).unwrap();
    external
        .execute(
            "DELETE FROM memory_fts WHERE id=?1",
            [&c.facts["language"].id],
        )
        .unwrap();
    let health = s
        .memory_diagnostics(Some(&c.project), Some("test-model"))
        .unwrap();
    assert_eq!(health["lexical_missing_or_outdated"], 1);
    assert_eq!(health["entities_without_facts_or_relations"], 1);
    assert!(health["pending_embeddings"].as_i64().unwrap() > 0);
    let before = s
        .memory_history(&c.facts["language"].id, Some(&c.project))
        .unwrap();
    assert_eq!(s.repair_memory_lexical_index().unwrap(), 1);
    assert_eq!(s.repair_memory_lexical_index().unwrap(), 0);
    assert_eq!(
        before,
        s.memory_history(&c.facts["language"].id, Some(&c.project))
            .unwrap()
    );
    assert_eq!(
        s.memory_diagnostics(Some(&c.other_project), None).unwrap()["entities_without_facts_or_relations"],
        0
    );
}
#[test]
fn evaluator_uses_unique_results_and_compares_same_corpus() {
    let m = retrieval_metrics(&["a".into(), "a".into(), "b".into()], &["a".into()], 3);
    assert_eq!(m["precision_at_k"], 1.0 / 3.0);
    assert_eq!(m["recall_at_k"], 1.0);
    assert_eq!(
        retrieval_metrics(&["unexpected".into()], &[], 5)["recall_at_k"],
        0.0
    );
    let (_dir, s) = database();
    let c = support::corpus(&s);
    let report = s.evaluate_memory_retrieval(&c.cases, 5).unwrap();
    assert_eq!(report["query_count"], 26);
    assert_eq!(report["runs"][1]["forbidden_hits"], 0);
    assert!(
        report["runs"][1]["mean_recall"].as_f64().unwrap()
            > report["runs"][0]["mean_recall"].as_f64().unwrap()
    );
}

#[test]
fn upgrade_keeps_existing_pending_reviews_reachable() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("legacy.db");
    let db = rusqlite::Connection::open(&path).unwrap();
    for migration in [
        include_str!("../migrations/001_initial.sql"),
        include_str!("../migrations/002_subagents.sql"),
        include_str!("../migrations/003_projects.sql"),
        include_str!("../migrations/004_planning.sql"),
        include_str!("../migrations/005_plan_reviews.sql"),
        include_str!("../migrations/006_subagent_inbox.sql"),
        include_str!("../migrations/007_adaptive_memory.sql"),
        include_str!("../migrations/008_designs.sql"),
        include_str!("../migrations/009_library.sql"),
        include_str!("../migrations/010_completion_audits.sql"),
        include_str!("../migrations/011_project_specs.sql"),
        include_str!("../migrations/012_tool_output_search.sql"),
    ] {
        db.execute_batch(migration).unwrap();
    }
    for (id, content) in [
        ("a", "The service permits external uploads"),
        ("b", "The service forbids external uploads"),
    ] {
        db.execute("INSERT INTO memories(id,title,content,source,normalized,created_at,updated_at) VALUES (?1,'Policy',?2,'owner',?2,'2026-01-01T00:00:00Z','2026-01-01T00:00:00Z')",rusqlite::params![id,content]).unwrap();
    }
    db.execute("INSERT INTO memory_review_pairs(first_id,second_id,reason) VALUES ('a','b','legacy overlap')",[]).unwrap();
    db.execute("UPDATE memory_maintenance_cursor SET seq=999", [])
        .unwrap();
    drop(db);
    let s = Store::open(&path).unwrap();
    let review = s.claim_memory_review().unwrap().unwrap();
    assert!(review["first_revision"].as_i64().unwrap() > 0);
    assert_eq!(review["reason"], "legacy overlap");
}

#[test]
fn general_context_survives_inside_a_project_without_other_project_facts() {
    let (_dir, s) = database();
    let c = support::corpus(&s);
    let m = s
        .save_memory(None, "User name", "The owner is Cristhian", "owner")
        .unwrap();
    let pack = s
        .memory_context_pack("Cristhian budget", Some(&c.project), 3000)
        .unwrap();
    assert!(pack["text"].as_str().unwrap().contains(&m.id));
    assert!(
        !pack["text"]
            .as_str()
            .unwrap()
            .contains(&c.facts["private"].id)
    );
}

#[test]
fn maintenance_bounds_scanning_and_resumes_its_cursor() {
    let (dir, s) = database();
    for n in 0..120 {
        s.save_memory(
            None,
            "Indexed fact",
            &format!("Unique stored fact number {n}"),
            "owner",
        )
        .unwrap();
    }
    let db = rusqlite::Connection::open(dir.path().join("test.db")).unwrap();
    db.execute("DELETE FROM memory_fts", []).unwrap();
    assert_eq!(s.repair_memory_lexical_index().unwrap(), 50);
    drop(s);
    let s = Store::open(&dir.path().join("test.db")).unwrap();
    assert_eq!(s.repair_memory_lexical_index().unwrap(), 50);
    assert_eq!(s.repair_memory_lexical_index().unwrap(), 20);
    assert_eq!(s.repair_memory_lexical_index().unwrap(), 0);
    assert_eq!(
        s.memory_diagnostics(None, None).unwrap()["lexical_missing_or_outdated"],
        0
    );
}
