//! `syscribe next` and `syscribe done <PI>` (GH #485, `REQ-TRS-NEXT-001`, `REQ-TRS-DONE-001`): the two calls
//! any agent harness needs around its own loop — what to work on, and whether it may stop.
//!
//! Both are read-only. They compose what the model already knows: claims, blockers, questions, the
//! agent policy, the evidence rule (`E719`/`W310`), `validate`, the stereotype gates and `trace-code`.

use std::path::Path;

use serde_json::{json, Value};
use syscribe_model::config::ValidateConfig;
use syscribe_model::element::RawElement;
use syscribe_model::resolver::Resolver;
use syscribe_model::validator::{validate_with_config, Severity};
use syscribe_model::workflow::{self, label};

pub const NEXT_USAGE: &str = "Usage: syscribe -m <root> next [--limit N] [--json]";
pub const DONE_USAGE: &str = "Usage: syscribe -m <root> done <PI-id> [--json]";

// ── next ─────────────────────────────────────────────────────────────────────

/// The `next` report as JSON (shared with the MCP tool).
pub fn next_json(model_root: &Path, elements: &[RawElement], limit: usize) -> Value {
    let policy = syscribe_model::agent_policy::load(model_root);
    let report = workflow::next(elements, syscribe_model::agent_policy::default_human(policy.as_ref()));
    let shown: Vec<Value> = report
        .candidates
        .iter()
        .take(limit)
        .map(|c| {
            json!({
                "id": c.id, "qualifiedName": c.qualified_name, "name": c.name, "status": c.status,
                "resumable": c.resumable, "achieves": c.achieves, "hazards": c.hazards,
                "brief": format!("syscribe -m {} context {}", model_root.display(), c.id),
            })
        })
        .collect();
    json!({
        "schemaVersion": "1.0",
        "next": shown.first().cloned().unwrap_or(Value::Null),
        "candidates": shown,
        "candidateCount": report.candidates.len(),
        "skipped": report.skipped.iter().map(|s| json!({"id": s.id, "reason": s.reason})).collect::<Vec<_>>(),
    })
}

pub fn cmd_next(model_root: &Path, elements: &[RawElement], args: &[String]) -> i32 {
    let json_out = args.iter().any(|a| a == "--json");
    let mut limit = 5usize;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => {}
            "--limit" => match args.get(i + 1).and_then(|v| v.parse::<usize>().ok()).filter(|n| *n > 0) {
                Some(n) => {
                    limit = n;
                    i += 1;
                }
                None => {
                    eprintln!("next: --limit expects a positive integer\n{NEXT_USAGE}");
                    return 1;
                }
            },
            other => {
                eprintln!("next: unexpected argument '{other}'\n{NEXT_USAGE}");
                return 1;
            }
        }
        i += 1;
    }
    let doc = next_json(model_root, elements, limit);
    if json_out {
        println!("{}", serde_json::to_string_pretty(&doc).unwrap());
        return if doc["next"].is_null() { 2 } else { 0 };
    }
    let cands = doc["candidates"].as_array().cloned().unwrap_or_default();
    if cands.is_empty() {
        println!("Nothing to take: no unclaimed, unblocked PlanningItem is waiting.");
    } else {
        println!("# Next\n");
        for (k, c) in cands.iter().enumerate() {
            println!(
                "{}. {} — {}  [{}{}]",
                k + 1,
                c["id"].as_str().unwrap_or(""),
                c["name"].as_str().unwrap_or(""),
                c["status"].as_str().unwrap_or(""),
                if c["resumable"] == true { ", resumable" } else { "" }
            );
            for h in c["hazards"].as_array().into_iter().flatten() {
                println!("     ! {}", h.as_str().unwrap_or(""));
            }
        }
        let n = doc["candidateCount"].as_u64().unwrap_or(0) as usize;
        if n > cands.len() {
            println!("\n({} more; raise --limit)", n - cands.len());
        }
        println!("\nBrief:  {}", cands[0]["brief"].as_str().unwrap_or(""));
        println!("Claim:  syscribe -m {} claim {} --by <you>", model_root.display(), cands[0]["id"].as_str().unwrap_or(""));
    }
    let skipped = doc["skipped"].as_array().cloned().unwrap_or_default();
    if !skipped.is_empty() {
        println!("\n## Not offered\n");
        for s in &skipped {
            println!("- {} — {}", s["id"].as_str().unwrap_or(""), s["reason"].as_str().unwrap_or(""));
        }
    }
    if cands.is_empty() {
        2
    } else {
        0
    }
}

// ── done ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum Verdict {
    Pass,
    Fail,
    /// Not applicable (nothing configured), neither a pass nor a block.
    Skip,
    /// Satisfied, but a person must act (the policy reserves `done` for one).
    Handoff,
}

impl Verdict {
    fn name(&self) -> &'static str {
        match self {
            Verdict::Pass => "pass",
            Verdict::Fail => "fail",
            Verdict::Skip => "skip",
            Verdict::Handoff => "handoff",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Check {
    pub name: &'static str,
    pub verdict: Verdict,
    pub detail: String,
}

fn check(name: &'static str, verdict: Verdict, detail: impl Into<String>) -> Check {
    Check { name, verdict, detail: detail.into() }
}

/// Run the done gate for `item_ref`; `Err` is an unknown or non-PlanningItem reference.
/// `handoff` says why the caller cannot set `done` itself (the CLI asks the agent policy about this invocation; the MCP
/// server assumes an agent), or `None` when it may.
pub fn done_checks(
    model_root: &Path,
    elements: &[RawElement],
    config: &ValidateConfig,
    item_ref: &str,
    handoff: &dyn Fn(&RawElement) -> Option<String>,
) -> Result<(String, Vec<Check>), String> {
    let resolver = Resolver::new(elements);
    let item = resolver
        .resolve_ref(elements, item_ref)
        .filter(|e| workflow::is_planning_item(e))
        .ok_or_else(|| format!("'{item_ref}' is not a PlanningItem of the model"))?;
    let id = label(item);
    let mut out: Vec<Check> = Vec::new();

    // Steps: a composite is done when every step is.
    let kids = workflow::children(elements, item);
    let open: Vec<String> = kids.iter().filter(|k| k.frontmatter.status.as_deref() != Some("done")).map(|k| label(k)).collect();
    out.push(if kids.is_empty() {
        check("steps", Verdict::Skip, "no child items")
    } else if open.is_empty() {
        check("steps", Verdict::Pass, format!("all {} step(s) are done", kids.len()))
    } else {
        check("steps", Verdict::Fail, format!("{} step(s) not done: {}", open.len(), open.join(", ")))
    });

    // Questions: an open decision is unfinished work.
    let policy = syscribe_model::agent_policy::load(model_root);
    let default_human = syscribe_model::agent_policy::default_human(policy.as_ref());
    let open_qs: Vec<String> = syscribe_model::questions::parse(item)
        .0
        .into_iter()
        .filter(|q| q.status == "open")
        .map(|q| format!("{}{}", q.reference(), if q.effective_answer_by(default_human) == "human" { " (for a person)" } else { "" }))
        .collect();
    out.push(if open_qs.is_empty() { check("questions", Verdict::Pass, "none open") } else { check("questions", Verdict::Fail, format!("open: {}", open_qs.join(", "))) });

    // Blockers.
    let unmet = workflow::unmet_blockers(elements, &resolver, item);
    out.push(if unmet.is_empty() { check("blockers", Verdict::Pass, "none unmet") } else { check("blockers", Verdict::Fail, format!("unmet: {}", unmet.join(", "))) });

    // Evidence, validate: judge the model as it would be with the item done.
    let mut candidate: Vec<RawElement> = elements.to_vec();
    if let Some(c) = candidate.iter_mut().find(|e| e.qualified_name == item.qualified_name) {
        c.frontmatter.status = Some("done".to_string());
    }
    let findings = validate_with_config(&candidate, config).findings;
    let mine: Vec<String> = findings
        .iter()
        .filter(|f| f.file == item.file_path && matches!(f.code, "E719" | "W310"))
        .map(|f| format!("{}: {}", f.code, f.message))
        .collect();
    out.push(if mine.is_empty() { check("evidence", Verdict::Pass, "the item and its requirements are evidenced") } else { check("evidence", Verdict::Fail, mine.join("; ")) });
    // The evidence rule has its own line above; do not count it twice.
    let errors: Vec<_> = findings.iter().filter(|f| f.severity == Severity::Error && !(f.file == item.file_path && f.code == "E719")).collect();
    out.push(if errors.is_empty() {
        check("validate", Verdict::Pass, "no errors")
    } else {
        check(
            "validate",
            Verdict::Fail,
            format!("{} error(s): {}", errors.len(), errors.iter().take(3).map(|f| format!("{} {}", f.code, f.file)).collect::<Vec<_>>().join("; ")),
        )
    });

    // The stereotype gates.
    let conds = syscribe_model::tagged_values::load_gate_conditions(model_root);
    let gate = crate::query::meta_gate_hits(elements, elements.iter(), &conds)
        .map(|h| h.into_iter().map(|(c, q)| format!("{c} on {q}")).collect::<Vec<_>>())
        .and_then(|mut h| crate::covtree::coverage_gates(elements, config, model_root).map(|c| {
            h.extend(c);
            h
        }));
    out.push(match gate {
        Ok(h) if h.is_empty() => check("gates", Verdict::Pass, "no stereotype gate tripped"),
        Ok(h) => check("gates", Verdict::Fail, format!("tripped: {}", h.join("; "))),
        Err(m) => check("gates", Verdict::Fail, m),
    });

    // trace-code, when configured.
    out.push(if syscribe_model::trace_code::TraceCodeConfig::load(model_root).is_none() {
        check("trace-code", Verdict::Skip, "no [traceCode] table")
    } else {
        match crate::tracecode::analyse_local(model_root, elements, config, None) {
            Ok(r) if r.clean() => check("trace-code", Verdict::Pass, "every code unit is named or baselined"),
            Ok(r) => check("trace-code", Verdict::Fail, format!("{} new untraced unit(s), {} stale baseline entr(ies)", r.new_untraced().count(), r.stale.len())),
            Err(e) => check("trace-code", Verdict::Fail, e),
        }
    });

    // May this invocation set `done`?
    out.push(match handoff(item) {
        Some(why) => check("status", Verdict::Handoff, why),
        None => check("status", Verdict::Pass, "this invocation may set status done"),
    });
    Ok((id, out))
}

/// `ready`: no check failed. A handoff is a pass that needs a person.
pub fn ready(checks: &[Check]) -> bool {
    !checks.iter().any(|c| c.verdict == Verdict::Fail)
}

pub fn done_json(id: &str, checks: &[Check]) -> Value {
    json!({
        "schemaVersion": "1.0",
        "item": id,
        "ready": ready(checks),
        "needsPerson": checks.iter().any(|c| c.verdict == Verdict::Handoff),
        "missing": checks.iter().filter(|c| c.verdict == Verdict::Fail).map(|c| format!("{}: {}", c.name, c.detail)).collect::<Vec<_>>(),
        "checks": checks.iter().map(|c| json!({"check": c.name, "result": c.verdict.name(), "detail": c.detail})).collect::<Vec<_>>(),
    })
}

pub fn cmd_done(model_root: &Path, elements: &[RawElement], config: &ValidateConfig, args: &[String]) -> i32 {
    let json_out = args.iter().any(|a| a == "--json");
    let refs: Vec<&String> = args.iter().filter(|a| !a.starts_with("--")).collect();
    if refs.len() != 1 || args.iter().any(|a| a.starts_with("--") && a != "--json") {
        eprintln!("{DONE_USAGE}");
        return 1;
    }
    let (id, checks) = match done_checks(model_root, elements, config, refs[0], &crate::policy_gate::done_refusal) {
        Ok(r) => r,
        Err(m) => {
            eprintln!("done: {m}");
            return 1;
        }
    };
    let ok = ready(&checks);
    if json_out {
        println!("{}", serde_json::to_string_pretty(&done_json(&id, &checks)).unwrap());
        return if ok { 0 } else { 2 };
    }
    println!("# {id} — {}\n", if ok { "ready to be done" } else { "not ready" });
    for c in &checks {
        let mark = match c.verdict {
            Verdict::Pass => "ok  ",
            Verdict::Fail => "MISS",
            Verdict::Skip => "n/a ",
            Verdict::Handoff => "TURN",
        };
        println!("{mark} {:<11} {}", c.name, c.detail);
    }
    if ok {
        if checks.iter().any(|c| c.verdict == Verdict::Handoff) {
            println!("\nEverything holds; a person sets `status: done` (`syscribe -m {} set {id} status=done --as-human`).", model_root.display());
        } else {
            println!("\nEverything holds: `syscribe -m {} set {id} status=done --by <you>`.", model_root.display());
        }
    } else {
        println!("\nNot done: fix the MISS lines, then run this again.");
    }
    if ok {
        0
    } else {
        2
    }
}
