//! Portable manual skill storage. No model calls, translations or script execution.
use anyhow::{Context, Result, ensure};
use fenix_files::private::{Lock, PrivateDirectory};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::Read,
    path::{Component, Path},
};
pub const MAX_BYTES: usize = 128 * 1024;
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bundle {
    pub name: String,
    pub files: BTreeMap<String, String>,
    #[serde(default)]
    pub executable: BTreeSet<String>,
}
impl Bundle {
    pub fn validate(&self) -> Result<()> {
        ensure!(valid_name(&self.name), "Invalid skill name");
        ensure!(
            !self.files.is_empty() && self.files.len() <= 128,
            "A publication supports 1–128 text files"
        );
        ensure!(self.files.contains_key("SKILL.md"), "SKILL.md is required");
        ensure!(
            self.files["SKILL.md"].len() <= 32_000 && !self.files["SKILL.md"].trim().is_empty(),
            "Invalid SKILL.md size"
        );
        ensure!(
            self.files.values().map(String::len).sum::<usize>() <= MAX_BYTES,
            "Skill exceeds 128 KiB"
        );
        metadata(&self.files["SKILL.md"])?;
        for (path, content) in &self.files {
            validate_path(path)?;
            ensure!(
                !content.contains('\0'),
                "Binary content requires manual review"
            );
            ensure!(
                !self
                    .files
                    .keys()
                    .any(|other| other.starts_with(&format!("{path}/"))),
                "Conflicting file paths"
            );
        }
        ensure!(
            self.executable
                .iter()
                .all(|path| self.files.contains_key(path)),
            "Unknown executable path"
        );
        Ok(())
    }
    pub fn hash(&self) -> String {
        format!(
            "{:x}",
            Sha256::digest(serde_json::to_vec(self).expect("Serializable bundle"))
        )
    }
}
fn validate_path(path: &str) -> Result<()> {
    ensure!(
        !path.is_empty()
            && path.len() <= 240
            && !path.contains('\\')
            && !path.contains(':')
            && !path.chars().any(char::is_control),
        "Invalid resource path"
    );
    ensure!(
        path.split('/').count() <= 12
            && path
                .split('/')
                .all(|p| !p.is_empty() && !p.starts_with('.') && portable_component(p)),
        "Hidden or ambiguous paths require manual review"
    );
    ensure!(
        Path::new(path)
            .components()
            .all(|p| matches!(p, Component::Normal(_))),
        "Resource must remain inside skill"
    );
    Ok(())
}

pub fn metadata(text: &str) -> Result<serde_yaml_ng::Value> {
    if let Some(rest) = text
        .strip_prefix("---\n")
        .or_else(|| text.strip_prefix("---\r\n"))
    {
        let end = rest
            .lines()
            .scan(0, |offset, line| {
                let start = *offset;
                *offset += line.len()
                    + if rest.as_bytes().get(start + line.len()) == Some(&b'\r') {
                        2
                    } else {
                        1
                    };
                Some((start, line))
            })
            .find(|(_, line)| *line == "---")
            .map(|(offset, _)| offset)
            .context("Unclosed skill YAML frontmatter")?;
        Ok(serde_yaml_ng::from_str(&rest[..end])?)
    } else {
        // Compatibility with early manually installed skills without frontmatter.
        let header = text
            .lines()
            .take_while(|line| !line.trim().is_empty())
            .collect::<Vec<_>>()
            .join("\n");
        Ok(serde_yaml_ng::from_str(&header).unwrap_or_default())
    }
}

fn portable_component(name: &str) -> bool {
    if name.ends_with(['.', ' ']) {
        return false;
    }
    let stem = name
        .split('.')
        .next()
        .unwrap_or_default()
        .to_ascii_uppercase();
    !matches!(
        stem.as_str(),
        "CON" | "PRN" | "AUX" | "NUL" | "CONIN$" | "CONOUT$"
    ) && !["COM", "LPT"].iter().any(|prefix| {
        stem.strip_prefix(prefix).is_some_and(|suffix| {
            matches!(
                suffix,
                "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "¹" | "²" | "³"
            )
        })
    })
}
pub fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && portable_component(name)
        && name.len() <= 80
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

pub struct Library {
    root: PrivateDirectory,
}
impl Library {
    pub fn open(path: &Path) -> Result<Self> {
        Ok(Self {
            root: PrivateDirectory::open(path)?,
        })
    }
    pub fn lock(&self) -> Result<Lock> {
        Lock::exclusive(&self.root)
    }
    pub fn read_skill(&self, name: &str) -> Result<(String, String)> {
        ensure!(valid_name(name), "Invalid skill name");
        let dir = self.root.subdir(Path::new(name), false)?;
        let content = String::from_utf8(dir.read(Path::new("SKILL.md"), 32_000)?)?;
        let revision = format!("{:x}", Sha256::digest(content.as_bytes()));
        Ok((content, revision))
    }
    pub fn replace_skill(&self, name: &str, expected: &str, content: &str) -> Result<()> {
        ensure!(valid_name(name), "Invalid skill name");
        ensure!(
            !content.trim().is_empty() && content.len() <= 32_000,
            "Invalid skill size"
        );
        metadata(content)?;
        let (before, revision) = self.read_skill(name)?;
        ensure!(
            revision == expected,
            "This skill changed elsewhere. Reopen it before saving"
        );
        self.root.subdir(Path::new(name), false)?.replace(
            Path::new("SKILL.md"),
            before.as_bytes(),
            content.as_bytes(),
        )
    }
    pub fn export(&self, name: &str) -> Result<Bundle> {
        ensure!(valid_name(name), "Invalid skill name");
        let _lock = self.lock()?;
        let dir = self.root.subdir(Path::new(name), false)?;
        let mut bundle = Bundle {
            name: name.into(),
            files: BTreeMap::new(),
            executable: BTreeSet::new(),
        };
        collect(&dir, "", &mut bundle, &mut 0, &mut 0)?;
        #[cfg(windows)]
        if dir.exists(Path::new(".fenix-executables.json"))? {
            bundle.executable =
                serde_json::from_slice(&dir.read(Path::new(".fenix-executables.json"), 32_000)?)?;
        }
        bundle.validate()?;
        Ok(bundle)
    }
    pub fn install(&self, bundle: &Bundle, name: &str) -> Result<()> {
        bundle.validate()?;
        ensure!(valid_name(name), "Invalid installation name");
        let _lock = self.lock()?;
        ensure!(
            !self.root.exists(Path::new(name))?,
            "Skill already exists; existing edits are never overwritten"
        );
        let staging = format!(".import-{}", uuid::Uuid::new_v4());
        let result = (|| -> Result<()> {
            let dir = self.root.subdir(Path::new(&staging), true)?;
            for (path, content) in &bundle.files {
                dir.create(
                    Path::new(path),
                    content.as_bytes(),
                    bundle.executable.contains(path),
                )?;
            }
            #[cfg(windows)]
            dir.create(
                Path::new(".fenix-executables.json"),
                &serde_json::to_vec(&bundle.executable)?,
                false,
            )?;
            dir.sync()?;
            // All cooperating writers hold the same lock. A populated destination
            // is never replaced; an empty directory created externally also aborts.
            ensure!(
                !self.root.exists(Path::new(name))?,
                "Skill appeared during installation"
            );
            self.root
                .rename_noreplace(Path::new(&staging), Path::new(name))?;
            Ok(())
        })();
        if result.is_err() {
            let _ = self.root.remove_tree(Path::new(&staging));
        }
        result
    }
    /// Caller holds lock and records a deletion tombstone before detaching.
    pub fn detach(&self, name: &str, expected: &str) -> Result<String> {
        let (_, revision) = self.read_skill(name)?;
        ensure!(
            revision == expected,
            "This skill changed elsewhere. Reopen it before deleting"
        );
        let removed = format!(".deleted-{}", uuid::Uuid::new_v4());
        self.root.rename(Path::new(name), Path::new(&removed))?;
        Ok(removed)
    }
    pub fn remove_detached(&self, name: &str) -> Result<()> {
        ensure!(
            name.starts_with(".deleted-") && !name.contains('/') && !name.contains('\\'),
            "Invalid detached directory"
        );
        self.root.remove_tree(Path::new(name))
    }
}
fn collect(
    dir: &PrivateDirectory,
    prefix: &str,
    bundle: &mut Bundle,
    bytes: &mut usize,
    entries: &mut usize,
) -> Result<()> {
    for entry in dir.entries()? {
        *entries += 1;
        ensure!(*entries <= 256, "Too many bundle entries");
        let entry = entry?;
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| anyhow::anyhow!("Non UTF-8 filename"))?;
        #[cfg(windows)]
        if name == ".fenix-executables.json" && prefix.is_empty() {
            continue;
        }
        let relative = if prefix.is_empty() {
            name.clone()
        } else {
            format!("{prefix}/{name}")
        };
        validate_path(&relative)?;
        let kind = entry.file_type()?;
        if kind.is_dir() {
            collect(
                &dir.subdir(Path::new(&name), false)?,
                &relative,
                bundle,
                bytes,
                entries,
            )?;
        } else {
            ensure!(
                kind.is_file() && !kind.is_symlink(),
                "Links and special files cannot be published"
            );
            ensure!(bundle.files.len() < 128, "Too many skill files");
            let file = dir.open_file(Path::new(&name))?;
            let metadata = file.metadata()?;
            let mut content = String::new();
            file.take((MAX_BYTES - *bytes + 1) as u64)
                .read_to_string(&mut content)
                .context("Only UTF-8 text bundles can be published")?;
            *bytes += content.len();
            ensure!(*bytes <= MAX_BYTES, "Skill exceeds 128 KiB");
            if fenix_files::private::executable(&metadata) {
                bundle.executable.insert(relative.clone());
            }
            bundle.files.insert(relative, content);
        }
    }
    Ok(())
}
