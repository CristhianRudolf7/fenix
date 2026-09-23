use anyhow::{Context, Result, bail, ensure};
use std::{
    collections::BTreeMap,
    io::{Cursor, Read, Write},
};
use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};
pub(crate) const W: &str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";
pub(crate) const S: &str = "http://schemas.openxmlformats.org/spreadsheetml/2006/main";
pub(crate) const A: &str = "http://schemas.openxmlformats.org/drawingml/2006/main";
pub(crate) const P: &str = "http://schemas.openxmlformats.org/presentationml/2006/main";
pub(crate) const R: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";
pub(crate) const MAX_BYTES: usize = 64 * 1024 * 1024;
#[derive(Default)]
pub struct Package {
    pub(crate) parts: BTreeMap<String, Vec<u8>>,
}
impl Package {
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        ensure!(bytes.len() <= MAX_BYTES, "Document exceeds 64 MiB");
        let mut archive = ZipArchive::new(Cursor::new(bytes))?;
        ensure!(archive.len() <= 4096, "Too many package parts");
        let mut parts = BTreeMap::new();
        let mut total = 0;
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            if file.is_dir() {
                continue;
            }
            let name = file.name().to_owned();
            ensure!(
                !name.starts_with('/')
                    && !name.contains('\\')
                    && !name
                        .split('/')
                        .any(|s| s == ".." || s == "." || s.is_empty()),
                "Invalid package path"
            );
            ensure!(!parts.contains_key(&name), "Duplicate package part");
            ensure!(file.size() <= MAX_BYTES as u64, "Package part too large");
            let mut data = Vec::new();
            (&mut file)
                .take((MAX_BYTES - total + 1) as u64)
                .read_to_end(&mut data)?;
            total += data.len();
            ensure!(total <= MAX_BYTES, "Expanded package exceeds 64 MiB");
            if name.ends_with(".xml") || name.ends_with(".rels") {
                roxmltree::Document::parse(std::str::from_utf8(&data)?)?;
            }
            parts.insert(name, data);
        }
        let package = Self { parts };
        package.kind()?;
        ensure!(
            package.parts.contains_key("[Content_Types].xml")
                && package.parts.contains_key("_rels/.rels"),
            "Missing package manifest"
        );
        Ok(package)
    }
    pub(crate) fn kind(&self) -> Result<&'static str> {
        let candidates = [
            ("word/document.xml", "docx"),
            ("xl/workbook.xml", "xlsx"),
            ("ppt/presentation.xml", "pptx"),
        ];
        let found: Vec<_> = candidates
            .iter()
            .filter(|(p, _)| self.parts.contains_key(*p))
            .collect();
        ensure!(
            found.len() == 1,
            "Not a supported DOCX, XLSX or PPTX package"
        );
        Ok(found[0].1)
    }
    pub(crate) fn xml(&self, name: &str) -> Result<&str> {
        Ok(std::str::from_utf8(
            self.parts.get(name).context("Package part not found")?,
        )?)
    }
    pub(crate) fn add(&mut self, name: &str, xml: String) {
        self.parts.insert(name.into(), xml.into_bytes());
    }
    pub fn encode(&self) -> Result<Vec<u8>> {
        ensure!(
            self.parts.values().map(Vec::len).sum::<usize>() <= MAX_BYTES,
            "Document exceeds expanded size limit"
        );
        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
        for (name, data) in &self.parts {
            if name.ends_with(".xml") || name.ends_with(".rels") {
                roxmltree::Document::parse(std::str::from_utf8(data)?)?;
            }
            writer.start_file(
                name,
                SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated),
            )?;
            writer.write_all(data)?;
        }
        Ok(writer.finish()?.into_inner())
    }
    pub(crate) fn shared_strings(&self) -> Result<Vec<String>> {
        if !self.parts.contains_key("xl/sharedStrings.xml") {
            return Ok(vec![]);
        }
        let doc = roxmltree::Document::parse(self.xml("xl/sharedStrings.xml")?)?;
        Ok(doc
            .descendants()
            .filter(|n| n.has_tag_name((S, "si")))
            .map(|n| {
                n.descendants()
                    .filter(|n| n.has_tag_name((S, "t")))
                    .filter_map(|n| n.text())
                    .collect()
            })
            .collect())
    }
    pub(crate) fn invalidate_calculation(&mut self) -> Result<()> {
        // Keeping shared/array formula groups coherent requires a calculation engine.
        for (name, data) in &self.parts {
            if name.starts_with("xl/worksheets/") && name.ends_with(".xml") {
                let doc = roxmltree::Document::parse(std::str::from_utf8(data)?)?;
                if doc.descendants().any(|n| {
                    n.has_tag_name((S, "f"))
                        && matches!(n.attribute("t"), Some("shared" | "array" | "dataTable"))
                }) {
                    bail!(
                        "Editing workbooks with shared/array/data-table formulas is not supported"
                    );
                }
            }
        }
        let worksheets: Vec<_> = self
            .parts
            .keys()
            .filter(|name| name.starts_with("xl/worksheets/") && name.ends_with(".xml"))
            .cloned()
            .collect();
        for name in worksheets {
            let xml = self.xml(&name)?.to_owned();
            let doc = roxmltree::Document::parse(&xml)?;
            let mut ranges = Vec::new();
            for cell in doc.descendants().filter(|n| n.has_tag_name((S, "c"))) {
                if cell.children().any(|n| n.has_tag_name((S, "f"))) {
                    ranges.extend(
                        cell.children()
                            .filter(|n| n.has_tag_name((S, "v")))
                            .map(|n| n.range()),
                    );
                }
            }
            ranges.sort_by_key(|r| std::cmp::Reverse(r.start));
            if !ranges.is_empty() {
                let mut after = xml.clone();
                for range in ranges {
                    after.replace_range(range, "");
                }
                self.add(&name, after);
            }
        }
        // Request recalculation on open; never present a cached result as recalculated.
        let xml = self.xml("xl/workbook.xml")?.to_owned();
        let doc = roxmltree::Document::parse(&xml)?;
        let replacement = "<calcPr xmlns=\"http://schemas.openxmlformats.org/spreadsheetml/2006/main\" fullCalcOnLoad=\"1\" forceFullCalc=\"1\" calcMode=\"auto\"/>";
        let after = if let Some(node) = doc.descendants().find(|n| n.has_tag_name((S, "calcPr"))) {
            let mut after = xml.clone();
            after.replace_range(node.range(), replacement);
            after
        } else {
            let end = xml.rfind("</").context("Malformed workbook")?;
            format!("{}{}{}", &xml[..end], replacement, &xml[end..])
        };
        self.add("xl/workbook.xml", after);
        self.parts.remove("xl/calcChain.xml");
        for part in ["xl/_rels/workbook.xml.rels", "[Content_Types].xml"] {
            if !self.parts.contains_key(part) {
                continue;
            }
            let text = self.xml(part)?.to_owned();
            let doc = roxmltree::Document::parse(&text)?;
            let mut ranges: Vec<_> = doc
                .descendants()
                .filter(|n| {
                    n.attribute("Type")
                        .is_some_and(|t| t.ends_with("/calcChain"))
                        || n.attribute("PartName") == Some("/xl/calcChain.xml")
                })
                .map(|n| n.range())
                .collect();
            ranges.sort_by_key(|r| std::cmp::Reverse(r.start));
            let mut after = text.clone();
            for range in ranges {
                after.replace_range(range, "");
            }
            self.add(part, after);
        }
        Ok(())
    }
}
pub(crate) fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
pub(crate) fn rels(items: &[(String, String, String)]) -> String {
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">{}</Relationships>",
        items
            .iter()
            .map(|(id, kind, target)| format!(
                "<Relationship Id=\"{}\" Type=\"{R}/{}\" Target=\"{}\"/>",
                escape(id),
                escape(kind),
                escape(target)
            ))
            .collect::<String>()
    )
}
