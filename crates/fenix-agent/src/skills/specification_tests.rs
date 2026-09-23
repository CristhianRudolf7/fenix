use super::*;

const PREVIOUS: &[(&str, &str)] = &[
    (
        "requirements-and-planning",
        include_str!(
            "../../tests/fixtures/skills/spec-consistency-v1/requirements-and-planning.md"
        ),
    ),
    (
        "software-implementation",
        include_str!("../../tests/fixtures/skills/spec-consistency-v1/software-implementation.md"),
    ),
    (
        "completion-verification",
        include_str!("../../tests/fixtures/skills/spec-consistency-v1/completion-verification.md"),
    ),
];

#[tokio::test]
async fn symbol_navigation_skill_upgrades_only_the_unchanged_stock_revision() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let rt = Runtime::open(&dir.path().join("data"), Some(&dir.path().join("skills"))).await?;
    let path = rt.skills_dir.join("software-implementation/SKILL.md");
    let previous = include_str!("../../tests/fixtures/skills/software-symbol-navigation-v1.md");
    fs::write(&path, previous)?;
    rt.set_skill_enabled("software-implementation", false)?;
    rt.install_resources()?;
    let expected = DEFAULTS
        .iter()
        .find(|(name, _)| *name == "software-implementation")
        .unwrap()
        .1;
    assert_eq!(fs::read_to_string(&path)?, expected);
    assert!(rt.skill_load("software-implementation").is_err());
    let customized = format!("{previous}\nOwner's additional coding instruction.\n");
    fs::write(&path, &customized)?;
    rt.install_resources()?;
    assert_eq!(fs::read_to_string(&path)?, customized);
    rt.shutdown().await?;
    Ok(())
}

#[tokio::test]
async fn specification_skills_upgrade_without_overwriting_owner_changes() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let rt = Runtime::open(&dir.path().join("data"), Some(&dir.path().join("skills"))).await?;
    for (name, body) in PREVIOUS {
        fs::write(rt.skills_dir.join(name).join("SKILL.md"), body)?;
    }
    rt.set_skill_enabled("completion-verification", false)?;
    rt.install_resources()?;
    rt.install_resources()?;
    for (name, _) in PREVIOUS {
        let expected = DEFAULTS.iter().find(|(n, _)| n == name).unwrap().1;
        assert_eq!(
            fs::read_to_string(rt.skills_dir.join(name).join("SKILL.md"))?,
            expected
        );
    }
    assert_eq!(
        rt.store
            .get::<bool>("skill_enabled", "completion-verification")?,
        Some(false)
    );
    let resources = [
        "requirements-and-planning/references/specification-consistency.md",
        "completion-verification/references/requirement-reconciliation.md",
    ];
    for relative in resources {
        let expected = SKILL_RESOURCES
            .iter()
            .find(|(p, _)| p == &relative)
            .unwrap()
            .1;
        assert_eq!(fs::read(rt.skills_dir.join(relative))?, expected);
        fs::write(
            rt.skills_dir.join(relative),
            "Owner's reference customization",
        )?;
    }
    for (name, body) in PREVIOUS {
        fs::write(
            rt.skills_dir.join(name).join("SKILL.md"),
            format!("{body}\nOwner's additional project constraint.\n"),
        )?;
    }
    rt.install_resources()?;
    for (name, body) in PREVIOUS {
        assert_eq!(
            fs::read_to_string(rt.skills_dir.join(name).join("SKILL.md"))?,
            format!("{body}\nOwner's additional project constraint.\n")
        );
    }
    for relative in resources {
        assert_eq!(
            fs::read_to_string(rt.skills_dir.join(relative))?,
            "Owner's reference customization"
        );
    }
    Ok(())
}

#[tokio::test]
async fn specification_queries_find_planning_and_delivery_skills() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let rt = Runtime::open(&dir.path().join("data"), Some(&dir.path().join("skills"))).await?;
    for (query, expected) in [
        (
            "Revisar requisitos ambiguos y contradicciones de especificación",
            "requirements-and-planning",
        ),
        (
            "Check specification consistency and conflicting requirements before planning",
            "requirements-and-planning",
        ),
        (
            "Verify delivery specification gaps and stale tests",
            "completion-verification",
        ),
        (
            "Verifica incumplimientos y requisitos pendientes de la entrega",
            "completion-verification",
        ),
        (
            "Implementa un parser CSV por línea de comandos",
            "software-implementation",
        ),
    ] {
        let found = rt.search_skills(query, 3)?;
        assert!(
            found["skills"]
                .as_array()
                .unwrap()
                .iter()
                .any(|s| s["skill"]["name"] == expected),
            "{query}: expected {expected}; got {found}"
        );
    }
    Ok(())
}
