//! Small, conservative recognizer for owner-declared executable examples.
//! This never executes extracted text and is not a shell parser or semantic grader.
use super::*;
use sha2::{Digest, Sha256};
use std::path::Path;

fn steps(command: &str) -> Vec<(String, String)> {
    if command.contains(['\'', '"', '`', '$', '\\', '<', '|']) {
        return Vec::new();
    }
    let mut found = Vec::new();
    for line in command.split("&&").flat_map(|s| s.split(['\n', ';'])) {
        let words: Vec<_> = line.split_whitespace().collect();
        let Some(first) = words.first() else { continue };
        let program = Path::new(first)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(first);
        let compiler = matches!(
            program,
            "gcc" | "g++" | "cc" | "c++" | "clang" | "clang++" | "rustc"
        );
        let interpreter = Regex::new(r"^(?:python[0-9.]*|node|ruby|perl)$")
            .unwrap()
            .is_match(program);
        if compiler || interpreter {
            // Inline snippets and module invocations are not source-file execution.
            if interpreter && words.iter().any(|w| matches!(*w, "-c" | "-m" | "-e")) {
                continue;
            }
            if let Some(target) = words.iter().skip(1).find(|w| {
                !w.starts_with('-')
                    && (if compiler {
                        [".c", ".cc", ".cpp", ".cxx", ".rs"]
                            .iter()
                            .any(|ext| w.ends_with(ext))
                    } else {
                        !w.starts_with('>')
                    })
            }) {
                found.push((program.to_owned(), (*target).to_owned()));
            }
        } else if first.starts_with('/') && words.len() > 1 {
            found.push(((*first).to_owned(), (*first).to_owned()));
        }
    }
    found
}

pub(super) fn register(request: &str, state: &mut Value) {
    let lower = request.to_lowercase();
    if [
        "do not run",
        "don't run",
        "no ejecutes",
        "no ejecutar",
        "do not test",
        "no hagas pruebas",
    ]
    .iter()
    .any(|s| lower.contains(s))
    {
        return;
    }
    for span in Regex::new(r"`([^`\n]+)`").unwrap().captures_iter(request) {
        let quote = &span[1];
        let declared = steps(quote);
        for (program, artifact) in &declared {
            // Relative/complex examples stay under the existing explicit contract tools.
            if !artifact.starts_with('/') {
                continue;
            }
            let hash = format!("{:x}", Sha256::digest(format!("{program}\n{artifact}")));
            let id = format!("invocation_{}", &hash[..24]);
            let items = state["items"].as_array_mut().unwrap();
            if !items.iter().any(|item| item["id"] == id) {
                let run_artifact = if is_compiler(program) {
                    declared
                        .iter()
                        .find(|(p, _)| p.starts_with('/'))
                        .map(|(_, path)| path)
                } else {
                    None
                };
                items.push(json!({"id":id,"kind":"invocation","program":program,"artifact":artifact,
                    "source_quote":quote,"status":"pending","run_artifact":run_artifact,
                    "instruction":"Run this owner-declared execution path with representative concrete arguments after the latest edit. For a compiler followed by a binary, compile to an explicit output and execute that exact output in the same successful && chain; a preexisting binary is not build evidence. Use a simple direct command; no masked failures. Exit success proves execution only: also inspect its output against the requested behavior. Repair missing dependencies within existing authorization."}));
            }
        }
    }
}

pub(super) fn matches(item: &Value, command: &str, cwd: Option<&str>) -> bool {
    if item["run_artifact"].is_string() && !linked_build_run(item, command, cwd) {
        return false;
    }
    steps(command).iter().any(|(program, path)| {
        let resolved = if Path::new(path).is_absolute() {
            Path::new(path).to_path_buf()
        } else {
            Path::new(cwd.unwrap_or("")).join(path)
        };
        item["program"] == *program
            && item["artifact"]
                .as_str()
                .is_some_and(|expected| Path::new(expected) == resolved)
    })
}

fn is_compiler(program: &str) -> bool {
    matches!(
        program,
        "gcc" | "g++" | "cc" | "c++" | "clang" | "clang++" | "rustc"
    )
}

fn resolve(path: &str, cwd: Option<&str>) -> std::path::PathBuf {
    let joined = Path::new(cwd.unwrap_or("")).join(path);
    let mut normalized = std::path::PathBuf::new();
    for part in joined.components() {
        match part {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            part => normalized.push(part.as_os_str()),
        }
    }
    normalized
}

// Deliberately supports direct compile-and-run checks, not arbitrary build systems.
// Requiring the same chain avoids treating a preexisting executable as a new build.
fn linked_build_run(item: &Value, command: &str, cwd: Option<&str>) -> bool {
    if !unmasked(command) || command.contains(['\n', ';', '\'', '"', '`', '$', '\\', '<', '>']) {
        return false;
    }
    let segments: Vec<_> = command
        .split("&&")
        .map(|s| s.split_whitespace().collect::<Vec<_>>())
        .collect();
    let expected = item["run_artifact"].as_str().unwrap();
    for (index, words) in segments.iter().enumerate() {
        let Some(program) = words.first() else {
            continue;
        };
        if *program == "cd" {
            return false;
        }
        let program = Path::new(program)
            .file_name()
            .and_then(|x| x.to_str())
            .unwrap_or("");
        if item["program"] != program || !is_compiler(program) {
            continue;
        }
        if words.iter().any(|w| {
            matches!(*w, "-c" | "-S" | "-E" | "--emit" | "--out-dir")
                || w.starts_with("--emit=")
                || w.starts_with("--out-dir=")
        }) {
            continue;
        }
        let source = item["artifact"].as_str().unwrap();
        if !words
            .iter()
            .skip(1)
            .any(|w| resolve(w, cwd) == resolve(source, cwd))
        {
            continue;
        }
        let explicit = words.windows(2).find(|w| w[0] == "-o").map(|w| w[1]);
        let default = if program == "rustc" {
            Path::new(source)
                .file_stem()
                .and_then(|p| p.to_str())
                .unwrap_or("")
        } else {
            "a.out"
        };
        let output = resolve(explicit.unwrap_or(default), cwd);
        if output != resolve(expected, cwd) {
            continue;
        }
        for next in &segments[index + 1..] {
            let Some(first) = next.first() else { continue };
            // Intervening steps could overwrite outputs or change working directory.
            return resolve(first, cwd) == output
                && (first.starts_with('/') || first.starts_with("./"));
        }
    }
    false
}

pub(super) fn unmasked(command: &str) -> bool {
    (!(command.contains(';') || command.contains('\n'))
        || command.trim_start().starts_with("set -e\n"))
        && !command.contains("||")
        && !command.contains('|')
        && !command.contains("<<")
        && !Regex::new(r"(?:;|\n)\s*(?:true|exit\s+0)\s*;?\s*$")
            .unwrap()
            .is_match(command)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn declared_execution_needs_fresh_receipts_in_each_requested_mode() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let rt = Runtime::open(&dir.path().join("state"), Some(&dir.path().join("skills"))).await?;
        let task = rt.store.enqueue("Run `python3 /work/report.py 5` and `gcc /work/report.c -o /work/report && /work/report 5`.", "web", None)?;
        let run = |id: &str, cmd: &str, code: i64| {
            rt.record_completion_check(
                &task.id,
                id,
                "command_run",
                &json!({"command":cmd,"cwd":"/work"}),
                &json!({"exit_code":code,"output":"5"}),
                false,
            )
        };
        assert_eq!(
            rt.completion_requirements(&task.id)?["items"]
                .as_array()
                .unwrap()
                .len(),
            3
        );
        run("missing", "python3 /work/report.py 5", 127)?;
        run(
            "compile",
            "gcc /work/report.c -o /work/report && /work/report 5",
            0,
        )?;
        assert!(rt.validate_task_completion(&task.id).is_err());
        run("fake", "echo python3 /work/report.py 5", 0)?;
        run("masked", "python3 /work/report.py 5 || true", 0)?;
        assert!(rt.validate_task_completion(&task.id).is_err());
        run("python", "python3 report.py 5", 0)?;
        rt.validate_task_completion(&task.id)?;
        rt.record_completion_check(
            &task.id,
            "edit",
            "file_edit",
            &json!({"path":"/work/report.py"}),
            &json!({"ok":true}),
            false,
        )?;
        assert!(rt.validate_task_completion(&task.id).is_err());
        run(
            "all",
            "python3 /work/report.py 5 && gcc /work/report.c -o /work/report && /work/report 5",
            0,
        )?;
        rt.validate_task_completion(&task.id)?;
        rt.shutdown().await?;
        Ok(())
    }

    #[test]
    fn compiled_example_requires_the_binary_just_built() {
        let mut state = json!({"revision":0,"items":[]});
        register("Run `rustc /work/main.rs && /work/main 5`.", &mut state);
        let item = &state["items"][0];
        assert!(!matches(
            item,
            "rustc /work/main.rs && /work/main 5",
            Some("/tmp")
        ));
        assert!(matches(
            item,
            "rustc /work/main.rs -o /work/main && /work/main 5",
            Some("/tmp")
        ));
        assert!(matches(item, "rustc main.rs && ./main 5", Some("/work")));
        assert!(!matches(
            item,
            "rustc /work/main.rs -o /tmp/old && /work/main 5",
            Some("/work")
        ));
        assert!(!matches(
            item,
            "rustc /work/main.rs -o /work/main && cp /tmp/old /work/main && /work/main 5",
            Some("/work")
        ));
        assert!(!unmasked("false; /work/main 5"));
        assert!(!matches(item, "rustc /work/main.rs", Some("/work")));
    }

    #[test]
    fn does_not_turn_paths_or_snippets_or_prohibited_runs_into_checks() {
        for request in [
            "Create `/work/note.md`.",
            "Do not run `python3 /work/a.py 5`.",
            "Example: `python3 -c 'print(1)'`",
            "Read `cat /work/data.txt`.",
        ] {
            let mut state = json!({"revision":0,"items":[]});
            register(request, &mut state);
            assert!(state["items"].as_array().unwrap().is_empty(), "{request}");
        }
        assert!(steps("cat << EOF\npython3 /work/a.py 5\nEOF").is_empty());
        assert!(steps("echo python3 /work/a.py 5").is_empty());
    }
}
