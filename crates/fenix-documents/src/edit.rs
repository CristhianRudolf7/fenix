use crate::{
    author::{Cell, cell_xml, coordinates},
    package::{S, escape},
};
use anyhow::{Context, Result, ensure};
use std::ops::Range;

/// Match decoded paragraph text, including text split over formatting runs.
pub(crate) fn replace_text(
    xml: &str,
    ns: &str,
    expected: &str,
    replacement: &str,
) -> Result<String> {
    ensure!(!expected.is_empty(), "Expected text must not be empty");
    let doc = roxmltree::Document::parse(xml)?;
    let mut matches = Vec::new();
    for paragraph in doc.descendants().filter(|n| n.has_tag_name((ns, "p"))) {
        let nodes: Vec<_> = paragraph
            .descendants()
            .filter(|n| n.has_tag_name((ns, "t")))
            .collect();
        let text: String = nodes.iter().filter_map(|n| n.text()).collect();
        for (start, _) in text.match_indices(expected) {
            matches.push((nodes.clone(), start, start + expected.len()));
        }
    }
    ensure!(
        matches.len() == 1,
        "Expected text must match exactly once within a paragraph in this part; inspect a smaller target or provide more context"
    );
    let (nodes, start, end) = matches.pop().unwrap();
    let mut offset = 0;
    let mut changes = Vec::new();
    let mut inserted = false;
    for node in nodes {
        let text = node.text().unwrap_or("");
        let node_end = offset + text.len();
        if node_end > start && offset < end {
            let from = start.saturating_sub(offset);
            let to = (end - offset).min(text.len());
            let value = format!(
                "{}{}{}",
                &text[..from],
                if inserted { "" } else { replacement },
                &text[to..]
            );
            inserted = true;
            let range = node.range();
            let original = &xml[range.clone()];
            let begin = original.find('>').context("Missing text start")? + 1;
            let finish = original.rfind('<').context("Missing text end")?;
            // Preserve attributes/prefix and add whitespace preservation when absent.
            let mut opening = original[..begin].to_owned();
            if !opening.contains("xml:space=") {
                opening.insert_str(begin - 1, " xml:space=\"preserve\"");
            }
            changes.push((
                range,
                format!("{opening}{}{}", escape(&value), &original[finish..]),
            ));
        }
        offset = node_end;
    }
    apply(xml, changes)
}
fn apply(xml: &str, mut changes: Vec<(Range<usize>, String)>) -> Result<String> {
    changes.sort_by_key(|(range, _)| std::cmp::Reverse(range.start));
    let mut after = xml.to_owned();
    for (range, value) in changes {
        after.replace_range(range, &value);
    }
    roxmltree::Document::parse(&after)?;
    Ok(after)
}
fn append_element(
    xml: &str,
    node: roxmltree::Node<'_, '_>,
    child: &str,
) -> Result<(Range<usize>, String)> {
    let range = node.range();
    let raw = &xml[range.clone()];
    if let Some(opening) = raw.strip_suffix("/>") {
        let name = raw[1..]
            .split(|c: char| c.is_whitespace() || c == '/' || c == '>')
            .next()
            .context("Element name missing")?;
        Ok((range, format!("{opening}>{child}</{name}>")))
    } else {
        let end = range.start + raw.rfind("</").context("Missing closing element")?;
        Ok((end..end, child.into()))
    }
}
pub(crate) fn set_cell(xml: &str, address: &str, value: &Cell) -> Result<String> {
    let (column, row) = coordinates(address)?;
    let doc = roxmltree::Document::parse(xml)?;
    let data = doc
        .descendants()
        .find(|n| n.has_tag_name((S, "sheetData")))
        .context("Missing sheetData")?;
    let existing: Vec<_> = data
        .descendants()
        .filter(|n| n.has_tag_name((S, "c")) && n.attribute("r") == Some(address))
        .collect();
    ensure!(existing.len() <= 1, "Duplicate cell address");
    let mut changes = Vec::new();
    if let Some(cell) = existing.first() {
        changes.push((cell.range(), cell_xml(address, value, cell.attribute("s"))?));
    } else if let Some(target) = data.children().find(|n| {
        n.has_tag_name((S, "row"))
            && n.attribute("r").and_then(|r| r.parse::<usize>().ok()) == Some(row)
    }) {
        let cell = cell_xml(address, value, None)?;
        if let Some(next) = target.children().find(|n| {
            n.has_tag_name((S, "c"))
                && n.attribute("r")
                    .and_then(|r| coordinates(r).ok())
                    .is_some_and(|(c, _)| c > column)
        }) {
            changes.push((next.range().start..next.range().start, cell));
        } else {
            changes.push(append_element(xml, target, &cell)?);
        }
    } else {
        let row_xml = format!(
            "<row xmlns=\"{S}\" r=\"{row}\">{}</row>",
            cell_xml(address, value, None)?
        );
        if let Some(next) = data.children().find(|n| {
            n.has_tag_name((S, "row"))
                && n.attribute("r")
                    .and_then(|r| r.parse::<usize>().ok())
                    .is_some_and(|r| r > row)
        }) {
            changes.push((next.range().start..next.range().start, row_xml));
        } else {
            changes.push(append_element(xml, data, &row_xml)?);
        }
    }
    // Remove stale optional range hints and formula cached values from this worksheet.
    for node in doc
        .descendants()
        .filter(|n| n.has_tag_name((S, "dimension")))
    {
        changes.push((node.range(), String::new()));
    }
    for cell in data
        .descendants()
        .filter(|n| n.has_tag_name((S, "c")) && n.attribute("r") != Some(address))
    {
        if cell.children().any(|n| n.has_tag_name((S, "f"))) {
            for cached in cell.children().filter(|n| n.has_tag_name((S, "v"))) {
                changes.push((cached.range(), String::new()));
            }
        }
    }
    apply(xml, changes)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn replaces_across_runs_and_preserves_other_markup() {
        let xml = format!(
            "<w:document xmlns:w=\"{}\"><w:body><w:p><w:r><w:rPr><w:b/></w:rPr><w:t>Hello &amp; </w:t></w:r><w:r><w:t>world</w:t></w:r></w:p></w:body></w:document>",
            crate::package::W
        );
        let after =
            replace_text(&xml, crate::package::W, "Hello & world", "¡Buenos días!").unwrap();
        assert!(after.contains("<w:b/>"));
        assert!(after.contains("¡Buenos días!"));
        assert!(!after.contains("world"));
        assert!(replace_text(&xml, crate::package::W, "missing", "new").is_err());
    }
    #[test]
    fn inserts_cells_in_sorted_rows_and_preserves_styles() {
        let xml = format!(
            "<worksheet xmlns=\"{S}\"><dimension ref=\"B2:B2\"/><sheetData><row r=\"2\"><c r=\"B2\" s=\"3\"><v>5</v></c></row></sheetData></worksheet>"
        );
        let first = set_cell(&xml, "A1", &Cell::Number(1.)).unwrap();
        let second = set_cell(&first, "A2", &Cell::Text("test".into())).unwrap();
        let third = set_cell(&second, "B2", &Cell::Bool(true)).unwrap();
        assert!(third.find("r=\"A1\"").unwrap() < third.find("r=\"A2\"").unwrap());
        assert!(third.find("r=\"A2\"").unwrap() < third.find("r=\"B2\"").unwrap());
        assert!(third.contains("s=\"3\""));
        assert!(!third.contains("dimension"));
        let empty = format!("<worksheet xmlns=\"{S}\"><sheetData/></worksheet>");
        assert!(
            set_cell(&empty, "C3", &Cell::Number(3.))
                .unwrap()
                .contains("r=\"C3\"")
        );
    }
}
