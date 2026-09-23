use std::{env, fs, path::Path};

fn collect(directory: &Path, base: &Path, resources: &mut Vec<String>) {
    for entry in fs::read_dir(directory).expect("Read bundled skill resources") {
        let entry = entry.expect("Read skill resource entry");
        let kind = entry.file_type().expect("Read skill resource type");
        assert!(
            !kind.is_symlink(),
            "Bundled skills must not contain symlinks"
        );
        if kind.is_dir() {
            collect(&entry.path(), base, resources);
        } else if kind.is_file() {
            let relative = entry.path().strip_prefix(base).unwrap().to_owned();
            let relative = relative.to_str().expect("UTF-8 skill path");
            // Historical migration snapshots are not loadable skill resources.
            if !relative.ends_with("/SKILL.v1.md") {
                resources.push(relative.to_owned());
            }
        }
    }
}

fn main() {
    let manifest = env::var("CARGO_MANIFEST_DIR").unwrap();
    let root = Path::new(&manifest)
        .join("../../resources/skills")
        .canonicalize()
        .expect("Locate bundled skills");
    println!("cargo:rerun-if-changed={}", root.display());
    let mut names = Vec::new();
    let mut resources = Vec::new();
    for entry in fs::read_dir(&root).unwrap() {
        let entry = entry.unwrap();
        assert!(!entry.file_type().unwrap().is_symlink());
        if entry.file_type().unwrap().is_dir() && entry.path().join("SKILL.md").is_file() {
            names.push(entry.file_name().into_string().expect("UTF-8 skill name"));
            collect(&entry.path(), &root, &mut resources);
        }
    }
    names.sort();
    resources.sort();
    let mut output = String::from("const DEFAULTS: &[(&str, &str)] = &[\n");
    for name in &names {
        let path = root.join(name).join("SKILL.md");
        output.push_str(&format!("({name:?}, include_str!({path:?})),\n"));
    }
    output.push_str("];\nconst SKILL_RESOURCES: &[(&str, &[u8])] = &[\n");
    for relative in &resources {
        if relative.ends_with("/SKILL.md") {
            continue;
        }
        let path = root.join(relative);
        output.push_str(&format!("({relative:?}, include_bytes!({path:?})),\n"));
    }
    output.push_str("];\n");
    fs::write(
        Path::new(&env::var("OUT_DIR").unwrap()).join("bundled_skills.rs"),
        output,
    )
    .expect("Write bundled skill catalog");
}
