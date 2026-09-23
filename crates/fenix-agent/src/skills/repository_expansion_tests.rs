use super::*;

const ADDED: &[&str] = &[
    "systematic-review",
    "research-evidence-audit",
    "web-animation",
    "parametric-cad",
    "cad-drawing-and-fabrication",
    "robot-description",
    "3d-printing-preparation",
    "huggingface-hub",
    "model-selection",
    "ml-demo-apps",
    "browser-ml-inference",
];

const PREVIOUS: &[(&str, &str)] = &[
    (
        "literature-review",
        include_str!("../../tests/fixtures/skills/repository-expansion-v1/literature-review.md"),
    ),
    (
        "scientific-writing",
        include_str!("../../tests/fixtures/skills/repository-expansion-v1/scientific-writing.md"),
    ),
    (
        "peer-review",
        include_str!("../../tests/fixtures/skills/repository-expansion-v1/peer-review.md"),
    ),
    (
        "bibliography-management",
        include_str!(
            "../../tests/fixtures/skills/repository-expansion-v1/bibliography-management.md"
        ),
    ),
    (
        "frontend-design-and-accessibility",
        include_str!(
            "../../tests/fixtures/skills/repository-expansion-v1/frontend-design-and-accessibility.md"
        ),
    ),
    (
        "ml-training",
        include_str!("../../tests/fixtures/skills/repository-expansion-v1/ml-training.md"),
    ),
    (
        "ml-dataset-preparation",
        include_str!(
            "../../tests/fixtures/skills/repository-expansion-v1/ml-dataset-preparation.md"
        ),
    ),
    (
        "ml-inference-deployment",
        include_str!(
            "../../tests/fixtures/skills/repository-expansion-v1/ml-inference-deployment.md"
        ),
    ),
    (
        "experiment-and-benchmark-design",
        include_str!(
            "../../tests/fixtures/skills/repository-expansion-v1/experiment-and-benchmark-design.md"
        ),
    ),
];

#[tokio::test]
async fn repository_capabilities_are_discoverable_without_taking_over_neighboring_tasks()
-> Result<()> {
    let cases: Vec<serde_json::Value> = serde_json::from_str(include_str!(
        "../../../../docs/skills/repository-expansion-cases.json"
    ))?;
    let dir = tempfile::tempdir()?;
    let skills = dir.path().join("skills");
    let rt = Runtime::open(&dir.path().join("data"), Some(&skills)).await?;
    let mut failures = Vec::new();
    let mut baseline_guards = std::collections::HashMap::new();
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
                "repository-expansion: {}",
                serde_json::json!({
                    "phase": phase, "query": query, "expected": expected,
                    "kind": case["kind"], "rank": rank, "results": names,
                    "expected_available": phase == "candidate" || !ADDED.contains(&expected)
                })
            );
            if phase == "baseline" && case["kind"] == "guard" {
                baseline_guards.insert(
                    query.to_owned(),
                    (rank, names.first().map(|name| (*name).to_owned())),
                );
            }
            if phase == "candidate" {
                let guard = case["kind"] == "guard";
                let previous = baseline_guards.get(query);
                // Existing routing misses remain visible in the paired results.
                // For those cases, this expansion must preserve the prior winner.
                let existing_miss = guard && previous.is_some_and(|row| row.0.is_none());
                if rank.is_none() && !existing_miss {
                    failures.push(format!("{query}: expected {expected}, got {names:?}"));
                }
                if existing_miss && names.first().copied() != previous.unwrap().1.as_deref() {
                    failures.push(format!(
                        "Existing routing miss changed winner: {query}: {names:?}"
                    ));
                }
                if guard && names.first().is_some_and(|n| ADDED.contains(n)) {
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
async fn repository_parent_upgrades_preserve_custom_text_and_disabled_state() -> Result<()> {
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
async fn repository_helpers_and_references_are_bundled_and_respect_activation() -> Result<()> {
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
        checked >= 20,
        "Expected the actual helper/reference resources"
    );
    for name in ADDED {
        rt.set_skill_enabled(name, false)?;
    }
    rt.install_resources()?;
    for name in ADDED {
        assert!(rt.skill_load(name).is_err());
    }
    assert!(
        rt.skill_resource("parametric-cad", "scripts/inspect_geometry.py")
            .is_err()
    );
    rt.shutdown().await?;
    Ok(())
}
