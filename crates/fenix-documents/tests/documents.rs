use fenix_documents::{DocumentSpec, create, inspect, modify, sha256};
use serde_json::{Value, json};
fn build(value: Value) -> Vec<u8> {
    create(serde_json::from_value::<DocumentSpec>(value).unwrap()).unwrap()
}
#[test]
fn all_formats_roundtrip_and_conditional_edits() {
    for (spec, part, operation, expected) in [
        (
            json!({"format":"docx","paragraphs":[{"text":"Report & evidence","heading":1},{"text":"Status pending"}],"tables":[[["Name","Value"],["Trial","10"]]]}),
            "word/document.xml",
            json!({"action":"replace_text","expected":"Status pending","replacement":"Status complete <verified>"}),
            "Status complete <verified>",
        ),
        (
            json!({"format":"xlsx","sheets":[{"name":"Sales","rows":[["Amount",2],["Total",{"formula":"SUM(B1:B1)"}]]}]}),
            "xl/worksheets/sheet1.xml",
            json!({"action":"set_cell","cell":"D9","value":"=literal-not-formula"}),
            "=literal-not-formula",
        ),
        (
            json!({"format":"pptx","slides":[{"title":"Quarterly review","bullets":["Status pending","Next steps"]}]}),
            "ppt/slides/slide1.xml",
            json!({"action":"replace_text","expected":"Status pending","replacement":"Status complete"}),
            "Status complete",
        ),
    ] {
        let bytes = build(spec);
        let initial = inspect(&bytes, 0, 100).unwrap();
        assert!(initial["total_blocks"].as_u64().unwrap() > 0);
        let mut operation = operation;
        operation["part"] = json!(part);
        assert!(modify(&bytes, "stale-hash", &operation).is_err());
        let updated = modify(&bytes, &sha256(&bytes), &operation).unwrap();
        let after = inspect(&updated, 0, 100).unwrap();
        assert!(after.to_string().contains(expected));
        assert_ne!(sha256(&bytes), sha256(&updated));
    }
}
#[test]
fn formulas_are_explicit_and_pagination_is_bounded() {
    let bytes = build(
        json!({"format":"xlsx","sheets":[{"name":"Data","rows":[["=SUM(1,2)",{"formula":"SUM(1,2)"},true,null,5.25]]}]}),
    );
    let page = inspect(&bytes, 0, 1).unwrap();
    assert_eq!(page["blocks"][0]["value"], "=SUM(1,2)");
    assert!(page["blocks"][0]["formula"].is_null());
    assert_eq!(page["next_offset"], 1);
    let page = inspect(&bytes, 1, 1).unwrap();
    assert_eq!(page["blocks"][0]["formula"], "SUM(1,2)");
    assert_eq!(page["blocks"][0]["cached_value_only"], true);
}
#[test]
fn rejects_invalid_specs_and_archives() {
    assert!(
        create(
            serde_json::from_value(
                json!({"format":"xlsx","sheets":[{"name":"Bad/Name","rows":[]}]})
            )
            .unwrap()
        )
        .is_err()
    );
    assert!(create(serde_json::from_value(json!({"format":"pptx","slides":[]})).unwrap()).is_err());
    assert!(inspect(b"not a zip", 0, 20).is_err());
}
