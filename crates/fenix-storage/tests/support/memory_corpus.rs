use fenix_storage::{
    EntityInput, Memory, MemoryEvidence, MemoryMetadata, ProjectInput, RelationInput,
    RelationSchema, Store, memory_eval::MemoryEvalCase,
};
use std::collections::BTreeMap;

pub struct Corpus {
    pub project: String,
    pub other_project: String,
    pub facts: BTreeMap<String, Memory>,
    pub anchor: String,
    pub cases: Vec<MemoryEvalCase>,
}
pub fn corpus(s: &Store) -> Corpus {
    let project = s
        .save_project(
            None,
            &ProjectInput {
                name: "Research".into(),
                ..Default::default()
            },
        )
        .unwrap()
        .id;
    let other_project = s
        .save_project(
            None,
            &ProjectInput {
                name: "Office".into(),
                ..Default::default()
            },
        )
        .unwrap()
        .id;
    let mut facts = BTreeMap::new();
    for (key, title, content, kind, scope) in [
        (
            "language",
            "Code language",
            "Source code uses Rust",
            "preference",
            project.as_str(),
        ),
        (
            "writing",
            "Writing style",
            "Documents use concise Spanish prose",
            "preference",
            project.as_str(),
        ),
        (
            "launch",
            "Launch decision",
            "We approved the beta release",
            "decision",
            project.as_str(),
        ),
        (
            "deadline",
            "Delivery",
            "Deliver the prototype by Friday",
            "commitment",
            project.as_str(),
        ),
        (
            "instrument",
            "Equipment",
            "The laboratory uses a spectrometer",
            "fact",
            project.as_str(),
        ),
        (
            "owner",
            "Ownership",
            "Marta coordinates Helios",
            "fact",
            project.as_str(),
        ),
        (
            "dependency",
            "Dependency",
            "Helios supports Aurora",
            "fact",
            project.as_str(),
        ),
        (
            "meeting",
            "Reunión",
            "La reunión del laboratorio será el martes",
            "commitment",
            project.as_str(),
        ),
        (
            "budget",
            "Budget",
            "The approved budget is 900 soles",
            "constraint",
            project.as_str(),
        ),
        (
            "private",
            "Office budget",
            "The office budget is 12000 soles",
            "fact",
            other_project.as_str(),
        ),
    ] {
        let m = s
            .save_memory_record(
                None,
                title,
                content,
                "owner",
                Some(scope),
                None,
                Some(MemoryMetadata {
                    kind: kind.into(),
                    evidence: vec![MemoryEvidence {
                        reference: format!("fixture:{key}"),
                        quote: content.into(),
                    }],
                    ..Default::default()
                }),
            )
            .unwrap();
        facts.insert(key.into(), m);
    }
    let entity = |name: &str, kind: &str, aliases: Vec<String>| {
        s.save_entity(
            None,
            Some(&project),
            EntityInput {
                name: name.into(),
                kind: kind.into(),
                aliases,
                description: String::new(),
            },
        )
        .unwrap()
    };
    let aurora = entity(
        "Aurora",
        "research",
        vec!["Study Aurora".into(), "Estudio Aurora".into()],
    );
    let helios = entity("Helios", "project", vec![]);
    let marta = entity("Marta", "person", vec![]);
    for (predicate, from, to, key) in [
        ("supports", &helios, &aurora, "dependency"),
        ("coordinates", &marta, &helios, "owner"),
    ] {
        s.save_relation_schema(
            Some(&project),
            RelationSchema {
                name: predicate.into(),
                description: predicate.into(),
                source_kinds: vec![],
                target_kinds: vec![],
                status: "active".into(),
            },
        )
        .unwrap();
        s.save_relation(
            Some(&project),
            RelationInput {
                source_id: from.id.clone(),
                target_id: to.id.clone(),
                predicate: predicate.into(),
                evidence_memory_id: facts[key].id.clone(),
                status: "confirmed".into(),
            },
        )
        .unwrap();
    }
    let mut cases = vec![];
    for (name, query, keys) in [
        ("code language", "Rust", vec!["language"]),
        ("code exact title", "Code language", vec!["language"]),
        ("writing", "concise Spanish", vec!["writing"]),
        ("writing title", "Writing style", vec!["writing"]),
        ("decision", "beta release", vec!["launch"]),
        ("deadline", "prototype Friday", vec!["deadline"]),
        ("instrument", "spectrometer", vec!["instrument"]),
        ("equipment title", "Equipment", vec!["instrument"]),
        ("meeting Spanish", "reunión martes", vec!["meeting"]),
        ("budget scoped", "budget", vec!["budget"]),
        (
            "graph English",
            "Who coordinates Study Aurora?",
            vec!["owner", "dependency"],
        ),
        (
            "graph Spanish",
            "¿Quién participa en Estudio Aurora?",
            vec!["owner", "dependency"],
        ),
        ("graph alias", "Study Aurora", vec!["owner", "dependency"]),
        ("graph canonical", "Aurora", vec!["owner", "dependency"]),
        ("neighbor", "Helios", vec!["owner", "dependency"]),
        ("person", "Marta", vec!["owner", "dependency"]),
        ("unknown", "quasarxylophone", vec![]),
        ("no substring entity", "Auroralight", vec![]),
    ] {
        cases.push(MemoryEvalCase {
            name: name.into(),
            query: query.into(),
            project_id: Some(project.clone()),
            relevant: keys.iter().map(|key| facts[*key].id.clone()).collect(),
            forbidden: vec![facts["private"].id.clone()],
        });
    }
    cases.push(MemoryEvalCase {
        name: "project isolation".into(),
        query: "Marta Aurora budget".into(),
        project_id: Some(other_project.clone()),
        relevant: vec![facts["private"].id.clone()],
        forbidden: vec![facts["owner"].id.clone(), facts["budget"].id.clone()],
    });
    cases.push(MemoryEvalCase {
        name: "exact ID".into(),
        query: facts["launch"].id.clone(),
        project_id: Some(project.clone()),
        relevant: vec![facts["launch"].id.clone()],
        forbidden: vec![],
    });
    let previous = s
        .save_memory_record(
            None,
            "Theme preference",
            "Prefers bright violet panels",
            "owner",
            Some(&project),
            None,
            Some(MemoryMetadata {
                kind: "preference".into(),
                ..Default::default()
            }),
        )
        .unwrap();
    let current = s
        .save_memory_record(
            None,
            "Theme preference",
            "Prefers neutral silver panels",
            "owner",
            Some(&project),
            None,
            Some(MemoryMetadata {
                kind: "preference".into(),
                supersedes: Some(previous.id.clone()),
                ..Default::default()
            }),
        )
        .unwrap();
    let withdrawn = s
        .save_memory_scoped(
            None,
            "Temporary code",
            "Obsolete marker turmaline",
            "owner",
            Some(&project),
            None,
        )
        .unwrap();
    s.delete_memory_scoped(&withdrawn.id, Some(&project))
        .unwrap();
    let candidate = s
        .save_memory_record(
            None,
            "Unconfirmed",
            "Hypothesis amethyst",
            "owner",
            Some(&project),
            None,
            Some(MemoryMetadata {
                status: "candidate".into(),
                ..Default::default()
            }),
        )
        .unwrap();
    let expired = s
        .save_memory_record(
            None,
            "Expired",
            "Old appointment topaz",
            "owner",
            Some(&project),
            None,
            Some(MemoryMetadata {
                valid_until: Some("2020-01-01T00:00:00Z".into()),
                ..Default::default()
            }),
        )
        .unwrap();
    let yes = s
        .save_memory_scoped(
            None,
            "Upload policy",
            "The service permits external uploads",
            "owner",
            Some(&project),
            None,
        )
        .unwrap();
    let no = s
        .save_memory_scoped(
            None,
            "Upload policy",
            "The service forbids external uploads",
            "owner",
            Some(&project),
            None,
        )
        .unwrap();
    for (name, query, relevant, forbidden) in [
        (
            "corrected preference",
            "Theme preference",
            vec![current.id.clone()],
            vec![previous.id.clone()],
        ),
        ("superseded detail", "violet", vec![], vec![previous.id]),
        ("withdrawn detail", "turmaline", vec![], vec![withdrawn.id]),
        (
            "candidate exclusion",
            "amethyst",
            vec![],
            vec![candidate.id],
        ),
        ("expiry exclusion", "topaz", vec![], vec![expired.id]),
        (
            "conflicting evidence remains visible",
            "Upload policy",
            vec![yes.id, no.id],
            vec![],
        ),
    ] {
        cases.push(MemoryEvalCase {
            name: name.into(),
            query: query.into(),
            project_id: Some(project.clone()),
            relevant,
            forbidden,
        });
    }
    Corpus {
        project,
        other_project,
        facts,
        anchor: aurora.id,
        cases,
    }
}
