use super::*;

const ADDED: &[&str] = &[
    "developer-experience-review",
    "code-health-review",
    "engineering-retrospective",
];

const PREVIOUS: &[(&str, &str)] = &[
    (
        "architecture-design",
        include_str!("../../tests/fixtures/skills/gstack-v1/architecture-design.md"),
    ),
    (
        "browser-regression-testing",
        include_str!("../../tests/fixtures/skills/gstack-v1/browser-regression-testing.md"),
    ),
    (
        "frontend-design-and-accessibility",
        include_str!("../../tests/fixtures/skills/gstack-v1/frontend-design-and-accessibility.md"),
    ),
    (
        "performance-diagnosis",
        include_str!("../../tests/fixtures/skills/gstack-v1/performance-diagnosis.md"),
    ),
    (
        "release-and-ci",
        include_str!("../../tests/fixtures/skills/gstack-v1/release-and-ci.md"),
    ),
    (
        "requirements-and-planning",
        include_str!("../../tests/fixtures/skills/gstack-v1/requirements-and-planning.md"),
    ),
    (
        "skill-authoring",
        include_str!("../../tests/fixtures/skills/gstack-v1/skill-authoring.md"),
    ),
    (
        "technical-documentation",
        include_str!("../../tests/fixtures/skills/gstack-v1/technical-documentation.md"),
    ),
];

#[tokio::test]
async fn gstack_discovery_compares_original_catalog_and_preserves_neighbor_routing() -> Result<()> {
    let cases: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("../../../../docs/skills/gstack-cases.json"))?;
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
                .map(|v| v["skill"]["name"].as_str().unwrap())
                .collect();
            let rank = names
                .iter()
                .position(|name| *name == expected)
                .map(|i| i + 1);
            println!(
                "gstack-discovery: {}",
                serde_json::json!({"phase":phase,"query":query,"expected":expected,"rank":rank,"results":names,"kind":case["kind"]})
            );
            if phase == "candidate" {
                if rank.is_none() {
                    failures.push(format!("{query}: expected {expected}, got {names:?}"));
                }
                if case["kind"] == "guard" && names.first().is_some_and(|n| ADDED.contains(n)) {
                    failures.push(format!(
                        "New skill took over neighboring task: {query}: {names:?}"
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
async fn gstack_stock_upgrades_preserve_customization_activation_and_reference_content()
-> Result<()> {
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
    for (name, body) in PREVIOUS {
        fs::write(
            rt.skills_dir.join(name).join("SKILL.md"),
            format!("{body}\nOwner custom guidance.\n"),
        )?;
    }
    let resource = "references/release-observation.md";
    rt.set_skill_enabled("release-and-ci", true)?;
    assert!(
        rt.skill_resource("release-and-ci", resource)?
            .contains("# Release observation")
    );
    fs::write(
        rt.skills_dir.join("release-and-ci").join(resource),
        "Owner release policy",
    )?;
    rt.install_resources()?;
    for (name, body) in PREVIOUS {
        assert_eq!(
            fs::read_to_string(rt.skills_dir.join(name).join("SKILL.md"))?,
            format!("{body}\nOwner custom guidance.\n")
        );
    }
    assert_eq!(
        rt.skill_resource("release-and-ci", resource)?,
        "Owner release policy"
    );
    for name in ADDED {
        assert!(!rt.skill_load(name)?.is_empty());
        rt.set_skill_enabled(name, false)?;
        rt.install_resources()?;
        assert!(rt.skill_load(name).is_err());
    }
    rt.shutdown().await?;
    Ok(())
}
