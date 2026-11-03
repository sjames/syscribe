//! `check-staged` — validate only what is staged, for a pre-commit hook (GH #445,
//! `REQ-TRS-CHECKSTAGED-001`).
//!
//! The staged model files, plus the elements that directly reference them, are the *touched* set.
//! The same checks as `validate` run over the model — unprojected, and under each Configuration a
//! touched element is active in — and only findings on touched elements are reported. When the
//! model directory has unstaged or untracked changes the check runs on an export of the index, so it
//! judges what would be committed; a deletion or rename also reports every finding it introduced
//! (a dependant left dangling), compared with `HEAD`.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use syscribe_model::config::{detect_git_root, ValidateConfig};
use syscribe_model::element::{ElementType, RawElement};
use syscribe_model::validator::{validate_with_config, Finding, Severity};

const MARKER: &str = "# syscribe:check-staged";

fn git(root: &Path, args: &[&str]) -> Result<String, String> {
    let o = std::process::Command::new("git").arg("-C").arg(root).args(args).output().map_err(|e| format!("cannot run git: {e}"))?;
    if !o.status.success() {
        return Err(format!("git {}: {}", args.join(" "), String::from_utf8_lossy(&o.stderr).trim()));
    }
    Ok(String::from_utf8_lossy(&o.stdout).into_owned())
}

/// One staged path and its change letter (`A`, `C`, `M`, `D`, `R`, …).
struct Staged {
    status: char,
    path: String,
}

fn staged_files(git_root: &Path, model_rel: &str) -> Result<Vec<Staged>, String> {
    let mut args = vec!["diff", "--cached", "--name-status", "-z", "--no-renames"];
    if !model_rel.is_empty() {
        args.push("--");
        args.push(model_rel);
    }
    let out = git(git_root, &args)?;
    let mut parts = out.split('\0').filter(|s| !s.is_empty());
    let mut v = Vec::new();
    while let (Some(status), Some(path)) = (parts.next(), parts.next()) {
        v.push(Staged { status: status.chars().next().unwrap_or('M'), path: path.to_string() });
    }
    Ok(v)
}

/// Does the model directory have changes that are not staged (modified, deleted or untracked files)?
fn model_dir_is_dirty(git_root: &Path, model_rel: &str) -> Result<bool, String> {
    let mut args = vec!["status", "--porcelain=v1", "-z", "--untracked-files=all"];
    if !model_rel.is_empty() {
        args.push("--");
        args.push(model_rel);
    }
    let out = git(git_root, &args)?;
    Ok(out.split('\0').filter(|l| l.len() > 2).any(|l| l.starts_with("??") || l.as_bytes()[1] != b' '))
}

fn severity_word(s: &Severity) -> &'static str {
    match s {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Info => "info",
    }
}

/// Write the pre-commit hook. Refuses to overwrite a hook it did not write.
fn install_hook(git_root: &Path, model_rel: &str) -> i32 {
    let path = match git(git_root, &["rev-parse", "--git-path", "hooks/pre-commit"]) {
        Ok(p) => {
            let p = PathBuf::from(p.trim());
            if p.is_absolute() { p } else { git_root.join(p) }
        }
        Err(m) => {
            eprintln!("check-staged: {m}");
            return 1;
        }
    };
    if let Ok(existing) = std::fs::read_to_string(&path) {
        if !existing.contains(MARKER) {
            eprintln!(
                "check-staged: {} already exists and was not installed by syscribe; add this line to it instead:\n\n  syscribe -m '{}' check-staged\n",
                path.display(),
                model_rel_or_dot(model_rel)
            );
            return 1;
        }
    }
    let root = model_rel_or_dot(model_rel).replace('\'', "'\\''");
    let script = format!(
        "#!/bin/sh\n{MARKER} (installed by `syscribe check-staged --install-hook`; safe to delete)\n\
         if ! command -v syscribe >/dev/null 2>&1; then\n  echo \"syscribe not found on PATH; skipping the staged-model check\" >&2\n  exit 0\nfi\n\
         cd \"$(git rev-parse --show-toplevel)\" || exit 0\nexec syscribe -m '{root}' check-staged\n"
    );
    if let Some(dir) = path.parent() {
        if let Err(e) = std::fs::create_dir_all(dir) {
            eprintln!("check-staged: cannot create {}: {e}", dir.display());
            return 1;
        }
    }
    if let Err(e) = std::fs::write(&path, script) {
        eprintln!("check-staged: cannot write {}: {e}", path.display());
        return 1;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755));
    }
    println!("Installed the pre-commit hook at {}.", path.display());
    0
}

fn model_rel_or_dot(rel: &str) -> &str {
    if rel.is_empty() { "." } else { rel }
}

/// `args` are the tokens after `check-staged`. Returns the exit code: `0` clean, `1` errors or a usage
/// problem, `2` a warning gated by `--deny`/`--warnings-as-errors`/`--max-warnings`.
pub fn cmd_check_staged(model_root: &Path, args: &[String]) -> i32 {
    let json = args.iter().any(|a| a == "--json");
    let fix_hints = args.iter().any(|a| a == "--fix-hints");
    let skip_configs = args.iter().any(|a| a == "--skip-configs");
    let gate = crate::parse_gate_options(args);

    let abs = match std::fs::canonicalize(model_root) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("check-staged: {}: {e}", model_root.display());
            return 1;
        }
    };
    let Some(git_root) = detect_git_root(&abs) else {
        eprintln!("check-staged: {} is not inside a git repository", model_root.display());
        return 1;
    };
    let model_rel = abs.strip_prefix(&git_root).unwrap_or(Path::new("")).to_string_lossy().replace('\\', "/");
    if args.iter().any(|a| a == "--install-hook") {
        return install_hook(&git_root, &model_rel);
    }

    let staged = match staged_files(&git_root, &model_rel) {
        Ok(s) => s,
        Err(m) => {
            eprintln!("check-staged: {m}");
            return 1;
        }
    };
    if staged.is_empty() {
        if json {
            println!("{}", serde_json::json!({ "schemaVersion": "1.0", "stagedFiles": [], "checked": [], "findings": [], "ok": true }));
        } else {
            println!("check-staged: no staged model files — nothing to check.");
        }
        return 0;
    }

    // Judge what would be committed: the working tree when it matches the index, else an export of the index.
    let dirty = match model_dir_is_dirty(&git_root, &model_rel) {
        Ok(d) => d,
        Err(m) => {
            eprintln!("check-staged: {m}");
            return 1;
        }
    };
    let exported;
    let (root, tree_base): (PathBuf, PathBuf) = if dirty {
        exported = match syscribe_model::gitrev::materialize_index(&abs) {
            Ok(x) => x,
            Err(m) => {
                eprintln!("check-staged: {m}");
                return 1;
            }
        };
        let base = if model_rel.is_empty() { exported.root.clone() } else { exported.root.ancestors().nth(model_rel.split('/').count()).unwrap_or(&exported.root).to_path_buf() };
        (exported.root.clone(), base)
    } else {
        (abs.clone(), git_root.clone())
    };
    let elements = match syscribe_model::walker::walk_model(&root) {
        Ok(e) => e,
        Err(err) => {
            eprintln!("check-staged: cannot load the model: {err}");
            return 1;
        }
    };
    let config = ValidateConfig::with_model_root(&root);

    // git-root-relative path of an element's file, in whichever tree was loaded.
    let rel_of = |file_path: &str| -> Option<String> {
        let p = crate::impactfiles::lenient_path(Path::new(file_path));
        let base = std::fs::canonicalize(&tree_base).unwrap_or_else(|_| tree_base.clone());
        p.strip_prefix(&base).ok().map(|r| r.to_string_lossy().replace('\\', "/"))
    };

    let staged_set: HashSet<&str> = staged.iter().map(|s| s.path.as_str()).collect();
    let mut config_changed = false;
    for s in &staged {
        let name = Path::new(&s.path).file_name().and_then(|n| n.to_str()).unwrap_or("");
        let is_element_file = elements.iter().any(|e| rel_of(&e.file_path).as_deref() == Some(s.path.as_str()));
        if !is_element_file && (name.starts_with('.') || s.path.contains("/.syscribe/")) {
            config_changed = true;
        }
    }
    let seeds: Vec<&RawElement> = if config_changed {
        elements.iter().collect()
    } else {
        elements.iter().filter(|e| rel_of(&e.file_path).is_some_and(|r| staged_set.contains(r.as_str()))).collect()
    };
    let resolver = syscribe_model::resolver::Resolver::new(&elements);
    let seed_q: Vec<String> = seeds.iter().map(|e| e.qualified_name.clone()).collect();
    let dependants = crate::impact::traverse(&elements, &resolver, &seed_q, crate::impact::Direction::Downstream, Some(1), &|_| true);
    let mut touched_q: HashSet<String> = seed_q.iter().cloned().collect();
    touched_q.extend(dependants.iter().map(|n| n.qname.clone()));
    let touched_files: HashSet<String> = elements.iter().filter(|e| touched_q.contains(&e.qualified_name)).map(|e| e.file_path.clone()).collect();

    // The unprojected model, then each Configuration a touched element is active in.
    let mut findings: Vec<Finding> = validate_with_config(&elements, &config).findings;
    let configs: Vec<&RawElement> = elements.iter().filter(|e| e.frontmatter.element_type == Some(ElementType::Configuration)).collect();
    if !skip_configs && !configs.is_empty() && syscribe_model::feature_model::has_feature_model(&elements) {
        for cfg in configs {
            let id = cfg.frontmatter.id.clone().unwrap_or_else(|| cfg.qualified_name.clone());
            let syscribe_model::projection::SelectionOutcome::Resolved(sel) = syscribe_model::projection::resolve_selection(&elements, &id) else { continue };
            let active: HashSet<String> = syscribe_model::projection::project(&elements, &sel).into_iter().map(|e| e.qualified_name).collect();
            if touched_q.iter().any(|q| active.contains(q)) {
                findings.extend(syscribe_model::projection::validate_projected(&elements, &config, &sel));
            }
        }
        syscribe_model::projection::set_current_config(None);
    }

    // A deletion or rename can leave a dependant dangling: report whatever it introduced, too.
    let removes = staged.iter().any(|s| matches!(s.status, 'D' | 'R'));
    let marks: Vec<bool> = if removes {
        let source = syscribe_model::finding_baseline::Source::Rev("HEAD".to_string());
        match syscribe_model::finding_baseline::mark_new(&source, &findings, &elements, &config, &|e, c| validate_with_config(e, c).findings) {
            Ok(m) => m,
            Err(m) => {
                eprintln!("check-staged: {m}");
                return 1;
            }
        }
    } else {
        vec![false; findings.len()]
    };
    let mut seen: HashSet<(String, String, String)> = HashSet::new();
    let mut report: Vec<Finding> = Vec::new();
    for (f, is_new) in findings.into_iter().zip(marks) {
        if (touched_files.contains(&f.file) || is_new) && seen.insert((f.code.to_string(), f.file.clone(), f.message.clone())) {
            report.push(f);
        }
    }
    report.sort_by(|a, b| (a.severity.clone() as u8, &a.file, a.code).cmp(&(b.severity.clone() as u8, &b.file, b.code)));

    let show = |f: &str| rel_of(f).unwrap_or_else(|| f.to_string());
    let errors = report.iter().filter(|f| f.severity == Severity::Error).count();
    let warns: Vec<&Finding> = report.iter().filter(|f| f.severity == Severity::Warning).collect();
    let denied: Vec<&Finding> = if gate.warnings_as_errors { warns.clone() } else { warns.iter().filter(|f| gate.deny.contains(f.code)).copied().collect() };
    let denied_infos = report.iter().filter(|f| f.severity == Severity::Info && gate.deny.contains(f.code)).count();
    let over_max = gate.max_warnings.is_some_and(|m| warns.len() > m);
    let code = if errors > 0 { 1 } else if !denied.is_empty() || denied_infos > 0 || over_max { 2 } else { 0 };

    if json {
        let items: Vec<serde_json::Value> = report
            .iter()
            .map(|f| serde_json::json!({ "code": f.code, "severity": severity_word(&f.severity), "file": show(&f.file), "message": f.message }))
            .collect();
        let mut checked: Vec<&str> = elements.iter().filter(|e| touched_q.contains(&e.qualified_name)).map(|e| e.frontmatter.id.as_deref().unwrap_or(&e.qualified_name)).collect();
        checked.sort();
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "schemaVersion": "1.0",
                "stagedFiles": staged.iter().map(|s| s.path.clone()).collect::<Vec<_>>(),
                "checked": checked,
                "judged": if dirty { "index" } else { "working tree" },
                "findings": items,
                "ok": code == 0,
            }))
            .unwrap()
        );
        return code;
    }
    println!(
        "check-staged: {} staged model file(s); {} element(s) checked ({} direct dependant(s)){}",
        staged.len(),
        touched_q.len(),
        dependants.len(),
        if dirty { "; judged on the staged content (the working tree has other changes)" } else { "" }
    );
    if report.is_empty() {
        println!("OK — no findings on the staged elements.");
        return 0;
    }
    println!("\n| Code | File | Message |\n|---|---|---|");
    for f in &report {
        println!("| {} | {} | {} |", f.code, show(&f.file), f.message);
    }
    println!("\n{errors} error(s), {} warning(s){}", warns.len(), if code == 2 { " — gated" } else { "" });
    if fix_hints {
        let mut codes: Vec<&str> = report.iter().map(|f| f.code).collect();
        codes.sort();
        codes.dedup();
        println!("\nHow to fix:");
        for c in codes {
            if let Some((text, mut cmds)) = crate::explain::describe(c) {
                // A finding's own message often names the exact fix (`scaffold-gherkin <TC> --fix`).
                for f in report.iter().filter(|f| f.code == c) {
                    for cmd in crate::explain::commands_in_message(&f.message) {
                        if !cmds.contains(&cmd) {
                            cmds.push(cmd);
                        }
                    }
                }
                println!("- {c}: {}", text.split(". ").next().unwrap_or(&text));
                for cmd in cmds {
                    println!("    syscribe help {cmd}");
                }
            }
        }
    }
    if code != 0 {
        eprintln!("check-staged: the commit should not go ahead{}", if code == 2 { " (a gated warning)" } else { "" });
    }
    code
}
