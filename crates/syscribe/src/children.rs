//! `children` — the direct children of a tree node with counters, one call per expanded node (GH #425,
//! `REQ-TRS-CHILDREN-001`). Two views: `containment` (packages and elements by qualified name) and
//! `configurations` (Configurations following `derivedFrom:` and `subConfigurations:`). The data function is shared
//! by the CLI, the MCP tool and the LSP request `syscribe/children`.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use serde_json::{json, Value};
use syscribe_model::element::{ElementType, RawElement};
use syscribe_model::resolver::Resolver;
use syscribe_model::validator::{Finding, Severity};

pub const DEFAULT_LIMIT: usize = 200;
pub const MAX_LIMIT: usize = 1000;

#[derive(Debug, Clone, Default)]
pub struct Request {
    /// The node to expand (qualified name or id); `None` is the root of the view.
    pub of: Option<String>,
    /// `containment` (default) or `configurations`.
    pub view: Option<String>,
    pub offset: usize,
    pub limit: Option<usize>,
}

pub type Fail = (i32, String);

#[derive(Default, Clone)]
struct Counters {
    errors: usize,
    warnings: usize,
}

fn ident(e: &RawElement) -> String {
    e.frontmatter.id.clone().unwrap_or_else(|| e.qualified_name.clone())
}

fn type_name(e: &RawElement) -> &'static str {
    crate::query::type_label(e.frontmatter.element_type.as_ref().unwrap_or(&ElementType::Unknown))
}

/// Findings per qualified name, and rolled up per ancestor package (`subtree`).
fn finding_counts(elements: &[RawElement], findings: &[Finding]) -> (HashMap<String, Counters>, HashMap<String, Counters>) {
    let by_file: HashMap<&str, &str> = elements.iter().map(|e| (e.file_path.as_str(), e.qualified_name.as_str())).collect();
    let (mut own, mut sub): (HashMap<String, Counters>, HashMap<String, Counters>) = Default::default();
    for f in findings {
        let Some(q) = by_file.get(f.file.as_str()) else { continue };
        let bump = |c: &mut Counters| match f.severity {
            Severity::Error => c.errors += 1,
            _ => c.warnings += 1,
        };
        bump(own.entry((*q).to_string()).or_default());
        let mut path = *q;
        loop {
            bump(sub.entry(path.to_string()).or_default());
            match path.rsplit_once("::") {
                Some((head, _)) => path = head,
                None => break,
            }
        }
    }
    (own, sub)
}

pub fn compute(elements: &[RawElement], findings: &[Finding], req: &Request) -> Result<Value, Fail> {
    let limit = req.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
    let resolver = Resolver::new(elements);
    let (own, sub) = finding_counts(elements, findings);

    // Reverse indices used for counters.
    let (mut derived, mut verifying): (HashMap<String, usize>, HashMap<String, usize>) = Default::default();
    for e in elements {
        for r in e.frontmatter.derived_from.iter().flatten() {
            if let Some(t) = resolver.resolve_ref(elements, r) {
                if e.frontmatter.element_type != Some(ElementType::Configuration) {
                    *derived.entry(t.qualified_name.clone()).or_default() += 1;
                }
            }
        }
        if e.frontmatter.element_type == Some(ElementType::TestCase) {
            for r in e.frontmatter.verifies.iter().flatten() {
                if let Some(t) = resolver.resolve_ref(elements, r) {
                    *verifying.entry(t.qualified_name.clone()).or_default() += 1;
                }
            }
        }
    }

    let view = req.view.as_deref().unwrap_or("containment");
    let (nodes, total_children_of): (Vec<Value>, Box<dyn Fn(&str) -> usize>) = match view {
        "containment" => containment(elements, &resolver, req, &own, &sub, &derived, &verifying)?,
        "configurations" => configurations(elements, &resolver, req, &own)?,
        o => return Err((2, format!("unknown view '{o}'; use containment or configurations"))),
    };
    let _ = total_children_of;
    let total = nodes.len();
    let page: Vec<Value> = nodes.into_iter().skip(req.offset).take(limit).collect();
    let next = req.offset + page.len();
    Ok(json!({
        "schemaVersion": "1.0",
        "view": view,
        "of": req.of,
        "total": total,
        "offset": req.offset,
        "limit": limit,
        "nextOffset": if next < total { json!(next) } else { Value::Null },
        "children": page,
    }))
}

type Built = (Vec<Value>, Box<dyn Fn(&str) -> usize>);

fn containment(
    elements: &[RawElement],
    resolver: &Resolver,
    req: &Request,
    own: &HashMap<String, Counters>,
    sub: &HashMap<String, Counters>,
    derived: &HashMap<String, usize>,
    verifying: &HashMap<String, usize>,
) -> Result<Built, Fail> {
    let parent: Option<String> = match &req.of {
        None => None,
        Some(k) => {
            let q = resolver.resolve_ref(elements, k).map(|e| e.qualified_name.clone()).unwrap_or_else(|| k.replace('/', "::"));
            // A directory package without an _index.md has no element, only descendants.
            let exists = elements.iter().any(|e| e.qualified_name == q || e.qualified_name.starts_with(&format!("{q}::")));
            if !exists {
                return Err((1, format!("element not found: {k}")));
            }
            Some(q)
        }
    };
    let prefix = parent.as_ref().map(|p| format!("{p}::"));
    // child segment -> (the element with exactly that qualified name, descendant count)
    let mut kids: BTreeMap<String, (Option<&RawElement>, usize)> = BTreeMap::new();
    for e in elements {
        let rest = match &prefix {
            None => e.qualified_name.as_str(),
            Some(p) => match e.qualified_name.strip_prefix(p.as_str()) {
                Some(r) => r,
                None => continue,
            },
        };
        if rest.is_empty() {
            continue;
        }
        let (seg, deeper) = match rest.split_once("::") {
            Some((s, _)) => (s, true),
            None => (rest, false),
        };
        let slot = kids.entry(seg.to_string()).or_insert((None, 0));
        if deeper {
            slot.1 += 1;
        } else {
            slot.0 = Some(e);
        }
    }
    let q_of = |seg: &str| match &parent {
        None => seg.to_string(),
        Some(p) => format!("{p}::{seg}"),
    };
    let mut rows: Vec<(bool, String, Value)> = Vec::new();
    for (seg, (elem, nested)) in &kids {
        let q = q_of(seg);
        let is_package = *nested > 0 || elem.is_some_and(|e| matches!(e.frontmatter.element_type, Some(ElementType::Package | ElementType::LibraryPackage | ElementType::Namespace)));
        let mut counters = json!({ "children": nested });
        if let Some(c) = derived.get(&q) {
            counters["derivedChildren"] = json!(c);
        }
        if let Some(c) = verifying.get(&q) {
            counters["verifyingTests"] = json!(c);
        }
        let f = if is_package { sub.get(&q) } else { own.get(&q) };
        counters["errors"] = json!(f.map_or(0, |c| c.errors));
        counters["warnings"] = json!(f.map_or(0, |c| c.warnings));
        let node = json!({
            "qualifiedName": q,
            "id": elem.map(ident).unwrap_or_else(|| q.clone()),
            "name": elem.and_then(|e| e.frontmatter.name.clone()).unwrap_or_else(|| seg.clone()),
            "type": elem.map(type_name).unwrap_or("Package"),
            "status": elem.and_then(|e| e.frontmatter.status.clone()),
            "file": elem.map(|e| e.file_path.clone()),
            "package": is_package,
            "virtual": elem.is_none(),
            "hasChildren": *nested > 0,
            "counters": counters,
        });
        rows.push((!is_package, seg.to_lowercase(), node));
    }
    // Packages first, then elements, each by name.
    rows.sort_by(|a, b| (a.0, &a.1).cmp(&(b.0, &b.1)));
    Ok((rows.into_iter().map(|r| r.2).collect(), Box::new(|_| 0)))
}

fn configurations(elements: &[RawElement], resolver: &Resolver, req: &Request, own: &HashMap<String, Counters>) -> Result<Built, Fail> {
    let confs: Vec<&RawElement> = elements.iter().filter(|e| e.frontmatter.element_type == Some(ElementType::Configuration)).collect();
    // child qname -> parents it derives from, and parent qname -> sub-configurations it consolidates.
    let mut derived_of: HashMap<&str, Vec<&RawElement>> = HashMap::new();
    let mut is_child: BTreeSet<&str> = BTreeSet::new();
    let mut subs_of: HashMap<&str, Vec<&RawElement>> = HashMap::new();
    for c in &confs {
        for r in c.frontmatter.derived_from.iter().flatten() {
            if let Some(p) = resolver.resolve_ref(elements, r) {
                if p.frontmatter.element_type == Some(ElementType::Configuration) {
                    derived_of.entry(p.qualified_name.as_str()).or_default().push(c);
                    is_child.insert(c.qualified_name.as_str());
                }
            }
        }
        for r in c.frontmatter.sub_configurations.iter().flatten() {
            if let Some(s) = resolver.resolve_ref(elements, r) {
                if s.frontmatter.element_type == Some(ElementType::Configuration) {
                    subs_of.entry(c.qualified_name.as_str()).or_default().push(s);
                    is_child.insert(s.qualified_name.as_str());
                }
            }
        }
    }
    let rel_children = |q: &str| -> Vec<(&'static str, &RawElement)> {
        let mut v: Vec<(&'static str, &RawElement)> = derived_of.get(q).into_iter().flatten().map(|c| ("derivedFrom", *c)).collect();
        v.extend(subs_of.get(q).into_iter().flatten().map(|c| ("subConfiguration", *c)));
        v.sort_by_key(|(_, c)| ident(c));
        v.dedup_by_key(|(_, c)| c.qualified_name.clone());
        v
    };
    let list: Vec<(&'static str, &RawElement)> = match &req.of {
        None => {
            let mut roots: Vec<&RawElement> = confs.iter().copied().filter(|c| !is_child.contains(c.qualified_name.as_str())).collect();
            roots.sort_by_key(|c| ident(c));
            roots.into_iter().map(|c| ("root", c)).collect()
        }
        Some(k) => {
            let e = resolver.resolve_ref(elements, k).ok_or_else(|| (1, format!("element not found: {k}")))?;
            if e.frontmatter.element_type != Some(ElementType::Configuration) {
                return Err((1, format!("{k} is not a Configuration")));
            }
            rel_children(e.qualified_name.as_str())
        }
    };
    let nodes = list
        .into_iter()
        .map(|(rel, c)| {
            let q = c.qualified_name.as_str();
            let n = rel_children(q).len();
            json!({
                "qualifiedName": c.qualified_name,
                "id": ident(c),
                "name": c.frontmatter.name,
                "type": "Configuration",
                "status": c.frontmatter.status,
                "file": c.file_path,
                "relation": rel,
                "package": false,
                "virtual": false,
                "hasChildren": n > 0,
                "bindingStage": c.frontmatter.binding_stage,
                "counters": { "children": n, "errors": own.get(q).map_or(0, |x| x.errors), "warnings": own.get(q).map_or(0, |x| x.warnings) },
            })
        })
        .collect();
    Ok((nodes, Box::new(|_| 0)))
}

const USAGE: &str = "Usage: syscribe -m <root> children [<node>] [--view containment|configurations] [--offset N] [--limit N] [--json]";

pub fn parse_args(args: &[String]) -> Result<(Request, bool), String> {
    let (mut req, mut json) = (Request::default(), false);
    let mut it = args.iter();
    while let Some(a) = it.next() {
        let mut val = |f: &str| it.next().cloned().ok_or_else(|| format!("children: {f} needs a value"));
        let num = |f: &str, v: String| v.parse::<usize>().map_err(|_| format!("children: {f} expects a non-negative integer"));
        match a.as_str() {
            "--json" => json = true,
            "--view" => req.view = Some(val("--view")?),
            "--offset" => req.offset = num("--offset", val("--offset")?)?,
            "--limit" => req.limit = Some(num("--limit", val("--limit")?)?),
            f if f.starts_with('-') => return Err(format!("children: unknown option '{f}'")),
            _ if req.of.is_none() => req.of = Some(a.clone()),
            _ => return Err(USAGE.to_string()),
        }
    }
    Ok((req, json))
}

pub fn cmd_children(elements: &[RawElement], findings: &[Finding], args: &[String]) -> i32 {
    let (req, json) = match parse_args(args) {
        Ok(p) => p,
        Err(m) => {
            eprintln!("{m}");
            return 2;
        }
    };
    let out = match compute(elements, findings, &req) {
        Ok(o) => o,
        Err((c, m)) => {
            eprintln!("{m}");
            return c;
        }
    };
    if json {
        println!("{}", serde_json::to_string_pretty(&out).unwrap());
        return 0;
    }
    println!("# {} ({} of {})\n", req.of.as_deref().unwrap_or("root"), out["children"].as_array().map_or(0, Vec::len), out["total"]);
    for c in out["children"].as_array().into_iter().flatten() {
        let k = &c["counters"];
        println!(
            "- {}{} · {} · {}  [{} child, {} err, {} warn]",
            c["id"].as_str().unwrap_or(""),
            if c["hasChildren"] == true { "/" } else { "" },
            c["type"].as_str().unwrap_or(""),
            c["status"].as_str().unwrap_or("-"),
            k["children"],
            k["errors"],
            k["warnings"]
        );
    }
    if let Some(n) = out["nextOffset"].as_u64() {
        println!("\nMore: --offset {n}");
    }
    0
}
