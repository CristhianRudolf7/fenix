use crate::Runtime;
use anyhow::{Context, Result, ensure};
use fenix_storage::{LibraryChunk, LibraryDocument};
use serde_json::{Value, json};
use std::{path::Path, process::Stdio, time::Duration};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader};

pub fn chunks(doc: &LibraryDocument, blocks: Vec<(String, Value)>) -> Result<Vec<LibraryChunk>> {
    let mut out = vec![];
    for (text, location) in blocks {
        let chars = text.chars().collect::<Vec<_>>();
        let mut at = 0;
        while at < chars.len() {
            let mut end = (at + 1800).min(chars.len());
            if end < chars.len()
                && let Some(pos) = chars[at + 900..end]
                    .iter()
                    .rposition(|c| *c == '\n' || *c == ' ')
            {
                end = at + 900 + pos + 1;
            }
            let text = chars[at..end]
                .iter()
                .collect::<String>()
                .trim_end()
                .to_owned();
            if !text.trim().is_empty() {
                let mut loc = location.clone();
                loc["character_start"] = json!(at);
                loc["character_end"] = json!(end);
                out.push(LibraryChunk {
                    id: fenix_storage::id(),
                    document_id: doc.id.clone(),
                    revision: doc.revision,
                    ordinal: out.len(),
                    hash: fenix_documents::sha256(text.as_bytes()),
                    text,
                    location: loc,
                });
                ensure!(
                    out.len() <= 20000,
                    "Document exceeds 20,000 fragments; split it into smaller documents"
                );
            }
            at = end;
        }
    }
    Ok(out)
}
pub fn text_blocks(bytes: &[u8], format: &str) -> Result<Vec<(String, Value)>> {
    if matches!(format, "docx" | "xlsx" | "pptx") {
        let result = fenix_documents::extract(bytes)?;
        if format == "xlsx" {
            return spreadsheet_blocks(&result);
        }
        let mut blocks = vec![];
        for b in result["blocks"]
            .as_array()
            .context("Missing document blocks")?
        {
            let value = b["text"].as_str().or(b["value"].as_str()).unwrap_or("");
            let text = if b["cell"].is_string() {
                format!(
                    "{} {}: {}{}",
                    b["part"].as_str().unwrap_or(""),
                    b["cell"].as_str().unwrap_or(""),
                    value,
                    if b["cached_value_only"] == true {
                        " (cached formula value; not recalculated)"
                    } else {
                        ""
                    }
                )
            } else {
                value.into()
            };
            let mut location = b.clone();
            location.as_object_mut().unwrap().remove("text");
            location.as_object_mut().unwrap().remove("value");
            blocks.push((text, location));
        }
        return Ok(blocks);
    }
    let text = std::str::from_utf8(bytes)
        .context("Text documents must use UTF-8")?
        .trim_start_matches('\u{feff}');
    ensure!(!text.contains('\0'), "Binary data is not a text document");
    if format == "csv" {
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(false)
            .flexible(true)
            .from_reader(text.as_bytes());
        let mut blocks = vec![];
        for (index, record) in reader.records().enumerate() {
            ensure!(index < 20000, "CSV exceeds 20,000 rows; split the document");
            let row = record.context("Malformed CSV record")?;
            let line = row.position().map(|p| p.line());
            let cells = row
                .iter()
                .enumerate()
                .map(|(i, v)| format!("Column {}: {}", i + 1, v))
                .collect::<Vec<_>>();
            blocks.push((
                format!("Row {}\n{}", index + 1, cells.join("\n")),
                json!({"row":index+1,"line_start":line,"columns":row.len(),"format":"csv"}),
            ));
        }
        return Ok(blocks);
    }
    let mut blocks = vec![];
    let mut section = String::new();
    let mut part = String::new();
    let mut start = 1;
    for (i, line) in text.lines().enumerate() {
        if format == "md" && line.starts_with('#') && !part.is_empty() {
            blocks.push((
                std::mem::take(&mut part),
                json!({"section":section,"line_start":start,"line_end":i}),
            ));
            start = i + 1;
        }
        if format == "md" && line.starts_with('#') {
            section = line.trim_start_matches('#').trim().to_owned();
        }
        part.push_str(line);
        part.push('\n');
        if part.len() > 6000 || (line.trim().is_empty() && !part.trim().is_empty()) {
            blocks.push((
                std::mem::take(&mut part),
                json!({"section":section,"line_start":start,"line_end":i+1}),
            ));
            start = i + 2;
        }
    }
    if !part.trim().is_empty() {
        blocks.push((
            part,
            json!({"section":section,"line_start":start,"line_end":text.lines().count()}),
        ));
    }
    Ok(blocks)
}
/// Keep neighboring spreadsheet cells together without assuming the first row is a header.
fn spreadsheet_blocks(result: &Value) -> Result<Vec<(String, Value)>> {
    let mut rows: std::collections::BTreeMap<(String, u64), Vec<Value>> =
        std::collections::BTreeMap::new();
    for cell in result["blocks"].as_array().context("Missing cells")? {
        let address = cell["cell"].as_str().context("Missing cell address")?;
        let row = address
            .trim_start_matches(|c: char| c.is_ascii_alphabetic() || c == '$')
            .parse::<u64>()
            .context("Invalid cell row")?;
        rows.entry((cell["part"].as_str().unwrap_or("").into(), row))
            .or_default()
            .push(cell.clone());
    }
    Ok(rows.into_iter().map(|((part,row),cells)| {
        let text = cells.iter().map(|c| format!("{}: {}{}", c["cell"].as_str().unwrap_or(""),c["value"].as_str().unwrap_or(""),
            if c["cached_value_only"]==true {" (cached formula value; not recalculated)"} else {""})).collect::<Vec<_>>().join("\n");
        let refs = cells.iter().map(|c| json!({"cell":c["cell"],"formula":c["formula"],"cached_value_only":c["cached_value_only"]})).collect::<Vec<_>>();
        (format!("{part}, row {row}\n{text}"),json!({"part":part,"row":row,"cells":refs}))
    }).collect())
}

impl Runtime {
    pub(super) async fn library_extract(
        &self,
        doc: &LibraryDocument,
    ) -> Result<(Vec<LibraryChunk>, Vec<String>)> {
        let bytes = self.library_original(doc)?;
        if !matches!(doc.format.as_str(), "pdf" | "png" | "jpg" | "jpeg") {
            let format = doc.format.clone();
            let blocks =
                tokio::task::spawn_blocking(move || text_blocks(&bytes, &format)).await??;
            let warnings = if matches!(doc.format.as_str(), "docx" | "xlsx" | "pptx") {
                vec!["Office extraction preserves text/cell references, not visual layout. Formula values are cached, never recalculated; embedded images are not transcribed.".into()]
            } else {
                vec![]
            };
            return Ok((chunks(doc, blocks)?, warnings));
        }
        let settings = self
            .settings()?
            .context("Configure the agent workspace before extracting PDF/images")?;
        let workspace = if let Some(id) = &doc.project_id {
            self.store
                .project(id)?
                .workspace
                .map(std::path::PathBuf::from)
                .unwrap_or(settings.workspace)
        } else {
            settings.workspace
        };
        self.validate_user_workspace(&workspace)?;
        let path = workspace.join(".fenix-documents").join(format!(
            "{}-{}-{}.{}",
            doc.id,
            doc.revision,
            fenix_storage::id(),
            doc.format
        ));
        fenix_execution::fs::create(&path, &bytes)?;
        let result=async {
            let mut start=1;let mut blocks=vec![];let mut warnings=vec![];
            loop {
                ensure!(!self.stop.is_cancelled(),"Extraction interrupted");
                ensure!(self.store.library_document(&doc.id,None)?.revision==doc.revision,"Document superseded during extraction");
                let req=json!({"path":path,"sha256":doc.sha256,"mode":"auto","start_page":start,"page_count":20,"dpi":200});
                let mut command=crate::documents::local_extraction_command(&self.data_dir.join("processes"), &req)?;
                command.current_dir(&workspace).stdout(Stdio::piped()).stderr(Stdio::piped()).kill_on_drop(true);
                let (mut child,_tree)=fenix_process::spawn_program(command)?;
                let stdout=child.stdout.take().context("Missing extractor stdout")?;
                let stderr=child.stderr.take().context("Missing extractor stderr")?;
                let read=async {
                    let stdout=async {
                        let mut reader=BufReader::new(stdout);let mut size=0;let mut events=vec![];
                        loop {let mut line=vec![];let n=(&mut reader).take(65537).read_until(b'\n',&mut line).await?;if n==0{break;}size+=n;ensure!(n<=65536 && size<=12_000_000,"Extraction output exceeds limits");events.push(serde_json::from_slice::<Value>(&line)?);}
                        Ok::<_,anyhow::Error>(events)
                    };
                    let stderr=async {let mut buf=vec![];stderr.take(65537).read_to_end(&mut buf).await?;ensure!(buf.len()<=65536,"Extractor diagnostics exceeded limit");Ok::<_,anyhow::Error>(())};
                    let (events,_)=tokio::try_join!(stdout,stderr)?;
                    let code=child.wait().await?;
                    ensure!(code.success() || code.code()==Some(2),"Document extractor failed; verify local Python dependencies and input");
                    Ok::<_,anyhow::Error>(events)
                };
                let events=tokio::select!{_=self.stop.cancelled()=>anyhow::bail!("Extraction interrupted"),v=tokio::time::timeout(Duration::from_secs(320),read)=>v.context("Extractor timed out")??};
                let mut summary=None;
                for event in events {
                    match event["event"].as_str() {
                        Some("block")=>{let mut loc=event.clone();let text=loc.as_object_mut().unwrap().remove("text").and_then(|v|v.as_str().map(str::to_owned)).unwrap_or_default();blocks.push((text,loc));}
                        Some("page")=>{if event["status"]!="complete" || event["warnings"].as_array().is_some_and(|w|!w.is_empty()){warnings.push(format!("Page {}: {}",event["page"],event["warnings"]));}}
                        Some("page_error")=>warnings.push(format!("Page {} could not be extracted",event["page"])),
                        Some("summary")=>summary=Some(event),_=>{}
                    }
                }
                let summary=summary.context("Extraction did not provide a completion receipt")?;
                ensure!(summary["sha256"]==doc.sha256,"Extraction source mismatch");
                if summary["status"]!="complete"{warnings.push(format!("Incomplete page batch starting at {start}"));}
                match summary["next_page"].as_u64(){Some(next)=>{ensure!(next>start && next<=2000,"Invalid extractor continuation");start=next;},None=>break}
                ensure!(blocks.len()<=40000,"Extraction too large; split the document");
            }
            Ok::<_,anyhow::Error>((chunks(doc,blocks)?,warnings))
        }.await;
        let _ = fenix_execution::fs::remove(Path::new(&path), &bytes);
        result
    }
}
