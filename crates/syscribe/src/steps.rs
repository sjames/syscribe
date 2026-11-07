//! `steps <PI>` — a PlanningItem's breakdown as a tree with a derived roll-up (GH #436,
//! `REQ-TRS-PISTEP-001`). A step is a child PlanningItem; this adds a *view*, never a field, and
//! raises no validation finding. The data function is shared with the MCP `steps` tool.

use serde::Serialize;
use std::collections::{BTreeMap, HashMap};
use syscribe_model::element::{ElementType, RawElement};
use syscribe_model::questions;
use syscribe_model::resolver::Resolver;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Node {
    pub id: String,
    pub qualified_name: String,
    pub name: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub claimed_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assigned_to: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub blocked_by: Vec<String>,
    pub evidence: usize,
    /// `evidence: - commit:` entries (GH #455): `{commit, sha?, subject?, status}`.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub commits: Vec<serde_json::Value>,
    pub open_questions: usize,
    /// A `done` leaf with no evidence entry at all (`E719` territory) — shown, not judged here.
    pub done_without_evidence: bool,
    pub children: Vec<Node>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Rollup {
    pub leaves: usize,
    pub done: usize,
    pub by_status: BTreeMap<String, usize>,
}

fn label(e: &RawElement) -> String {
    e.frontmatter.id.clone().unwrap_or_else(|| e.qualified_name.clone())
}

fn is_pi(e: &RawElement) -> bool {
    e.frontmatter.element_type == Some(ElementType::PlanningItem)
}

/// Build the tree rooted at `root` (id or qualified name). `Err` if it is not found or not a PlanningItem.
pub fn tree(elements: &[RawElement], root: &str) -> Result<Node, String> {
    let resolver = Resolver::new(elements);
    let r = resolver.resolve_ref(elements, root).ok_or_else(|| format!("element not found: {root}"))?;
    if !is_pi(r) {
        return Err(format!("{root} is not a PlanningItem"));
    }
    // parent qname -> children, in id order. `parent:` is a single scalar reference.
    let mut kids: HashMap<&str, Vec<&RawElement>> = HashMap::new();
    for e in elements.iter().filter(|e| is_pi(e)) {
        if let Some(p) = e.frontmatter.parent.as_deref().and_then(|p| resolver.resolve_ref(elements, p)) {
            kids.entry(p.qualified_name.as_str()).or_default().push(e);
        }
    }
    for v in kids.values_mut() {
        v.sort_by_key(|e| label(e));
    }
    // Every commit entry of the model, resolved in one batch against the repository holding the
    // root item's file (GH #455).
    let dir = std::path::Path::new(&r.file_path).parent().map(|d| if d.as_os_str().is_empty() { std::path::Path::new(".") } else { d });
    let commits = syscribe_model::commit_evidence::CommitCheck::new(
        dir,
        elements.iter().filter(|e| is_pi(e)).flat_map(|e| e.frontmatter.evidence.iter().flatten()),
    );
    fn build(e: &RawElement, kids: &HashMap<&str, Vec<&RawElement>>, seen: &mut Vec<String>, cc: &syscribe_model::commit_evidence::CommitCheck) -> Node {
        seen.push(e.qualified_name.clone());
        // `seen` guards against a `parent:` cycle (E712), which must not hang the view.
        let mut children: Vec<Node> = Vec::new();
        for c in kids.get(e.qualified_name.as_str()).into_iter().flatten() {
            if !seen.contains(&c.qualified_name) {
                children.push(build(c, kids, seen, cc));
            }
        }
        let commits = e
            .frontmatter
            .evidence
            .iter()
            .flatten()
            .filter_map(syscribe_model::commit_evidence::commit_value)
            .map(|raw| syscribe_model::commit_evidence::describe(cc, &raw))
            .collect();
        seen.pop();
        let evidence = e.frontmatter.evidence.as_ref().map(|v| v.len()).unwrap_or(0);
        let status = e.frontmatter.status.clone().unwrap_or_default();
        let leaf = children.is_empty();
        Node {
            id: label(e),
            qualified_name: e.qualified_name.clone(),
            name: e.frontmatter.name.clone().unwrap_or_default(),
            done_without_evidence: leaf && status == "done" && evidence == 0,
            status,
            claimed_by: e.frontmatter.claimed_by.clone(),
            assigned_to: e.frontmatter.assigned_to.clone(),
            blocked_by: e.frontmatter.blocked_by.clone().unwrap_or_default(),
            evidence,
            commits,
            open_questions: questions::parse(e).0.iter().filter(|q| q.status == "open").count(),
            children,
        }
    }
    Ok(build(r, &kids, &mut Vec::new(), &commits))
}

/// Leaves of the tree (a childless root is its own only leaf).
fn leaves<'a>(n: &'a Node, out: &mut Vec<&'a Node>) {
    if n.children.is_empty() {
        out.push(n);
    }
    n.children.iter().for_each(|c| leaves(c, out));
}

pub fn rollup(root: &Node) -> Rollup {
    let mut ls = Vec::new();
    leaves(root, &mut ls);
    let mut by_status: BTreeMap<String, usize> = BTreeMap::new();
    for l in &ls {
        *by_status.entry(l.status.clone()).or_default() += 1;
    }
    Rollup { leaves: ls.len(), done: ls.iter().filter(|l| l.status == "done").count(), by_status }
}

/// View-only notes where a composite item's status disagrees with its leaves. Never a finding.
pub fn hints(root: &Node) -> Vec<String> {
    let mut out = Vec::new();
    fn walk(n: &Node, out: &mut Vec<String>) {
        if !n.children.is_empty() {
            let mut ls = Vec::new();
            leaves(n, &mut ls);
            let unfinished = ls.iter().filter(|l| l.status != "done").count();
            if unfinished == 0 && n.status != "done" {
                out.push(format!("{}: all {} leaves are done but the item is `{}`", n.id, ls.len(), n.status));
            } else if unfinished > 0 && n.status == "done" {
                out.push(format!("{}: marked `done` but {} of {} leaves are not", n.id, unfinished, ls.len()));
            }
        }
        n.children.iter().for_each(|c| walk(c, out));
    }
    walk(root, &mut out);
    out
}

/// Remove subtrees that are entirely `done` (the root is always kept).
pub fn prune_done(n: &Node, is_root: bool) -> Option<Node> {
    let children: Vec<Node> = n.children.iter().filter_map(|c| prune_done(c, false)).collect();
    if !is_root && n.status == "done" && children.is_empty() {
        return None;
    }
    Some(Node { children, ..n.clone() })
}

fn suffix(n: &Node) -> String {
    let mut s = format!("  [{}]", n.status);
    if let Some(a) = &n.assigned_to {
        s.push_str(&format!("  @{a}"));
    }
    if let Some(c) = &n.claimed_by {
        s.push_str(&format!("  claimed by {c}"));
    }
    if !n.blocked_by.is_empty() {
        s.push_str(&format!("  blocked by {}", n.blocked_by.join(", ")));
    }
    if n.open_questions > 0 {
        s.push_str(&format!("  ?{}", n.open_questions));
    }
    if n.done_without_evidence {
        s.push_str("  no evidence");
    }
    for c in &n.commits {
        let short = c["sha"].as_str().or(c["commit"].as_str()).unwrap_or("").chars().take(10).collect::<String>();
        match c["status"].as_str() {
            Some("resolved") | Some("unchecked") => s.push_str(&format!("  commit {short}")),
            Some(other) => s.push_str(&format!("  commit {short} ({other})")),
            None => {}
        }
    }
    s
}

fn render(n: &Node, prefix: &str, last: bool, root: bool, depth: Option<usize>, level: usize, out: &mut String) {
    if root {
        out.push_str(&format!("{}  {}{}\n", n.id, n.name, suffix(n)));
    } else {
        out.push_str(&format!("{prefix}{}{}  {}{}\n", if last { "└─ " } else { "├─ " }, n.id, n.name, suffix(n)));
    }
    let child_prefix = if root { String::new() } else { format!("{prefix}{}", if last { "   " } else { "│  " }) };
    if depth.is_some_and(|d| level >= d) {
        if !n.children.is_empty() {
            out.push_str(&format!("{child_prefix}└─ … {} more below (raise --depth)\n", count(n) - 1));
        }
        return;
    }
    let len = n.children.len();
    for (i, c) in n.children.iter().enumerate() {
        render(c, &child_prefix, i + 1 == len, false, depth, level + 1, out);
    }
}

fn count(n: &Node) -> usize {
    1 + n.children.iter().map(count).sum::<usize>()
}

/// `args` are the tokens after `steps`: `<PI> [--open] [--depth N] [--json]`. Returns the exit code.
pub fn cmd_steps(elements: &[RawElement], args: &[String]) -> i32 {
    let json = args.iter().any(|a| a == "--json");
    let open = args.iter().any(|a| a == "--open");
    let depth = match args.windows(2).find(|w| w[0] == "--depth") {
        None => None,
        Some(w) => match w[1].parse::<usize>() {
            Ok(n) => Some(n),
            Err(_) => {
                eprintln!("steps: --depth expects a non-negative integer, got '{}'", w[1]);
                return 1;
            }
        },
    };
    let mut skip = false;
    let item = args.iter().find(|a| {
        if skip {
            skip = false;
            return false;
        }
        if *a == "--depth" {
            skip = true;
            return false;
        }
        !a.starts_with("--")
    });
    let Some(item) = item else {
        eprintln!("Usage: syscribe -m <root> steps <PI-id> [--open] [--depth N] [--json]");
        return 1;
    };
    let full = match tree(elements, item) {
        Ok(t) => t,
        Err(m) => {
            eprintln!("steps: {m}");
            return 1;
        }
    };
    // The roll-up and hints describe the whole breakdown, whatever the display hides.
    let roll = rollup(&full);
    let notes = hints(&full);
    let shown = if open { prune_done(&full, true).unwrap_or_else(|| full.clone()) } else { full.clone() };
    if json {
        let doc = serde_json::json!({ "schemaVersion": "1.0", "rollup": roll, "hints": notes, "tree": shown });
        println!("{}", serde_json::to_string_pretty(&doc).unwrap());
        return 0;
    }
    println!("# {} — {}  [{}]", full.id, full.name, full.status);
    if full.children.is_empty() {
        println!("\nNo child items: {} has no steps (a step is a child PlanningItem with `parent: {}`).", full.id, full.id);
    } else {
        let mut parts: Vec<String> = Vec::new();
        for s in ["in_progress", "blocked", "todo"] {
            if let Some(n) = roll.by_status.get(s) {
                parts.push(format!("{n} {}", s.replace('_', " ")));
            }
        }
        println!("\n{} of {} leaves done{}", roll.done, roll.leaves, if parts.is_empty() { String::new() } else { format!(" · {}", parts.join(" · ")) });
    }
    println!();
    let mut out = String::new();
    render(&shown, "", true, true, depth, 0, &mut out);
    print!("{out}");
    if !notes.is_empty() {
        println!("\nNotes (not findings):");
        for n in &notes {
            println!("- {n}");
        }
    }
    0
}
