//! Behavioral coverage report (§20, GH #72). Read-only: reports how completely the active
//! TestCases exercise the behavioral elements (ActionDef/Action/StateDef/State), via four
//! coverage paths (source overlap, requirement chain, test function, allocation).

use std::collections::{BTreeSet, HashMap};
use syscribe_model::{
    element::{ElementType, RawElement},
    resolver::Resolver,
};

fn is_behavioral(e: &RawElement) -> bool {
    matches!(
        e.frontmatter.element_type,
        Some(ElementType::ActionDef) | Some(ElementType::Action) | Some(ElementType::StateDef) | Some(ElementType::State)
    )
}

fn id_of(e: &RawElement) -> &str {
    e.frontmatter.id.as_deref().unwrap_or(&e.qualified_name)
}

/// Normalised path-containment: is `child` under (or equal to) `parent`?
/// A leading `repo:` prefix (model-relative source link) is stripped from both sides.
fn path_under(child: &str, parent: &str) -> bool {
    let norm = |s: &str| s.strip_prefix("repo:").unwrap_or(s).trim_end_matches('/').to_string();
    let c = norm(child);
    let p = norm(parent);
    c == p || c.starts_with(&format!("{}/", p))
}

/// Qualified names an element's `typedBy:` / `supertype:` / `allocatedTo:` point to (resolved).
fn refs_of(v: &Option<serde_yaml::Value>, resolver: &Resolver, elements: &[RawElement]) -> Vec<String> {
    let raw: Vec<&str> = match v {
        Some(serde_yaml::Value::String(s)) => vec![s.as_str()],
        Some(serde_yaml::Value::Sequence(seq)) => seq.iter().filter_map(|x| x.as_str()).collect(),
        _ => vec![],
    };
    raw.into_iter().filter_map(|r| resolver.resolve_ref(elements, r).map(|e| e.qualified_name.clone())).collect()
}

fn str_refs(list: &Option<Vec<String>>, resolver: &Resolver, elements: &[RawElement]) -> Vec<String> {
    list.as_deref()
        .unwrap_or(&[])
        .iter()
        .filter_map(|r| resolver.resolve_ref(elements, r).map(|e| e.qualified_name.clone()))
        .collect()
}

pub struct BcovOptions<'a> {
    pub scope: Option<&'a str>,
    pub depth: Option<usize>,
    pub format: &'a str,
    pub uncovered_only: bool,
    pub include_planned: bool,
}

struct Row {
    qname: String,
    etype: String,
    covered_by: BTreeSet<String>,
    planned_by: BTreeSet<String>,
}

pub fn cmd_behavioral_coverage(elements: &[RawElement], opts: &BcovOptions) {
    let resolver = Resolver::new(elements);

    let (scope_label, scope_prefix) = match opts.scope {
        None => ("<model>".to_string(), None),
        Some(q) => match resolver.resolve_ref(elements, q) {
            Some(s) => (s.qualified_name.clone(), Some(s.qualified_name.clone())),
            None => {
                eprintln!("behavioral-coverage: scope '{}' does not resolve.", q);
                std::process::exit(1);
            }
        },
    };

    // Behavioral elements in scope (at depth).
    let in_scope = |e: &RawElement| -> bool {
        let Some(prefix) = &scope_prefix else { return true };
        if &e.qualified_name == prefix {
            return false; // the scope element itself is not an axis row
        }
        let Some(rest) = e.qualified_name.strip_prefix(&format!("{}::", prefix)) else { return false };
        match opts.depth {
            Some(d) => rest.matches("::").count() < d,
            None => true,
        }
    };
    let behaviors: Vec<&RawElement> = elements.iter().filter(|e| is_behavioral(e) && in_scope(e)).collect();

    // req qname → satisfying architecture elements (E with satisfies: R).
    let mut req_satisfiers: HashMap<String, Vec<&RawElement>> = HashMap::new();
    for e in elements {
        for rq in str_refs(&e.frontmatter.satisfies, &resolver, elements) {
            req_satisfiers.entry(rq).or_default().push(e);
        }
    }

    // Active and planned TestCases.
    let testcases: Vec<&RawElement> = elements.iter().filter(|e| Resolver::is_native_testcase(e)).collect();

    // Does TestCase tc cover behavioral element b? Returns true on any of the four paths.
    let covers = |tc: &RawElement, b: &RawElement| -> bool {
        let bfm = &b.frontmatter;
        let impl_paths = bfm.implemented_by.as_deref().unwrap_or(&[]);
        // Path 1: direct source overlap.
        if let Some(sf) = &tc.frontmatter.source_file {
            if impl_paths.iter().any(|p| path_under(sf, p)) {
                return true;
            }
        }
        // Path 3: test function file under implementedBy.
        if let Some(tfs) = &tc.frontmatter.test_functions {
            for tf in tfs {
                if let Some(m) = tf.as_mapping() {
                    if let Some(file) = m.get(serde_yaml::Value::from("file")).and_then(|v| v.as_str()) {
                        if impl_paths.iter().any(|p| path_under(file, p)) {
                            return true;
                        }
                    }
                }
            }
        }
        // Paths 2 & 4: requirement chain to a satisfying element typed-by/specialising/allocated-to B.
        for rq in str_refs(&tc.frontmatter.verifies, &resolver, elements) {
            if let Some(es) = req_satisfiers.get(&rq) {
                for e in es {
                    let typed = refs_of(&e.frontmatter.typed_by, &resolver, elements);
                    let supers = refs_of(&e.frontmatter.supertype, &resolver, elements);
                    let alloc = str_refs(&e.frontmatter.allocated_to, &resolver, elements);
                    if typed.contains(&b.qualified_name)
                        || supers.contains(&b.qualified_name)
                        || alloc.contains(&b.qualified_name)
                    {
                        return true;
                    }
                }
            }
        }
        false
    };

    let mut rows: Vec<Row> = Vec::new();
    for b in &behaviors {
        let mut covered_by = BTreeSet::new();
        let mut planned_by = BTreeSet::new();
        for tc in &testcases {
            let active = tc.frontmatter.status.as_deref() == Some("active");
            if active {
                if covers(tc, b) {
                    covered_by.insert(id_of(tc).to_string());
                }
            } else if opts.include_planned && covers(tc, b) {
                planned_by.insert(id_of(tc).to_string());
            }
        }
        rows.push(Row {
            qname: b.qualified_name.clone(),
            etype: b.frontmatter.element_type.as_ref().map(|t| format!("{:?}", t)).unwrap_or_default(),
            covered_by,
            planned_by,
        });
    }
    rows.sort_by(|a, b| a.qname.cmp(&b.qname));

    // Totals are computed over all behavioral elements, before any display filtering.
    let total = behaviors.len();
    let covered = rows.iter().filter(|r| !r.covered_by.is_empty()).count();
    let pct = if total == 0 { 0.0 } else { 100.0 * covered as f64 / total as f64 };

    if opts.uncovered_only {
        rows.retain(|r| r.covered_by.is_empty());
    }

    if opts.format == "json" {
        let els: Vec<serde_json::Value> = rows
            .iter()
            .map(|r| {
                let mut o = serde_json::json!({
                    "qname": r.qname,
                    "type": r.etype,
                    "covered": !r.covered_by.is_empty(),
                    "coveredBy": r.covered_by.iter().cloned().collect::<Vec<_>>(),
                });
                if opts.include_planned {
                    o["plannedBy"] = serde_json::json!(r.planned_by.iter().cloned().collect::<Vec<_>>());
                }
                o
            })
            .collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "scope": scope_label, "covered": covered, "total": total,
                "coverage_pct": (pct * 10.0).round() / 10.0, "elements": els
            }))
            .unwrap()
        );
        return;
    }

    println!("Behavioral Coverage — {} (active tests only)\n", scope_label);
    if rows.is_empty() {
        println!("(no behavioral elements in scope)");
        return;
    }
    let name_w = rows.iter().map(|r| r.qname.len()).max().unwrap_or(8).max(8);
    let type_w = rows.iter().map(|r| r.etype.len()).max().unwrap_or(9).max(9);
    let mut header = format!("{:<nw$}  {:<tw$}  Covered  Test Cases", "Element", "Type", nw = name_w, tw = type_w);
    if opts.include_planned {
        header.push_str("  Planned");
    }
    println!("{}", header);
    for r in &rows {
        let mark = if r.covered_by.is_empty() { "✗" } else { "✓" };
        let tcs = if r.covered_by.is_empty() { "—".to_string() } else { r.covered_by.iter().cloned().collect::<Vec<_>>().join(", ") };
        let mut line = format!("{:<nw$}  {:<tw$}  {:<7}  {}", r.qname, r.etype, mark, tcs, nw = name_w, tw = type_w);
        if opts.include_planned {
            let p = if r.planned_by.is_empty() { "—".to_string() } else { r.planned_by.iter().cloned().collect::<Vec<_>>().join(", ") };
            line.push_str(&format!("  {}", p));
        }
        println!("{}", line);
    }
    println!("\nCoverage: {} / {} ({:.1}%)", covered, total, pct);
}
