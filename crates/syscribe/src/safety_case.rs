//! `safety-case` — renders the GSN (Goal Structuring Notation) safety-argument
//! tree for issue #20.
//!
//! For each top `SafetyGoal` (or only the one named on the command line) the view
//! walks:
//!   Goal
//!     → the `Argument`s whose `supports` names it (recursing into sub-Arguments)
//!         → each Argument's `evidence` (Requirements / TestCases as leaves)
//!     → the implicit chain `SafetyGoal ← Requirement (derivedFromSafetyGoal)
//!       ← TestCase (verifies)` so the view is useful even with no Argument nodes
//!     → any `AssumptionOfUse` whose `appliesTo` names the goal or an argument
//!       under it.
//!
//! Read-only: reuses `Resolver`; reuses `tc_verdict` (issue #21) to annotate
//! TestCase leaves with their ingested verdict when a results sidecar is present.

use std::collections::HashSet;
use syscribe_model::{
    element::{ElementType, RawElement},
    resolver::Resolver,
    results::ResultsData,
};

use crate::query::{tc_verdict, TcVerdict};

/// What kind of node a resolved reference is, for prefixing.
enum NodeKind {
    Argument,
    Requirement,
    TestCase,
    Assumption,
    Other(String),
}

fn classify(elem: &RawElement) -> NodeKind {
    match elem.frontmatter.element_type {
        Some(ElementType::Argument) => NodeKind::Argument,
        Some(ElementType::Requirement) => NodeKind::Requirement,
        Some(ElementType::TestCase) => NodeKind::TestCase,
        Some(ElementType::AssumptionOfUse) => NodeKind::Assumption,
        Some(ref t) => NodeKind::Other(format!("{:?}", t)),
        None => NodeKind::Other("Unknown".into()),
    }
}

/// Best display id for an element (stable id, else qualified name).
fn disp_id(elem: &RawElement) -> &str {
    elem.frontmatter.id.as_deref().unwrap_or(&elem.qualified_name)
}

fn disp_title(elem: &RawElement) -> &str {
    elem.frontmatter.name.as_deref().unwrap_or("")
}

/// Verdict suffix for a TestCase leaf (`[pass]` / `[fail]` / `[unknown]`).
fn verdict_suffix(tc: &RawElement, results: Option<&ResultsData>) -> &'static str {
    match tc_verdict(tc, results) {
        TcVerdict::Pass => " [pass]",
        TcVerdict::Fail => " [fail]",
        TcVerdict::Unknown => " [unknown]",
    }
}

// ── Entry point ─────────────────────────────────────────────────────────────

/// Render the safety-case view. `goal_filter` is an optional SG id/qname; `json`
/// switches between the GSN-style text tree and the JSON document.
/// `no_implicit` suppresses the implicit SafetyGoal→Requirement→TestCase fold-in for all goals.
/// `sidecar_loaded` indicates whether a results sidecar was ingested (suppresses the [unknown] footnote).
pub fn cmd_safety_case(
    elements: &[RawElement],
    resolver: &Resolver,
    goal_filter: &str,
    results: Option<&ResultsData>,
    json: bool,
    no_implicit: bool,
    sidecar_loaded: bool,
) {
    // Collect the top SafetyGoals (all of them, or only the named one).
    let goals: Vec<&RawElement> = elements
        .iter()
        .filter(|e| Resolver::is_safety_goal(e))
        .filter(|e| {
            goal_filter.is_empty()
                || disp_id(e) == goal_filter
                || e.qualified_name == goal_filter
                || e.frontmatter.id.as_deref() == Some(goal_filter)
        })
        .collect();

    if goals.is_empty() {
        if goal_filter.is_empty() {
            println!("No SafetyGoal elements found — nothing to render.");
        } else {
            println!("No SafetyGoal matching '{}' found.", goal_filter);
        }
        return;
    }

    if json {
        render_json(elements, resolver, &goals, results, no_implicit, sidecar_loaded);
    } else {
        render_text(elements, resolver, &goals, results, no_implicit, sidecar_loaded);
    }
}

// ── Argument / evidence walking ─────────────────────────────────────────────

/// The Arguments whose `supports` names `target_id`/`target_qname`.
fn supporting_arguments<'a>(
    elements: &'a [RawElement],
    target_id: &str,
    target_qname: &str,
) -> Vec<&'a RawElement> {
    elements
        .iter()
        .filter(|e| Resolver::is_argument(e))
        .filter(|e| {
            e.frontmatter
                .supports
                .as_deref()
                .unwrap_or(&[])
                .iter()
                .any(|s| s == target_id || s == target_qname)
        })
        .collect()
}

/// The AssumptionsOfUse whose `appliesTo` names `target_id`/`target_qname`.
fn assumptions_for<'a>(
    elements: &'a [RawElement],
    target_id: &str,
    target_qname: &str,
) -> Vec<&'a RawElement> {
    elements
        .iter()
        .filter(|e| Resolver::is_assumption_of_use(e))
        .filter(|e| {
            e.frontmatter
                .applies_to
                .as_deref()
                .unwrap_or(&[])
                .iter()
                .any(|s| s == target_id || s == target_qname)
        })
        .collect()
}

/// Requirements directly derived from the goal (`derivedFromSafetyGoal`).
fn derived_requirements<'a>(
    elements: &'a [RawElement],
    goal_id: &str,
) -> Vec<&'a RawElement> {
    elements
        .iter()
        .filter(|e| Resolver::is_native_requirement(e))
        .filter(|e| e.frontmatter.derived_from_safety_goal.as_deref() == Some(goal_id))
        .collect()
}

/// TestCases that `verifies` the given requirement id.
fn verifying_testcases<'a>(elements: &'a [RawElement], req_id: &str) -> Vec<&'a RawElement> {
    elements
        .iter()
        .filter(|e| Resolver::is_native_testcase(e))
        .filter(|e| {
            e.frontmatter
                .verifies
                .as_deref()
                .unwrap_or(&[])
                .iter()
                .any(|v| v == req_id)
        })
        .collect()
}

// ── Text rendering ──────────────────────────────────────────────────────────

fn render_text(
    elements: &[RawElement],
    resolver: &Resolver,
    goals: &[&RawElement],
    results: Option<&ResultsData>,
    no_implicit: bool,
    sidecar_loaded: bool,
) {
    let mut any_unknown = false;

    for goal in goals {
        let gid = disp_id(goal);
        println!("[SafetyGoal] {} — {}", gid, disp_title(goal));

        let args = supporting_arguments(elements, gid, &goal.qualified_name);
        // Suppress implicit fold-in when --no-implicit OR goal has explicit Arguments.
        let suppress_implicit = no_implicit || !args.is_empty();
        let dreqs = if suppress_implicit {
            vec![]
        } else {
            derived_requirements(elements, gid)
        };
        let assumps = assumptions_for(elements, gid, &goal.qualified_name);

        let total = args.len() + dreqs.len() + assumps.len();
        let mut idx = 0usize;
        let mut guard: HashSet<String> = HashSet::new();

        for arg in &args {
            idx += 1;
            let last = idx == total;
            print_argument(elements, resolver, arg, "", last, results, &mut guard, &mut any_unknown);
        }
        for req in &dreqs {
            idx += 1;
            let last = idx == total;
            let conn = if last { "└──" } else { "├──" };
            println!("{} [evidence:Requirement] {} — {}", conn, disp_id(req), disp_title(req));
            let child_indent = if last { "    " } else { "│   " };
            let tcs = verifying_testcases(elements, disp_id(req));
            let tcn = tcs.len();
            for (i, tc) in tcs.iter().enumerate() {
                let tlast = i + 1 == tcn;
                let tconn = if tlast { "└──" } else { "├──" };
                let vs = verdict_suffix(tc, results);
                if vs == " [unknown]" { any_unknown = true; }
                println!(
                    "{}{} [evidence:TestCase] {} — {}{}",
                    child_indent, tconn, disp_id(tc), disp_title(tc), vs
                );
            }
        }
        for aou in &assumps {
            idx += 1;
            let last = idx == total;
            let conn = if last { "└──" } else { "├──" };
            println!("{} [AoU] {} — {}", conn, disp_id(aou), disp_title(aou));
        }
        println!();
    }

    if any_unknown && !sidecar_loaded {
        println!("(verdicts unknown — run `syscribe ingest-results` to populate)");
    }
}

/// Recursively print an Argument node and its evidence children.
fn print_argument(
    elements: &[RawElement],
    resolver: &Resolver,
    arg: &RawElement,
    indent: &str,
    last: bool,
    results: Option<&ResultsData>,
    guard: &mut HashSet<String>,
    any_unknown: &mut bool,
) {
    let conn = if last { "└──" } else { "├──" };
    let kind = arg.frontmatter.argument_type.as_deref().unwrap_or("claim");
    println!("{}{} [{}] {} — {}", indent, conn, kind, disp_id(arg), disp_title(arg));

    let aid = disp_id(arg).to_string();
    if !guard.insert(aid) {
        return;
    }

    let child_indent = format!("{}{}", indent, if last { "    " } else { "│   " });

    let ev: Vec<&str> = arg
        .frontmatter
        .evidence
        .as_deref()
        .unwrap_or(&[])
        .iter()
        .map(|s| s.as_str())
        .collect();
    let assumps = assumptions_for(elements, disp_id(arg), &arg.qualified_name);
    let total = ev.len() + assumps.len();
    let mut idx = 0usize;

    for r in &ev {
        idx += 1;
        let elast = idx == total;
        match resolver.resolve_ref(elements, r) {
            None => {
                let c = if elast { "└──" } else { "├──" };
                println!("{}{} [unresolved] {}", child_indent, c, r);
            }
            Some(target) => match classify(target) {
                NodeKind::Argument => {
                    print_argument(elements, resolver, target, &child_indent, elast, results, guard, any_unknown);
                }
                NodeKind::Requirement => {
                    let c = if elast { "└──" } else { "├──" };
                    println!("{}{} [evidence:Requirement] {} — {}", child_indent, c, disp_id(target), disp_title(target));
                }
                NodeKind::TestCase => {
                    let c = if elast { "└──" } else { "├──" };
                    let vs = verdict_suffix(target, results);
                    if vs == " [unknown]" { *any_unknown = true; }
                    println!(
                        "{}{} [evidence:TestCase] {} — {}{}",
                        child_indent, c, disp_id(target), disp_title(target), vs
                    );
                }
                NodeKind::Assumption => {
                    let c = if elast { "└──" } else { "├──" };
                    println!("{}{} [AoU] {} — {}", child_indent, c, disp_id(target), disp_title(target));
                }
                NodeKind::Other(t) => {
                    let c = if elast { "└──" } else { "├──" };
                    println!("{}{} [evidence:{}] {} — {}", child_indent, c, t, disp_id(target), disp_title(target));
                }
            },
        }
    }
    for aou in &assumps {
        idx += 1;
        let elast = idx == total;
        let c = if elast { "└──" } else { "├──" };
        println!("{}{} [AoU] {} — {}", child_indent, c, disp_id(aou), disp_title(aou));
    }
}

// ── JSON rendering ──────────────────────────────────────────────────────────

fn render_json(
    elements: &[RawElement],
    resolver: &Resolver,
    goals: &[&RawElement],
    results: Option<&ResultsData>,
    no_implicit: bool,
    sidecar_loaded: bool,
) {
    let mut any_unknown = false;

    let goals_json: Vec<serde_json::Value> = goals
        .iter()
        .map(|goal| {
            let gid = disp_id(goal);
            let args = supporting_arguments(elements, gid, &goal.qualified_name);
            let suppress_implicit = no_implicit || !args.is_empty();
            let mut guard: HashSet<String> = HashSet::new();
            let args_json: Vec<serde_json::Value> = args
                .iter()
                .map(|a| argument_json(elements, resolver, a, results, &mut guard, &mut any_unknown))
                .collect();

            // Implicit fold-in: derived requirements with their verifying tests.
            let reqs_json: Vec<serde_json::Value> = if suppress_implicit {
                vec![]
            } else {
                derived_requirements(elements, gid)
                    .iter()
                    .map(|req| {
                        let tcs: Vec<serde_json::Value> = verifying_testcases(elements, disp_id(req))
                            .iter()
                            .map(|tc| {
                                let v = testcase_json(tc, results);
                                if v["verdict"] == "unknown" { any_unknown = true; }
                                v
                            })
                            .collect();
                        serde_json::json!({
                            "id": disp_id(req),
                            "title": disp_title(req),
                            "testCases": tcs,
                        })
                    })
                    .collect()
            };

            let assumps_json: Vec<serde_json::Value> = assumptions_for(elements, gid, &goal.qualified_name)
                .iter()
                .map(|a| serde_json::json!({ "id": disp_id(a), "title": disp_title(a) }))
                .collect();

            serde_json::json!({
                "id": gid,
                "title": disp_title(goal),
                "arguments": args_json,
                "requirements": reqs_json,
                "assumptions": assumps_json,
            })
        })
        .collect();

    let mut doc = serde_json::json!({ "goals": goals_json });
    if any_unknown && !sidecar_loaded {
        doc.as_object_mut().unwrap().insert("verdictsUnknown".into(), serde_json::json!(true));
    }
    println!("{}", serde_json::to_string_pretty(&doc).unwrap());
}

fn argument_json(
    elements: &[RawElement],
    resolver: &Resolver,
    arg: &RawElement,
    results: Option<&ResultsData>,
    guard: &mut HashSet<String>,
    any_unknown: &mut bool,
) -> serde_json::Value {
    let kind = arg.frontmatter.argument_type.clone().unwrap_or_else(|| "claim".into());
    let aid = disp_id(arg).to_string();

    if !guard.insert(aid.clone()) {
        return serde_json::json!({
            "id": aid, "argumentType": kind, "title": disp_title(arg), "cycle": true,
        });
    }

    let mut sub_args: Vec<serde_json::Value> = Vec::new();
    let mut reqs: Vec<serde_json::Value> = Vec::new();
    let mut tcs: Vec<serde_json::Value> = Vec::new();
    let mut others: Vec<serde_json::Value> = Vec::new();

    for r in arg.frontmatter.evidence.as_deref().unwrap_or(&[]) {
        match resolver.resolve_ref(elements, r) {
            None => others.push(serde_json::json!({ "ref": r, "resolved": false })),
            Some(target) => match classify(target) {
                NodeKind::Argument => {
                    sub_args.push(argument_json(elements, resolver, target, results, guard, any_unknown))
                }
                NodeKind::Requirement => {
                    reqs.push(serde_json::json!({ "id": disp_id(target), "title": disp_title(target) }))
                }
                NodeKind::TestCase => {
                    let v = testcase_json(target, results);
                    if v["verdict"] == "unknown" { *any_unknown = true; }
                    tcs.push(v);
                }
                NodeKind::Assumption => {
                    others.push(serde_json::json!({ "id": disp_id(target), "kind": "AssumptionOfUse" }))
                }
                NodeKind::Other(t) => {
                    others.push(serde_json::json!({ "id": disp_id(target), "kind": t }))
                }
            },
        }
    }

    let assumps: Vec<serde_json::Value> = assumptions_for(elements, disp_id(arg), &arg.qualified_name)
        .iter()
        .map(|a| serde_json::json!({ "id": disp_id(a), "title": disp_title(a) }))
        .collect();

    serde_json::json!({
        "id": aid,
        "argumentType": kind,
        "title": disp_title(arg),
        "arguments": sub_args,
        "requirements": reqs,
        "testCases": tcs,
        "assumptions": assumps,
        "other": others,
    })
}

fn testcase_json(tc: &RawElement, results: Option<&ResultsData>) -> serde_json::Value {
    let verdict = match tc_verdict(tc, results) {
        TcVerdict::Pass => "pass",
        TcVerdict::Fail => "fail",
        TcVerdict::Unknown => "unknown",
    };
    serde_json::json!({ "id": disp_id(tc), "title": disp_title(tc), "verdict": verdict })
}
