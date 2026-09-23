use super::*;

const ADDED: &[&str] = &[
    "agent-security-evaluation",
    "api-security-testing",
    "vulnerability-management",
    "security-artifact-triage",
];

const PREVIOUS: &[(&str, &str)] = &[
    (
        "authentication-integration",
        include_str!("../../tests/fixtures/skills/cybersecurity-v1/authentication-integration.md"),
    ),
    (
        "backup-and-restore",
        include_str!("../../tests/fixtures/skills/cybersecurity-v1/backup-and-restore.md"),
    ),
    (
        "container-workflows",
        include_str!("../../tests/fixtures/skills/cybersecurity-v1/container-workflows.md"),
    ),
    (
        "dependency-and-secret-audit",
        include_str!("../../tests/fixtures/skills/cybersecurity-v1/dependency-and-secret-audit.md"),
    ),
    (
        "detection-engineering",
        include_str!("../../tests/fixtures/skills/cybersecurity-v1/detection-engineering.md"),
    ),
    (
        "identity-access-review",
        include_str!("../../tests/fixtures/skills/cybersecurity-v1/identity-access-review.md"),
    ),
    (
        "incident-response",
        include_str!("../../tests/fixtures/skills/cybersecurity-v1/incident-response.md"),
    ),
    (
        "kubernetes-operations",
        include_str!("../../tests/fixtures/skills/cybersecurity-v1/kubernetes-operations.md"),
    ),
    (
        "network-diagnostics",
        include_str!("../../tests/fixtures/skills/cybersecurity-v1/network-diagnostics.md"),
    ),
    (
        "phishing-triage",
        include_str!("../../tests/fixtures/skills/cybersecurity-v1/phishing-triage.md"),
    ),
    (
        "rag-evaluation",
        include_str!("../../tests/fixtures/skills/cybersecurity-v1/rag-evaluation.md"),
    ),
    (
        "release-and-ci",
        include_str!("../../tests/fixtures/skills/cybersecurity-v1/release-and-ci.md"),
    ),
    (
        "security-code-review",
        include_str!("../../tests/fixtures/skills/cybersecurity-v1/security-code-review.md"),
    ),
];

#[tokio::test]
async fn cybersecurity_capabilities_are_discoverable_without_taking_over_neighboring_tasks()
-> Result<()> {
    let cases: Vec<serde_json::Value> = serde_json::from_str(include_str!(
        "../../../../docs/skills/cybersecurity-cases.json"
    ))?;
    let dir = tempfile::tempdir()?;
    let skills = dir.path().join("skills");
    let rt = Runtime::open(&dir.path().join("data"), Some(&skills)).await?;
    let mut failures = Vec::new();
    for phase in ["baseline", "candidate"] {
        for name in ADDED {
            rt.set_skill_enabled(name, phase == "candidate")?;
        }
        if phase == "baseline" {
            for (name, content) in PREVIOUS {
                fs::write(skills.join(name).join("SKILL.md"), content)?;
            }
        } else {
            rt.install_resources()?;
        }
        for case in &cases {
            let query = case["query"].as_str().unwrap();
            let expected = case["expected"].as_str().unwrap();
            let results = rt.search_skills(query, 3)?;
            let names: Vec<_> = results["skills"]
                .as_array()
                .unwrap()
                .iter()
                .map(|row| row["skill"]["name"].as_str().unwrap())
                .collect();
            let rank = names
                .iter()
                .position(|name| *name == expected)
                .map(|i| i + 1);
            println!(
                "cybersecurity: {}",
                serde_json::json!({
                    "phase": phase, "query": query, "expected": expected,
                    "kind": case["kind"], "rank": rank, "results": names,
                    "expected_available": phase == "candidate" || !ADDED.contains(&expected)
                })
            );
            if phase == "candidate" {
                if rank.is_none() {
                    failures.push(format!("{query}: expected {expected}, got {names:?}"));
                }
                if case["kind"] == "guard" && names.first().is_some_and(|n| ADDED.contains(n)) {
                    failures.push(format!(
                        "New workflow took over neighboring task: {query}: {names:?}"
                    ));
                }
            }
        }
    }
    rt.shutdown().await?;
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[tokio::test]
async fn cybersecurity_parent_upgrades_preserve_custom_text_and_disabled_state() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let skills = dir.path().join("skills");
    let rt = Runtime::open(&dir.path().join("data"), Some(&skills)).await?;
    for (name, content) in PREVIOUS {
        fs::write(skills.join(name).join("SKILL.md"), content)?;
        rt.set_skill_enabled(name, false)?;
    }
    rt.install_resources()?;
    for (name, _) in PREVIOUS {
        let expected = DEFAULTS.iter().find(|(n, _)| n == name).unwrap().1;
        assert_eq!(
            fs::read_to_string(skills.join(name).join("SKILL.md"))?,
            expected
        );
        assert!(rt.skill_load(name).is_err());
    }
    for (name, previous) in PREVIOUS {
        fs::write(
            skills.join(name).join("SKILL.md"),
            format!("{previous}\nOwner instructions.\n"),
        )?;
        rt.set_skill_enabled(name, true)?;
    }
    rt.install_resources()?;
    for (name, previous) in PREVIOUS {
        assert_eq!(
            rt.skill_load(name)?,
            format!("{previous}\nOwner instructions.\n")
        );
    }
    rt.shutdown().await?;
    Ok(())
}

#[tokio::test]
async fn cybersecurity_helpers_and_references_are_bundled_and_respect_activation() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let rt = Runtime::open(&dir.path().join("data"), Some(&dir.path().join("skills"))).await?;
    let affected: Vec<_> = ADDED
        .iter()
        .copied()
        .chain(PREVIOUS.iter().map(|(name, _)| *name))
        .collect();
    let mut checked = 0;
    for (path, bytes) in SKILL_RESOURCES {
        let (name, resource) = path.split_once('/').unwrap();
        if affected.contains(&name) {
            assert_eq!(
                rt.skill_resource(name, resource)?.as_bytes(),
                *bytes,
                "{path}"
            );
            checked += 1;
        }
    }
    assert!(
        checked >= 15,
        "Expected the actual helper/reference resources"
    );
    for name in ADDED {
        rt.set_skill_enabled(name, false)?;
    }
    rt.install_resources()?;
    for name in ADDED {
        assert!(rt.skill_load(name).is_err());
    }
    rt.set_skill_enabled("backup-and-restore", false)?;
    assert!(
        rt.skill_resource("backup-and-restore", "scripts/compare_manifests.py")
            .is_err()
    );
    rt.shutdown().await?;
    Ok(())
}
