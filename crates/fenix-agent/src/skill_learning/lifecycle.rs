//! Journaled skill revisions and deterministic maintenance; no approval/model calls.
use super::*;

#[derive(Clone, Serialize, Deserialize)]
struct Revision {
    name: String,
    before: Option<String>,
    after: String,
    previous_version: Option<Value>,
}

impl Runtime {
    pub(super) fn learned_skill_editable(&self, name: &str, content: &str) -> Result<bool> {
        let usage = self
            .store
            .get::<Value>("skill_usage", name)?
            .unwrap_or_default();
        let version = self.store.get::<Value>("learned_skill_versions", name)?;
        Ok(usage["pinned"] != true
            && version.is_some_and(|v| v["sha256"].as_str() == Some(hash(content).as_str()))
            && self.skills()?.iter().any(|s| s.name == name && s.enabled))
    }

    pub(super) async fn activate_learned_skill(&self, job: &mut ReviewJob) -> Result<Value> {
        let _guard = self.library_write.lock().await;
        // Re-read after the lock: two consumers must not activate/revert stale state.
        if let Some(saved) = self.store.get::<ReviewJob>("skill_learning", &job.id)? {
            *job = saved;
        }
        if job.status == "active" {
            return Ok(json!({"status":"active","already_applied":true}));
        }
        ensure!(
            matches!(
                job.status.as_str(),
                "validated" | "applying" | "activation_blocked"
            ),
            "Candidate is not validated"
        );
        ensure!(
            self.store
                .get::<bool>("skill_learning_settings", "enabled")?
                != Some(false),
            "Learning is disabled"
        );
        let c = job.candidate.clone().context("Candidate missing")?;
        self.validate_learned_candidate(&c, &job.evidence)?;
        let usage = self
            .store
            .get::<Value>("skill_usage", &c.name)?
            .unwrap_or_default();
        ensure!(usage["pinned"] != true, "Pinned skill is protected");
        let saved = self
            .store
            .get::<Revision>("learned_skill_revisions", &job.id)?;
        let revision = if let Some(saved) = saved {
            ensure!(
                saved.name == c.name && saved.after == c.content,
                "Revision identity changed"
            );
            saved
        } else {
            let previous_version = self.store.get::<Value>("learned_skill_versions", &c.name)?;
            let before = if let Some(expected) = &c.base_hash {
                let old = self.skill_load(&c.name)?;
                ensure!(&hash(&old) == expected, "Skill changed since review");
                ensure!(
                    self.learned_skill_editable(&c.name, &old)?,
                    "Manual, bundled or pinned skills cannot be automatically rewritten"
                );
                // Keep support files and require existing Markdown resource references to survive.
                let package = self.skill_bundle(&c.name)?;
                let old_metadata = frontmatter(&old)?;
                let new_metadata = frontmatter(&c.content)?;
                for key in ["category", "required_tools"] {
                    ensure!(
                        old_metadata[key] == new_metadata[key],
                        "Candidate changes protected metadata: {key}"
                    );
                }
                for path in package.files.keys().filter(|p| p.as_str() != "SKILL.md") {
                    ensure!(
                        !old.contains(path) || c.content.contains(path),
                        "Candidate drops an existing resource reference: {path}"
                    );
                }
                Some(old)
            } else {
                ensure!(
                    !self.skills_dir.join(&c.name).try_exists()?,
                    "Skill name already exists; do not overwrite it"
                );
                // Exact procedure duplicate detection ignores frontmatter/name changes.
                let body = procedure(&c.content);
                for skill in self.skills()? {
                    if let Ok(bundle) = self.skill_bundle(&skill.name) {
                        ensure!(
                            procedure(&bundle.files["SKILL.md"]) != body,
                            "An existing skill already contains this procedure"
                        );
                    }
                }
                None
            };
            let revision = Revision {
                name: c.name.clone(),
                before,
                after: c.content.clone(),
                previous_version,
            };
            self.store
                .put("learned_skill_revisions", &job.id, &revision)?;
            revision
        };
        job.status = "applying".into();
        self.store.put("skill_learning", &job.id, job)?;
        let path = self.skills_dir.join(&c.name).join("SKILL.md");
        if path.try_exists()? {
            let current = self.skill_bundle(&c.name)?.files["SKILL.md"].clone();
            if current != revision.after {
                ensure!(
                    revision.before.as_ref() == Some(&current),
                    "Current skill diverged; preserve owner changes"
                );
                fenix_execution::fs::replace(&path, current.as_bytes(), revision.after.as_bytes())?;
            }
        } else {
            ensure!(revision.before.is_none(), "Original skill disappeared");
            self.install_skill_bundle(
                &crate::skills::bundle::Bundle {
                    name: c.name.clone(),
                    files: BTreeMap::from([("SKILL.md".into(), c.content.clone())]),
                    executable: BTreeSet::new(),
                },
                &c.name,
            )?;
        }
        self.store.put("learned_skill_versions", &c.name, &json!({"candidate_id":job.id,"sha256":hash(&c.content),"source_task":job.task_id,"activated_at":fenix_storage::now()}))?;
        self.translation_notify.notify_one();
        job.status = "active".into();
        self.store.put("skill_learning", &job.id, job)?;
        self.store.event(
            &job.task_id,
            "skill_learning_activated",
            &json!({"id":job.id,"name":c.name,"sha256":hash(&c.content),"approval_required":false}),
        )?;
        Ok(
            json!({"status":"active","name":c.name,"version":hash(&c.content),"approval_required":false,"original_resources_preserved":true}),
        )
    }

    pub(super) async fn rollback_learned_skill(&self, job: &mut ReviewJob) -> Result<Value> {
        let _guard = self.library_write.lock().await;
        if let Some(saved) = self.store.get::<ReviewJob>("skill_learning", &job.id)? {
            *job = saved;
        }
        if job.status == "rolled_back" {
            return Ok(json!({"status":"rolled_back","already_applied":true}));
        }
        ensure!(
            matches!(
                job.status.as_str(),
                "active" | "rolling_back" | "rollback_blocked"
            ),
            "Only an active revision can be restored"
        );
        let revision = self
            .store
            .get::<Revision>("learned_skill_revisions", &job.id)?
            .context("No recoverable revision")?;
        let current = self.skill_bundle(&revision.name)?.files["SKILL.md"].clone();
        let version = self
            .store
            .get::<Value>("learned_skill_versions", &revision.name)?;
        if job.status == "active" {
            ensure!(
                version
                    .as_ref()
                    .is_some_and(|v| v["candidate_id"] == job.id),
                "A newer revision is active; restore it first"
            );
            ensure!(
                current == revision.after,
                "Skill changed after activation; preserve owner edits"
            );
        } else {
            ensure!(
                current == revision.after || revision.before.as_ref() == Some(&current),
                "Skill changed during restoration"
            );
        }
        job.status = "rolling_back".into();
        self.store.put("skill_learning", &job.id, job)?;
        if let Some(before) = &revision.before {
            if current != *before {
                fenix_execution::fs::replace(
                    &self.skills_dir.join(&revision.name).join("SKILL.md"),
                    current.as_bytes(),
                    before.as_bytes(),
                )?;
            }
            if let Some(previous) = &revision.previous_version {
                self.store
                    .put("learned_skill_versions", &revision.name, previous)?;
            } else {
                self.store
                    .delete("learned_skill_versions", &revision.name)?;
            }
        } else {
            // A newly learned skill is disabled, never deleted; resources stay recoverable.
            self.set_skill_enabled(&revision.name, false)?;
        }
        job.status = "rolled_back".into();
        self.store.put("skill_learning", &job.id, job)?;
        self.store.event(
            &job.task_id,
            "skill_learning_rolled_back",
            &json!({"id":job.id,"name":revision.name}),
        )?;
        self.translation_notify.notify_one();
        Ok(json!({"status":"rolled_back","name":revision.name,"deleted":false}))
    }

    pub(super) async fn maintain_learned_skills_if_due(&self) -> Result<()> {
        let last = self
            .store
            .get::<String>("skill_learning_settings", "last_maintenance")?;
        if last
            .as_deref()
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .is_some_and(|t| (chrono::Utc::now() - t.with_timezone(&chrono::Utc)).num_hours() < 24)
        {
            return Ok(());
        }
        self.learned_skill_maintenance(true).await?;
        self.store.put(
            "skill_learning_settings",
            "last_maintenance",
            &fenix_storage::now(),
        )
    }

    pub(super) async fn learned_skill_maintenance(&self, apply: bool) -> Result<Vec<Value>> {
        let _guard = self.library_write.lock().await;
        // Include paused schedules and stored workflows: future work must retain its procedures.
        let mut references = self.store.records::<Value>("schedules")?;
        references.extend(self.store.records::<Value>("workflows")?);
        let reference_text = serde_json::to_string(&references)?;
        let mut entries = Vec::new();
        for skill in self.skills()? {
            let Some(version) = self
                .store
                .get::<Value>("learned_skill_versions", &skill.name)?
            else {
                continue;
            };
            let usage = self
                .store
                .get::<Value>("skill_usage", &skill.name)?
                .unwrap_or(json!({"uses":0,"pinned":false}));
            let date = [
                usage["last_used_at"].as_str(),
                version["activated_at"].as_str(),
                usage["unarchived_at"].as_str(),
            ]
            .into_iter()
            .flatten()
            .filter_map(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .max();
            let idle_days = date
                .map(|t| {
                    (chrono::Utc::now() - t.with_timezone(&chrono::Utc))
                        .num_days()
                        .max(0)
                })
                .unwrap_or(0);
            let unchanged = self.skill_bundle(&skill.name).ok().is_some_and(|b| {
                version["sha256"].as_str() == Some(hash(&b.files["SKILL.md"]).as_str())
            });
            let protected =
                usage["pinned"] == true || reference_text.contains(&skill.name) || !unchanged;
            let archive = skill.enabled && idle_days >= 90 && !protected;
            if apply && archive {
                self.set_skill_enabled(&skill.name, false)?;
                self.store.put("skill_learning_archives", &skill.name, &json!({"name":skill.name,"at":fenix_storage::now(),"reason":"unused_90_days","version":version}))?;
            }
            entries.push(json!({"name":skill.name,"enabled":skill.enabled && !(apply && archive),"usage":usage,"idle_days":idle_days,"protected":protected,"state":if !skill.enabled || apply && archive {"archived"} else if idle_days>=30&&!protected {"stale"} else {"active"},"archive_candidate":archive,"version":version}));
        }
        Ok(entries)
    }
}
fn procedure(content: &str) -> String {
    content
        .strip_prefix("---\n")
        .and_then(|s| s.split_once("\n---").map(|(_, body)| body))
        .unwrap_or(content)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn frontmatter(content: &str) -> Result<serde_yaml_ng::Value> {
    let yaml = content
        .strip_prefix("---\n")
        .and_then(|s| s.split_once("\n---"))
        .context("Skill requires YAML frontmatter")?
        .0;
    Ok(serde_yaml_ng::from_str(yaml)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    async fn fixture() -> Result<(tempfile::TempDir, Arc<Runtime>, ReviewJob)> {
        let dir = tempfile::tempdir()?;
        let rt = Runtime::open(&dir.path().join("state"), Some(&dir.path().join("skills"))).await?;
        let task = rt.store.enqueue_bound(
            "Inspect delivery",
            "web",
            None,
            &fenix_storage::TaskBinding {
                bound: true,
                ..Default::default()
            },
        )?;
        let job = ReviewJob { id:task.id.clone(), task_id:task.id, status:"validated".into(), attempts:1,
            evidence:vec![json!({"call_id":"one"}),json!({"call_id":"two"})], skills:vec![], error:None,
            candidate:Some(Candidate { name:"learned-delivery".into(),content:"---\nname: learned-delivery\ndescription: Inspect delivery artifacts\nversion: 1.0.0\ncategory: engineering\n---\n## When to use\nInspect a delivery.\n## Procedure\nRead the delivery manifest.\n## Validation\nVerify listed artifacts.\n## Limitations\nReport missing evidence.".into(), rationale:"Reusable procedure".into(),evidence_call_ids:vec!["one".into(),"two".into()],examples:vec![Example { request:"Inspect delivery".into(),expected_behavior:"Check manifest".into() },Example { request:"Missing artifact".into(),expected_behavior:"Report absence".into() }],base_hash:None }) };
        rt.store.put("skill_learning", &job.id, &job)?;
        Ok((dir, rt, job))
    }
    fn revision(rt: &Runtime, first: &ReviewJob) -> Result<ReviewJob> {
        let task = rt.store.enqueue_bound(
            "Improve delivery validation",
            "web",
            None,
            &fenix_storage::TaskBinding {
                bound: true,
                ..Default::default()
            },
        )?;
        let mut job = first.clone();
        job.id = task.id.clone();
        job.task_id = task.id;
        job.status = "validated".into();
        let c = job.candidate.as_mut().unwrap();
        c.base_hash = Some(hash(&c.content));
        c.content
            .push_str("\nCheck artifact hashes when the manifest supplies them.");
        rt.store.put("skill_learning", &job.id, &job)?;
        Ok(job)
    }
    #[tokio::test]
    async fn automatic_activation_is_idempotent_and_rollback_preserves_resources() -> Result<()> {
        let (_dir, rt, mut first) = fixture().await?;
        rt.activate_learned_skill(&mut first).await?;
        let name = first.candidate.as_ref().unwrap().name.clone();
        std::fs::write(
            rt.skills_dir.join(&name).join("reference.md"),
            "Keep this resource",
        )?;
        let mut next = revision(&rt, &first)?;
        rt.activate_learned_skill(&mut next).await?;
        assert_eq!(
            rt.activate_learned_skill(&mut next).await?["already_applied"],
            true
        );
        assert!(rt.pending.lock().await.is_empty());
        assert!(rt.rollback_learned_skill(&mut first).await.is_err()); // Newer revision must be undone first.
        rt.rollback_learned_skill(&mut next).await?;
        assert_eq!(
            rt.skill_load(&name)?,
            first.candidate.as_ref().unwrap().content
        );
        assert_eq!(
            std::fs::read_to_string(rt.skills_dir.join(&name).join("reference.md"))?,
            "Keep this resource"
        );
        rt.rollback_learned_skill(&mut first).await?;
        assert!(
            !rt.skills()?
                .iter()
                .find(|s| s.name == name)
                .unwrap()
                .enabled
        );
        assert!(rt.skills_dir.join(&name).join("SKILL.md").exists());
        rt.skill_learning(
            &first.task_id,
            &json!({"action":"unarchive","name":name}),
            &CancellationToken::new(),
        )
        .await?;
        assert!(rt.skill_load(&name).is_ok());
        rt.shutdown().await?;
        Ok(())
    }
    #[tokio::test]
    async fn pinned_manual_and_changed_skills_are_not_rewritten() -> Result<()> {
        let (_dir, rt, mut first) = fixture().await?;
        rt.activate_learned_skill(&mut first).await?;
        let name = first.candidate.as_ref().unwrap().name.clone();
        let mut next = revision(&rt, &first)?;
        rt.store
            .put("skill_usage", &name, &json!({"pinned":true}))?;
        assert!(rt.activate_learned_skill(&mut next).await.is_err());
        rt.store
            .put("skill_usage", &name, &json!({"pinned":false}))?;
        let path = rt.skills_dir.join(&name).join("SKILL.md");
        let owner = format!(
            "{}\nOwner addition",
            first.candidate.as_ref().unwrap().content
        );
        std::fs::write(&path, &owner)?;
        assert!(rt.activate_learned_skill(&mut next).await.is_err());
        assert!(rt.rollback_learned_skill(&mut first).await.is_err());
        assert_eq!(std::fs::read_to_string(path)?, owner);
        assert!(!rt.learned_skill_editable("local-file-work", &rt.skill_load("local-file-work")?)?);
        rt.shutdown().await?;
        Ok(())
    }
    #[tokio::test]
    async fn updates_preserve_capability_metadata_and_resource_links() -> Result<()> {
        let (_dir, rt, mut first) = fixture().await?;
        let c = first.candidate.as_mut().unwrap();
        c.content = c.content.replace(
            "category: engineering",
            "category: engineering\nrequired_tools: [file_read]",
        );
        c.content.push_str("\nSee [Reference](reference.md).\n");
        rt.store.put("skill_learning", &first.id, &first)?;
        rt.activate_learned_skill(&mut first).await?;
        let name = first.candidate.as_ref().unwrap().name.clone();
        std::fs::write(
            rt.skills_dir.join(&name).join("reference.md"),
            "Required reference",
        )?;
        let mut next = revision(&rt, &first)?;
        let reviewed = next.candidate.as_ref().unwrap().content.clone();
        next.candidate.as_mut().unwrap().content = reviewed.replace(
            "required_tools: [file_read]",
            "required_tools: [command_run]",
        );
        rt.store.put("skill_learning", &next.id, &next)?;
        assert!(rt.activate_learned_skill(&mut next).await.is_err());
        next.candidate.as_mut().unwrap().content =
            reviewed.replace("See [Reference](reference.md).", "Reference omitted.");
        rt.store.put("skill_learning", &next.id, &next)?;
        assert!(rt.activate_learned_skill(&mut next).await.is_err());
        assert_eq!(rt.skill_load(&name)?, first.candidate.unwrap().content);
        rt.shutdown().await?;
        Ok(())
    }
    #[tokio::test]
    async fn interrupted_install_finishes_from_durable_revision_without_a_model() -> Result<()> {
        let (_dir, rt, mut job) = fixture().await?;
        let c = job.candidate.clone().unwrap();
        rt.store.put(
            "learned_skill_revisions",
            &job.id,
            &Revision {
                name: c.name.clone(),
                before: None,
                after: c.content.clone(),
                previous_version: None,
            },
        )?;
        job.status = "applying".into();
        rt.store.put("skill_learning", &job.id, &job)?;
        rt.install_skill_bundle(
            &crate::skills::bundle::Bundle {
                name: c.name.clone(),
                files: BTreeMap::from([("SKILL.md".into(), c.content.clone())]),
                executable: BTreeSet::new(),
            },
            &c.name,
        )?;
        rt.activate_learned_skill(&mut job).await?;
        assert_eq!(job.status, "active");
        assert_eq!(rt.skill_load(&c.name)?, c.content);
        rt.shutdown().await?;
        Ok(())
    }
    #[tokio::test]
    async fn maintenance_protects_scheduled_pinned_and_manual_changes() -> Result<()> {
        let (_dir, rt, mut job) = fixture().await?;
        rt.activate_learned_skill(&mut job).await?;
        let name = job.candidate.as_ref().unwrap().name.clone();
        let mut version = rt
            .store
            .get::<Value>("learned_skill_versions", &name)?
            .unwrap();
        version["activated_at"] =
            json!((chrono::Utc::now() - chrono::Duration::days(100)).to_rfc3339());
        rt.store.put("learned_skill_versions", &name, &version)?;
        rt.store.put(
            "schedules",
            "paused",
            &json!({"enabled":false,"prompt":format!("Use {name}")}),
        )?;
        rt.learned_skill_maintenance(true).await?;
        assert!(rt.skill_load(&name).is_ok());
        rt.store.delete("schedules", "paused")?;
        rt.store
            .put("skill_usage", &name, &json!({"pinned":true}))?;
        rt.learned_skill_maintenance(true).await?;
        assert!(rt.skill_load(&name).is_ok());
        rt.store
            .put("skill_usage", &name, &json!({"pinned":false}))?;
        rt.learned_skill_maintenance(true).await?;
        assert!(rt.skill_load(&name).is_err());
        assert!(rt.skills_dir.join(&name).exists());
        rt.skill_learning(
            &job.task_id,
            &json!({"action":"unarchive","name":name}),
            &CancellationToken::new(),
        )
        .await?;
        rt.learned_skill_maintenance(true).await?;
        assert!(rt.skill_load(&name).is_ok());
        rt.shutdown().await?;
        Ok(())
    }
    #[tokio::test]
    async fn rejects_unvalidated_and_duplicate_procedures_without_approval() -> Result<()> {
        let (_dir, rt, mut job) = fixture().await?;
        job.status = "rejected".into();
        rt.store.put("skill_learning", &job.id, &job)?;
        assert!(rt.activate_learned_skill(&mut job).await.is_err());
        job.status = "validated".into();
        rt.store.put("skill_learning", &job.id, &job)?;
        rt.activate_learned_skill(&mut job).await?;
        let mut duplicate = revision(&rt, &job)?;
        let c = duplicate.candidate.as_mut().unwrap();
        c.name = "learned-delivery-copy".into();
        c.base_hash = None;
        c.content = job
            .candidate
            .as_ref()
            .unwrap()
            .content
            .replace("name: learned-delivery\n", "name: learned-delivery-copy\n");
        rt.store.put("skill_learning", &duplicate.id, &duplicate)?;
        assert!(rt.activate_learned_skill(&mut duplicate).await.is_err());
        assert!(!rt.skills_dir.join("learned-delivery-copy").exists());
        assert!(rt.pending.lock().await.is_empty());
        rt.shutdown().await?;
        Ok(())
    }
}
