use crate::package::{A, P, Package, R, S, W, escape as x, rels};
use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Serialize, Deserialize)]
#[serde(tag = "format", rename_all = "lowercase", deny_unknown_fields)]
pub enum DocumentSpec {
    Docx {
        paragraphs: Vec<Paragraph>,
        #[serde(default)]
        tables: Vec<Vec<Vec<String>>>,
    },
    Xlsx {
        sheets: Vec<Sheet>,
    },
    Pptx {
        slides: Vec<Slide>,
    },
}
impl DocumentSpec {
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Docx { .. } => "docx",
            Self::Xlsx { .. } => "xlsx",
            Self::Pptx { .. } => "pptx",
        }
    }
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Paragraph {
    pub text: String,
    #[serde(default)]
    pub heading: u8,
    #[serde(default)]
    pub bold: bool,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sheet {
    pub name: String,
    pub rows: Vec<Vec<Cell>>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Slide {
    pub title: String,
    #[serde(default)]
    pub bullets: Vec<String>,
}
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum Cell {
    Empty,
    Bool(bool),
    Number(f64),
    Text(String),
    Formula { formula: String },
}
pub(crate) fn address(column: usize, row: usize) -> String {
    let mut column = column;
    let mut letters = String::new();
    while column > 0 {
        column -= 1;
        letters.insert(0, (b'A' + (column % 26) as u8) as char);
        column /= 26;
    }
    format!("{letters}{row}")
}
pub(crate) fn coordinates(address: &str) -> Result<(usize, usize)> {
    let split = address
        .find(|c: char| !c.is_ascii_uppercase())
        .unwrap_or(address.len());
    let (letters, numbers) = address.split_at(split);
    ensure!(
        !letters.is_empty()
            && letters.len() <= 3
            && !numbers.is_empty()
            && !numbers.starts_with('0'),
        "Invalid A1 cell address"
    );
    let column = letters
        .bytes()
        .fold(0usize, |n, c| n * 26 + usize::from(c - b'A' + 1));
    let row: usize = numbers.parse()?;
    ensure!(
        (1..=16384).contains(&column) && (1..=1048576).contains(&row),
        "Cell outside worksheet limits"
    );
    Ok((column, row))
}
pub(crate) fn cell_xml(address: &str, cell: &Cell, style: Option<&str>) -> Result<String> {
    coordinates(address)?;
    let style = style
        .map(|s| format!(" s=\"{}\"", x(s)))
        .unwrap_or_default();
    let content = match cell {
        Cell::Empty => format!("<c xmlns=\"{S}\" r=\"{address}\"{style}/>"),
        Cell::Bool(value) => format!(
            "<c xmlns=\"{S}\" r=\"{address}\"{style} t=\"b\"><v>{}</v></c>",
            u8::from(*value)
        ),
        Cell::Number(value) => {
            ensure!(value.is_finite(), "Cell number must be finite");
            format!("<c xmlns=\"{S}\" r=\"{address}\"{style}><v>{value}</v></c>")
        }
        Cell::Text(value) => {
            ensure!(
                value.chars().count() <= 32767,
                "Cell text exceeds 32767 characters"
            );
            format!(
                "<c xmlns=\"{S}\" r=\"{address}\"{style} t=\"inlineStr\"><is><t xml:space=\"preserve\">{}</t></is></c>",
                x(value)
            )
        }
        Cell::Formula { formula } => {
            let formula = formula.strip_prefix('=').unwrap_or(formula);
            ensure!(!formula.is_empty(), "Empty formula");
            format!(
                "<c xmlns=\"{S}\" r=\"{address}\"{style}><f>{}</f></c>",
                x(formula)
            )
        }
    };
    Ok(content)
}
fn paragraph(p: &Paragraph) -> Result<String> {
    ensure!(p.heading <= 6, "Heading must be 0 through 6");
    let props = if p.heading > 0 {
        format!("<w:pPr><w:pStyle w:val=\"Heading{}\"/></w:pPr>", p.heading)
    } else {
        String::new()
    };
    let bold = if p.bold { "<w:rPr><w:b/></w:rPr>" } else { "" };
    let runs = p
        .text
        .split('\n')
        .map(|line| {
            format!(
                "<w:r>{bold}<w:t xml:space=\"preserve\">{}</w:t></w:r>",
                x(line)
            )
        })
        .collect::<Vec<_>>()
        .join("<w:r><w:br/></w:r>");
    Ok(format!("<w:p>{props}{runs}</w:p>"))
}
pub(crate) fn build(spec: DocumentSpec) -> Result<Package> {
    let mut package = Package::default();
    let mut types: Vec<(String, String)> = Vec::new();
    let main;
    match spec {
        DocumentSpec::Docx { paragraphs, tables } => {
            main = "word/document.xml";
            let mut body = String::new();
            for p in paragraphs {
                body.push_str(&paragraph(&p)?);
            }
            for table in tables {
                ensure!(
                    !table.is_empty() && table.iter().all(|row| !row.is_empty()),
                    "Tables need rows and cells"
                );
                body.push_str("<w:tbl><w:tblPr><w:tblW w:w=\"0\" w:type=\"auto\"/><w:tblBorders><w:top w:val=\"single\"/><w:left w:val=\"single\"/><w:bottom w:val=\"single\"/><w:right w:val=\"single\"/><w:insideH w:val=\"single\"/><w:insideV w:val=\"single\"/></w:tblBorders></w:tblPr>");
                let columns = table[0].len();
                ensure!(
                    table.iter().all(|row| row.len() == columns),
                    "Table rows must have the same number of columns"
                );
                body.push_str("<w:tblGrid>");
                for _ in 0..columns {
                    body.push_str(&format!("<w:gridCol w:w=\"{}\"/>", 9026 / columns));
                }
                body.push_str("</w:tblGrid>");
                for row in table {
                    body.push_str("<w:tr>");
                    for text in row {
                        body.push_str("<w:tc>");
                        body.push_str(&paragraph(&Paragraph {
                            text,
                            heading: 0,
                            bold: false,
                        })?);
                        body.push_str("</w:tc>");
                    }
                    body.push_str("</w:tr>");
                }
                body.push_str("</w:tbl>");
            }
            package.add(main,format!("<w:document xmlns:w=\"{W}\"><w:body>{body}<w:sectPr><w:pgSz w:w=\"11906\" w:h=\"16838\"/><w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\" w:header=\"720\" w:footer=\"720\" w:gutter=\"0\"/></w:sectPr></w:body></w:document>"));
            let headings: String = (1..=6).map(|n|format!("<w:style w:type=\"paragraph\" w:styleId=\"Heading{n}\"><w:name w:val=\"heading {n}\"/><w:basedOn w:val=\"Normal\"/><w:pPr><w:keepNext/><w:outlineLvl w:val=\"{}\"/></w:pPr><w:rPr><w:b/><w:sz w:val=\"{}\"/></w:rPr></w:style>",n-1,36-2*n)).collect();
            package.add("word/styles.xml",format!("<w:styles xmlns:w=\"{W}\"><w:style w:type=\"paragraph\" w:default=\"1\" w:styleId=\"Normal\"><w:name w:val=\"Normal\"/><w:rPr><w:rFonts w:ascii=\"Calibri\" w:hAnsi=\"Calibri\"/><w:sz w:val=\"22\"/></w:rPr></w:style>{headings}</w:styles>"));
            package.add(
                "word/_rels/document.xml.rels",
                rels(&[("rId1".into(), "styles".into(), "styles.xml".into())]),
            );
            types.push((main.into(), "wordprocessingml.document.main".into()));
            types.push(("word/styles.xml".into(), "wordprocessingml.styles".into()));
        }
        DocumentSpec::Xlsx { sheets } => {
            main = "xl/workbook.xml";
            ensure!(!sheets.is_empty(), "Workbook needs a sheet");
            let mut names = HashSet::new();
            let mut listing = String::new();
            let mut links = Vec::new();
            for (index, sheet) in sheets.into_iter().enumerate() {
                ensure!(
                    !sheet.name.is_empty()
                        && sheet.name.chars().count() <= 31
                        && !sheet.name.contains(['[', ']', ':', '*', '?', '/', '\\'])
                        && !sheet.name.starts_with('\'')
                        && !sheet.name.ends_with('\''),
                    "Invalid worksheet name"
                );
                ensure!(
                    names.insert(sheet.name.to_lowercase()),
                    "Duplicate worksheet name"
                );
                ensure!(sheet.rows.len() <= 1048576, "Too many rows");
                let id = index + 1;
                let mut rows = String::new();
                for (r, row) in sheet.rows.iter().enumerate() {
                    ensure!(row.len() <= 16384, "Too many columns");
                    rows.push_str(&format!("<row r=\"{}\">", r + 1));
                    for (c, cell) in row.iter().enumerate() {
                        rows.push_str(&cell_xml(&address(c + 1, r + 1), cell, None)?);
                    }
                    rows.push_str("</row>");
                }
                let path = format!("xl/worksheets/sheet{id}.xml");
                package.add(
                    &path,
                    format!("<worksheet xmlns=\"{S}\"><sheetData>{rows}</sheetData></worksheet>"),
                );
                types.push((path, "spreadsheetml.worksheet".into()));
                listing.push_str(&format!(
                    "<sheet name=\"{}\" sheetId=\"{id}\" r:id=\"rId{id}\"/>",
                    x(&sheet.name)
                ));
                links.push((
                    format!("rId{id}"),
                    "worksheet".into(),
                    format!("worksheets/sheet{id}.xml"),
                ));
            }
            package.add(main,format!("<workbook xmlns=\"{S}\" xmlns:r=\"{R}\"><sheets>{listing}</sheets><calcPr fullCalcOnLoad=\"1\" forceFullCalc=\"1\"/></workbook>"));
            package.add("xl/_rels/workbook.xml.rels", rels(&links));
            types.push((main.into(), "spreadsheetml.sheet.main".into()));
        }
        DocumentSpec::Pptx { slides } => {
            main = "ppt/presentation.xml";
            ensure!(!slides.is_empty(), "Presentation needs a slide");
            let mut listing = String::new();
            let mut links = vec![(
                "rIdMaster".into(),
                "slideMaster".into(),
                "slideMasters/slideMaster1.xml".into(),
            )];
            for (index, slide) in slides.into_iter().enumerate() {
                ensure!(
                    slide.bullets.len() <= 10,
                    "Split more than ten bullets across multiple slides"
                );
                let id = index + 1;
                let title = shape(
                    2,
                    "Title",
                    &[slide.title],
                    400000,
                    300000,
                    11000000,
                    900000,
                    3200,
                    false,
                );
                let body = shape(
                    3,
                    "Content",
                    &slide.bullets,
                    600000,
                    1500000,
                    10800000,
                    4800000,
                    2200,
                    true,
                );
                let path = format!("ppt/slides/slide{id}.xml");
                package.add(&path,format!("<p:sld xmlns:p=\"{P}\" xmlns:a=\"{A}\" xmlns:r=\"{R}\"><p:cSld><p:spTree>{}{title}{body}</p:spTree></p:cSld><p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr></p:sld>",group()));
                package.add(
                    &format!("ppt/slides/_rels/slide{id}.xml.rels"),
                    rels(&[(
                        "rIdLayout".into(),
                        "slideLayout".into(),
                        "../slideLayouts/slideLayout1.xml".into(),
                    )]),
                );
                listing.push_str(&format!(
                    "<p:sldId id=\"{}\" r:id=\"rId{id}\"/>",
                    256 + index
                ));
                links.push((
                    format!("rId{id}"),
                    "slide".into(),
                    format!("slides/slide{id}.xml"),
                ));
                types.push((path, "presentationml.slide".into()));
            }
            package.add(main,format!("<p:presentation xmlns:p=\"{P}\" xmlns:a=\"{A}\" xmlns:r=\"{R}\"><p:sldMasterIdLst><p:sldMasterId id=\"2147483648\" r:id=\"rIdMaster\"/></p:sldMasterIdLst><p:sldIdLst>{listing}</p:sldIdLst><p:sldSz cx=\"12192000\" cy=\"6858000\"/><p:notesSz cx=\"6858000\" cy=\"9144000\"/></p:presentation>"));
            package.add("ppt/_rels/presentation.xml.rels", rels(&links));
            add_presentation_parts(&mut package, &mut types);
            types.push((main.into(), "presentationml.presentation.main".into()));
        }
    }
    package.add(
        "_rels/.rels",
        rels(&[("rId1".into(), "officeDocument".into(), main.into())]),
    );
    let overrides: String = types.iter().map(|(path,kind)| format!("<Override PartName=\"/{path}\" ContentType=\"application/vnd.openxmlformats-officedocument.{kind}+xml\"/>")).collect();
    package.add("[Content_Types].xml",format!("<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\"><Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/><Default Extension=\"xml\" ContentType=\"application/xml\"/>{overrides}</Types>"));
    Ok(package)
}
fn group() -> &'static str {
    "<p:nvGrpSpPr><p:cNvPr id=\"1\" name=\"\"/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr><a:xfrm><a:off x=\"0\" y=\"0\"/><a:ext cx=\"0\" cy=\"0\"/><a:chOff x=\"0\" y=\"0\"/><a:chExt cx=\"0\" cy=\"0\"/></a:xfrm></p:grpSpPr>"
}
#[allow(clippy::too_many_arguments)]
fn shape(
    id: u32,
    name: &str,
    paragraphs: &[String],
    left: u32,
    top: u32,
    width: u32,
    height: u32,
    size: u32,
    bullets: bool,
) -> String {
    let text: String = paragraphs.iter().map(|text| format!("<a:p><a:pPr><a:buNone/></a:pPr><a:r><a:rPr lang=\"en-US\" sz=\"{size}\"><a:solidFill><a:srgbClr val=\"111111\"/></a:solidFill><a:latin typeface=\"Calibri\"/></a:rPr><a:t>{}{}</a:t></a:r><a:endParaRPr lang=\"en-US\" sz=\"{size}\"/></a:p>",if bullets {"• "} else {""},x(text))).collect();
    let text = if text.is_empty() {
        "<a:p/>".to_owned()
    } else {
        text
    };
    format!(
        "<p:sp><p:nvSpPr><p:cNvPr id=\"{id}\" name=\"{name}\"/><p:cNvSpPr txBox=\"1\"/><p:nvPr/></p:nvSpPr><p:spPr><a:xfrm><a:off x=\"{left}\" y=\"{top}\"/><a:ext cx=\"{width}\" cy=\"{height}\"/></a:xfrm><a:prstGeom prst=\"rect\"><a:avLst/></a:prstGeom><a:noFill/></p:spPr><p:txBody><a:bodyPr wrap=\"square\"><a:spAutoFit/></a:bodyPr><a:lstStyle/>{text}</p:txBody></p:sp>"
    )
}
fn add_presentation_parts(package: &mut Package, types: &mut Vec<(String, String)>) {
    let colors = "<p:clrMap accent1=\"accent1\" accent2=\"accent2\" accent3=\"accent3\" accent4=\"accent4\" accent5=\"accent5\" accent6=\"accent6\" bg1=\"lt1\" bg2=\"lt2\" folHlink=\"folHlink\" hlink=\"hlink\" tx1=\"dk1\" tx2=\"dk2\"/>";
    package.add("ppt/slideMasters/slideMaster1.xml",format!("<p:sldMaster xmlns:p=\"{P}\" xmlns:a=\"{A}\" xmlns:r=\"{R}\"><p:cSld><p:spTree>{}</p:spTree></p:cSld>{colors}<p:sldLayoutIdLst><p:sldLayoutId id=\"2147483649\" r:id=\"rIdLayout\"/></p:sldLayoutIdLst><p:txStyles><p:titleStyle/><p:bodyStyle/><p:otherStyle/></p:txStyles></p:sldMaster>",group()));
    package.add(
        "ppt/slideMasters/_rels/slideMaster1.xml.rels",
        rels(&[
            (
                "rIdLayout".into(),
                "slideLayout".into(),
                "../slideLayouts/slideLayout1.xml".into(),
            ),
            (
                "rIdTheme".into(),
                "theme".into(),
                "../theme/theme1.xml".into(),
            ),
        ]),
    );
    package.add("ppt/slideLayouts/slideLayout1.xml",format!("<p:sldLayout xmlns:p=\"{P}\" xmlns:a=\"{A}\" type=\"blank\" preserve=\"1\"><p:cSld name=\"Blank\"><p:spTree>{}</p:spTree></p:cSld><p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr></p:sldLayout>",group()));
    package.add(
        "ppt/slideLayouts/_rels/slideLayout1.xml.rels",
        rels(&[(
            "rIdMaster".into(),
            "slideMaster".into(),
            "../slideMasters/slideMaster1.xml".into(),
        )]),
    );
    let colors: String = [
        ("dk1", "111111"),
        ("lt1", "FFFFFF"),
        ("dk2", "333333"),
        ("lt2", "EEEEEE"),
        ("accent1", "2563EB"),
        ("accent2", "16A34A"),
        ("accent3", "EA580C"),
        ("accent4", "9333EA"),
        ("accent5", "0891B2"),
        ("accent6", "DB2777"),
        ("hlink", "2563EB"),
        ("folHlink", "9333EA"),
    ]
    .iter()
    .map(|(name, color)| format!("<a:{name}><a:srgbClr val=\"{color}\"/></a:{name}>"))
    .collect();
    let font = "<a:latin typeface=\"Calibri\"/><a:ea typeface=\"\"/><a:cs typeface=\"\"/>";
    let fill = "<a:solidFill><a:schemeClr val=\"phClr\"/></a:solidFill>";
    package.add("ppt/theme/theme1.xml",format!("<a:theme xmlns:a=\"{A}\" name=\"Fenix\"><a:themeElements><a:clrScheme name=\"Fenix\">{colors}</a:clrScheme><a:fontScheme name=\"Fenix\"><a:majorFont>{font}</a:majorFont><a:minorFont>{font}</a:minorFont></a:fontScheme><a:fmtScheme name=\"Fenix\"><a:fillStyleLst>{}</a:fillStyleLst><a:lnStyleLst>{}</a:lnStyleLst><a:effectStyleLst>{}</a:effectStyleLst><a:bgFillStyleLst>{}</a:bgFillStyleLst></a:fmtScheme></a:themeElements></a:theme>",fill.repeat(3),format!("<a:ln w=\"9525\">{fill}<a:prstDash val=\"solid\"/></a:ln>").repeat(3),"<a:effectStyle><a:effectLst/></a:effectStyle>".repeat(3),fill.repeat(3)));
    for (path, kind) in [
        (
            "ppt/slideMasters/slideMaster1.xml",
            "presentationml.slideMaster",
        ),
        (
            "ppt/slideLayouts/slideLayout1.xml",
            "presentationml.slideLayout",
        ),
        ("ppt/theme/theme1.xml", "theme"),
    ] {
        types.push((path.into(), kind.into()));
    }
}
