//! `safety-case` — renders the GSN (Goal Structuring Notation) safety-argument
//! tree (issues #20, #217).
//!
//! The traversal, node/status/verdict types and the completeness analysis live
//! in [`syscribe_model::safety_case`] so other renderers (diagram derivers) can
//! reuse them; this module only formats that tree as text or JSON and maps the
//! results-sidecar TestCase verdict onto the model-crate [`Verdict`].
//!
//! Read-only: reuses `Resolver` and `tc_verdict` (issue #21).

use syscribe_model::{
    element::RawElement,
    resolver::Resolver,
    results::ResultsData,
    safety_case::{
        self, BuildOptions, Completeness, GoalTree, NodeKind, SafetyCase, SafetyCaseNode, Verdict,
    },
};

use crate::query::{tc_verdict, TcVerdict};

/// Render the safety-case view. `goal_filter` is an optional SG id/qname; `format`
/// picks the GSN-style text tree (default), the JSON document, or a DOT / Mermaid
/// GSN diagram.
/// `no_implicit` suppresses the implicit SafetyGoal→Requirement→TestCase fold-in.
/// `sidecar_loaded` indicates whether a results sidecar was ingested (suppresses the [unknown] footnote).
///
/// Returns the process exit code: 0 on success, 1 when `goal_filter` names no
/// `SafetyGoal` (the message goes to stderr).
pub fn cmd_safety_case(
    elements: &[RawElement],
    resolver: &Resolver,
    goal_filter: &str,
    results: Option<&ResultsData>,
    format: Format,
    no_implicit: bool,
    sidecar_loaded: bool,
) -> i32 {
    // REQ-TRS-LINKTYPE-006 — a `coverage = true` user-defined link extending
    // verifies/derivedFrom counts as that base link in the argument tree
    // (read-only report). Same element order, so `resolver` stays valid.
    let cov = syscribe_model::link_types::coverage_view(elements, &syscribe_model::link_types::active());
    let elements: &[RawElement] = &cov;

    let verdict_of = |tc: &RawElement| match tc_verdict(tc, results) {
        TcVerdict::Pass => Verdict::Pass,
        TcVerdict::Fail => Verdict::Fail,
        TcVerdict::Unknown => Verdict::Unknown,
    };
    let case = safety_case::build(
        elements,
        resolver,
        goal_filter,
        BuildOptions { include_implicit: !no_implicit },
        &verdict_of,
    );

    if case.goals.is_empty() {
        if goal_filter.is_empty() {
            println!("No SafetyGoal elements found — nothing to render.");
            return 0;
        }
        eprintln!("Error: no SafetyGoal matching '{}' found.", goal_filter);
        return 1;
    }

    // Retained failure evidence of the failing TestCases in the tree (GH #258).
    let mut failure_details: Vec<(String, syscribe_model::results::FailureNote)> = Vec::new();
    let mut seen_tc: std::collections::HashSet<String> = std::collections::HashSet::new();
    for g in &case.goals {
        g.root.walk(&mut |n| {
            if n.verdict == Some(Verdict::Fail) && seen_tc.insert(n.id.clone()) {
                if let Some(tc) = n.qualified_name.as_deref().and_then(|q| elements.iter().find(|e| e.qualified_name == q)) {
                    for note in syscribe_model::results::failure_notes(tc, results) {
                        failure_details.push((n.id.clone(), note));
                    }
                }
            }
        });
    }
    match format {
        Format::Json => render_json(&case, sidecar_loaded, &failure_details),
        Format::Text => render_text(&case, sidecar_loaded, &failure_details),
        // GH #223: the same tree as a GSN diagram (the node shapes, status
        // tones, undeveloped diamonds and test verdicts of the web view).
        Format::Dot | Format::Mermaid => {
            let name = if goal_filter.is_empty() { "Safety case" } else { goal_filter };
            let graph = syscribe_model::vis::derive::safety_case::graph_of_case(&case, name);
            if format == Format::Dot {
                print!("{}", syscribe_model::vis::render_dot(&graph));
            } else if let Some(text) = syscribe_model::vis::render_mermaid(&graph, &|_| None) {
                print!("{text}");
            }
        }
    }
    0
}

/// The output format of `safety-case`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// The GSN-style text tree (the default).
    Text,
    Json,
    /// Graphviz DOT of the GSN diagram.
    Dot,
    /// A Mermaid flowchart of the GSN diagram.
    Mermaid,
}

impl Format {
    pub const VALUES: &'static [&'static str] = &["text", "json", "dot", "mermaid"];

    pub fn parse(s: &str) -> Option<Format> {
        Some(match s {
            "text" => Format::Text,
            "json" => Format::Json,
            "dot" => Format::Dot,
            "mermaid" => Format::Mermaid,
            _ => return None,
        })
    }
}

// ── Text rendering ──────────────────────────────────────────────────────────

fn node_line(n: &SafetyCaseNode) -> String {
    let mut s = match n.kind {
        NodeKind::Unresolved => format!("[unresolved] {}", n.id),
        _ => format!("[{}] {} — {}", n.kind.label(), n.id, n.title),
    };
    if let Some(k) = n.decomposition_kind.as_deref() {
        s.push_str(&format!(" [decomposition: {k}]"));
    }
    if let Some(v) = n.verdict {
        s.push_str(&format!(" [{}]", v.as_str()));
    }
    if n.implicit {
        s.push_str(" (implicit)");
    }
    if n.cycle {
        s.push_str(" [CYCLE]");
    }
    if n.undeveloped {
        s.push_str(" [UNDEVELOPED]");
    }
    s
}

/// Print a goal's subtree. A Requirement or Argument node that has children and was already
/// printed earlier in this goal's tree is shown once more as `… (see above)` without its
/// children (GH #247), as `connectivity` does with `(*)`. Leaves (a TestCase) always print.
fn print_children(children: &[SafetyCaseNode], indent: &str, expanded: &mut std::collections::HashSet<String>) {
    let total = children.len();
    for (i, c) in children.iter().enumerate() {
        let last = i + 1 == total;
        let conn = if last { "└──" } else { "├──" };
        let key = format!("{}|{}", c.kind.label(), c.id);
        let repeat = !c.children.is_empty() && !expanded.insert(key);
        if repeat {
            println!("{}{} {} (see above)", indent, conn, node_line(c));
            continue;
        }
        println!("{}{} {}", indent, conn, node_line(c));
        let child_indent = format!("{}{}", indent, if last { "    " } else { "│   " });
        print_children(&c.children, &child_indent, expanded);
    }
}

fn render_text(case: &SafetyCase, sidecar_loaded: bool, failure_details: &[(String, syscribe_model::results::FailureNote)]) {
    for g in &case.goals {
        println!(
            "[SafetyGoal] {} — {} [{}]{}",
            g.root.id,
            g.root.title,
            g.verdict.as_str().to_uppercase(),
            if g.root.undeveloped { " [UNDEVELOPED]" } else { "" }
        );
        print_children(&g.root.children, "", &mut std::collections::HashSet::new());
        println!();
    }
    print_completeness(&case.completeness);
    if !failure_details.is_empty() {
        println!("Failure details:");
        for (tc, n) in failure_details {
            let msg = n.message.as_deref().map(|m| format!(" — {}", m.split_whitespace().collect::<Vec<_>>().join(" "))).unwrap_or_default();
            let time = n.time.map(|t| format!(" ({t}s)")).unwrap_or_default();
            println!("  {tc}: {}{msg}{time}", n.function);
        }
    }
    if case.any_unknown() && !sidecar_loaded {
        println!("(verdicts unknown — run `syscribe ingest-results` to populate)");
    }
}

fn print_completeness(c: &Completeness) {
    println!("Completeness:");
    println!(
        "  goals: {} ({} supported, {} incomplete, {} failing)",
        c.goals, c.goals_supported, c.goals_incomplete, c.goals_failing
    );
    println!(
        "  requirements: {} ({} leaf without a verifying test)",
        c.requirements, c.requirements_without_tests
    );
    println!(
        "  test cases: {} ({} pass, {} fail, {} unknown)",
        c.testcases, c.tests_pass, c.tests_fail, c.tests_unknown
    );
    println!("  undeveloped nodes: {}", c.undeveloped_nodes);
    for id in &c.undeveloped_ids {
        println!("    - {id}");
    }
    if c.unresolved_refs > 0 || c.cycles > 0 {
        println!("  unresolved references: {}  cycles: {}", c.unresolved_refs, c.cycles);
    }
}

// ── JSON rendering ──────────────────────────────────────────────────────────

fn completeness_json(c: &Completeness) -> serde_json::Value {
    serde_json::json!({
        "goals": c.goals,
        "goalsSupported": c.goals_supported,
        "goalsIncomplete": c.goals_incomplete,
        "goalsFailing": c.goals_failing,
        "requirements": c.requirements,
        "requirementsWithoutTests": c.requirements_without_tests,
        "testCases": c.testcases,
        "testsPass": c.tests_pass,
        "testsFail": c.tests_fail,
        "testsUnknown": c.tests_unknown,
        "undevelopedNodes": c.undeveloped_nodes,
        "undevelopedIds": c.undeveloped_ids,
        "unresolvedRefs": c.unresolved_refs,
        "cycles": c.cycles,
    })
}

/// A node as JSON. Children are bucketed by kind (`arguments`, `requirements`,
/// `testCases`, `assumptions`, `other`) — the long-standing shape — alongside
/// the status fields.
fn node_json(n: &SafetyCaseNode) -> serde_json::Value {
    let mut arguments = Vec::new();
    let mut requirements = Vec::new();
    let mut test_cases = Vec::new();
    let mut assumptions = Vec::new();
    let mut other = Vec::new();
    for c in &n.children {
        match &c.kind {
            k if k.is_argument() => arguments.push(node_json(c)),
            NodeKind::Requirement => requirements.push(node_json(c)),
            NodeKind::TestCase => test_cases.push(node_json(c)),
            NodeKind::AssumptionOfUse => {
                assumptions.push(serde_json::json!({ "id": c.id, "title": c.title }))
            }
            NodeKind::Unresolved => other.push(serde_json::json!({ "ref": c.id, "resolved": false })),
            NodeKind::Other(t) => other.push(serde_json::json!({ "id": c.id, "kind": t })),
            _ => {}
        }
    }
    let mut v = serde_json::json!({
        "id": n.id,
        "title": n.title,
        "status": n.status.as_str(),
        "undeveloped": n.undeveloped,
    });
    let o = v.as_object_mut().unwrap();
    match &n.kind {
        k if k.is_argument() => {
            o.insert("argumentType".into(), serde_json::json!(k.label()));
        }
        NodeKind::TestCase => {
            o.insert("verdict".into(), serde_json::json!(n.verdict.unwrap_or(Verdict::Unknown).as_str()));
        }
        NodeKind::Requirement => {
            o.insert("implicit".into(), serde_json::json!(n.implicit));
            if let Some(k) = &n.decomposition_kind {
                o.insert("decompositionKind".into(), serde_json::json!(k));
            }
        }
        _ => {}
    }
    if n.cycle {
        o.insert("cycle".into(), serde_json::json!(true));
    }
    if n.kind != NodeKind::TestCase {
        o.insert("arguments".into(), serde_json::Value::Array(arguments));
        o.insert("requirements".into(), serde_json::Value::Array(requirements));
        o.insert("testCases".into(), serde_json::Value::Array(test_cases));
        o.insert("assumptions".into(), serde_json::Value::Array(assumptions));
        o.insert("other".into(), serde_json::Value::Array(other));
    }
    v
}

fn goal_json(g: &GoalTree) -> serde_json::Value {
    let mut v = node_json(&g.root);
    let o = v.as_object_mut().unwrap();
    o.insert("verdict".into(), serde_json::json!(g.verdict.as_str()));
    o.insert("completeness".into(), completeness_json(&g.completeness));
    // A goal has no direct TestCases / other evidence; keep the legacy keys only.
    o.remove("testCases");
    o.remove("other");
    v
}

fn render_json(case: &SafetyCase, sidecar_loaded: bool, failure_details: &[(String, syscribe_model::results::FailureNote)]) {
    let goals: Vec<serde_json::Value> = case.goals.iter().map(goal_json).collect();
    let mut doc = serde_json::json!({
        "goals": goals,
        "completeness": completeness_json(&case.completeness),
        "failureDetails": failure_details.iter().map(|(tc, n)| serde_json::json!({"testCase": tc, "function": n.function, "message": n.message, "time": n.time})).collect::<Vec<_>>(),
    });
    if case.any_unknown() && !sidecar_loaded {
        doc.as_object_mut().unwrap().insert("verdictsUnknown".into(), serde_json::json!(true));
    }
    println!("{}", serde_json::to_string_pretty(&doc).unwrap());
}
