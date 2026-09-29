//! Visual results are private, versioned task artifacts, never executable host pages.
pub mod diagram;
use crate::Runtime;
use anyhow::{Context, Result, bail, ensure};
use fenix_storage::{Design, DesignInput};
use serde_json::{Value, json};
use std::{
    io::{Read, Write},
    path::Path,
    sync::OnceLock,
};
const MAX_FILE: usize = 64 * 1024 * 1024;
const KINDS: &[&str] = &[
    "web",
    "diagram",
    "presentation",
    "chart",
    "document",
    "spreadsheet",
    "image",
    "video",
];

pub fn design_templates() -> &'static Vec<Value> {
    static CATALOG: OnceLock<Vec<Value>> = OnceLock::new();
    CATALOG.get_or_init(|| {
        serde_json::from_str(include_str!("../../../resources/designs/catalog.json"))
            .expect("Validated bundled design catalog")
    })
}
pub fn template(id: &str) -> Result<Value> {
    design_templates()
        .iter()
        .find(|t| t["id"] == id)
        .cloned()
        .context("Template not found")
}
pub fn sample_video(id: &str) -> Result<&'static [u8]> {
    match id {
        "motion-intro" => Ok(include_bytes!(
            "../../../resources/designs/media/motion-intro.webm"
        )),
        "data-motion" => Ok(include_bytes!(
            "../../../resources/designs/media/data-motion.webm"
        )),
        _ => bail!("Sample not found"),
    }
}
fn text<'a>(value: &'a Value, name: &str, limit: usize) -> Result<&'a str> {
    let s = value[name]
        .as_str()
        .with_context(|| format!("{name} must be text"))?;
    ensure!(
        !s.trim().is_empty() && s.len() <= limit,
        "Invalid {name} length"
    );
    Ok(s)
}
pub fn validate_design(input: &DesignInput) -> Result<()> {
    ensure!(
        !input.title.trim().is_empty() && input.title.len() <= 180,
        "Title must contain 1–180 bytes"
    );
    ensure!(
        KINDS.contains(&input.kind.as_str()),
        "Unsupported design kind"
    );
    let c = &input.content;
    ensure!(
        c.is_object() && c.to_string().len() <= 1_000_000,
        "Design content must be an object up to 1 MB"
    );
    if let Some(brand) = c.get("brand") {
        for key in ["accent", "background", "text"] {
            let color = brand[key].as_str().context("Brand color required")?;
            ensure!(
                color.len() == 7
                    && color.starts_with('#')
                    && color[1..].bytes().all(|b| b.is_ascii_hexdigit()),
                "Invalid brand color"
            );
        }
        ensure!(
            matches!(
                brand["font"].as_str(),
                Some("sans-serif" | "serif" | "monospace")
            ),
            "Invalid brand font"
        );
    }
    match input.kind.as_str() {
        "web" => {
            text(c, "html", 500_000)?;
        }
        "diagram" => {
            diagram::prepare(&mut c.clone())?;
        }
        "presentation" => {
            let slides = c["slides"].as_array().context("slides required")?;
            ensure!(
                !slides.is_empty() && slides.len() <= 200,
                "Expected 1–200 slides"
            );
            for slide in slides {
                text(slide, "title", 1000)?;
                text(slide, "body", 20_000)?;
            }
        }
        "chart" => {
            ensure!(
                matches!(c["chart_type"].as_str(), Some("bar" | "line" | "pie")),
                "chart_type must be bar, line or pie"
            );
            let labels = c["labels"].as_array().context("labels required")?;
            let values = c["values"].as_array().context("values required")?;
            ensure!(
                !labels.is_empty() && labels.len() <= 500 && labels.len() == values.len(),
                "Aligned labels and values required, up to 500 points"
            );
            ensure!(
                labels
                    .iter()
                    .all(|x| x.as_str().is_some_and(|s| s.len() <= 500))
                    && values
                        .iter()
                        .all(|x| x.as_f64().is_some_and(f64::is_finite)),
                "Invalid chart labels or values"
            );
            if c["chart_type"] == "pie" {
                ensure!(
                    values.iter().all(|x| x.as_f64().unwrap() >= 0.0)
                        && values.iter().any(|x| x.as_f64().unwrap() > 0.0),
                    "Pie values must be nonnegative with a positive total"
                );
            }
        }
        "spreadsheet" => {
            let cols = c["columns"].as_array().context("columns required")?;
            let rows = c["rows"].as_array().context("rows required")?;
            ensure!(
                !cols.is_empty() && cols.len() <= 200 && rows.len() <= 20_000,
                "Table size exceeded"
            );
            ensure!(
                cols.iter().all(|x| x.is_string())
                    && rows
                        .iter()
                        .all(|r| r.as_array().is_some_and(|r| r.len() == cols.len()
                            && r.iter().all(|v| v.is_null()
                                || v.is_string()
                                || v.is_number()
                                || v.is_boolean()))),
                "Rows must align with scalar columns"
            );
        }
        "document" if c.get("asset_id").is_none() => {
            text(c, "text", 500_000)?;
        }
        "image" if c.get("asset_id").is_none() => {
            text(c, "svg", 500_000)?;
        }
        "video" if c.get("sample").is_some() => {
            sample_video(text(c, "sample", 80)?)?;
        }
        _ => {
            text(c, "asset_id", 80)?;
        }
    }
    Ok(())
}
impl Runtime {
    pub fn save_visual(
        &self,
        existing: Option<&str>,
        task: Option<&str>,
        mut input: DesignInput,
    ) -> Result<Design> {
        if input.kind == "diagram" {
            diagram::prepare(&mut input.content)?;
        }
        validate_design(&input)?;
        if let Some(project) = &input.project_id {
            self.store.project(project)?;
        }
        if let Some(id) = &input.template_id {
            template(id)?;
        }
        if let Some(id) = input.content["asset_id"].as_str() {
            let meta: Value = self
                .store
                .get("design_asset", id)?
                .context("Asset not found")?;
            ensure!(
                meta["project_id"] == json!(input.project_id),
                "Asset belongs to a different project"
            );
            let mime = meta["mime"].as_str().unwrap_or("");
            ensure!(
                match input.kind.as_str() {
                    "image" => mime.starts_with("image/"),
                    "video" => mime.starts_with("video/"),
                    "document" => mime == "application/pdf",
                    _ => false,
                },
                "Asset type does not match design kind"
            );
        }
        self.store
            .save_design_with(existing, task, input, |design| self.archive_diagram(design))
    }

    /// The database is authoritative; revision files are durable, portable snapshots.
    /// Files are written before the DB commit. An interrupted save may leave an
    /// uncommitted snapshot, which is never listed as a published revision.
    pub fn archive_diagram(&self, design: &Design) -> Result<()> {
        if design.kind != "diagram" {
            return Ok(());
        }
        let paths = self.diagram_files(design)?;
        let document = serde_json::to_vec_pretty(design)?;
        let source = design.content["source"]
            .as_str()
            .context("Diagram source missing")?;
        snapshot(Path::new(paths["document"].as_str().unwrap()), &document)?;
        snapshot(
            Path::new(paths["mermaid"].as_str().unwrap()),
            source.as_bytes(),
        )?;
        Ok(())
    }

    pub fn diagram_files(&self, design: &Design) -> Result<Value> {
        ensure!(
            design
                .id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-'),
            "Invalid design ID"
        );
        let directory = self.data_dir.join("diagrams").join(&design.id);
        let stem = format!("v{:06}", design.revision);
        Ok(
            json!({"directory":directory,"document":directory.join(format!("{stem}.json")),"mermaid":directory.join(format!("{stem}.mmd"))}),
        )
    }
    pub fn use_design_template(&self, id: &str, project: Option<String>) -> Result<Design> {
        let t = template(id)?;
        self.save_visual(
            None,
            None,
            DesignInput {
                title: t["title"].as_str().unwrap().into(),
                kind: t["kind"].as_str().unwrap().into(),
                content: t["content"].clone(),
                template_id: Some(id.into()),
                project_id: project,
                expected_revision: None,
            },
        )
    }
    pub fn design_brand(&self) -> Result<Value> {
        Ok(self.store.get("design_brand","default")?.unwrap_or(json!({"name":"","accent":"#244a39","background":"#faf9f6","text":"#17211c","font":"sans-serif"})))
    }
    pub fn save_design_brand(&self, value: &Value) -> Result<Value> {
        ensure!(
            value["name"].as_str().is_some_and(|s| s.len() <= 100),
            "Brand name required"
        );
        for key in ["accent", "background", "text"] {
            let s = value[key].as_str().context("Color required")?;
            ensure!(
                s.len() == 7 && s.starts_with('#') && s[1..].bytes().all(|c| c.is_ascii_hexdigit()),
                "Colors must use #RRGGBB"
            );
        }
        ensure!(
            matches!(
                value["font"].as_str(),
                Some("sans-serif" | "serif" | "monospace")
            ),
            "Unsupported font"
        );
        if let Some(id) = value["logo_asset_id"].as_str() {
            let asset: Value = self
                .store
                .get("design_asset", id)?
                .context("Logo asset not found")?;
            ensure!(
                asset["mime"]
                    .as_str()
                    .is_some_and(|s| s.starts_with("image/"))
                    && asset["project_id"].is_null(),
                "Logo must be an image in general scope"
            );
        }
        let clean = json!({"name":value["name"],"accent":value["accent"],"background":value["background"],"text":value["text"],"font":value["font"],"logo_asset_id":value["logo_asset_id"]});
        self.store.put("design_brand", "default", &clean)?;
        Ok(clean)
    }
    pub fn import_visual(
        &self,
        filename: &str,
        bytes: &[u8],
        project: Option<String>,
        task: Option<&str>,
    ) -> Result<Design> {
        ensure!(
            !bytes.is_empty() && bytes.len() <= MAX_FILE,
            "File must contain 1 byte–64 MiB"
        );
        if let Some(p) = &project {
            self.store.project(p)?;
        }
        let name = Path::new(filename)
            .file_name()
            .and_then(|x| x.to_str())
            .context("Filename required")?;
        ensure!(name.len() <= 180, "Filename too long");
        let ext = Path::new(name)
            .extension()
            .and_then(|x| x.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        let mut input = DesignInput {
            title: name.into(),
            kind: String::new(),
            content: json!({}),
            project_id: project.clone(),
            template_id: None,
            expected_revision: None,
        };
        match ext.as_str() {
            "html" | "htm" => {
                input.kind = "web".into();
                input.content = json!({"html":std::str::from_utf8(bytes)?});
            }
            "mmd" | "mermaid" => {
                input.kind = "diagram".into();
                input.content = json!({"source":std::str::from_utf8(bytes)?});
            }
            "txt" | "md" => {
                input.kind = "document".into();
                input.content = json!({"text":std::str::from_utf8(bytes)?});
            }
            "json" => {
                let parsed: DesignInput = serde_json::from_slice(bytes)?;
                input.title = parsed.title;
                input.kind = parsed.kind;
                input.content = parsed.content;
            }
            "csv" => {
                let mut reader = csv::Reader::from_reader(bytes);
                let columns = reader
                    .headers()?
                    .iter()
                    .map(str::to_owned)
                    .collect::<Vec<_>>();
                let mut rows = Vec::new();
                for row in reader.records().take(20_001) {
                    rows.push(row?.iter().map(str::to_owned).collect::<Vec<_>>());
                }
                input.kind = "spreadsheet".into();
                input.content = json!({"columns":columns,"rows":rows,"caption":"Imported CSV; formulas are displayed as text."});
            }
            "docx" | "xlsx" | "pptx" => {
                let data = fenix_documents::inspect(bytes, 0, 20_000)?;
                let blocks = data["blocks"].as_array().context("No document blocks")?;
                let note = "Content preview; original layout, embedded images and formula recalculation are not reproduced.";
                if ext == "xlsx" {
                    input.kind = "spreadsheet".into();
                    input.content = json!({"columns":["Sheet","Cell","Value","Formula"],"rows":blocks.iter().map(|b|json!([b["part"],b["cell"],b["value"],b["formula"]])).collect::<Vec<_>>(),"caption":note});
                } else if ext == "pptx" {
                    let mut groups = std::collections::BTreeMap::<String, Vec<String>>::new();
                    for b in blocks {
                        groups
                            .entry(b["part"].as_str().unwrap_or("Slide").into())
                            .or_default()
                            .push(b["text"].as_str().unwrap_or("").into());
                    }
                    let mut groups = groups
                        .into_iter()
                        .filter(|(k, _)| k.starts_with("ppt/slides/"))
                        .collect::<Vec<_>>();
                    groups.sort_by_key(|(k, _)| {
                        k.chars()
                            .filter(char::is_ascii_digit)
                            .collect::<String>()
                            .parse::<usize>()
                            .unwrap_or(0)
                    });
                    input.kind = "presentation".into();
                    input.content = json!({"slides":groups.into_iter().map(|(k,v)|json!({"title":v.first().filter(|s|!s.is_empty()).unwrap_or(&k),"body":v.join("\n")})).collect::<Vec<_>>(),"caption":note});
                } else {
                    input.kind = "document".into();
                    input.content = json!({"text":blocks.iter().filter_map(|b|b["text"].as_str()).collect::<Vec<_>>().join("\n\n"),"caption":note});
                }
                input.content["extraction"] =
                    json!({"total":data["total_blocks"],"next_offset":data["next_offset"]});
            }
            _ => {
                let mime = match ext.as_str() {
                    "pdf" if bytes.starts_with(b"%PDF-") => "application/pdf",
                    "png" if bytes.starts_with(b"\x89PNG\r\n\x1a\n") => "image/png",
                    "jpg" | "jpeg" if bytes.starts_with(b"\xff\xd8\xff") => "image/jpeg",
                    "webp" if bytes.starts_with(b"RIFF") && bytes.get(8..12) == Some(b"WEBP") => {
                        "image/webp"
                    }
                    "gif" if bytes.starts_with(b"GIF8") => "image/gif",
                    "svg" => {
                        std::str::from_utf8(bytes)?;
                        "image/svg+xml"
                    }
                    "mp4" if bytes.get(4..8) == Some(b"ftyp") => "video/mp4",
                    "webm" if bytes.starts_with(b"\x1a\x45\xdf\xa3") => "video/webm",
                    _ => bail!("Unsupported file format or invalid file signature"),
                };
                let id = fenix_storage::id();
                let dir = self.data_dir.join("design-assets");
                std::fs::create_dir_all(&dir)?;
                ensure!(
                    std::fs::canonicalize(&dir)? == dir,
                    "Asset directory must not be a symlink"
                );
                let mut file = std::fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(dir.join(&id))?;
                use std::os::unix::fs::PermissionsExt;
                file.set_permissions(std::fs::Permissions::from_mode(0o600))?;
                file.write_all(bytes)?;
                file.sync_all()?;
                self.store.put(
                    "design_asset",
                    &id,
                    &json!({"mime":mime,"filename":name,"project_id":project,"size":bytes.len()}),
                )?;
                input.kind = if mime.starts_with("video/") {
                    "video"
                } else if mime.starts_with("image/") {
                    "image"
                } else {
                    "document"
                }
                .into();
                input.content = json!({"asset_id":id,"filename":name});
            }
        }
        self.save_visual(None, task, input)
    }
    pub fn design_asset(&self, id: &str) -> Result<(Value, Vec<u8>)> {
        ensure!(
            id.len() == 36 && id.bytes().all(|c| c.is_ascii_hexdigit() || c == b'-'),
            "Invalid asset ID"
        );
        let meta = self
            .store
            .get("design_asset", id)?
            .context("Asset not found")?;
        let path = self.data_dir.join("design-assets").join(id);
        ensure!(
            std::fs::canonicalize(&path)? == path && std::fs::symlink_metadata(&path)?.is_file(),
            "Invalid asset path"
        );
        let file = fenix_execution::fs::open(&path)?;
        ensure!(file.metadata()?.len() <= MAX_FILE as u64, "Asset too large");
        let mut bytes = Vec::new();
        file.take(MAX_FILE as u64 + 1).read_to_end(&mut bytes)?;
        ensure!(bytes.len() <= MAX_FILE, "Asset too large");
        Ok((meta, bytes))
    }
    pub(crate) fn visual_tool(
        &self,
        task: &str,
        workspace: &Path,
        name: &str,
        args: &Value,
    ) -> Result<Value> {
        let scope = self.store.task_binding(task)?.project_id;
        match name {
            "design_templates" => {
                let query = args["query"].as_str().unwrap_or("").to_lowercase();
                if let Some(id) = args["id"].as_str() {
                    return template(id);
                }
                let rows = design_templates()
                    .iter()
                    .filter(|x| {
                        query.is_empty()
                            || format!(
                                "{} {} {} {}",
                                x["title"], x["title_es"], x["tags"], x["kind"]
                            )
                            .to_lowercase()
                            .contains(&query)
                    })
                    .skip(args["offset"].as_u64().unwrap_or(0) as usize)
                    .take(20)
                    .map(|x| {
                        let mut x = x.clone();
                        let content = x.as_object_mut().unwrap().remove("content");
                        // Palette and layout names help match stated color/style preferences.
                        for key in ["theme", "composition"] {
                            if let Some(value) = content.as_ref().and_then(|c| c.get(key)) {
                                x[key] = value.clone();
                            }
                        }
                        x
                    })
                    .collect::<Vec<_>>();
                let favorites = self
                    .store
                    .records::<Value>("design_favorite")?
                    .into_iter()
                    .filter_map(|f| f["id"].as_str().map(str::to_owned))
                    .collect::<Vec<_>>();
                let brand_saved = self
                    .store
                    .get::<Value>("design_brand", "default")?
                    .is_some();
                Ok(
                    json!({"templates":rows,"brand":self.design_brand()?,"brand_saved":brand_saved,"favorite_template_ids":favorites,"instruction":"Use id to load a selected template. Sample data is illustrative. Favorites and a saved brand are user preferences; an unsaved brand is only a default."}),
                )
            }
            "design_present" => {
                let ids = args["template_ids"]
                    .as_array()
                    .context("template_ids required")?;
                ensure!(!ids.is_empty() && ids.len() <= 8, "Present 1–8 templates");
                let mut options = Vec::new();
                for id in ids {
                    let mut t = template(id.as_str().context("Template ID must be text")?)?;
                    t.as_object_mut().unwrap().remove("content");
                    t["project_id"] = json!(scope);
                    options.push(t);
                }
                let offer = json!({"id":fenix_storage::id(),"task_id":task,"project_id":scope,"templates":options,"created_at":fenix_storage::now()});
                self.store
                    .put("design_offer", offer["id"].as_str().unwrap(), &offer)?;
                Ok(offer)
            }
            "design_list" => Ok(
                json!({"designs":self.store.designs(scope.as_deref(),true,args["offset"].as_u64().unwrap_or(0) as usize,20)?}),
            ),
            "design_read" => {
                let d = self
                    .store
                    .design(text(args, "id", 80)?, args["revision"].as_u64())?;
                ensure!(d.project_id == scope, "Design belongs to another project");
                let mut value = serde_json::to_value(&d)?;
                if d.kind == "diagram" {
                    self.archive_diagram(&d)?;
                    value["files"] = self.diagram_files(&d)?;
                }
                Ok(value)
            }
            "diagram_edit" => {
                let id = text(args, "id", 80)?;
                let d = self.store.design(id, None)?;
                ensure!(
                    d.project_id == scope && d.kind == "diagram",
                    "Diagram belongs to another scope or has another kind"
                );
                let expected = args["expected_revision"]
                    .as_u64()
                    .context("expected_revision required")?;
                ensure!(
                    expected == d.revision,
                    "Diagram changed; read it before editing"
                );
                let content = diagram::edit(&d.content, &args["operations"])?;
                let saved = self.save_visual(
                    Some(id),
                    Some(task),
                    DesignInput {
                        title: args["title"].as_str().unwrap_or(&d.title).into(),
                        kind: "diagram".into(),
                        content,
                        project_id: scope,
                        template_id: d.template_id,
                        expected_revision: Some(expected),
                    },
                )?;
                Ok(
                    json!({"id":saved.id,"revision":saved.revision,"files":self.diagram_files(&saved)?,"visible_in":"Design and chat preview panel"}),
                )
            }
            "design_publish" => {
                let mut input: DesignInput = serde_json::from_value(args["design"].clone())?;
                ensure!(
                    input.project_id.is_none() || input.project_id == scope,
                    "Cannot publish in another project"
                );
                input.project_id = scope;
                let d = self.save_visual(args["id"].as_str(), Some(task), input)?;
                let mut result = json!({"id":d.id,"title":d.title,"kind":d.kind,"revision":d.revision,"project_id":d.project_id,"visible_in":"Design and chat preview panel"});
                if d.kind == "diagram" {
                    result["files"] = self.diagram_files(&d)?;
                }
                Ok(result)
            }
            "design_import" => {
                let path = self.file_path(workspace, text(args, "path", 4096)?, false)?;
                let file = fenix_execution::fs::open(&path)?;
                ensure!(
                    file.metadata()?.is_file() && file.metadata()?.len() <= MAX_FILE as u64,
                    "Expected a file up to 64 MiB"
                );
                let mut bytes = Vec::new();
                file.take(MAX_FILE as u64 + 1).read_to_end(&mut bytes)?;
                let d = self.import_visual(
                    path.file_name()
                        .and_then(|x| x.to_str())
                        .context("Filename required")?,
                    &bytes,
                    scope,
                    Some(task),
                )?;
                Ok(
                    json!({"id":d.id,"title":d.title,"kind":d.kind,"revision":d.revision,"visible_in":"Design and chat preview panel"}),
                )
            }
            _ => bail!("Unknown design tool"),
        }
    }
}

fn snapshot(path: &Path, bytes: &[u8]) -> Result<()> {
    let matches =
        || fenix_execution::fs::read(path, 2_000_000).is_ok_and(|current| current == bytes);
    if matches() {
        return Ok(());
    }
    match fenix_execution::fs::write(path, bytes) {
        // Two readers may both lazily archive the same historical revision.
        Err(_) if matches() => Ok(()),
        result => result,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn diagram_agent_edits_archive_revisions_and_fail_without_advancing() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let rt = Runtime::open(&dir.path().join("state"), Some(&dir.path().join("skills"))).await?;
        let task = rt.store.enqueue("Create a purchase flow", "web", None)?;
        let result = rt.visual_tool(
            &task.id,
            dir.path(),
            "design_publish",
            &json!({"design":{
                "title":"Purchases", "kind":"diagram", "content":{"graph":{"version":1,
                "nodes":[{"id":"a","label":"Request"},{"id":"b","label":"Approved"}],
                "edges":[{"id":"ab","from":"a","to":"b"}]}}
            }}),
        )?;
        let id = result["id"].as_str().unwrap();
        let file = result["files"]["document"].as_str().unwrap();
        let first = std::fs::read(file)?;
        let second = rt.visual_tool(
            &task.id,
            dir.path(),
            "diagram_edit",
            &json!({"id":id,"expected_revision":1,"operations":[
                {"op":"put_node","value":{"id":"b","label":"Manager approved"}}
            ]}),
        )?;
        assert_eq!(second["revision"], 2);
        assert_eq!(std::fs::read(file)?, first);
        let archived: Design = serde_json::from_slice(&std::fs::read(
            second["files"]["document"].as_str().unwrap(),
        )?)?;
        assert_eq!(
            archived.content["graph"]["nodes"][1]["label"],
            "Manager approved"
        );
        assert!(rt.visual_tool(&task.id, dir.path(), "diagram_edit", &json!({"id":id,"expected_revision":1,"operations":[{"op":"direction","value":"TB"}]})).is_err());
        assert!(
            rt.visual_tool(
                &task.id,
                dir.path(),
                "diagram_edit",
                &json!({"id":id,"expected_revision":2,"operations":[{"op":"remove_node","id":"b"}]})
            )
            .is_err()
        );
        // A filesystem error must not publish a new database revision.
        let blocked =
            Path::new(second["files"]["directory"].as_str().unwrap()).join("v000003.json");
        std::os::unix::fs::symlink(dir.path().join("outside"), &blocked)?;
        assert!(rt.visual_tool(&task.id, dir.path(), "diagram_edit", &json!({"id":id,"expected_revision":2,"operations":[{"op":"direction","value":"TB"}]})).is_err());
        assert_eq!(rt.store.design(id, None)?.revision, 2);
        assert!(!dir.path().join("outside").exists());
        let project = rt
            .store
            .save_project(None, &serde_json::from_value(json!({"name":"Private"}))?)?;
        let private = rt.use_design_template("decision-flow", Some(project.id))?;
        assert!(
            rt.visual_tool(
                &task.id,
                dir.path(),
                "diagram_edit",
                &json!({"id":private.id,"expected_revision":1,"operations":[]})
            )
            .is_err()
        );
        rt.shutdown().await?;
        Ok(())
    }
    #[tokio::test]
    async fn catalog_revisions_scope_and_file_imports_are_real() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let rt = Runtime::open(&dir.path().join("data"), Some(&dir.path().join("skills"))).await?;
        let mut ids = std::collections::HashSet::new();
        for t in design_templates() {
            ensure!(ids.insert(t["id"].as_str().unwrap()), "Duplicate template");
            let d = rt.use_design_template(t["id"].as_str().unwrap(), None)?;
            ensure!(d.revision == 1, "Initial revision");
        }
        assert_eq!(ids.len(), 72);
        let task = rt.store.enqueue("Create a visual", "web", None)?;
        let result=rt.visual_tool(&task.id,dir.path(),"design_publish",&json!({"design":{"title":"Tool result","kind":"chart","content":{"chart_type":"bar","labels":["A"],"values":[2]}}}))?;
        assert_eq!(
            rt.store
                .design(result["id"].as_str().unwrap(), None)?
                .task_id
                .as_deref(),
            Some(task.id.as_str())
        );
        let project = rt.store.save_project(
            None,
            &serde_json::from_value(json!({"name":"Other project"}))?,
        )?;
        let private = rt.use_design_template("decision-flow", Some(project.id))?;
        assert!(
            rt.visual_tool(
                &task.id,
                dir.path(),
                "design_read",
                &json!({"id":private.id})
            )
            .is_err()
        );
        assert!(rt.visual_tool(&task.id,dir.path(),"design_publish",&json!({"id":private.id,"design":{"title":"Wrong scope","kind":"document","content":{"text":"No"},"expected_revision":1}})).is_err());
        let offer = rt.visual_tool(
            &task.id,
            dir.path(),
            "design_present",
            &json!({"template_ids":["decision-flow","login-sequence"]}),
        )?;
        assert_eq!(offer["templates"][1]["id"], "login-sequence");
        rt.store
            .put("design_favorite", "pitch-deck", &json!({"id":"pitch-deck"}))?;
        let listed = rt.visual_tool(
            &task.id,
            dir.path(),
            "design_templates",
            &json!({"query":"pitch"}),
        )?;
        assert_eq!(listed["favorite_template_ids"], json!(["pitch-deck"]));
        assert_eq!(listed["brand_saved"], false);
        let pitch = listed["templates"]
            .as_array()
            .unwrap()
            .iter()
            .find(|t| t["id"] == "pitch-deck")
            .unwrap();
        assert_eq!(pitch["theme"], "midnight");
        assert_eq!(pitch["composition"], "venture-stack");
        assert!(pitch.get("content").is_none());

        let d = rt.use_design_template("decision-flow", None)?;
        let mut input = DesignInput {
            title: "Updated diagram".into(),
            kind: "diagram".into(),
            content: json!({"source":"flowchart LR\n A --> B"}),
            project_id: None,
            template_id: None,
            expected_revision: Some(1),
        };
        assert_eq!(
            rt.save_visual(Some(&d.id), None, input.clone())?.revision,
            2
        );
        assert!(rt.save_visual(Some(&d.id), None, input.clone()).is_err());
        assert_eq!(rt.store.design(&d.id, Some(1))?.title, d.title);
        input.project_id = Some("unknown".into());
        assert!(rt.save_visual(None, None, input).is_err());
        let csv = rt.import_visual("quoted.csv", b"Name,Note\nAlex,\"a,b\"\n", None, None)?;
        assert_eq!(csv.content["rows"][0][1], "a,b");
        let spec = fenix_documents::DocumentSpec::Docx {
            paragraphs: vec![fenix_documents::Paragraph {
                text: "Check me".into(),
                heading: 0,
                bold: false,
            }],
            tables: vec![],
        };
        let bytes = fenix_documents::create(spec)?;
        assert!(
            rt.import_visual("sample.docx", &bytes, None, None)?.content["text"]
                .as_str()
                .unwrap()
                .contains("Check me")
        );
        let pdf = rt.import_visual("sample.pdf", b"%PDF-1.4\n%%EOF", None, None)?;
        let id = pdf.content["asset_id"].as_str().unwrap();
        assert_eq!(rt.design_asset(id)?.1, b"%PDF-1.4\n%%EOF");
        assert!(rt.design_asset("../fenix.sqlite3").is_err());
        assert!(
            rt.import_visual("bad.mp4", b"not a video", None, None)
                .is_err()
        );
        let other_dir = tempfile::tempdir()?;
        let other = Runtime::open(other_dir.path(), Some(&other_dir.path().join("skills"))).await?;
        assert!(other.store.design(&d.id, None).is_err());
        assert!(other.design_asset(id).is_err());
        assert!(rt.save_design_brand(&json!({"name":"X","accent":"url(https://evil)","background":"#ffffff","text":"#111111","font":"serif"})).is_err());
        rt.shutdown().await?;
        other.shutdown().await?;
        Ok(())
    }
    #[test]
    fn malformed_chart_and_table_are_rejected() {
        for (kind, content) in [
            (
                "chart",
                json!({"chart_type":"pie","labels":["x"],"values":[-2]}),
            ),
            ("spreadsheet", json!({"columns":["x"],"rows":[[1,2]]})),
            ("web", json!({"source":"wrong"})),
        ] {
            assert!(
                validate_design(&DesignInput {
                    title: "Invalid".into(),
                    kind: kind.into(),
                    content,
                    project_id: None,
                    template_id: None,
                    expected_revision: None
                })
                .is_err()
            );
        }
    }
}
