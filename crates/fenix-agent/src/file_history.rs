//! Private content-addressed file versions; no remote repository or user Git configuration.
use crate::Runtime;
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    io::Write,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio_util::sync::CancellationToken;
const MAX_FILE: u64 = 32 * 1024 * 1024;
const MAX_STORE: u64 = 512 * 1024 * 1024;
#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct Version {
    pub id: String,
    pub task_id: String,
    workspace: PathBuf,
    path: PathBuf,
    before: Option<String>,
    after: Option<String>,
    permissions: Option<u32>,
    status: String,
    operation: String,
    created_at: String,
}
fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn current(path: &Path) -> Result<Option<Vec<u8>>> {
    match fs::symlink_metadata(path) {
        Ok(meta) => {
            ensure!(
                meta.is_file() && !meta.file_type().is_symlink(),
                "History only supports regular files"
            );
            Ok(Some(fenix_execution::fs::read(path, MAX_FILE)?))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}
impl Runtime {
    fn history_blob(&self, bytes: &[u8]) -> Result<String> {
        let id = hash(bytes);
        let root = self.data_dir.join("file-history");
        fs::create_dir_all(&root)?;
        fs::set_permissions(&root, fs::Permissions::from_mode(0o700))?;
        let path = root.join(&id);
        if path.exists() {
            ensure!(
                hash(&fenix_execution::fs::read(&path, MAX_FILE)?) == id,
                "Corrupt history blob"
            );
            return Ok(id);
        }
        let used: u64 = fs::read_dir(&root)?
            .map(|e| Ok(e?.metadata()?.len()))
            .collect::<Result<Vec<u64>>>()?
            .into_iter()
            .sum();
        ensure!(
            used.saturating_add(bytes.len() as u64) <= MAX_STORE,
            "File history reached 512 MiB; storage maintenance is required before more managed writes"
        );
        let mut file = tempfile::NamedTempFile::new_in(&root)?;
        file.write_all(bytes)?;
        file.as_file().sync_all()?;
        match file.persist_noclobber(&path) {
            Ok(_) => {}
            Err(e) if e.error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(e) => return Err(e.error.into()),
        };
        fs::File::open(root)?.sync_all()?;
        Ok(id)
    }
    fn history_bytes(&self, id: &str) -> Result<Vec<u8>> {
        ensure!(
            id.len() == 64 && id.bytes().all(|c| c.is_ascii_hexdigit()),
            "Invalid blob identity"
        );
        let bytes =
            fenix_execution::fs::read(&self.data_dir.join("file-history").join(id), MAX_FILE)?;
        ensure!(hash(&bytes) == id, "Corrupt history blob");
        Ok(bytes)
    }
    pub(crate) fn history_begin(
        &self,
        task: &str,
        workspace: &Path,
        paths: &[String],
        operation: &str,
    ) -> Result<Vec<Version>> {
        ensure!(paths.len() <= 64, "Too many history paths");
        let mut versions = Vec::new();
        for input in paths {
            let path = self.file_path(workspace, input, true)?;
            let before = current(&path)?;
            let version = Version {
                id: fenix_storage::id(),
                task_id: task.into(),
                workspace: workspace.into(),
                path: path.clone(),
                before: before
                    .as_deref()
                    .map(|b| self.history_blob(b))
                    .transpose()?,
                after: None,
                permissions: fs::symlink_metadata(&path)
                    .ok()
                    .map(|m| m.permissions().mode()),
                status: "pending".into(),
                operation: operation.into(),
                created_at: fenix_storage::now(),
            };
            self.store.put("file_versions", &version.id, &version)?;
            versions.push(version);
        }
        Ok(versions)
    }
    #[cfg(test)]
    pub(crate) fn history_finish(&self, versions: Vec<Version>) -> Result<()> {
        let mut expected = BTreeMap::new();
        for v in &versions {
            expected.insert(
                v.path.clone(),
                (v.before.clone(), current(&v.path)?.as_deref().map(hash)),
            );
        }
        self.history_finish_expected(versions, &expected)
    }
    fn history_finish_expected(
        &self,
        mut versions: Vec<Version>,
        expected: &BTreeMap<PathBuf, (Option<String>, Option<String>)>,
    ) -> Result<()> {
        for version in &mut versions {
            version.after = current(&version.path)?
                .as_deref()
                .map(|b| self.history_blob(b))
                .transpose()?;
            version.status = if version.before == version.after {
                "unchanged"
            } else if expected
                .get(&version.path)
                .is_some_and(|(before, after)| before == &version.before && after == &version.after)
            {
                "complete"
            } else {
                "pending"
            }
            .into();
            self.store.put("file_versions", &version.id, version)?;
        }
        Ok(())
    }
    pub(crate) fn history_finish_receipt(
        &self,
        versions: Vec<Version>,
        result: &Result<Value>,
    ) -> Result<()> {
        let mut expected = BTreeMap::new();
        if let Ok(receipt) = result {
            let hash_field = |v: &Value| v.as_str().map(str::to_owned);
            if let Some(path) = receipt["path"].as_str()
                && receipt["sha256"].is_string()
            {
                expected.insert(
                    PathBuf::from(path),
                    (
                        hash_field(&receipt["before_sha256"]),
                        hash_field(&receipt["sha256"]),
                    ),
                );
            }
            for item in receipt["applied"].as_array().into_iter().flatten() {
                if let (Some(path), Some(destination)) =
                    (item["path"].as_str(), item["destination"].as_str())
                {
                    if path != destination {
                        expected.insert(
                            PathBuf::from(path),
                            (hash_field(&item["before_sha256"]), None),
                        );
                        expected.insert(
                            PathBuf::from(destination),
                            (None, hash_field(&item["sha256"])),
                        );
                    } else {
                        expected.insert(
                            PathBuf::from(path),
                            (
                                hash_field(&item["before_sha256"]),
                                hash_field(&item["sha256"]),
                            ),
                        );
                    }
                }
            }
        }
        self.history_finish_expected(versions, &expected)
    }
    pub(crate) fn history_mutation_paths(&self, name: &str, args: &Value) -> Result<Vec<String>> {
        if name == "file_patch" {
            return crate::coding::patch::patch_paths(args);
        }
        Ok(vec![
            args["path"]
                .as_str()
                .context("File path required")?
                .to_owned(),
        ])
    }
    pub(crate) async fn file_history(
        self: &Arc<Self>,
        task: &str,
        settings: &crate::config::Settings,
        args: &Value,
        cancel: &CancellationToken,
    ) -> Result<Value> {
        let target = args["task_id"].as_str().unwrap_or(task);
        self.authorize_task_read(task, target)?;
        let workspace = self.task_workspace(task, &settings.workspace)?;
        let mut versions: Vec<Version> = self.store.records("file_versions")?;
        versions.retain(|v| v.task_id == target && v.workspace == workspace);
        versions.sort_by(|a, b| a.created_at.cmp(&b.created_at).then(a.id.cmp(&b.id)));
        let action = args["action"].as_str().context("Action required")?;
        if action == "list" {
            let offset = args["offset"].as_u64().unwrap_or(0) as usize;
            return Ok(
                json!({"total":versions.len(),"versions":versions.iter().rev().skip(offset).take(50).collect::<Vec<_>>(),"next_offset":offset+50}),
            );
        }
        if let Some(id) = args["id"].as_str() {
            versions.retain(|v| v.id == id);
        }
        ensure!(
            !versions.is_empty(),
            "No matching versions in this task/workspace"
        );
        if action == "inspect" {
            ensure!(args["id"].is_string(), "inspect requires id");
            let v = &versions[0];
            let preview = |id: &Option<String>| -> Result<Value> {
                Ok(match id {
                    None => Value::Null,
                    Some(id) => {
                        let b = self.history_bytes(id)?;
                        json!({"sha256":id,"bytes":b.len(),"preview":crate::redaction::truncate(&self.redactor.text(&String::from_utf8_lossy(&b)),12000),"binary":std::str::from_utf8(&b).is_err()})
                    }
                })
            };
            return Ok(
                json!({"version":v,"before":preview(&v.before)?,"after":preview(&v.after)?}),
            );
        }
        ensure!(action == "restore", "Unknown history action");
        ensure!(
            self.store.task_binding(task)?.project_id
                == self.store.task_binding(target)?.project_id,
            "Read access to another project cannot authorize restoration"
        );
        ensure!(
            self.store.task(task)?.origin != "subagent",
            "Only the master can restore versions"
        );
        // Each path restores its earliest selected before-image, provided the latest selected after-image still matches.
        let mut grouped: BTreeMap<PathBuf, (Version, Version)> = BTreeMap::new();
        for v in versions {
            ensure!(
                v.status != "pending",
                "Incomplete write: inspect the actual file; automatic restoration is unsafe"
            );
            if v.status == "unchanged" {
                continue;
            }
            grouped
                .entry(v.path.clone())
                .and_modify(|pair| pair.1 = v.clone())
                .or_insert((v.clone(), v));
        }
        ensure!(!grouped.is_empty(), "No changed files to restore");
        let paths: Vec<_> = grouped.keys().cloned().collect();
        for path in &paths {
            let rules = self.pending_repository_rules(
                task,
                &workspace,
                "file_edit",
                &json!({"path":path}),
            )?;
            if !rules.is_empty() {
                return Ok(
                    json!({"executed":false,"repository_instructions":rules,"instruction":"Read the applicable guidance before retrying restoration; no files changed."}),
                );
            }
        }
        let _lease = self.effect_lease_for(task, Some(&paths), cancel).await?;
        let _tracking = self.file_effects.begin(task);
        let runtime = self.clone();
        let task = task.to_owned();
        let cancel = cancel.clone();
        tokio::task::spawn_blocking(move || -> Result<Value> {
            let _lease=_lease; let _tracking=_tracking;
            let mut prepared=Vec::new();
            for (path,(first,last)) in grouped {
                runtime.file_path(&workspace,&path.to_string_lossy(),true)?;
                let now=current(&path)?;
                ensure!(now.as_deref().map(hash)==last.after,"File changed after the selected version: {}",path.display());
                let before=first.before.as_deref().map(|id|runtime.history_bytes(id)).transpose()?;
                if let Some(bytes)=&before && let Ok(text)=std::str::from_utf8(bytes) { runtime.check_instruction_write(&path,text)?; }
                prepared.push((path,now,before,first.permissions));
            }
            let originals:Vec<_>=prepared.iter().map(|(p,..)|p.to_string_lossy().into_owned()).collect();
            let expected=prepared.iter().map(|(path,now,before,_)|(path.clone(),(now.as_deref().map(hash),before.as_deref().map(hash)))).collect::<BTreeMap<_,_>>();
            let snapshots=runtime.history_begin(&task,&workspace,&originals,"restore")?;
            let mut restored=Vec::new();
            let outcome=(||->Result<()> {
                for (path,now,before,permissions) in prepared {
                    ensure!(!cancel.is_cancelled(),"Restoration cancelled; inspect completed paths");
                    runtime.store.check_plan_worker(&task)?;
                    match (&now,&before) {
                        (Some(now),Some(before))=>fenix_execution::fs::replace(&path,now,before)?,
                        (Some(now),None)=>fenix_execution::fs::remove(&path,now)?,
                        (None,Some(before))=>fenix_execution::fs::create_with_permissions(&path,before,permissions.map(fs::Permissions::from_mode))?,
                        _=>{},
                    }
                    restored.push(path);
                } Ok(())
            })();
            runtime.history_finish_expected(snapshots,&expected)?;
            runtime.store.event(&task,"file_history_restore",&json!({"restored":restored,"error":outcome.as_ref().err().map(ToString::to_string)}))?;
            outcome?;
            Ok(json!({"restored":restored,"external_effects_reverted":false}))
        }).await?
    }
}
