//! Enforcing the project's agent policy on the writing commands (GH #442, `ADR-SYS-AGENTPOLICY-001`).
//!
//! The policy ([`syscribe_model::agent_policy`]) applies to a change made by an *agent*: one that names an actor
//! (`--by`) and does not declare itself a person (`--as-human`). When `require_by` is set, a model-writing
//! command that names neither is refused. Protected paths are enforced twice — an up-front check in the
//! single-element writers for a clear message, and a guard in the write layer as a backstop for every other
//! command — and are refused even under `--dry-run`, so a dry run reports what a real run would be refused.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use syscribe_model::agent_policy::{self, Policy, Refusal};
use syscribe_model::element::RawElement;

static POLICY: OnceLock<Option<Policy>> = OnceLock::new();
static ROOTS: OnceLock<(PathBuf, Option<PathBuf>)> = OnceLock::new();

pub fn policy() -> Option<&'static Policy> {
    POLICY.get().and_then(|p| p.as_ref())
}

/// This invocation is an agent's: it names an actor and has not declared a person.
pub fn agent_active() -> bool {
    let g = crate::provenance::get();
    g.by.is_some() && !g.as_human
}

/// Does `command args…` change the model? (`require_by` applies to these.)
pub fn is_model_write(command: &str, args: &[String]) -> bool {
    let has = |f: &str| args.iter().any(|a| a == f);
    match command {
        "set" | "claim" | "release" | "move" | "import-reqif" | "ingest-results" | "bind" | "new" => true,
        "applies-when" => has("--set") || has("--clear"),
        "scaffold-gherkin" => has("--fix"),
        "baseline" => args.first().map(String::as_str) == Some("create"),
        "suspect" => args.first().map(String::as_str) == Some("accept"),
        "trace-code" => has("--write-baseline"),
        _ => false,
    }
}

/// The path spelled the ways a policy glob may name it: relative to the repository and to the model root.
fn candidates(path: &Path) -> Vec<String> {
    let Some((model_root, git_root)) = ROOTS.get() else { return vec![path.to_string_lossy().into_owned()] };
    let mut out = Vec::new();
    if let Some(root) = git_root {
        if let Some(r) = crate::impactfiles::to_git_rel(root, path) {
            out.push(r);
        }
    }
    let abs = crate::impactfiles::lenient_path(path);
    let m = std::fs::canonicalize(model_root).unwrap_or_else(|_| model_root.clone());
    if let Ok(r) = abs.strip_prefix(&m) {
        out.push(r.to_string_lossy().replace('\\', "/"));
    }
    if out.is_empty() {
        out.push(path.to_string_lossy().into_owned());
    }
    out
}

/// Load the policy and arm the enforcement for this invocation. `Err` is a refusal (exit 1).
pub fn init(model_root: &Path, command: &str, args: &[String]) -> Result<(), String> {
    let policy = agent_policy::load(model_root);
    let _ = POLICY.set(policy);
    // Questions reserved to a person (GH #457) are guarded in the write layer for an agent, policy or not:
    // a question's own `answerBy: human` reserves it without any `[agents]` table.
    if agent_active() {
        syscribe_model::fsio::set_content_guard(|path, new| {
            if path.extension().is_none_or(|e| e != "md") {
                return Ok(());
            }
            let old = std::fs::read_to_string(path).ok();
            agent_policy::check_answer_content(policy_ref(), old.as_deref(), &String::from_utf8_lossy(new)).map_err(|r| r.to_string())
        });
    }
    let Some(p) = policy_ref() else { return Ok(()) };
    let g = crate::provenance::get();
    if p.require_by && is_model_write(command, args) && g.by.is_none() && !g.as_human {
        return Err(Policy::actor_required().to_string());
    }
    // Two things are enforced in the write layer, which every writer goes through, so a command added later
    // is covered without anyone remembering to list it: protected paths (for an agent), and — under
    // `require_by` — that a write to the model names its actor. The up-front checks give a clearer message
    // and refuse before any work is done; this is what makes them complete.
    let agent = agent_active();
    let needs_actor = p.require_by && g.by.is_none() && !g.as_human;
    if (agent && !p.protected_paths.is_empty()) || needs_actor {
        let git_root = std::fs::canonicalize(model_root).ok().and_then(|a| syscribe_model::config::detect_git_root(&a));
        let _ = ROOTS.set((model_root.to_path_buf(), git_root));
        let model_abs = std::fs::canonicalize(model_root).unwrap_or_else(|_| model_root.to_path_buf());
        let sidecar_writer = Some(command.to_string());
        syscribe_model::fsio::set_guard(move |path| {
            if needs_actor && is_model_file(&model_abs, path) {
                return Err(Policy::actor_required().to_string());
            }
            if agent {
                // The evidence sidecar stays protected from every writer but the commands the policy names.
                if sidecar_writer.as_deref().is_some_and(|c| p.sidecar_writers.iter().any(|w| w == c)) && is_sidecar_file(&model_abs, path) {
                    return Ok(());
                }
                p.check_path(&candidates(path)).map_err(|r| r.to_string())?;
            }
            Ok(())
        });
    }
    Ok(())
}

/// A file under `<model>/.syscribe/` — the verdict/evidence sidecars — but not the `scripts/` tooling, which no
/// sidecar writer touches and which is executable code.
fn is_sidecar_file(model_abs: &Path, path: &Path) -> bool {
    let abs = crate::impactfiles::lenient_path(path);
    abs.strip_prefix(model_abs.join(".syscribe")).is_ok_and(|rel| rel.components().next().is_some_and(|c| c.as_os_str() != "scripts"))
}

/// A write inside the model directory, other than the tool's own regenerable cache.
fn is_model_file(model_abs: &Path, path: &Path) -> bool {
    let abs = crate::impactfiles::lenient_path(path);
    abs.strip_prefix(model_abs).is_ok_and(|rel| {
        let c: Vec<_> = rel.components().map(|c| c.as_os_str().to_string_lossy().into_owned()).collect();
        !(c.first().map(String::as_str) == Some(".syscribe") && c.get(1).map(String::as_str) == Some("cache"))
    })
}

fn policy_ref() -> Option<&'static Policy> {
    policy()
}

fn refuse(r: Refusal) -> ! {
    eprintln!("Refusing: {r}");
    std::process::exit(1)
}

/// An agent may not write this element's file.
pub fn check_path(elem: &RawElement) {
    if let (Some(p), true) = (policy(), agent_active()) {
        if let Err(r) = p.check_path(&candidates(Path::new(&elem.file_path))) {
            refuse(r);
        }
    }
}

/// An agent may not write any of these files (a `move` rewrites several).
pub fn check_paths(paths: &[PathBuf]) {
    if let (Some(p), true) = (policy(), agent_active()) {
        for path in paths {
            if let Err(r) = p.check_path(&candidates(path)) {
                refuse(r);
            }
        }
    }
}

/// An agent may not answer a question reserved to a person (GH #457): `new_content` is what the element's
/// file would become. Checked up front so a dry run is refused exactly as the real write would be.
pub fn check_answers(elem: &RawElement, new_content: &str) {
    if !agent_active() {
        return;
    }
    let old = std::fs::read_to_string(&elem.file_path).ok();
    if let Err(r) = agent_policy::check_answer_content(policy(), old.as_deref(), new_content) {
        refuse(r);
    }
}

/// An agent may not set this status, and may not mark an item done until `require_before_done` holds.
pub fn check_status(model_root: &Path, elements: &[RawElement], elem: &RawElement, status: &str) {
    let (Some(p), true) = (policy(), agent_active()) else { return };
    let ty = elem.frontmatter.element_type.as_ref().map(syscribe_model::baseline::element_type_name).unwrap_or_default();
    if let Err(r) = p.check_status(&ty, status) {
        refuse(r);
    }
    let config = syscribe_model::config::ValidateConfig::with_model_root(model_root);
    if let Err(r) = p.check_before_done(elements, &config, &elem.qualified_name, status) {
        refuse(r);
    }
}

/// The findings `validate` adds for the policy: malformed `[agents]` entries and, from git history, agent
/// commits that changed a protected path or set a human-only status, and (`W323`) reserved questions whose
/// answer an agent wrote — the last also without a policy, for a question that says `answerBy: human`.
pub fn audit_findings(model_root: &Path, elements: &[RawElement]) -> Vec<syscribe_model::validator::Finding> {
    let p = agent_policy::load(model_root);
    let mut out = p.as_ref().map(|p| agent_policy::audit(model_root, p)).unwrap_or_default();
    out.extend(agent_policy::answer_audit(model_root, elements, p.as_ref()));
    out
}

/// `policy [--json]` — the effective policy and how this invocation is treated.
pub fn cmd_policy(model_root: &Path, args: &[String]) -> i32 {
    let json = args.iter().any(|a| a == "--json");
    let Some(p) = agent_policy::load(model_root) else {
        if json {
            println!("{}", serde_json::json!({ "schemaVersion": "1.0", "configured": false }));
        } else {
            println!("No [agents] policy is configured in {}/.syscribe.toml.", model_root.display());
        }
        return 0;
    };
    let g = crate::provenance::get();
    let treated = if g.as_human {
        "as a person (--as-human)"
    } else if g.by.is_some() {
        "as an agent (--by is given)"
    } else {
        "as a person (no --by)"
    };
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "schemaVersion": "1.0",
                "configured": true,
                "protectedPaths": p.protected_paths,
                "humanOnlyStatus": p.human_only_status,
                "humanOnlyAnswers": p.human_only_answers,
                "requireBeforeDone": p.require_before_done,
                "requireBy": p.require_by,
                "actors": p.actors,
                "auditLimit": p.audit_limit,
                "problems": p.problems,
                "thisInvocation": treated,
            }))
            .unwrap()
        );
        return 0;
    }
    println!("# Agent policy\n");
    let rules = p.describe();
    if rules.is_empty() {
        println!("(no rules set)");
    }
    for r in &rules {
        println!("- {r}");
    }
    if !p.actors.is_empty() {
        println!("\nGit authors counted as agents in the audit: {}.", p.actors.join(", "));
    }
    for m in &p.problems {
        println!("\nignored: {m}");
    }
    println!("\nThis invocation is treated {treated}.");
    0
}
