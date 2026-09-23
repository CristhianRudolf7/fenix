use super::*;

const ADDED: &[&str] = &[
    "automation-audit",
    "notification-triage",
    "llm-cost-optimization",
    "patent-research",
    "inventory-planning",
];

const PREVIOUS: &[(&str, &str)] = &[
    (
        "agent-workflow-diagnosis",
        include_str!("../../tests/fixtures/skills/ecc-v1/agent-workflow-diagnosis.md"),
    ),
    (
        "api-contract-design",
        include_str!("../../tests/fixtures/skills/ecc-v1/api-contract-design.md"),
    ),
    (
        "api-integration",
        include_str!("../../tests/fixtures/skills/ecc-v1/api-integration.md"),
    ),
    (
        "browser-regression-testing",
        include_str!("../../tests/fixtures/skills/ecc-v1/browser-regression-testing.md"),
    ),
    (
        "cache-design",
        include_str!("../../tests/fixtures/skills/ecc-v1/cache-design.md"),
    ),
    (
        "customer-support",
        include_str!("../../tests/fixtures/skills/ecc-v1/customer-support.md"),
    ),
    (
        "data-pipelines",
        include_str!("../../tests/fixtures/skills/ecc-v1/data-pipelines.md"),
    ),
    (
        "experiment-and-benchmark-design",
        include_str!("../../tests/fixtures/skills/ecc-v1/experiment-and-benchmark-design.md"),
    ),
    (
        "memory-and-reminders",
        include_str!("../../tests/fixtures/skills/ecc-v1/memory-and-reminders.md"),
    ),
    (
        "performance-diagnosis",
        include_str!("../../tests/fixtures/skills/ecc-v1/performance-diagnosis.md"),
    ),
    (
        "scheduled-workflows",
        include_str!("../../tests/fixtures/skills/ecc-v1/scheduled-workflows.md"),
    ),
    (
        "skill-authoring",
        include_str!("../../tests/fixtures/skills/ecc-v1/skill-authoring.md"),
    ),
    (
        "team-messaging",
        include_str!("../../tests/fixtures/skills/ecc-v1/team-messaging.md"),
    ),
];

const REFERENCES: &[(&str, &str)] = &[
    (
        "agent-workflow-diagnosis",
        "references/execution-boundaries.md",
    ),
    ("api-contract-design", "references/shared-contracts.md"),
    ("customer-support", "references/shipping-and-returns.md"),
    ("data-pipelines", "references/throughput-and-accounting.md"),
    ("scheduled-workflows", "references/bounded-execution.md"),
    ("skill-authoring", "references/library-maintenance.md"),
    (
        "experiment-and-benchmark-design",
        "references/agent-evaluations.md",
    ),
    ("patent-research", "references/record-handling.md"),
];

#[tokio::test]
async fn ecc_discovery_compares_baseline_and_preserves_neighboring_workflows() -> Result<()> {
    let cases: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("../../../../docs/skills/ecc-cases.json"))?;
    let dir = tempfile::tempdir()?;
    let rt = Runtime::open(&dir.path().join("data"), Some(&dir.path().join("skills"))).await?;
    let mut failures = Vec::new();
    for phase in ["baseline", "candidate"] {
        for name in ADDED {
            rt.set_skill_enabled(name, phase == "candidate")?;
        }
        if phase == "baseline" {
            for (name, body) in PREVIOUS {
                fs::write(rt.skills_dir.join(name).join("SKILL.md"), body)?;
            }
            // Remove new references as well: search indexes body/resource text.
            for (name, resource) in REFERENCES {
                if !ADDED.contains(name) {
                    fs::remove_file(rt.skills_dir.join(name).join(resource))?;
                }
            }
        } else {
            rt.install_resources()?;
        }
        for case in &cases {
            let query = case["query"].as_str().unwrap();
            let expected = case["expected"].as_str().unwrap();
            let result = rt.search_skills(query, 3)?;
            let names: Vec<_> = result["skills"]
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
                "ecc-discovery: {}",
                serde_json::json!({"phase":phase,"query":query,"expected":expected,"rank":rank,"results":names,"kind":case["kind"]})
            );
            if phase == "candidate" {
                if rank.is_none() {
                    failures.push(format!("{query}: expected {expected}, got {names:?}"));
                }
                if case["kind"] == "guard" && names.first().is_some_and(|name| ADDED.contains(name))
                {
                    failures.push(format!(
                        "New skill captured neighboring task: {query}: {names:?}"
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
async fn ecc_updates_preserve_customization_disabled_state_and_resource_boundaries() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let rt = Runtime::open(&dir.path().join("data"), Some(&dir.path().join("skills"))).await?;
    for (name, body) in PREVIOUS {
        fs::write(rt.skills_dir.join(name).join("SKILL.md"), body)?;
        rt.set_skill_enabled(name, false)?;
    }
    rt.install_resources()?;
    for (name, _) in PREVIOUS {
        let expected = DEFAULTS.iter().find(|(n, _)| n == name).unwrap().1;
        assert_eq!(
            fs::read_to_string(rt.skills_dir.join(name).join("SKILL.md"))?,
            expected
        );
        assert!(rt.skill_load(name).is_err());
    }
    for name in ADDED {
        assert!(!rt.skill_load(name)?.is_empty());
        rt.set_skill_enabled(name, false)?;
    }
    for (name, resource) in REFERENCES {
        assert!(rt.skill_resource(name, resource).is_err());
        rt.set_skill_enabled(name, true)?;
        assert!(!rt.skill_resource(name, resource)?.is_empty());
        assert!(
            rt.skill_resource(name, "../automation-audit/SKILL.md")
                .is_err()
        );
        fs::write(
            rt.skills_dir.join(name).join(resource),
            "Owner resource customization",
        )?;
        rt.set_skill_enabled(name, false)?;
    }
    for (name, body) in PREVIOUS {
        fs::write(
            rt.skills_dir.join(name).join("SKILL.md"),
            format!("{body}\nOwner custom instruction.\n"),
        )?;
    }
    rt.install_resources()?;
    for (name, body) in PREVIOUS {
        assert_eq!(
            fs::read_to_string(rt.skills_dir.join(name).join("SKILL.md"))?,
            format!("{body}\nOwner custom instruction.\n")
        );
        assert!(rt.skill_load(name).is_err());
    }
    for name in ADDED {
        assert!(rt.skill_load(name).is_err());
    }
    for (name, resource) in REFERENCES {
        rt.set_skill_enabled(name, true)?;
        assert_eq!(
            rt.skill_resource(name, resource)?,
            "Owner resource customization"
        );
    }
    rt.shutdown().await?;
    Ok(())
}
