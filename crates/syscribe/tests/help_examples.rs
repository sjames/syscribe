//! Help-page example drift gate.
//!
//! Every `syscribe -m <bundled-model>/ …` example line in the per-command help pages
//! (`prompts/help/*.md`) and in the top-level `syscribe --help` index is executed
//! against the bundled model it names (`model/`, `model_auto/`, `model_sil/`,
//! `model_mg/`, or an `examples/…/` model), from the workspace root. An example
//! passes when it exits `0` or `2` (a gate verdict) and prints nothing that says an
//! element, package, configuration or file it names does not exist — so a copied
//! example never fails on an element absent from the model it targets.
//!
//! Examples are never allowed to change the checked-in models:
//!
//! * commands that write files get their `--output`/`-o`/`--out`/`--svg`/`--prove`
//!   value redirected into a temp directory;
//! * mutating commands that support `--dry-run` (`set`, `move`, `claim`, `release`,
//!   `applies-when --set/--clear`, `plantuml` batch) are forced to `--dry-run`;
//! * `suspect accept <src> <tgt>` is checked by resolving both ends with
//!   `check-ref`; `scaffold-gherkin --fix` runs in preview mode;
//! * commands needing a live client, the network, git state or an external tool
//!   (`mcp`, `lsp`, `repos sync`, `baseline create`, `ingest-results`,
//!   `plantuml render` without `--dry-run`, `suspect accept --all*`) are skipped.
//!
//! (`summarize` refreshes its content-hash cache under `<root>/.syscribe/cache/`,
//! which is gitignored at any depth.)
//!
//! Placeholder examples (`-m <root>`) and examples naming a model root that isn't
//! bundled in this repository are not run.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().unwrap()
}

/// Split a shell-ish command line into words, honouring single/double quotes, and
/// stop at the first unquoted `|`, `>`, `<`, `;`, `&`, or ` #` comment.
fn shell_words(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut has = false;
    let mut quote: Option<char> = None;
    let mut prev_ws = true;
    for c in s.chars() {
        match quote {
            Some(q) if c == q => quote = None,
            Some(_) => cur.push(c),
            None => match c {
                '"' | '\'' => {
                    quote = Some(c);
                    has = true;
                }
                '|' | '>' | '<' | ';' | '&' => break,
                '#' if prev_ws => break,
                c if c.is_whitespace() => {
                    if has {
                        out.push(std::mem::take(&mut cur));
                        has = false;
                    }
                }
                '\\' => {}
                c => {
                    cur.push(c);
                    has = true;
                }
            },
        }
        prev_ws = c.is_whitespace();
    }
    if has {
        out.push(cur);
    }
    out
}

/// A bundled model root the example may target: one of the top-level demo models,
/// or a directory under `examples/`, that exists in the repository.
fn is_bundled_root(root: &str) -> bool {
    let t = root.trim_start_matches("./");
    let known = matches!(t, "model/" | "model_auto/" | "model_sil/" | "model_mg/") || t.starts_with("examples/");
    known && repo_root().join(t).is_dir()
}

#[derive(Debug)]
struct Example {
    origin: String,
    line: String,
    /// Arguments after the model root.
    args: Vec<String>,
    root: String,
}

fn collect_from(origin: &str, text: &str, out: &mut Vec<Example>) {
    for (i, line) in text.lines().enumerate() {
        let t = line.trim_start().trim_start_matches("$ ");
        let Some(rest) = t.strip_prefix("syscribe ") else { continue };
        let words = shell_words(rest);
        if words.len() < 2 || !(words[0] == "-m" || words[0] == "--model") {
            continue;
        }
        if !is_bundled_root(&words[1]) {
            continue;
        }
        out.push(Example {
            origin: format!("{origin}:{}", i + 1),
            line: t.to_string(),
            root: words[1].clone(),
            args: words[2..].to_vec(),
        });
    }
}

fn all_examples() -> Vec<Example> {
    let root = repo_root();
    let mut out = Vec::new();
    let mut pages: Vec<PathBuf> = std::fs::read_dir(root.join("prompts/help"))
        .unwrap()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "md"))
        .collect();
    pages.sort();
    for p in pages {
        let rel = p.strip_prefix(&root).unwrap().to_string_lossy().to_string();
        collect_from(&rel, &std::fs::read_to_string(&p).unwrap(), &mut out);
    }
    let usage = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("--help").output().unwrap();
    collect_from("syscribe --help", &String::from_utf8_lossy(&usage.stdout), &mut out);
    out
}

enum Plan {
    /// Not runnable here; the reason is documentation for the reader.
    #[allow(dead_code)]
    Skip(&'static str),
    Run(Vec<Vec<String>>),
}

fn has(args: &[String], flag: &str) -> bool {
    args.iter().any(|a| a == flag)
}

/// Turn one example into the argument vectors to execute (after `-m <root>`).
fn plan(ex: &Example, tmp: &Path, n: usize) -> Plan {
    let mut a = ex.args.clone();
    let cmd = a.first().cloned().unwrap_or_default();
    let sub = a.get(1).cloned().unwrap_or_default();
    match cmd.as_str() {
        "mcp" | "lsp" => return Plan::Skip("needs a live stdio client"),
        "ingest-results" => return Plan::Skip("needs an external results file"),
        "repos" if sub == "sync" => return Plan::Skip("needs the network / peer git repos"),
        "baseline" if sub == "create" => return Plan::Skip("seals git state"),
        "plantuml" if sub == "render" && !has(&a, "--dry-run") => return Plan::Skip("needs PlantUML"),
        "suspect" if sub == "accept" => {
            if a.len() < 4 || a[2].starts_with("--") {
                return Plan::Skip("bulk re-baselining writes the model");
            }
            return Plan::Run(vec![
                vec!["check-ref".into(), a[2].clone()],
                vec!["check-ref".into(), a[3].clone()],
            ]);
        }
        _ => {}
    }
    // Redirect file outputs into the temp dir.
    for i in 0..a.len() {
        if matches!(a[i].as_str(), "--output" | "-o" | "--out" | "--svg" | "--prove") {
            if let Some(v) = a.get(i + 1) {
                if v != "-" {
                    let name = Path::new(v).file_name().map(|f| f.to_string_lossy().to_string()).unwrap_or_default();
                    a[i + 1] = tmp.join(format!("{n}-{name}")).to_string_lossy().to_string();
                }
            }
        }
    }
    if cmd == "export-html" && !has(&a, "--out") {
        a.extend(["--out".into(), tmp.join(format!("{n}-html")).to_string_lossy().to_string()]);
    }
    if cmd == "scaffold-gherkin" {
        a.retain(|x| x != "--fix");
    }
    let dry_runnable = matches!(cmd.as_str(), "set" | "move" | "claim" | "release")
        || (cmd == "applies-when" && (has(&a, "--set") || has(&a, "--clear")))
        || (cmd == "plantuml" && !has(&a, "--output"));
    if dry_runnable && !has(&a, "--dry-run") {
        a.push("--dry-run".into());
    }
    Plan::Run(vec![a])
}

/// Output that means the example named something the model doesn't have.
fn not_found_marker(text: &str) -> Option<String> {
    let re = regex::Regex::new(
        r"(?i)(not found|does not resolve|doesn't resolve|no such file|does not exist|matches no |no children found|unknown (testplan|command|element|configuration|feature|profile|link)|is not defined|neither a known|no \w+ element found|nothing to claim|unrecognized|unexpected argument)",
    )
    .unwrap();
    re.find(text).map(|m| {
        let start = text[..m.start()].rfind('\n').map(|p| p + 1).unwrap_or(0);
        let end = text[m.end()..].find('\n').map(|p| m.end() + p).unwrap_or(text.len());
        text[start..end].trim().chars().take(200).collect()
    })
}

#[test]
fn help_page_examples_run_against_the_models_they_name() {
    let examples = all_examples();
    assert!(examples.len() > 50, "example scan found only {} examples", examples.len());
    let tmp = std::env::temp_dir().join(format!("syscribe-help-examples-{}", std::process::id()));
    std::fs::create_dir_all(&tmp).unwrap();
    let root = repo_root();
    let failures = Mutex::new(Vec::<String>::new());
    let next = std::sync::atomic::AtomicUsize::new(0);
    let workers = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4).min(8);
    std::thread::scope(|s| {
        for _ in 0..workers {
            s.spawn(|| loop {
                let n = next.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                let Some(ex) = examples.get(n) else { break };
                let Plan::Run(runs) = plan(ex, &tmp, n) else { continue };
                for args in runs {
                    let out = Command::new(env!("CARGO_BIN_EXE_syscribe"))
                        .current_dir(&root)
                        .env_remove("SYSCRIBE_MODEL")
                        .arg("-m")
                        .arg(&ex.root)
                        .args(&args)
                        .stdin(std::process::Stdio::null())
                        .output()
                        .unwrap();
                    let code = out.status.code().unwrap_or(-1);
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    let stdout = String::from_utf8_lossy(&out.stdout);
                    // Only the first lines of stdout: a "not found" notice is printed up
                    // front, while report bodies legitimately quote findings such as
                    // "sourceFile … does not exist on disk".
                    let head: String = stdout.lines().take(3).collect::<Vec<_>>().join("\n");
                    // lint-docs reports unresolvable references as its findings (exit 1).
                    let is_lint = args.first().is_some_and(|c| c == "lint-docs");
                    let lint_findings = is_lint && code == 1;
                    let problem = if code != 0 && code != 2 && !lint_findings {
                        Some(format!("exit {code}: {}", stderr.lines().next().unwrap_or("").trim()))
                    } else {
                        not_found_marker(&stderr)
                            .or_else(|| if is_lint { None } else { not_found_marker(&head) })
                            .map(|m| format!("output: {m}"))
                    };
                    if let Some(p) = problem {
                        failures.lock().unwrap().push(format!("{}: `{}` — {p}", ex.origin, ex.line));
                    }
                }
            });
        }
    });
    let _ = std::fs::remove_dir_all(&tmp);
    let mut f = failures.into_inner().unwrap();
    f.sort();
    assert!(f.is_empty(), "{} help example(s) fail against their bundled model:\n{}", f.len(), f.join("\n"));
}

#[test]
fn shell_words_handles_quotes_pipes_and_comments() {
    assert_eq!(shell_words(r#"find "brake release" --json | head"#), vec!["find", "brake release", "--json"]);
    assert_eq!(shell_words("validate   # the gate"), vec!["validate"]);
    assert_eq!(shell_words("export > out.json"), vec!["export"]);
    assert_eq!(shell_words("applies-when X --set 'A or B'"), vec!["applies-when", "X", "--set", "A or B"]);
}
