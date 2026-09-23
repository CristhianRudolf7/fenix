use super::*;

const PREVIOUS: &[(&str, &str)] = &[
    (
        "google-drive",
        include_str!("../../tests/fixtures/skills/google-workspace-v1/google-drive.md"),
    ),
    (
        "google-docs",
        include_str!("../../tests/fixtures/skills/google-workspace-v1/google-docs.md"),
    ),
    (
        "google-sheets",
        include_str!("../../tests/fixtures/skills/google-workspace-v1/google-sheets.md"),
    ),
    (
        "google-slides",
        include_str!("../../tests/fixtures/skills/google-workspace-v1/google-slides.md"),
    ),
    (
        "email-workflows",
        include_str!("../../tests/fixtures/skills/google-workspace-v1/email-workflows.md"),
    ),
    (
        "calendar-scheduling",
        include_str!("../../tests/fixtures/skills/google-workspace-v1/calendar-scheduling.md"),
    ),
    (
        "meeting-notes",
        include_str!("../../tests/fixtures/skills/google-workspace-v1/meeting-notes.md"),
    ),
    (
        "forms-and-surveys",
        include_str!("../../tests/fixtures/skills/google-workspace-v1/forms-and-surveys.md"),
    ),
    (
        "team-messaging",
        include_str!("../../tests/fixtures/skills/google-workspace-v1/team-messaging.md"),
    ),
    (
        "personal-organization",
        include_str!("../../tests/fixtures/skills/google-workspace-v1/personal-organization.md"),
    ),
];

#[tokio::test]
async fn workspace_skills_upgrade_stock_bodies_and_preserve_customizations() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let rt = Runtime::open(&dir.path().join("data"), Some(&dir.path().join("skills"))).await?;
    for (name, body) in PREVIOUS {
        fs::write(rt.skills_dir.join(name).join("SKILL.md"), body)?;
    }
    rt.set_skill_enabled("google-sheets", false)?;
    rt.install_resources()?;
    for (name, _) in PREVIOUS {
        let expected = DEFAULTS.iter().find(|(n, _)| n == name).unwrap().1;
        assert_eq!(
            fs::read_to_string(rt.skills_dir.join(name).join("SKILL.md"))?,
            expected
        );
    }
    assert_eq!(
        rt.store.get::<bool>("skill_enabled", "google-sheets")?,
        Some(false)
    );
    let custom = format!(
        "{}\nOwner's custom rule: use the finance template.\n",
        PREVIOUS[0].1
    );
    let path = rt.skills_dir.join("google-drive/SKILL.md");
    fs::write(&path, &custom)?;
    let resource = rt
        .skills_dir
        .join("google-workspace-workflows/references/native-operations.md");
    assert!(resource.exists());
    fs::write(&resource, "Owner's custom Workspace reference")?;
    rt.install_resources()?;
    assert_eq!(fs::read_to_string(path)?, custom);
    assert_eq!(
        fs::read_to_string(resource)?,
        "Owner's custom Workspace reference"
    );
    Ok(())
}

#[tokio::test]
async fn workspace_discovery_routes_new_workflows_and_existing_formats() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let rt = Runtime::open(&dir.path().join("data"), Some(&dir.path().join("skills"))).await?;
    for (query, expected) in [
        ("Google Tasks task lists", "google-workspace-workflows"),
        (
            "Google Contacts people contact records",
            "google-workspace-workflows",
        ),
        (
            "preparar reunión con documentos Google Workspace",
            "google-workspace-workflows",
        ),
        ("Edit Google Docs document tabs", "google-docs"),
        ("Actualizar fórmulas Google Sheets", "google-sheets"),
        ("Google Forms respuestas encuesta", "forms-and-surveys"),
        (
            "Resumen transcripciones reunión Google Meet",
            "meeting-notes",
        ),
        ("Reply Gmail thread attachments borrador", "email-workflows"),
    ] {
        let result = rt.search_skills(query, 3)?;
        let names: Vec<_> = result["skills"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v["skill"]["name"].as_str().unwrap())
            .collect();
        assert!(
            names.contains(&expected),
            "{query}: expected {expected}, got {names:?}"
        );
    }
    Ok(())
}
