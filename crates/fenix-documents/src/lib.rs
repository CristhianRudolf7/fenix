//! Fénix Documents: local OOXML authoring and surgical edits, without office services.
mod author;
mod edit;
mod package;
use anyhow::{Context, Result, bail, ensure};
pub use author::{Cell, DocumentSpec, Paragraph, Sheet, Slide};
pub use package::Package;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

pub fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub fn create(spec: DocumentSpec) -> Result<Vec<u8>> {
    author::build(spec)?.encode()
}

/// Returns paginated text/cells and stable part names; never evaluates embedded content.
pub fn inspect(bytes: &[u8], offset: usize, limit: usize) -> Result<Value> {
    inspect_inner(bytes, offset, limit.clamp(1, 100))
}
/// Complete bounded extraction for persistent indexing, without repeated ZIP/XML parsing.
pub fn extract(bytes: &[u8]) -> Result<Value> {
    let result = inspect_inner(bytes, 0, 20_001)?;
    ensure!(
        result["total_blocks"].as_u64().unwrap_or(0) <= 20_000,
        "Document exceeds 20,000 blocks; split it before indexing"
    );
    Ok(result)
}
fn inspect_inner(bytes: &[u8], offset: usize, limit: usize) -> Result<Value> {
    let package = Package::decode(bytes)?;
    let kind = package.kind()?;
    let mut blocks = Vec::new();
    let shared = package.shared_strings()?;
    let mut part_names: Vec<_> = package
        .parts
        .keys()
        .filter(|p| match kind {
            "docx" => {
                p.starts_with("word/")
                    && p.ends_with(".xml")
                    && !p.contains("/_rels/")
                    && (p.ends_with("document.xml")
                        || p.contains("header")
                        || p.contains("footer")
                        || p.ends_with("footnotes.xml")
                        || p.ends_with("endnotes.xml"))
            }
            "xlsx" => p.starts_with("xl/worksheets/") && p.ends_with(".xml"),
            "pptx" => {
                (p.starts_with("ppt/slides/") || p.starts_with("ppt/notesSlides/"))
                    && p.ends_with(".xml")
                    && !p.contains("/_rels/")
            }
            _ => false,
        })
        .cloned()
        .collect();
    part_names.sort_by_key(|name| {
        let digits: String = name.chars().filter(char::is_ascii_digit).collect();
        (
            name.trim_end_matches(|c: char| {
                c.is_ascii_digit() || c == '.' || c == 'x' || c == 'm' || c == 'l'
            })
            .to_owned(),
            digits.parse::<u64>().unwrap_or(0),
        )
    });
    for name in &part_names {
        let text = package.xml(name)?;
        let doc = roxmltree::Document::parse(text)?;
        if kind == "xlsx" {
            for node in doc
                .descendants()
                .filter(|n| n.has_tag_name((package::S, "c")))
            {
                let raw = node
                    .children()
                    .find(|n| n.has_tag_name((package::S, "v")))
                    .and_then(|n| n.text())
                    .unwrap_or("");
                let value = match node.attribute("t").unwrap_or("") {
                    "s" => shared
                        .get(
                            raw.parse::<usize>()
                                .context("Invalid shared string reference")?,
                        )
                        .context("Missing shared string")?
                        .clone(),
                    "inlineStr" => node
                        .descendants()
                        .filter(|n| n.has_tag_name((package::S, "t")))
                        .filter_map(|n| n.text())
                        .collect::<String>(),
                    _ => raw.to_owned(),
                };
                let formula = node
                    .children()
                    .find(|n| n.has_tag_name((package::S, "f")))
                    .and_then(|n| n.text());
                ensure!(
                    limit <= 100 || blocks.len() < 20_000,
                    "Document exceeds 20,000 blocks; split it before indexing"
                );
                blocks.push(json!({"part":name,"cell":node.attribute("r"),"value":value,"type":node.attribute("t").unwrap_or("n"),"formula":formula,"cached_value_only":formula.is_some()}));
            }
        } else {
            let ns = if kind == "docx" {
                package::W
            } else {
                package::A
            };
            for (index, node) in doc
                .descendants()
                .filter(|n| n.has_tag_name((ns, "p")))
                .enumerate()
            {
                let content: String = node
                    .descendants()
                    .filter(|n| n.has_tag_name((ns, "t")))
                    .filter_map(|n| n.text())
                    .collect();
                ensure!(
                    limit <= 100 || blocks.len() < 20_000,
                    "Document exceeds 20,000 blocks; split it before indexing"
                );
                blocks.push(json!({"part":name,"paragraph":index,"text":content}));
            }
        }
    }
    let limit = limit.clamp(1, 20_001);
    let total = blocks.len();
    let page: Vec<_> = blocks.into_iter().skip(offset).take(limit).collect();
    let next = offset.saturating_add(page.len());
    Ok(
        json!({"format":kind,"sha256":sha256(bytes),"parts":part_names,"blocks":page,"total_blocks":total,"next_offset":(next<total).then_some(next),"limitations":"Text/cell extraction, not a visual rendering. Formulas are stored, not recalculated; cached values may be stale. Embedded code and external links are never executed."}),
    )
}

/// Every edit is conditional on the exact inspected bytes. Parts not targeted are preserved.
pub fn modify(bytes: &[u8], expected_sha256: &str, operation: &Value) -> Result<Vec<u8>> {
    ensure!(
        sha256(bytes) == expected_sha256,
        "Document changed; read it again before editing"
    );
    let mut package = Package::decode(bytes)?;
    let part = operation["part"]
        .as_str()
        .context("part required; obtain it with document_read")?;
    match operation["action"].as_str() {
        Some("replace_text") => {
            ensure!(
                package.kind()? != "xlsx",
                "Use set_cell for spreadsheets; shared strings may affect multiple cells"
            );
            let ns = if package.kind()? == "docx" {
                package::W
            } else {
                package::A
            };
            let expected = operation["expected"]
                .as_str()
                .context("expected text required")?;
            let replacement = operation["replacement"]
                .as_str()
                .context("replacement required")?;
            let after = edit::replace_text(package.xml(part)?, ns, expected, replacement)?;
            package.parts.insert(part.into(), after.into_bytes());
        }
        Some("set_cell") => {
            ensure!(
                package.kind()? == "xlsx"
                    && part.starts_with("xl/worksheets/")
                    && part.ends_with(".xml"),
                "set_cell requires a worksheet part"
            );
            let address = operation["cell"].as_str().context("cell required")?;
            let cell: Cell = serde_json::from_value(operation["value"].clone())?;
            package.invalidate_calculation()?;
            let after = edit::set_cell(package.xml(part)?, address, &cell)?;
            package.parts.insert(part.into(), after.into_bytes());
        }
        _ => bail!("Supported edits: replace_text and set_cell"),
    }
    package.encode()
}

#[cfg(test)]
mod regression_tests {
    use super::*;
    #[test]
    fn edits_preserve_unrelated_binary_parts_and_reject_shared_formula_targets() -> Result<()> {
        let bytes = create(serde_json::from_value(
            json!({"format":"xlsx","sheets":[{"name":"Data","rows":[["Label",12]]}]}),
        )?)?;
        let mut package = Package::decode(&bytes)?;
        package
            .parts
            .insert("xl/media/image1.png".into(), vec![0, 1, 2, 255]);
        let source = package.encode()?;
        let after = modify(
            &source,
            &sha256(&source),
            &json!({"action":"set_cell","part":"xl/worksheets/sheet1.xml","cell":"B1","value":15}),
        )?;
        assert_eq!(
            Package::decode(&after)?.parts["xl/media/image1.png"],
            vec![0, 1, 2, 255]
        );
        assert_eq!(
            Package::decode(&after)?.parts["_rels/.rels"],
            package.parts["_rels/.rels"]
        );
        package.add("xl/worksheets/sheet1.xml",format!("<worksheet xmlns=\"{}\"><sheetData><row r=\"1\"><c r=\"A1\"><f t=\"shared\" si=\"0\" ref=\"A1:A2\">B1</f><v>12</v></c></row></sheetData></worksheet>",package::S));
        let source = package.encode()?;
        assert!(modify(&source,&sha256(&source),&json!({"action":"set_cell","part":"xl/worksheets/sheet1.xml","cell":"A1","value":0})).is_err());
        Ok(())
    }
    #[test]
    fn cell_edits_invalidate_formula_caches_on_other_sheets() -> Result<()> {
        let bytes = create(serde_json::from_value(
            json!({"format":"xlsx","sheets":[{"name":"Data","rows":[[12]]},{"name":"Totals","rows":[[{"formula":"Data!A1"}]]}]}),
        )?)?;
        let mut package = Package::decode(&bytes)?;
        let second = package
            .xml("xl/worksheets/sheet2.xml")?
            .replace("</f>", "</f><v>12</v>");
        package.add("xl/worksheets/sheet2.xml", second);
        let source = package.encode()?;
        let after = modify(
            &source,
            &sha256(&source),
            &json!({"action":"set_cell","part":"xl/worksheets/sheet1.xml","cell":"A1","value":99}),
        )?;
        let package = Package::decode(&after)?;
        assert!(
            !package
                .xml("xl/worksheets/sheet2.xml")?
                .contains("<v>12</v>")
        );
        assert!(
            package
                .xml("xl/workbook.xml")?
                .contains("forceFullCalc=\"1\"")
        );
        Ok(())
    }
}
