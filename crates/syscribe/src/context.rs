//! `context <ref>` — one bounded briefing bundle for an element or PlanningItem (GH #435,
//! `REQ-TRS-CONTEXT-001`). The data function ([`build`]) and the size-bounded renderer ([`render`]) are shared
//! with the MCP `context` tool.
//!
//! The bound is on the rendered output: [`render`] drops the least important items first, one at a time, until
//! the output fits, and says what it dropped, so a caller never has to guess what a short answer left out.

use serde_json::{json, Value};
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use syscribe_model::element::{ElementType, RawElement};
use syscribe_model::resolver::Resolver;
use syscribe_model::results::ResultsData;
use syscribe_model::validator::{Finding, Severity};

pub const DEFAULT_MAX_BYTES: usize = 16_000;
pub const MIN_MAX_BYTES: usize = 1_024;
/// Neighbours listed per direction before the size bound is even applied, so a hub cannot make building slow.
const NEIGHBOUR_CAP: usize = 400;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Markdown,
    Json,
}

#[derive(Debug, Clone, Default)]
pub struct Bundle {
    pub element: Value,
    pub body: String,
    pub parents: Vec<Value>,
    pub upstream: Vec<Value>,
    pub downstream: Vec<Value>,
    pub verification: Vec<Value>,
    pub planning: Vec<Value>,
    pub findings: Vec<Value>,
    /// Section name -> items left out by the size bound.
    pub omitted: BTreeMap<&'static str, usize>,
    pub body_truncated: bool,
}

fn ident(e: &RawElement) -> String {
    e.frontmatter.id.clone().unwrap_or_else(|| e.qualified_name.clone())
}

fn type_name(e: &RawElement) -> &'static str {
    crate::query::type_label(e.frontmatter.element_type.as_ref().unwrap_or(&ElementType::Unknown))
}

/// One line saying what the element is: its name, else the first prose line of its body.
fn summary(e: &RawElement) -> String {
    let s = e
        .frontmatter
        .name
        .clone()
        .filter(|n| !n.trim().is_empty())
        .or_else(|| e.doc.lines().map(str::trim).find(|l| !l.is_empty() && !l.starts_with('#') && !l.starts_with("```")).map(str::to_string))
        .unwrap_or_default();
    let s: String = s.chars().take(140).collect();
    s
}

fn brief(e: &RawElement) -> serde_json::Map<String, Value> {
    let mut m = serde_json::Map::new();
    m.insert("id".into(), json!(ident(e)));
    m.insert("type".into(), json!(type_name(e)));
    if let Some(s) = &e.frontmatter.status {
        m.insert("status".into(), json!(s));
    }
    m.insert("summary".into(), json!(summary(e)));
    m
}

fn with(mut m: serde_json::Map<String, Value>, k: &str, v: Value) -> Value {
    m.insert(k.into(), v);
    Value::Object(m)
}

/// Build the bundle for `key` (id or qualified name). `Err` when it names nothing.
pub fn build(elements: &[RawElement], results: Option<&ResultsData>, findings: &[Finding], key: &str, depth: usize) -> Result<Bundle, String> {
    let resolver = Resolver::new(elements);
    let root = resolver
        .resolve_ref(elements, key)
        .or_else(|| resolver.get(elements, key))
        .ok_or_else(|| format!("element not found: {key}{}", crate::suggest::did_you_mean(key, elements.iter().map(|e| e.frontmatter.id.as_deref().unwrap_or(&e.qualified_name)))))?;

    // Edges once: source qname -> [(relationship, target qname)].
    let mut out_edges: HashMap<&str, Vec<(String, &RawElement)>> = HashMap::new();
    let mut in_edges: HashMap<&str, Vec<(String, &RawElement)>> = HashMap::new();
    for e in elements {
        for (rel, tgt) in crate::query::outbound_refs(e) {
            if let Some(t) = resolver.resolve_ref(elements, &tgt) {
                if std::ptr::eq(t, e) {
                    continue;
                }
                out_edges.entry(e.qualified_name.as_str()).or_default().push((rel.clone(), t));
                in_edges.entry(t.qualified_name.as_str()).or_default().push((rel, e));
            }
        }
    }

    let walk = |edges: &HashMap<&str, Vec<(String, &RawElement)>>| -> Vec<Value> {
        let mut seen: HashSet<&str> = HashSet::from([root.qualified_name.as_str()]);
        let mut queue: VecDeque<(&RawElement, usize)> = VecDeque::from([(root, 0)]);
        let mut out: Vec<Value> = Vec::new();
        while let Some((e, d)) = queue.pop_front() {
            if d >= depth {
                continue;
            }
            let mut next: Vec<&(String, &RawElement)> = edges.get(e.qualified_name.as_str()).map(|v| v.iter().collect()).unwrap_or_default();
            next.sort_by(|a, b| (ident(a.1), &a.0).cmp(&(ident(b.1), &b.0)));
            for (rel, n) in next {
                if out.len() >= NEIGHBOUR_CAP {
                    return out;
                }
                if seen.insert(n.qualified_name.as_str()) {
                    let mut b = brief(n);
                    b.insert("relationship".into(), json!(rel));
                    b.insert("depth".into(), json!(d + 1));
                    b.insert("via".into(), json!(ident(e)));
                    out.push(Value::Object(b));
                    queue.push_back((n, d + 1));
                }
            }
        }
        out
    };
    let upstream = walk(&out_edges);
    let downstream = walk(&in_edges);

    // The qualified-name ancestors, nearest first.
    let mut parents = Vec::new();
    let mut qn = root.qualified_name.as_str();
    while let Some((head, _)) = qn.rsplit_once("::") {
        if let Some(p) = resolver.get(elements, head) {
            parents.push(Value::Object(brief(p)));
        }
        qn = head;
    }

    // Verifying tests: the TestCases that hold a `verifies` link to it, with the latest verdict and rig.
    let mut verification = Vec::new();
    let mut tests: Vec<&RawElement> = in_edges.get(root.qualified_name.as_str()).into_iter().flatten().filter(|(rel, e)| rel == "verifies" && Resolver::is_native_testcase(e)).map(|(_, e)| *e).collect();
    tests.sort_by_key(|t| ident(t));
    tests.dedup_by_key(|t| ident(t));
    for tc in tests {
        let verdict = match crate::query::tc_verdict(tc, results) {
            crate::query::TcVerdict::Pass => "pass",
            crate::query::TcVerdict::Fail => "fail",
            crate::query::TcVerdict::Waived => "waived",
            crate::query::TcVerdict::Unknown => "unknown",
        };
        let mut m = brief(tc);
        m.insert("verdict".into(), json!(verdict));
        if let Some(l) = &tc.frontmatter.test_level {
            m.insert("level".into(), json!(format!("{l:?}")));
        }
        let id = syscribe_model::result_join::canonical(tc);
        if let Some(r) = results.and_then(|r| r.records_for(&id).max_by_key(|r| crate::envmatrix::record_time(r).unwrap_or(i64::MIN))) {
            if let Some(p) = &r.provenance {
                if let Some(env) = &p.environment {
                    m.insert("environment".into(), json!(env));
                }
                if let Some(c) = p.build.as_ref().and_then(|b| b.commit.as_ref()) {
                    m.insert("build".into(), json!(c));
                }
                if let Some(t) = p.finished_at.as_ref().or(p.started_at.as_ref()) {
                    m.insert("at".into(), json!(t));
                }
            }
        }
        verification.push(Value::Object(m));
    }

    // PlanningItem: tree position, what it achieves, proof, claim and unresolved items.
    let mut planning = Vec::new();
    if root.frontmatter.element_type == Some(ElementType::PlanningItem) {
        let f = &root.frontmatter;
        if let Some(p) = f.parent.as_deref().and_then(|p| resolver.resolve_ref(elements, p)) {
            planning.push(with(brief(p), "role", json!("parent")));
        }
        let mut kids: Vec<&RawElement> = elements
            .iter()
            .filter(|e| e.frontmatter.element_type == Some(ElementType::PlanningItem) && e.frontmatter.parent.as_deref().and_then(|p| resolver.resolve_ref(elements, p)).is_some_and(|p| std::ptr::eq(p, root)))
            .collect();
        kids.sort_by_key(|k| ident(k));
        for k in kids {
            planning.push(with(brief(k), "role", json!("child")));
        }
        for a in f.achieves.iter().flatten() {
            let b = resolver.resolve_ref(elements, a).map(|e| brief(e)).unwrap_or_else(|| serde_json::Map::from_iter([("id".to_string(), json!(a))]));
            planning.push(with(b, "role", json!("achieves")));
        }
        for ev in f.evidence.iter().flatten() {
            planning.push(json!({"role": "evidence", "entry": serde_yaml_to_json(ev)}));
        }
        for b in f.blocked_by.iter().flatten() {
            planning.push(json!({"role": "blockedBy", "id": b}));
        }
        if let Some(c) = &f.claimed_by {
            planning.push(json!({"role": "claim", "by": c, "at": f.claimed_at}));
        }
        if let Some(a) = &f.assigned_to {
            planning.push(json!({"role": "assignedTo", "id": a}));
        }
        for q in syscribe_model::questions::parse(root).0.iter().filter(|q| q.status == "open") {
            planning.push(json!({"role": "openQuestion", "id": q.reference(), "text": q.text}));
        }
    }

    // Findings on the element and on its listed neighbours, errors first.
    let mut files: HashMap<&str, String> = HashMap::new();
    files.insert(root.file_path.as_str(), ident(root));
    for v in upstream.iter().chain(&downstream).chain(&verification) {
        if let Some(id) = v["id"].as_str() {
            if let Some(e) = resolver.resolve_ref(elements, id) {
                files.entry(e.file_path.as_str()).or_insert_with(|| ident(e));
            }
        }
    }
    let mut fs: Vec<Value> = findings
        .iter()
        .filter_map(|f| {
            let who = files.get(f.file.as_str())?;
            let sev = if matches!(f.severity, Severity::Error) { "error" } else { "warning" };
            Some(json!({"code": f.code, "severity": sev, "element": who, "message": f.message}))
        })
        .collect();
    fs.sort_by_key(|v| (v["severity"] != "error", v["element"].as_str().map(|s| s != ident(root)).unwrap_or(true)));

    let mut el = brief(root);
    el.insert("qualifiedName".into(), json!(root.qualified_name));
    el.insert("file".into(), json!(root.file_path));
    if let Some(n) = &root.frontmatter.name {
        el.insert("name".into(), json!(n));
    }
    Ok(Bundle { element: Value::Object(el), body: root.doc.trim().to_string(), parents, upstream, downstream, verification, planning, findings: fs, ..Default::default() })
}

fn serde_yaml_to_json(v: &serde_yaml::Value) -> Value {
    serde_json::to_value(v).unwrap_or(Value::Null)
}

// ── rendering with a bound ───────────────────────────────────────────────────

fn line(v: &Value) -> String {
    let g = |k: &str| v[k].as_str().unwrap_or("");
    let mut s = format!("- **{}**", if g("id").is_empty() { g("role") } else { g("id") });
    for k in ["role", "relationship", "type", "status", "verdict", "severity", "code"] {
        if !g(k).is_empty() && !(k == "role" && g("id").is_empty()) {
            s.push_str(&format!(" · {}", g(k)));
        }
    }
    for k in ["environment", "build", "at", "by", "element"] {
        if !g(k).is_empty() {
            s.push_str(&format!(" · {k} {}", &g(k)[..g(k).len().min(40)]));
        }
    }
    for k in ["summary", "message", "text"] {
        if !g(k).is_empty() {
            s.push_str(&format!(" — {}", g(k)));
            break;
        }
    }
    if let Some(d) = v["depth"].as_u64().filter(|d| *d > 1) {
        s.push_str(&format!(" (depth {d} via {})", g("via")));
    }
    if v.get("entry").is_some() {
        s.push_str(&format!(" {}", v["entry"]));
    }
    s
}

fn section(out: &mut String, title: &str, items: &[Value], omitted: usize) {
    if items.is_empty() && omitted == 0 {
        return;
    }
    out.push_str(&format!("\n## {title}\n\n"));
    for i in items {
        out.push_str(&line(i));
        out.push('\n');
    }
    if omitted > 0 {
        out.push_str(&format!("- _… {omitted} more omitted_\n"));
    }
}

fn om(b: &Bundle, k: &'static str) -> usize {
    b.omitted.get(k).copied().unwrap_or(0)
}

fn notice(b: &Bundle, max: usize) -> Option<Value> {
    if b.omitted.is_empty() && !b.body_truncated {
        return None;
    }
    let mut parts: Vec<String> = b.omitted.iter().map(|(k, n)| format!("{n} {k}")).collect();
    if b.body_truncated {
        parts.push("the end of the body".to_string());
    }
    Some(json!({"maxBytes": max, "omitted": b.omitted, "bodyTruncated": b.body_truncated, "message": format!("Truncated to fit {max} bytes; omitted: {}. Raise --max-bytes or lower --depth.", parts.join(", "))}))
}

fn render_once(b: &Bundle, fmt: Format, max: usize) -> String {
    let note = notice(b, max);
    match fmt {
        Format::Json => {
            let mut v = json!({
                "schemaVersion": "1.0",
                "element": b.element,
                "body": b.body,
                "parents": b.parents,
                "upstream": b.upstream,
                "downstream": b.downstream,
                "verification": b.verification,
                "planning": b.planning,
                "findings": b.findings,
            });
            if let Some(n) = note {
                v["truncated"] = n;
            }
            serde_json::to_string_pretty(&v).unwrap()
        }
        Format::Markdown => {
            let e = &b.element;
            let mut s = format!("# {} — {}\n\n{} · {}", e["id"].as_str().unwrap_or(""), e["summary"].as_str().unwrap_or(""), e["type"].as_str().unwrap_or(""), e["status"].as_str().unwrap_or("no status"));
            s.push_str(&format!(" · `{}`\n", e["file"].as_str().unwrap_or("")));
            if !b.body.is_empty() {
                s.push_str(&format!("\n{}{}\n", b.body, if b.body_truncated { " …" } else { "" }));
            }
            section(&mut s, "Planning", &b.planning, om(b, "planning"));
            section(&mut s, "Verification", &b.verification, om(b, "verification"));
            section(&mut s, "Upstream (this links to)", &b.upstream, om(b, "upstream"));
            section(&mut s, "Downstream (links to this)", &b.downstream, om(b, "downstream"));
            section(&mut s, "Findings", &b.findings, om(b, "findings"));
            section(&mut s, "Parents", &b.parents, om(b, "parents"));
            if let Some(n) = note {
                s.push_str(&format!("\n> **Truncated.** {}\n", n["message"].as_str().unwrap_or("")));
            }
            s
        }
    }
}

/// Render within `max` bytes, dropping the least important items first and saying what was dropped.
/// Order of loss: downstream, upstream, parents, findings, then verification, planning and finally the body —
/// the things that say what the element *is* and how it is verified go last.
pub fn render(mut b: Bundle, fmt: Format, max: usize) -> String {
    // A long body must not crowd out the links and verification state: it is held to a third of the bound first.
    let body_cap = max / 3;
    if b.body.len() > body_cap {
        let mut cut = body_cap;
        while !b.body.is_char_boundary(cut) {
            cut -= 1;
        }
        b.body.truncate(cut);
        b.body_truncated = true;
    }
    loop {
        let s = render_once(&b, fmt, max);
        if s.len() <= max {
            return s;
        }
        // Drop in chunks when far over, to keep large hubs quick.
        let over = s.len() - max;
        let chunk = (over / 400).max(1);
        macro_rules! shed {
            ($field:ident, $name:expr) => {
                if !b.$field.is_empty() {
                    let n = chunk.min(b.$field.len());
                    let keep = b.$field.len() - n;
                    b.$field.truncate(keep);
                    *b.omitted.entry($name).or_insert(0) += n;
                    continue;
                }
            };
        }
        shed!(downstream, "downstream");
        shed!(upstream, "upstream");
        shed!(parents, "parents");
        shed!(findings, "findings");
        shed!(verification, "verification");
        shed!(planning, "planning");
        if !b.body.is_empty() {
            let mut cut = b.body.len().saturating_sub(over.max(64)).min(b.body.len() / 2 * 2);
            while !b.body.is_char_boundary(cut) {
                cut -= 1;
            }
            b.body.truncate(cut);
            b.body_truncated = true;
            continue;
        }
        // Nothing left to drop: the header alone does not fit.
        return match fmt {
            Format::Json => json!({"schemaVersion": "1.0", "element": {"id": b.element["id"]}, "truncated": {"maxBytes": max, "message": "max-bytes is too small for even the element header"}}).to_string(),
            Format::Markdown => {
                let mut t = s;
                let mut cut = max;
                while !t.is_char_boundary(cut) {
                    cut -= 1;
                }
                t.truncate(cut);
                t
            }
        };
    }
}

pub const USAGE: &str = "Usage: syscribe -m <root> context <ref> [--depth N] [--max-bytes N] [--format text|markdown|json]";

/// Parse the tokens after `context`. `Err` is a usage message.
pub fn parse_args(args: &[String]) -> Result<(String, usize, usize, Format), String> {
    let mut key: Option<String> = None;
    let (mut depth, mut max, mut fmt) = (1usize, DEFAULT_MAX_BYTES, Format::Markdown);
    let mut it = args.iter();
    while let Some(a) = it.next() {
        let mut val = |flag: &str| it.next().cloned().ok_or_else(|| format!("context: {flag} needs a value"));
        match a.as_str() {
            "--depth" => depth = val("--depth")?.parse().map_err(|_| "context: --depth expects a non-negative integer".to_string())?,
            "--max-bytes" => {
                max = val("--max-bytes")?.parse().map_err(|_| "context: --max-bytes expects a positive integer".to_string())?;
                if max < MIN_MAX_BYTES {
                    return Err(format!("context: --max-bytes must be at least {MIN_MAX_BYTES}"));
                }
            }
            "--format" => {
                fmt = match val("--format")?.as_str() {
                    "text" | "markdown" => Format::Markdown,
                    "json" => Format::Json,
                    o => return Err(format!("context: unknown --format '{o}'; use text, markdown or json")),
                }
            }
            "--json" => fmt = Format::Json,
            f if f.starts_with("--") => return Err(format!("context: unknown option '{f}'")),
            _ if key.is_none() => key = Some(a.clone()),
            _ => return Err(USAGE.to_string()),
        }
    }
    Ok((key.ok_or_else(|| USAGE.to_string())?, depth, max, fmt))
}

/// `args` are the tokens after `context`. Returns the exit code.
pub fn cmd_context(elements: &[RawElement], results: Option<&ResultsData>, findings: &[Finding], args: &[String]) -> i32 {
    let (key, depth, max, fmt) = match parse_args(args) {
        Ok(p) => p,
        Err(m) => {
            eprintln!("{m}");
            return 2;
        }
    };
    match build(elements, results, findings, &key, depth) {
        Ok(b) => {
            println!("{}", render(b, fmt, max));
            0
        }
        Err(m) => {
            eprintln!("{m}");
            1
        }
    }
}
