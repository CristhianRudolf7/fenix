//! Generates interoperability fixtures in an explicitly supplied output directory.
use anyhow::Result;
use serde_json::json;
fn main() -> Result<()> {
    let directory =
        std::path::PathBuf::from(std::env::args().nth(1).expect("output directory required"));
    std::fs::create_dir_all(&directory)?;
    for (name, value) in [
        (
            "report.docx",
            json!({"format":"docx","paragraphs":[{"text":"Fénix — informe","heading":1},{"text":"Evidence & results <2026>.\nSecond line."}],"tables":[[["Item","Value"],["A","10"]]]}),
        ),
        (
            "budget.xlsx",
            json!({"format":"xlsx","sheets":[{"name":"Ventas","rows":[["Item","Amount"],["A",10],["B",20],["Total",{"formula":"SUM(B2:B3)"}]]},{"name":"Notas","rows":[["Literal","=hello"]]}]}),
        ),
        (
            "overview.pptx",
            json!({"format":"pptx","slides":[{"title":"Fénix Documents","bullets":["Native Rust engine","No external office provider"]},{"title":"Próximos pasos","bullets":["Validar documentos","Revisar resultados"]}]}),
        ),
    ] {
        std::fs::write(
            directory.join(name),
            fenix_documents::create(serde_json::from_value(value)?)?,
        )?;
    }
    Ok(())
}
