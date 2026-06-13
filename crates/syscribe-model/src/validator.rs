use std::collections::{HashMap, HashSet};
use petgraph::algo::toposort;
use petgraph::graph::DiGraph;
use petgraph::visit::EdgeRef;
use crate::config::ValidateConfig;
use crate::element::{ElementType, ParseIssue, RawElement};
use crate::graph::EdgeKind;
use crate::resolver::{
    is_adr_id, is_asset_id, is_aou_id, is_arg_id, is_at_id, is_atg_id, is_ats_id, is_basic_name, is_cm_id,
    is_cd_id, is_conf_id, is_csg_id, is_ds_id, is_fm_id, is_fmea_id, is_ft_id, is_fte_id, is_ftg_id, is_he_id,
    is_zn_id,
    is_req_id, is_rr_id, is_sc_id, is_sg_id, is_stable_id, is_tara_id, is_tc_id, is_test_plan_id, is_trd_id, is_ts_id,
    is_vr_id, Resolver,
};

/// A single validation finding.
#[derive(Debug, Clone)]
pub struct Finding {
    pub code: &'static str,
    pub file: String,
    pub message: String,
    pub severity: Severity,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    /// Informational: surfaces a fact (e.g. a planned, not-yet-implemented test)
    /// without failing validation. Never causes a non-zero exit on its own, but
    /// can be selected explicitly via `--deny <code>`.
    Info,
}

impl std::fmt::Display for Finding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let tag = match self.severity {
            Severity::Error => "ERROR",
            Severity::Warning => "WARN",
            Severity::Info => "INFO",
        };
        write!(f, "[{}] {} {}: {}", tag, self.code, self.file, self.message)
    }
}

pub struct ValidationResult {
    pub findings: Vec<Finding>,
    /// verifiedBy[req_id] = list of tc ids that have status:active
    pub verified_by: HashMap<String, Vec<String>>,
    /// derived_children[req_id] = list of child req ids
    pub derived_children: HashMap<String, Vec<String>>,
    /// REQ-TRS-MG-001 — refinedBy[req_id_or_qname] = use cases that `refines:` it.
    /// Keyed by the requirement's stable id when present, else its qualified name.
    pub refined_by: HashMap<String, Vec<String>>,
    /// REQ-TRS-MG-002 — actorIn[part_id_or_qname] = use cases naming the part as an
    /// actor. Keyed by the actor part's stable id when present, else its qualified
    /// name. Computed only when the MagicGrid gate is active.
    pub actor_in: HashMap<String, Vec<String>>,
    /// REQ-TRS-MG-008 — mopRefinedBy[moe_id_or_qname] = the MoPs (Measurements of
    /// Performance) whose `mg_mop_refines:` names this MoE. Keyed by the MoE's
    /// stable id when present, else its qualified name. Computed only when the
    /// MagicGrid gate is active.
    pub mop_refined_by: HashMap<String, Vec<String>>,
    /// REQ-TRS-ALLOC-001 — allocatedFrom[target_id_or_qname] = the source
    /// elements allocated to this target, aggregated over both authoring forms
    /// (`allocatedTo`-on-source and the standalone `Allocation` element). Keyed
    /// by the target's stable id when present, else its qualified name; each
    /// source is labelled by its stable id else qname.
    pub allocated_from: HashMap<String, Vec<String>>,
}

impl ValidationResult {
    pub fn errors(&self) -> impl Iterator<Item = &Finding> {
        self.findings.iter().filter(|f| f.severity == Severity::Error)
    }
    pub fn warnings(&self) -> impl Iterator<Item = &Finding> {
        self.findings.iter().filter(|f| f.severity == Severity::Warning)
    }
    pub fn infos(&self) -> impl Iterator<Item = &Finding> {
        self.findings.iter().filter(|f| f.severity == Severity::Info)
    }
}

/// Resolve a relative `href` path against a base directory into a normalised path string.
/// Handles `..` and `.` segments without touching the filesystem.
fn normalize_relative_path(base_dir: &str, href: &str) -> String {
    use std::path::Component;
    let combined = std::path::Path::new(base_dir).join(href);
    let mut parts: Vec<String> = Vec::new();
    for component in combined.components() {
        match component {
            Component::ParentDir => { parts.pop(); }
            Component::CurDir => {}
            Component::Normal(s) => parts.push(s.to_string_lossy().into_owned()),
            Component::RootDir => parts.clear(),
            Component::Prefix(_) => {}
        }
    }
    parts.join("/")
}

/// Collect the `name` of every `SendAction`/`AcceptAction` reachable through an
/// ActionDef sub-action tree (§22.4, W080). Recurses into `IfAction` `then:`/`else:`
/// branches and nested `subActions:` lists. Order-preserving, names only (the caller
/// builds the qualified name against the owning ActionDef).
fn collect_message_actions(sub_actions: &[serde_yaml::Value], out: &mut Vec<String>) {
    for sa in sub_actions {
        let serde_yaml::Value::Mapping(m) = sa else { continue };
        let kind = m
            .get(&serde_yaml::Value::String("kind".into()))
            .and_then(|v| v.as_str());
        let name = m
            .get(&serde_yaml::Value::String("name".into()))
            .and_then(|v| v.as_str());
        if let (Some(k), Some(n)) = (kind, name) {
            if k == "SendAction" || k == "AcceptAction" {
                out.push(n.to_string());
            }
        }
        for branch in ["then", "else", "subActions"] {
            if let Some(serde_yaml::Value::Sequence(seq)) =
                m.get(&serde_yaml::Value::String(branch.into()))
            {
                collect_message_actions(seq, out);
            }
        }
    }
}

/// A normalized state-machine transition edge (§8.8.3), extracted from either
/// authoring placement (nested under a `subStates:` entry, or top-level under the
/// `StateDef`'s `transitions:`) and either spelling (canonical
/// `source`/`target`/`accept` or the deprecated `from`/`to`/`trigger`). This is the
/// single edge primitive the state-machine completeness checks consume.
#[allow(dead_code)] // source/target/payload/has_guard are read by later W07x phases
#[derive(Debug, Clone)]
struct StateEdge {
    source: Option<String>,
    target: Option<String>,
    /// Accept payload — the `accept` string, `accept.payload`, or legacy `trigger`.
    payload: Option<String>,
    has_guard: bool,
    /// True when authored with the deprecated `from`/`to`/`trigger` keys (W075).
    legacy: bool,
}

/// Read a string-keyed field from a YAML mapping.
fn yaml_field<'a>(m: &'a serde_yaml::Mapping, k: &str) -> Option<&'a serde_yaml::Value> {
    m.get(&serde_yaml::Value::String(k.to_string()))
}

/// Extract the transition edges contributed by a substate roster (each substate's
/// nested `transitions:`, with the substate as implicit source) plus an optional
/// top-level/region-level `transitions:` list (explicit source). Normalizes the
/// canonical and deprecated spellings onto one edge model. §8.8.3.
fn transitions_from(
    sub_states: Option<&[serde_yaml::Value]>,
    top: Option<&[serde_yaml::Value]>,
) -> Vec<StateEdge> {
    fn parse_transition(t: &serde_yaml::Value, implicit_source: Option<&str>) -> Option<StateEdge> {
        let m = t.as_mapping()?;
        let src = yaml_field(m, "source").or_else(|| yaml_field(m, "from")).and_then(|v| v.as_str());
        let tgt = yaml_field(m, "target").or_else(|| yaml_field(m, "to")).and_then(|v| v.as_str());
        let payload = match yaml_field(m, "accept") {
            Some(serde_yaml::Value::String(s)) => Some(s.clone()),
            Some(serde_yaml::Value::Mapping(am)) => {
                yaml_field(am, "payload").and_then(|v| v.as_str()).map(String::from)
            }
            _ => yaml_field(m, "trigger").and_then(|v| v.as_str()).map(String::from),
        };
        let legacy = yaml_field(m, "from").is_some()
            || yaml_field(m, "to").is_some()
            || yaml_field(m, "trigger").is_some();
        let has_guard = yaml_field(m, "guard")
            .and_then(|v| v.as_str())
            .map(|s| !s.is_empty())
            .unwrap_or(false);
        Some(StateEdge {
            source: src.map(String::from).or_else(|| implicit_source.map(String::from)),
            target: tgt.map(String::from),
            payload,
            has_guard,
            legacy,
        })
    }

    let mut edges = Vec::new();
    if let Some(subs) = sub_states {
        for s in subs {
            let Some(sm) = s.as_mapping() else { continue };
            let name = yaml_field(sm, "name").and_then(|v| v.as_str());
            if let Some(serde_yaml::Value::Sequence(ts)) = yaml_field(sm, "transitions") {
                for t in ts {
                    if let Some(e) = parse_transition(t, name) {
                        edges.push(e);
                    }
                }
            }
        }
    }
    if let Some(ts) = top {
        for t in ts {
            if let Some(e) = parse_transition(t, None) {
                edges.push(e);
            }
        }
    }
    edges
}

/// Run the SysMLv2 flat-completeness checks over **one region** — a substate roster
/// plus the edge set scoped to it. `region` labels the parallel region for messages
/// (`None` for a top-level single-region machine). `W073`/`W074` (initial cardinality)
/// always apply; the reachability-flavoured `W070`/`W071`/`W072` apply only when the
/// region is flat (no composite substate), since those are refined for hierarchy/
/// parallelism in later phases. §22.1.
fn check_state_region(
    region: Option<&str>,
    sub_states: &[serde_yaml::Value],
    edges: &[StateEdge],
    file: &str,
    findings: &mut Vec<Finding>,
) {
    struct Sub {
        name: String,
        is_initial: bool,
        is_final: bool,
    }
    let roster: Vec<Sub> = sub_states
        .iter()
        .filter_map(|s| s.as_mapping())
        .filter_map(|m| {
            let name = yaml_field(m, "name")?.as_str()?.to_string();
            let flag = |k: &str| yaml_field(m, k) == Some(&serde_yaml::Value::Bool(true));
            Some(Sub { name, is_initial: flag("isInitial"), is_final: flag("isFinal") })
        })
        .collect();
    if roster.is_empty() {
        return;
    }
    let suffix = region.map(|r| format!(" in region '{}'", r)).unwrap_or_default();

    // W073 / W074 — initial-state cardinality (always checked).
    let initial_count = roster.iter().filter(|s| s.is_initial).count();
    if initial_count == 0 {
        findings.push(warning(
            "W073",
            file,
            &format!("state machine has no `isInitial: true` substate{} — no defined starting point", suffix),
        ));
    } else if initial_count > 1 {
        findings.push(warning(
            "W074",
            file,
            &format!("state machine has {} `isInitial: true` substates{} — a region has exactly one initial state", initial_count, suffix),
        ));
    }

    // W070/W071/W072 over this level's substates, treating composite substates as
    // single nodes; their interiors are checked by the recursive walk.
    let names: HashSet<&str> = roster.iter().map(|s| s.name.as_str()).collect();
    let mut indeg: HashMap<&str, usize> = names.iter().map(|n| (*n, 0)).collect();
    let mut outdeg: HashMap<&str, usize> = names.iter().map(|n| (*n, 0)).collect();
    for e in edges {
        if let Some(src) = e.source.as_deref() {
            if let Some(d) = outdeg.get_mut(src) {
                *d += 1;
            }
        }
        if let Some(tgt) = e.target.as_deref() {
            if let Some(d) = indeg.get_mut(tgt) {
                *d += 1;
            }
        }
    }
    for s in &roster {
        if !s.is_initial && indeg.get(s.name.as_str()) == Some(&0) {
            findings.push(warning(
                "W070",
                file,
                &format!("dead state '{}'{} — no incoming transition and not `isInitial`", s.name, suffix),
            ));
        }
        if !s.is_final && outdeg.get(s.name.as_str()) == Some(&0) {
            findings.push(warning(
                "W071",
                file,
                &format!("trap state '{}'{} — no outgoing transition and not `isFinal`", s.name, suffix),
            ));
        }
    }
    // W072 non-determinism — same source + same accept payload, no guard.
    let mut groups: std::collections::BTreeMap<(&str, &str), (usize, usize)> =
        std::collections::BTreeMap::new();
    for e in edges {
        if let (Some(src), Some(pl)) = (e.source.as_deref(), e.payload.as_deref()) {
            if names.contains(src) {
                let g = groups.entry((src, pl)).or_insert((0, 0));
                g.0 += 1;
                if e.has_guard {
                    g.1 += 1;
                }
            }
        }
    }
    for ((src, pl), (count, guarded)) in groups {
        if count >= 2 && guarded == 0 {
            findings.push(warning(
                "W072",
                file,
                &format!("non-determinism — {} transitions from '{}'{} accept the same payload '{}' with no guard", count, src, suffix, pl),
            ));
        }
    }
}

/// Recursively collect every state **name** and every transition **edge** in a state
/// machine, descending into inline-composite substates (a substate carrying its own
/// `subStates:`). A composite substate's own `transitions:` belong to its parent level
/// (extracted here with the substate as source); inner regions contribute their own
/// substates' nested transitions, so the recursion passes no sibling-level list down.
fn collect_machine(
    sub_states: &[serde_yaml::Value],
    sibling_top: Option<&[serde_yaml::Value]>,
    names: &mut HashSet<String>,
    edges: &mut Vec<StateEdge>,
) {
    for s in sub_states {
        let Some(sm) = s.as_mapping() else { continue };
        if let Some(n) = yaml_field(sm, "name").and_then(|v| v.as_str()) {
            names.insert(n.to_string());
        }
        if let Some(serde_yaml::Value::Sequence(inner)) = yaml_field(sm, "subStates") {
            collect_machine(inner, None, names, edges);
        }
    }
    edges.extend(transitions_from(Some(sub_states), sibling_top));
}

/// Recursively collect every state-machine **behavior reference** (`W079`): each state's
/// `entryAction`/`doAction`/`exitAction` and each transition's `effect`, given either as a
/// qualified-name string or a `{typedBy: <qn>}` map. `accept.payload` is intentionally
/// excluded (payloads frequently name informal event labels, not model elements).
fn collect_state_refs(sub_states: &[serde_yaml::Value], out: &mut Vec<String>) {
    fn add_behavior(v: Option<&serde_yaml::Value>, out: &mut Vec<String>) {
        match v {
            Some(serde_yaml::Value::String(s)) => out.push(s.clone()),
            Some(serde_yaml::Value::Mapping(m)) => {
                if let Some(t) = yaml_field(m, "typedBy").and_then(|x| x.as_str()) {
                    out.push(t.to_string());
                }
            }
            _ => {}
        }
    }
    for s in sub_states {
        let Some(sm) = s.as_mapping() else { continue };
        for k in ["entryAction", "doAction", "exitAction"] {
            add_behavior(yaml_field(sm, k), out);
        }
        if let Some(serde_yaml::Value::Sequence(ts)) = yaml_field(sm, "transitions") {
            for t in ts {
                if let Some(tm) = t.as_mapping() {
                    add_behavior(yaml_field(tm, "effect"), out);
                }
            }
        }
        if let Some(serde_yaml::Value::Sequence(inner)) = yaml_field(sm, "subStates") {
            collect_state_refs(inner, out);
        }
    }
}

/// Recursively check a state node and its descendants (§22.1). A non-parallel node is one
/// region: its substates are checked by [`check_state_region`] (composite substates as
/// nodes), then each inline-composite substate is recursed into. A parallel node's direct
/// substates are concurrent regions: arity (`W078`), each region recursed, and any
/// transition crossing two regions flagged (`W077`).
fn check_state_node(
    label: Option<&str>,
    sub_states: &[serde_yaml::Value],
    sibling_top: Option<&[serde_yaml::Value]>,
    is_parallel: bool,
    file: &str,
    findings: &mut Vec<Finding>,
) {
    let here = label.map(|l| format!(" '{}'", l)).unwrap_or_default();
    if is_parallel {
        let regions: Vec<&serde_yaml::Value> = sub_states.iter().collect();
        if regions.len() < 2 {
            findings.push(warning(
                "W078",
                file,
                &format!("`isParallel: true` state{} has {} region(s) — a parallel state needs at least two", here, regions.len()),
            ));
        }
        // Map each region's direct substate names to its region label (for W077).
        // A name appearing in more than one region is ambiguous and excluded.
        let mut name_region: HashMap<String, Option<String>> = HashMap::new();
        let mut all_edges = transitions_from(Some(sub_states), sibling_top);
        for region in &regions {
            let Some(rm) = region.as_mapping() else { continue };
            let rlabel = yaml_field(rm, "name").and_then(|v| v.as_str());
            let r_parallel = yaml_field(rm, "isParallel") == Some(&serde_yaml::Value::Bool(true));
            if let Some(serde_yaml::Value::Sequence(rsubs)) = yaml_field(rm, "subStates") {
                for cs in rsubs {
                    if let Some(n) = cs.as_mapping().and_then(|m| yaml_field(m, "name")).and_then(|v| v.as_str()) {
                        name_region
                            .entry(n.to_string())
                            .and_modify(|e| *e = None)
                            .or_insert(rlabel.map(String::from));
                    }
                }
                check_state_node(rlabel, rsubs, None, r_parallel, file, findings);
                all_edges.extend(transitions_from(Some(rsubs), None));
            }
        }
        for e in &all_edges {
            if let (Some(src), Some(tgt)) = (e.source.as_deref(), e.target.as_deref()) {
                if let (Some(Some(rs)), Some(Some(rt))) = (name_region.get(src), name_region.get(tgt)) {
                    if rs != rt {
                        findings.push(warning(
                            "W077",
                            file,
                            &format!("transition '{}' → '{}' crosses parallel regions ('{}' → '{}') — illegal in a parallel state", src, tgt, rs, rt),
                        ));
                    }
                }
            }
        }
    } else {
        let edges = transitions_from(Some(sub_states), sibling_top);
        check_state_region(label, sub_states, &edges, file, findings);
        for s in sub_states {
            let Some(sm) = s.as_mapping() else { continue };
            if let Some(serde_yaml::Value::Sequence(inner)) = yaml_field(sm, "subStates") {
                let slabel = yaml_field(sm, "name").and_then(|v| v.as_str());
                let s_parallel = yaml_field(sm, "isParallel") == Some(&serde_yaml::Value::Bool(true));
                check_state_node(slabel, inner, None, s_parallel, file, findings);
            }
        }
    }
}

// ── Budget expression language (§22.2, CalculationDef bodyLanguage: budget) ──────────

/// Read a numeric value from a YAML scalar that may be a number or a numeric string.
fn yaml_num(v: &serde_yaml::Value) -> Option<f64> {
    match v {
        serde_yaml::Value::Number(n) => n.as_f64(),
        serde_yaml::Value::String(s) => s.trim().parse::<f64>().ok(),
        _ => None,
    }
}

/// Numeric `value:`/`default:` of an inline `features:` entry named `name`.
fn feature_value(fm: &crate::element::RawFrontmatter, name: &str) -> Option<f64> {
    fm.features.as_ref()?.iter().find_map(|f| {
        let m = f.as_mapping()?;
        if yaml_field(m, "name").and_then(|v| v.as_str()) != Some(name) {
            return None;
        }
        yaml_field(m, "value").or_else(|| yaml_field(m, "default")).and_then(yaml_num)
    })
}

/// Top-level numeric `value:` of an element's frontmatter.
fn scalar_value(fm: &crate::element::RawFrontmatter) -> Option<f64> {
    fm.value.as_ref().and_then(yaml_num)
}

/// Resolve a budget `feature_ref` operand to a number: a bare name on the CalculationDef's
/// own features; a full qualified name carrying a scalar value; or `<owner>::<feature>`.
fn resolve_budget_operand(
    r: &str,
    calc_fm: &crate::element::RawFrontmatter,
    elements: &[RawElement],
    resolver: &Resolver,
) -> Option<f64> {
    if !r.contains("::") {
        if let Some(v) = feature_value(calc_fm, r) {
            return Some(v);
        }
    }
    if let Some(el) = resolver.resolve_ref(elements, r) {
        if let Some(v) = scalar_value(&el.frontmatter) {
            return Some(v);
        }
    }
    if let Some(pos) = r.rfind("::") {
        if let Some(el) = resolver.resolve_ref(elements, &r[..pos]) {
            if let Some(v) = feature_value(&el.frontmatter, &r[pos + 2..]) {
                return Some(v);
            }
        }
    }
    None
}

#[derive(Debug, PartialEq)]
enum BudTok {
    Num(f64),
    Ref(String),
    Plus,
    Minus,
    Star,
    Slash,
    LParen,
    RParen,
}

fn budget_tokenize(s: &str) -> Result<Vec<BudTok>, String> {
    let b = s.as_bytes();
    let mut i = 0;
    let mut toks = Vec::new();
    while i < b.len() {
        let c = b[i];
        if c.is_ascii_whitespace() {
            i += 1;
        } else if c == b'+' {
            toks.push(BudTok::Plus);
            i += 1;
        } else if c == b'-' {
            toks.push(BudTok::Minus);
            i += 1;
        } else if c == b'*' {
            toks.push(BudTok::Star);
            i += 1;
        } else if c == b'/' {
            toks.push(BudTok::Slash);
            i += 1;
        } else if c == b'(' {
            toks.push(BudTok::LParen);
            i += 1;
        } else if c == b')' {
            toks.push(BudTok::RParen);
            i += 1;
        } else if c.is_ascii_digit() || c == b'.' {
            let start = i;
            while i < b.len() && (b[i].is_ascii_digit() || b[i] == b'.' || b[i] == b'e' || b[i] == b'E') {
                i += 1;
            }
            let num = &s[start..i];
            toks.push(BudTok::Num(num.parse::<f64>().map_err(|_| format!("invalid number '{}'", num))?));
        } else if c.is_ascii_alphabetic() || c == b'_' {
            let start = i;
            while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_' || b[i] == b':') {
                i += 1;
            }
            toks.push(BudTok::Ref(s[start..i].to_string()));
        } else {
            return Err(format!("unexpected character '{}'", c as char));
        }
    }
    Ok(toks)
}

fn bud_factor(t: &[BudTok], p: &mut usize, res: &dyn Fn(&str) -> Option<f64>, unres: &mut Vec<String>) -> Result<f64, String> {
    match t.get(*p) {
        Some(BudTok::Num(n)) => {
            *p += 1;
            Ok(*n)
        }
        Some(BudTok::Ref(r)) => {
            *p += 1;
            match res(r) {
                Some(v) => Ok(v),
                None => {
                    unres.push(r.clone());
                    Ok(0.0)
                }
            }
        }
        Some(BudTok::LParen) => {
            *p += 1;
            let v = bud_expr(t, p, res, unres)?;
            if t.get(*p) != Some(&BudTok::RParen) {
                return Err("expected ')'".into());
            }
            *p += 1;
            Ok(v)
        }
        _ => Err("expected a number, reference, or '('".into()),
    }
}

fn bud_term(t: &[BudTok], p: &mut usize, res: &dyn Fn(&str) -> Option<f64>, unres: &mut Vec<String>) -> Result<f64, String> {
    let mut v = bud_factor(t, p, res, unres)?;
    while let Some(op) = t.get(*p) {
        match op {
            BudTok::Star => {
                *p += 1;
                v *= bud_factor(t, p, res, unres)?;
            }
            BudTok::Slash => {
                *p += 1;
                v /= bud_factor(t, p, res, unres)?;
            }
            _ => break,
        }
    }
    Ok(v)
}

fn bud_expr(t: &[BudTok], p: &mut usize, res: &dyn Fn(&str) -> Option<f64>, unres: &mut Vec<String>) -> Result<f64, String> {
    let mut v = bud_term(t, p, res, unres)?;
    while let Some(op) = t.get(*p) {
        match op {
            BudTok::Plus => {
                *p += 1;
                v += bud_term(t, p, res, unres)?;
            }
            BudTok::Minus => {
                *p += 1;
                v -= bud_term(t, p, res, unres)?;
            }
            _ => break,
        }
    }
    Ok(v)
}

/// Evaluate a budget expression. `Err` is a syntax error (E867); `Ok((value, unresolved))`
/// reports operands that resolved to no value (E868).
fn eval_budget(body: &str, res: &dyn Fn(&str) -> Option<f64>) -> Result<(f64, Vec<String>), String> {
    let toks = budget_tokenize(body)?;
    if toks.is_empty() {
        return Err("empty budget expression".into());
    }
    let mut p = 0;
    let mut unres = Vec::new();
    let v = bud_expr(&toks, &mut p, res, &mut unres)?;
    if p != toks.len() {
        return Err("trailing tokens after expression".into());
    }
    Ok((v, unres))
}

/// Reduce a constraint expression of the form `<lhs> <op> <number>` to `(op, bound)`,
/// for the best-effort W060 check. Compound constraints (`and`/`or`) return `None`.
fn constraint_simple_bound(expr: &str) -> Option<(&'static str, f64)> {
    if expr.contains(" and ") || expr.contains(" or ") {
        return None;
    }
    for op in ["<=", ">=", "==", "<", ">"] {
        if let Some(idx) = expr.find(op) {
            if let Ok(n) = expr[idx + op.len()..].trim().parse::<f64>() {
                return Some((op, n));
            }
        }
    }
    None
}

/// Map ASIL level string to a numeric rank for comparison (A=1, B=2, C=3, D=4).
fn asil_rank(level: &str) -> Option<u8> {
    match level.to_ascii_uppercase().as_str() {
        "A" => Some(1),
        "B" => Some(2),
        "C" => Some(3),
        "D" => Some(4),
        _ => None,
    }
}

/// Returns true when the child's integrity level is strictly lower than the source's.
/// Only comparable when both use the same standard; returns false for mixed standards.
fn integrity_is_lower(
    child_asil: Option<&str>, child_sil: Option<u8>,
    src_asil: Option<&str>,   src_sil:   Option<u8>,
) -> bool {
    if let (Some(ce), Some(se)) = (child_asil, src_asil) {
        let cr = asil_rank(ce).unwrap_or(0);
        let sr = asil_rank(se).unwrap_or(0);
        return cr < sr;
    }
    if let (Some(ce), Some(se)) = (child_sil, src_sil) {
        return ce < se;
    }
    false
}

/// Extract qualified name strings from a field that may be a YAML String or Sequence.
fn yaml_strings(v: &serde_yaml::Value) -> Vec<&str> {
    match v {
        serde_yaml::Value::String(s) => vec![s.as_str()],
        serde_yaml::Value::Sequence(seq) => seq.iter().filter_map(|x| x.as_str()).collect(),
        _ => vec![],
    }
}

/// Run all parse-time and model-time validation rules against a loaded element list.
///
/// Uses [`ValidateConfig::default`] — on-disk references resolve relative to the
/// current working directory. Callers that know the model root should prefer
/// [`validate_with_config`] so paths such as `sourceFile:` resolve correctly.
pub fn validate(elements: &[RawElement]) -> ValidationResult {
    validate_with_config(elements, &ValidateConfig::default())
}

/// Run all parse-time and model-time validation rules with explicit [`ValidateConfig`].
pub fn validate_with_config(elements: &[RawElement], config: &ValidateConfig) -> ValidationResult {
    let mut findings: Vec<Finding> = Vec::new();

    // Collect derive pass findings (E500, E501, E502) stored by the derive pass in walker.
    for elem in elements {
        for (code, file, message) in &elem.derive_findings {
            let sev = if code.starts_with('E') { Severity::Error } else { Severity::Warning };
            let static_code: &'static str = match code.as_str() {
                "E500" => "E500", "E501" => "E501", "E502" => "E502",
                _ => "E000",
            };
            findings.push(Finding { code: static_code, file: file.clone(), message: message.clone(), severity: sev });
        }
    }

    let resolver = Resolver::new(elements);

    // Segments some element claims as its own name (covers element names and
    // `_index.md` packages). A directory WITHOUT an `_index.md` owns no element,
    // so its namespace segment is W042-checked separately in the loop (GH #42).
    let owned_names: std::collections::HashSet<&str> = elements
        .iter()
        .filter_map(|e| e.qualified_name.rsplit("::").next())
        .collect();
    let mut flagged_dir_names: std::collections::HashSet<String> = std::collections::HashSet::new();

    // ── Parse-time checks (per-element) ──────────────────────────────────────

    for elem in elements {
        let file = elem.file_path.clone();
        let fm = &elem.frontmatter;

        // W041 (GH #39): custom_fields shape check. Each value must be a scalar
        // (string/number/bool/null) or a list of scalars; a nested map, or a list
        // containing a non-scalar, is flagged. Keys are freeform — only shape is
        // checked. Warning severity; gate with `--deny W041`.
        for (key, value) in &fm.custom_fields {
            if !is_custom_field_shape_ok(value) {
                findings.push(warning(
                    "W041",
                    &file,
                    &format!("custom field '{}' must be a scalar or a list of scalars", key),
                ));
            }
        }

        // E004: required fields for native elements
        if let Some(ElementType::TestCase) = &fm.element_type {
            if fm.id.is_none() {
                findings.push(error("E004", &file, "`id` is required on TestCase"));
            }
            if fm.name.is_none() {
                findings.push(error("E004", &file, "`name` is required on TestCase"));
            }
            if fm.status.is_none() {
                findings.push(error("E004", &file, "`status` is required on TestCase"));
            }
            if fm.test_level.is_none() {
                findings.push(error("E004", &file, "`testLevel` is required on TestCase"));
            }
            if fm.verifies.as_ref().map_or(true, |v| v.is_empty()) {
                findings.push(error("E013", &file, "`verifies` must have at least one entry on TestCase"));
            }
        }

        if let Some(ElementType::Requirement) = &fm.element_type {
            if let Some(ref id) = fm.id {
                if is_req_id(id) {
                    // native Requirement: check required fields
                    if fm.name.is_none() {
                        findings.push(error("E004", &file, "`name` is required on native Requirement"));
                    }
                    if fm.status.is_none() {
                        findings.push(error("E004", &file, "`status` is required on native Requirement"));
                    }
                }
            }
        }

        // E006: id pattern validation
        if let Some(ref id) = fm.id {
            let ty = &fm.element_type;
            let is_req = matches!(ty, Some(ElementType::Requirement));
            let is_tc = matches!(ty, Some(ElementType::TestCase));
            if is_req && !is_req_id(id) && !id.is_empty() {
                findings.push(error("E006", &file, &format!("`id` '{}' does not match REQ pattern", id)));
            }
            if is_tc && !is_tc_id(id) && !id.is_empty() {
                findings.push(error("E006", &file, &format!("`id` '{}' does not match TC pattern", id)));
            }
            // FeatureDef carries an OPTIONAL stable id; when present it must match the
            // FEAT-* pattern (REQ-TRS-ID-006). A FeatureDef without an id is unchanged;
            // its name remains the identity segment and is W042-checked separately.
            let is_feature_def = matches!(ty, Some(ElementType::FeatureDef));
            if is_feature_def && !crate::resolver::is_feat_id(id) && !id.is_empty() {
                findings.push(error(
                    "E006",
                    &file,
                    &format!("`id` '{}' does not match FEAT pattern", id),
                ));
            }
        }

        // E023: a stable-ID numeric suffix wider than the configured maximum
        // (REQ-TRS-ID-005 / GH #41). The grammar accepts 3+ digits structurally so
        // a long id still resolves; the digit cap is enforced here as a policy.
        // Only applies to a *numeric* final segment: FEAT ids need not end in a number
        // (REQ-TRS-ID-006), so a non-numeric trailing segment is not a digit suffix.
        if let Some(ref id) = fm.id {
            let last_seg = id.rsplit('-').next().unwrap_or("");
            let numeric_suffix = !last_seg.is_empty() && last_seg.bytes().all(|b| b.is_ascii_digit());
            if is_stable_id(id) && numeric_suffix {
                let suffix_len = last_seg.len();
                let max = config.id_digit_max();
                if suffix_len > max {
                    findings.push(error(
                        "E023",
                        &file,
                        &format!(
                            "`id` '{}' has a {}-digit suffix, exceeding the configured maximum of {} (`[ids] max_digits`)",
                            id, suffix_len, max
                        ),
                    ));
                }
            }
        }

        // E025: `name` is the single human-readable label on every element type
        // (REQ-TRS-NAME-002). The `title` field is removed as a label, so declaring
        // `title:` on ANY element — id-identified or name-identified — is E025. E024
        // (formerly: `name` on an id-identified type) is retired: `name` is now the
        // correct label on those types.
        if fm.title.is_some() {
            findings.push(error(
                "E025",
                &file,
                "the `title` field is removed — rename it to `name` (every element labels via `name`)",
            ));
        }

        // W043: a type reference into a known auto-imported built-in package
        // (`ScalarValues`, `Base`) naming a member that package does not declare — a
        // likely typo (REQ-TRS-LIB-001). Recognised members resolve cleanly; the
        // import-only packages (`SI`, `ISQ`, …) are not enumerated and never flagged.
        {
            use crate::resolver::{builtin_type_kind, BuiltinType};
            let k = |s: &str| serde_yaml::Value::String(s.to_string());
            let mut type_refs: Vec<&str> = Vec::new();
            if let Some(v) = &fm.supertype {
                type_refs.extend(yaml_strings(v));
            }
            if let Some(v) = &fm.typed_by {
                type_refs.extend(yaml_strings(v));
            }
            if let Some(rt) = &fm.return_type {
                type_refs.push(rt.as_str());
            }
            for v in fm
                .features
                .iter()
                .flatten()
                .chain(fm.connections.iter().flatten())
            {
                if let serde_yaml::Value::Mapping(m) = v {
                    if let Some(tb) = m.get(&k("typedBy")) {
                        type_refs.extend(yaml_strings(tb));
                    }
                }
            }
            for v in fm.parameters.iter().flatten() {
                if let serde_yaml::Value::Mapping(m) = v {
                    if let Some(t) = m.get(&k("type")) {
                        type_refs.extend(yaml_strings(t));
                    }
                }
            }
            for op in fm.operations.iter().flatten() {
                if let serde_yaml::Value::Mapping(m) = op {
                    if let Some(rt) = m.get(&k("returnType")) {
                        type_refs.extend(yaml_strings(rt));
                    }
                    if let Some(serde_yaml::Value::Sequence(params)) = m.get(&k("parameters")) {
                        for p in params {
                            if let serde_yaml::Value::Mapping(pm) = p {
                                if let Some(tb) = pm.get(&k("typedBy")) {
                                    type_refs.extend(yaml_strings(tb));
                                }
                            }
                        }
                    }
                }
            }
            for r in &type_refs {
                if let BuiltinType::UnknownMember { pkg, known, .. } = builtin_type_kind(r) {
                    findings.push(warning(
                        "W043",
                        &file,
                        &format!(
                            "'{}' is not a member of the built-in package `{}` (known members: {}) — check for a typo",
                            r, pkg, known.join(", ")
                        ),
                    ));
                }
            }
        }

        // W044: dimensional consistency between an element/feature's quantity type and
        // its unit (REQ-TRS-LIB-003). Fires only when BOTH the `typedBy:` (or a
        // parameter `type:`) resolves to a recognised ISQ quantity and the `unit:`
        // resolves to a recognised SI unit, and the two dimensions differ. Lenient when
        // either side is unrecognised.
        {
            use crate::units::{quantity_dimension, unit_dimension};
            let kk = |s: &str| serde_yaml::Value::String(s.to_string());
            for v in fm.features.iter().flatten().chain(fm.parameters.iter().flatten()) {
                if let serde_yaml::Value::Mapping(m) = v {
                    let q = m
                        .get(&kk("typedBy"))
                        .or_else(|| m.get(&kk("type")))
                        .and_then(|x| x.as_str());
                    let u = m.get(&kk("unit")).and_then(|x| x.as_str());
                    if let (Some(q), Some(u)) = (q, u) {
                        if let (Some(qd), Some(ud)) = (quantity_dimension(q), unit_dimension(u)) {
                            if qd != ud {
                                findings.push(warning(
                                    "W044",
                                    &file,
                                    &format!(
                                        "unit '{}' (dimension {}) is dimensionally inconsistent with quantity type '{}' (dimension {})",
                                        u, ud.human(), q, qd.human()
                                    ),
                                ));
                            }
                        }
                    }
                }
            }
        }

        // E317 / E318 / W045: stereotype applications via `metadata:` (REQ-TRS-META-001).
        // Each application must resolve to a MetadataDef (E317); if that def declares
        // `annotates:`, this element's type must be allowed (E318); each tagged-value key
        // must be a declared feature of the def (W045). Standard-library metadata packages
        // are recognised (no E317), and abstract metaclasses (Element/Definition/Usage) match.
        for app in crate::element::metadata_applications(&fm.metadata) {
            let stdlib_meta = matches!(
                app.def.split("::").next(),
                Some("ModelingMetadata") | Some("RiskMetadata")
            );
            match resolver.resolve_ref(elements, &app.def) {
                Some(def_el)
                    if matches!(
                        def_el.frontmatter.element_type,
                        Some(crate::element::ElementType::MetadataDef)
                    ) =>
                {
                    // E318: applicability constraint (`annotates:` metaclass names).
                    if let Some(ref allowed) = def_el.frontmatter.annotates {
                        let this_ty = fm
                            .element_type
                            .as_ref()
                            .map(|t| format!("{:?}", t))
                            .unwrap_or_default();
                        let is_def = this_ty.ends_with("Def");
                        let matches_mc = |mc: &str| -> bool {
                            mc == this_ty
                                || mc == "Element"
                                || (mc == "Definition" && is_def)
                                || (mc == "Usage" && !is_def)
                        };
                        if !allowed.iter().any(|a| matches_mc(a)) {
                            findings.push(error(
                                "E318",
                                &file,
                                &format!(
                                    "stereotype '{}' does not annotate a {} (its annotates is [{}])",
                                    app.def, this_ty, allowed.join(", ")
                                ),
                            ));
                        }
                    }
                    // W045: tagged-value keys must be declared features of the def.
                    let feat_names: Vec<String> = def_el
                        .frontmatter
                        .features
                        .iter()
                        .flatten()
                        .filter_map(|f| match f {
                            serde_yaml::Value::Mapping(m) => m
                                .get(serde_yaml::Value::from("name"))
                                .and_then(|v| v.as_str())
                                .map(str::to_string),
                            _ => None,
                        })
                        .collect();
                    for (k, _) in &app.values {
                        if !feat_names.contains(k) {
                            findings.push(warning(
                                "W045",
                                &file,
                                &format!(
                                    "tagged value '{}' is not a declared feature of stereotype '{}'",
                                    k, app.def
                                ),
                            ));
                        }
                    }
                }
                _ => {
                    // Recognised standard-library metadata packages resolve from the
                    // built-in inventory (no in-model file) — not an error.
                    if !stdlib_meta {
                        findings.push(error(
                            "E317",
                            &file,
                            &format!("`metadata:` application '{}' does not resolve to a MetadataDef", app.def),
                        ));
                    }
                }
            }
        }

        // W042: an element name that is not a SysMLv2 basic name (REQ-TRS-NAME-001 /
        // GH #42). The element's own name is the last `::` segment of its qualified
        // name; stable ids (REQ-*, TC-*, …) legitimately contain '-' and are exempt.
        // A non-basic name cannot be referenced in the tokenized expression contexts
        // (appliesWhen, parameterConstraints), where '-' is the subtraction operator.
        // Also exempt: elements whose declared `id:` is a stable id AND whose qname
        // segment differs from their `name:` label — the file stem is a lookup handle,
        // not the element's SysML name (e.g. `AOU-001-Desc.md` with `id: AOU-001`;
        // the label lives in `name:`). FeatureDef carries a stable id but is
        // name-identified (its `name` IS the qname segment), so it is not exempt.
        let elem_has_stable_id = fm.id.as_deref().is_some_and(is_stable_id);
        if let Some(seg) = elem.qualified_name.rsplit("::").next() {
            let seg_is_the_name = fm.name.as_deref().is_some_and(|n| n == seg);
            let id_identified_stem = elem_has_stable_id && !seg_is_the_name;
            if !seg.is_empty() && !is_basic_name(seg) && !is_stable_id(seg) && !id_identified_stem {
                let is_feature_def = matches!(fm.element_type, Some(ElementType::FeatureDef));
                let suffix = if is_feature_def {
                    " — a hyphen in a feature name causes E209 when the feature appears in an appliesWhen expression"
                } else {
                    ""
                };
                findings.push(warning(
                    "W042",
                    &file,
                    &format!(
                        "qualified-name segment '{}' is not a SysMLv2 basic name (letters/digits/_, not starting with a digit); rename using '_' or CamelCase{}",
                        seg, suffix
                    ),
                ));
            }
        }

        // W042 (namespace/directory segments): an ANCESTOR segment of this element's
        // qualified name that no element owns — a directory without an `_index.md` —
        // is still a referenceable namespace segment, so it must be a basic name too.
        // Flagged once per distinct directory name, attributed to the directory.
        let segs: Vec<&str> = elem.qualified_name.split("::").collect();
        if segs.len() >= 2 {
            for seg in &segs[..segs.len() - 1] {
                if seg.is_empty()
                    || owned_names.contains(seg)
                    || flagged_dir_names.contains(*seg)
                    || is_basic_name(seg)
                    || is_stable_id(seg)
                {
                    continue;
                }
                flagged_dir_names.insert((*seg).to_string());
                let parts: Vec<&str> = elem.file_path.split('/').collect();
                let dir = parts
                    .iter()
                    .position(|p| p == seg)
                    .map(|i| parts[..=i].join("/"))
                    .unwrap_or_else(|| file.clone());
                findings.push(warning(
                    "W042",
                    &dir,
                    &format!(
                        "namespace/directory name '{}' is not a SysMLv2 basic name (letters/digits/_); rename the directory using '_' or CamelCase",
                        seg
                    ),
                ));
            }
        }

        // E007: status enum
        if let Some(ref status) = fm.status {
            let ty = &fm.element_type;
            let is_tc = matches!(ty, Some(ElementType::TestCase));
            let is_req = matches!(ty, Some(ElementType::Requirement));
            if is_req {
                const REQ_STATUSES: &[&str] = &["draft", "review", "approved", "implemented", "verified"];
                if !REQ_STATUSES.contains(&status.as_str()) {
                    findings.push(error("E007", &file, &format!("unknown Requirement status '{}'", status)));
                }
            }
            if is_tc {
                const TC_STATUSES: &[&str] = &["draft", "review", "approved", "active", "retired"];
                if !TC_STATUSES.contains(&status.as_str()) {
                    findings.push(error("E007", &file, &format!("unknown TestCase status '{}'", status)));
                }
            }
        }

        // E008: testLevel
        if let Some(ref lvl) = fm.test_level {
            const LEVELS: &[&str] = &["L1", "L2", "L3", "L4", "L5"];
            if !LEVELS.contains(&lvl.as_str()) {
                findings.push(error("E008", &file, &format!("unknown testLevel '{}'", lvl)));
            }
        }
        // W809: securityTestMethod must be a recognised ISO/SAE 21434 §13 test method (REQ-TRS-SEC-008).
        if let Some(ref m) = fm.security_test_method {
            const METHODS: &[&str] = &["fuzz", "penetration_test", "security_regression", "vulnerability_scan", "threat_modeling"];
            if !METHODS.contains(&m.as_str()) {
                findings.push(warning("W809", &file, &format!(
                    "TestCase.securityTestMethod '{}' is not a recognised security test method — expected fuzz, penetration_test, security_regression, vulnerability_scan, or threat_modeling", m)));
            }
        }

        // ── Native TestPlan schema checks (GH #38; REQ-TRS-PLAN-001..004) ────
        if matches!(fm.element_type, Some(ElementType::TestPlan)) {
            // E600: required id / title / status, and TP-* id pattern.
            match &fm.id {
                None => findings.push(error("E600", &file, "`id` is required on TestPlan")),
                Some(id) if !is_test_plan_id(id) => findings.push(error(
                    "E600",
                    &file,
                    &format!("`id` '{}' does not match TP-* pattern", id),
                )),
                Some(_) => {}
            }
            if fm.name.is_none() {
                findings.push(error("E600", &file, "`name` is required on TestPlan"));
            }
            if fm.status.is_none() {
                findings.push(error("E600", &file, "`status` is required on TestPlan"));
            }

            // E604: status enum.
            if let Some(status) = &fm.status {
                const TP_STATUSES: &[&str] =
                    &["draft", "review", "approved", "active", "retired"];
                if !TP_STATUSES.contains(&status.as_str()) {
                    findings.push(error(
                        "E604",
                        &file,
                        &format!("unknown TestPlan status '{}'", status),
                    ));
                }
            }

            // W610: scope outside the recommended vocabulary (free-form accepted).
            if let Some(scope) = &fm.scope {
                const SCOPES: &[&str] = &[
                    "unit",
                    "smoke",
                    "integration",
                    "hil",
                    "certification",
                    "security",
                    "regression",
                ];
                if !SCOPES.contains(&scope.as_str()) {
                    findings.push(warning(
                        "W610",
                        &file,
                        &format!(
                            "scope '{}' is not in the recommended vocabulary (unit|smoke|integration|hil|certification|security|regression)",
                            scope
                        ),
                    ));
                }
            }

            // E602: selection.testLevels ⊆ L1–L5.  E605: selection.domains ⊆ system/hardware/software.
            if let Some(sel) = &fm.selection {
                if let Some(levels) = &sel.test_levels {
                    const LEVELS: &[&str] = &["L1", "L2", "L3", "L4", "L5"];
                    for lvl in levels {
                        if !LEVELS.contains(&lvl.as_str()) {
                            findings.push(error(
                                "E602",
                                &file,
                                &format!("selection.testLevels value '{}' is not one of L1–L5", lvl),
                            ));
                        }
                    }
                }
                if let Some(domains) = &sel.domains {
                    const DOMAINS: &[&str] = &["system", "hardware", "software"];
                    for d in domains {
                        if !DOMAINS.contains(&d.as_str()) {
                            findings.push(error(
                                "E605",
                                &file,
                                &format!(
                                    "selection.domains value '{}' is not one of system/hardware/software",
                                    d
                                ),
                            ));
                        }
                    }
                }
            }

            // E601: each testCases entry must resolve to a TestCase.
            // W613: an explicitly named TestCase whose status is draft/retired.
            if let Some(refs) = &fm.test_cases {
                for r in refs {
                    match resolver.resolve_ref(elements, r) {
                        Some(tc) if matches!(tc.frontmatter.element_type, Some(ElementType::TestCase)) => {
                            let st = tc.frontmatter.status.as_deref().unwrap_or("");
                            if st == "draft" || st == "retired" {
                                findings.push(warning(
                                    "W613",
                                    &file,
                                    &format!(
                                        "testCases names '{}' whose status is '{}' (a not-ready TestCase pinned into the plan)",
                                        r, st
                                    ),
                                ));
                            }
                        }
                        _ => findings.push(error(
                            "E601",
                            &file,
                            &format!("testCases entry '{}' does not resolve to a TestCase", r),
                        )),
                    }
                }
            }

            // E603: each demonstrates entry must resolve to a
            // Requirement/SafetyGoal/CybersecurityGoal/Argument.
            if let Some(refs) = &fm.demonstrates {
                for r in refs {
                    let ok = matches!(
                        resolver
                            .resolve_ref(elements, r)
                            .and_then(|t| t.frontmatter.element_type.clone()),
                        Some(
                            ElementType::Requirement
                                | ElementType::SafetyGoal
                                | ElementType::CybersecurityGoal
                                | ElementType::Argument
                        )
                    );
                    if !ok {
                        findings.push(error(
                            "E603",
                            &file,
                            &format!(
                                "demonstrates entry '{}' does not resolve to a Requirement/SafetyGoal/CybersecurityGoal/Argument",
                                r
                            ),
                        ));
                    }
                }
            }

            // E606: each configurations entry must resolve to a Configuration.
            if let Some(refs) = &fm.configurations {
                for r in refs {
                    let ok = resolver
                        .resolve_ref(elements, r)
                        .map(Resolver::is_configuration)
                        .unwrap_or(false);
                    if !ok {
                        findings.push(error(
                            "E606",
                            &file,
                            &format!("configurations entry '{}' does not resolve to a Configuration", r),
                        ));
                    }
                }
            }

            // ── computed-membership checks ───────────────────────────────────
            let members = crate::testplan::effective_testcases(elem, elements, &resolver);
            let configs = crate::testplan::plan_configs(elem, elements, &resolver);
            let status = fm.status.as_deref().unwrap_or("");

            // W612: empty effective TestCase set.
            if members.is_empty() {
                findings.push(warning(
                    "W612",
                    &file,
                    "TestPlan has an empty effective TestCase set",
                ));
            }

            // W611: a member TestCase active in NONE of the plan's configs
            // (escaping member). Dormant when the variability dimension is
            // inactive (no resolvable bound configs).
            if !configs.is_empty() {
                let pkg = crate::variability::package_conditions(elements);
                let feat_alias = crate::variability::feature_id_to_qname(elements);
                for tc in &members {
                    if !crate::testplan::member_active_in_any_config(tc, &configs, &pkg, &feat_alias) {
                        let tc_id = tc
                            .frontmatter
                            .id
                            .as_deref()
                            .unwrap_or(tc.qualified_name.as_str());
                        findings.push(warning(
                            "W611",
                            &file,
                            &format!(
                                "member TestCase '{}' is active in none of the plan's bound configurations (escaping member)",
                                tc_id
                            ),
                        ));
                    }
                }
            }

            // W614: an approved/active plan whose demonstrates names a Requirement
            // that no member TestCase verifies.
            if status == "approved" || status == "active" {
                if let Some(refs) = &fm.demonstrates {
                    for r in refs {
                        let target = match resolver.resolve_ref(elements, r) {
                            Some(t) if matches!(t.frontmatter.element_type, Some(ElementType::Requirement)) => t,
                            _ => continue,
                        };
                        // Goal-closure: the demonstrated requirement is covered if a
                        // member verifies it OR any requirement that derivesFrom it
                        // (transitively). A plan demonstrating a high-level/parent goal
                        // whose leaves are tested is the normal safety-case pattern —
                        // requiring a member to verify the parent directly would be the
                        // same parent/leaf false positive suppressed elsewhere (cf. GH
                        // #37, E312: a parent is verified through its leaves).
                        let covered = members.iter().any(|tc| {
                            tc.frontmatter.verifies.as_ref().is_some_and(|vs| {
                                vs.iter().any(|v| {
                                    resolver.resolve_ref(elements, v).is_some_and(|rt| {
                                        req_self_or_descendant_of(rt, target, elements, &resolver)
                                    })
                                })
                            })
                        });
                        if !covered {
                            findings.push(warning(
                                "W614",
                                &file,
                                &format!(
                                    "{} TestPlan demonstrates Requirement '{}' but no member TestCase verifies it",
                                    status, r
                                ),
                            ));
                        }
                    }
                }
            }

            // W615: results-gated — an approved plan with a member whose ingested
            // verdict is Fail/Missing. Only when a results sidecar is loaded.
            if status == "approved" {
                if let Some(results) = &config.results {
                    use crate::results::FnVerdict;
                    let func_key = serde_yaml::Value::String("function".into());
                    for tc in &members {
                        let Some(fns) = &tc.frontmatter.test_functions else {
                            continue;
                        };
                        for tf in fns {
                            if let serde_yaml::Value::Mapping(map) = tf {
                                if let Some(serde_yaml::Value::String(func)) = map.get(&func_key) {
                                    let v = results.verdict_for(func);
                                    if matches!(v, FnVerdict::Fail | FnVerdict::Missing) {
                                        let tc_id = tc
                                            .frontmatter
                                            .id
                                            .as_deref()
                                            .unwrap_or(tc.qualified_name.as_str());
                                        let what = if v == FnVerdict::Fail { "FAILED" } else { "was missing from" };
                                        findings.push(warning(
                                            "W615",
                                            &file,
                                            &format!(
                                                "approved TestPlan member TestCase '{}' has test function '{}' that {} the ingested results",
                                                tc_id, func, what
                                            ),
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // E009: silLevel 1–4
        if let Some(sil) = fm.sil_level {
            if !(1..=4).contains(&sil) {
                findings.push(error("E009", &file, &format!("silLevel {} out of range 1–4", sil)));
            }
        }

        // E010: asilLevel A–D
        if let Some(ref asil) = fm.asil_level {
            const ASIL: &[&str] = &["A", "B", "C", "D"];
            if !ASIL.contains(&asil.as_str()) {
                findings.push(error("E010", &file, &format!("unknown asilLevel '{}'", asil)));
            }
        }

        // E846: diagnosticCoverage / latentDiagnosticCoverage must be in 0.0–1.0
        // (ISO 26262-5 §8-9, GH #29). Documented for FaultTreeEvent but checked
        // generically wherever the fields appear.
        for (label, val) in [
            ("diagnosticCoverage", fm.diagnostic_coverage),
            ("latentDiagnosticCoverage", fm.latent_diagnostic_coverage),
        ] {
            if let Some(v) = val {
                if !(0.0..=1.0).contains(&v) {
                    findings.push(error("E846", &file,
                        &format!("`{}` {} is out of range 0.0–1.0", label, v)));
                }
            }
        }

        // E019: dalLevel A–E
        if let Some(ref dal) = fm.dal_level {
            const DAL: &[&str] = &["A", "B", "C", "D", "E"];
            if !DAL.contains(&dal.as_str()) {
                findings.push(error("E019", &file, &format!("unknown dalLevel '{}' — must be A, B, C, D, or E", dal)));
            }
        }

        // E020: verificationMethod enum
        if let Some(ref vm) = fm.verification_method {
            const METHODS: &[&str] = &["test", "inspection", "analysis", "demonstration"];
            if !METHODS.contains(&vm.as_str()) {
                findings.push(error("E020", &file, &format!("unknown verificationMethod '{}' — must be test, inspection, analysis, or demonstration", vm)));
            }
        }

        // E021: coverageTarget enum
        if let Some(ref ct) = fm.coverage_target {
            const TARGETS: &[&str] = &["statement", "branch", "MCDC"];
            if !TARGETS.contains(&ct.as_str()) {
                findings.push(error("E021", &file, &format!("unknown coverageTarget '{}' — must be statement, branch, or MCDC", ct)));
            }
        }

        // E022: requirementKind enum
        if let Some(ref rk) = fm.requirement_kind {
            const KINDS: &[&str] = &["stakeholder", "system", "software", "hardware"];
            if !KINDS.contains(&rk.as_str()) {
                findings.push(error("E022", &file, &format!("unknown requirementKind '{}' — must be stakeholder, system, software, or hardware", rk)));
            }
        }

        // W701: Requirement with asilLevel B/C/D should have verificationMethod
        if let Some(ElementType::Requirement) = &fm.element_type {
            if let Some(ref asil) = fm.asil_level {
                if matches!(asil.as_str(), "B" | "C" | "D") && fm.verification_method.is_none() {
                    findings.push(warning(
                        "W701",
                        &file,
                        &format!("Requirement with asilLevel: {} has no verificationMethod — add the frontmatter line `verificationMethod: test` (or: inspection | analysis | demonstration)", asil),
                    ));
                }
            }
        }

        // W807: security requirement (derivedFromCybersecurityGoal set) should have verificationMethod
        if matches!(fm.element_type, Some(ElementType::Requirement))
            && fm.derived_from_cybersecurity_goal.is_some()
            && fm.verification_method.is_none()
        {
            findings.push(warning(
                "W807",
                &file,
                "security Requirement (derivedFromCybersecurityGoal set) has no verificationMethod — add test, inspection, analysis, or demonstration",
            ));
        }

        // W703: asilLevel and dalLevel both present — these are different standards
        if fm.asil_level.is_some() && fm.dal_level.is_some() {
            findings.push(warning(
                "W703",
                &file,
                "both asilLevel (ISO 26262) and dalLevel (DO-178C) are set — these are different standards; validate under one or document the mapping",
            ));
        }

        // ── Tier 2: HazardousEvent (E800-E804) ───────────────────────────────
        if matches!(fm.element_type, Some(ElementType::HazardousEvent)) {
            // E800: required fields
            if fm.id.is_none() { findings.push(error("E800", &file, "`id` is required on HazardousEvent")); }
            if fm.name.is_none() { findings.push(error("E800", &file, "`name` is required on HazardousEvent")); }
            if fm.status.is_none() { findings.push(error("E800", &file, "`status` is required on HazardousEvent")); }
            // E804: id pattern
            if let Some(ref id) = fm.id {
                if !is_he_id(id) {
                    findings.push(error("E804", &file, &format!("`id` '{}' does not match HE-* pattern", id)));
                }
            }
            // E801: severity S0-S3
            if let Some(ref s) = fm.severity {
                if !["S0","S1","S2","S3"].contains(&s.as_str()) {
                    findings.push(error("E801", &file, &format!("HazardousEvent.severity '{}' must be S0, S1, S2, or S3", s)));
                }
            }
            // E802: exposure E0-E4
            if let Some(ref e) = fm.exposure {
                if !["E0","E1","E2","E3","E4"].contains(&e.as_str()) {
                    findings.push(error("E802", &file, &format!("HazardousEvent.exposure '{}' must be E0–E4", e)));
                }
            }
            // E803: controllability C0-C3
            if let Some(ref c) = fm.controllability {
                if !["C0","C1","C2","C3"].contains(&c.as_str()) {
                    findings.push(error("E803", &file, &format!("HazardousEvent.controllability '{}' must be C0, C1, C2, or C3", c)));
                }
            }
            // E833: IEC 61508 consequence Ca-Cd
            if let Some(ref c) = fm.consequence {
                if !["Ca","Cb","Cc","Cd"].contains(&c.as_str()) {
                    findings.push(error("E833", &file, &format!("HazardousEvent.consequence '{}' must be Ca, Cb, Cc, or Cd (IEC 61508 risk graph)", c)));
                }
            }
            // E834: IEC 61508 freqExposure Fa/Fb
            if let Some(ref fe) = fm.freq_exposure {
                if !["Fa","Fb"].contains(&fe.as_str()) {
                    findings.push(error("E834", &file, &format!("HazardousEvent.freqExposure '{}' must be Fa or Fb (IEC 61508 risk graph)", fe)));
                }
            }
            // E835: IEC 61508 avoidance Pa/Pb
            if let Some(ref av) = fm.avoidance {
                if !["Pa","Pb"].contains(&av.as_str()) {
                    findings.push(error("E835", &file, &format!("HazardousEvent.avoidance '{}' must be Pa or Pb (IEC 61508 risk graph)", av)));
                }
            }
            // E836: IEC 61508 demandRate W1-W3
            if let Some(ref dr) = fm.demand_rate {
                if !["W1","W2","W3"].contains(&dr.as_str()) {
                    findings.push(error("E836", &file, &format!("HazardousEvent.demandRate '{}' must be W1, W2, or W3 (IEC 61508 risk graph)", dr)));
                }
            }
        }

        // ── Tier 2: SafetyGoal (E805-E806, E837) ─────────────────────────────
        if matches!(fm.element_type, Some(ElementType::SafetyGoal)) {
            if fm.id.is_none() { findings.push(error("E805", &file, "`id` is required on SafetyGoal")); }
            if fm.name.is_none() { findings.push(error("E805", &file, "`name` is required on SafetyGoal")); }
            if fm.status.is_none() { findings.push(error("E805", &file, "`status` is required on SafetyGoal")); }
            if let Some(ref id) = fm.id {
                if !is_sg_id(id) {
                    findings.push(error("E806", &file, &format!("`id` '{}' does not match SG-* pattern", id)));
                }
            }
            // E837: plLevel enum (ISO 13849-1)
            if let Some(ref pl) = fm.pl_level {
                if !["a","b","c","d","e"].contains(&pl.as_str()) {
                    findings.push(error("E837", &file, &format!("SafetyGoal.plLevel '{}' must be a, b, c, d, or e (ISO 13849-1)", pl)));
                }
            }
            // W801: SafetyGoal should carry an integrity level (asilLevel, silLevel, or plLevel)
            if fm.asil_level.is_none() && fm.sil_level.is_none() && fm.pl_level.is_none() {
                findings.push(warning("W801", &file, "SafetyGoal has no integrity level — set asilLevel (ISO 26262), silLevel (IEC 61508), or plLevel (ISO 13849-1)"));
            }
        }

        // ── §8.18: GSN Argument (E852-E855, W040) ────────────────────────────
        // Issue #20 safety-argument layer. An Argument is a GSN node arguing for a
        // SafetyGoal or a parent Argument, discharged by evidence (Requirement /
        // TestCase / sub-Argument / AssumptionOfUse).
        if matches!(fm.element_type, Some(ElementType::Argument)) {
            if fm.id.is_none() { findings.push(error("E852", &file, "`id` is required on Argument")); }
            if fm.name.is_none() { findings.push(error("E852", &file, "`name` is required on Argument")); }
            if fm.status.is_none() { findings.push(error("E852", &file, "`status` is required on Argument")); }
            // E853: id pattern (ARG-*)
            if let Some(ref id) = fm.id {
                if !is_arg_id(id) {
                    findings.push(error("E853", &file, &format!("`id` '{}' does not match ARG-* pattern", id)));
                }
            }
            // E854: argumentType enum (absent → treated as claim).
            if let Some(ref at) = fm.argument_type {
                if !["claim","strategy","solution"].contains(&at.as_str()) {
                    findings.push(error("E854", &file, &format!("Argument.argumentType '{}' must be claim, strategy, or solution", at)));
                }
            }
            // E855: supports / evidence refs must resolve.
            if let Some(ref refs) = fm.supports {
                for r in refs {
                    if resolver.resolve_ref(elements, r).is_none() {
                        findings.push(error("E855", &file, &format!("Argument.supports '{}' does not resolve to any model element", r)));
                    }
                }
            }
            if let Some(ref refs) = fm.evidence {
                for r in refs {
                    if resolver.resolve_ref(elements, r).is_none() {
                        findings.push(error("E855", &file, &format!("Argument.evidence '{}' does not resolve to any model element", r)));
                    }
                }
            }
            // W040: a claim/strategy Argument that argues nothing (empty supports AND
            // empty evidence) is an orphan GSN node.
            let kind = fm.argument_type.as_deref().unwrap_or("claim");
            let no_supports = fm.supports.as_ref().map(|v| v.is_empty()).unwrap_or(true);
            let no_evidence = fm.evidence.as_ref().map(|v| v.is_empty()).unwrap_or(true);
            if matches!(kind, "claim" | "strategy") && no_supports && no_evidence {
                findings.push(warning("W040", &file, "Argument has neither `supports` nor `evidence` — an orphan GSN node arguing nothing"));
            }
        }

        // ── §8.18: GSN AssumptionOfUse / SRAC (E856-E858) ────────────────────
        if matches!(fm.element_type, Some(ElementType::AssumptionOfUse)) {
            if fm.id.is_none() { findings.push(error("E856", &file, "`id` is required on AssumptionOfUse")); }
            if fm.name.is_none() { findings.push(error("E856", &file, "`name` is required on AssumptionOfUse")); }
            if fm.status.is_none() { findings.push(error("E856", &file, "`status` is required on AssumptionOfUse")); }
            // E857: id pattern (AOU-*)
            if let Some(ref id) = fm.id {
                if !is_aou_id(id) {
                    findings.push(error("E857", &file, &format!("`id` '{}' does not match AOU-* pattern", id)));
                }
            }
            // E858: appliesTo refs must resolve.
            // E859: target must be SafetyGoal, CybersecurityGoal, Argument, or Requirement (REQ-TRS-SEC-004).
            if let Some(ref refs) = fm.applies_to {
                for r in refs {
                    match resolver.resolve_ref(elements, r) {
                        None => findings.push(error("E858", &file, &format!("AssumptionOfUse.appliesTo '{}' does not resolve to any model element", r))),
                        Some(target) => {
                            let ok = Resolver::is_safety_goal(target)
                                || Resolver::is_cybersecurity_goal(target)
                                || Resolver::is_argument(target)
                                || Resolver::is_native_requirement(target)
                                || matches!(target.frontmatter.element_type, Some(ElementType::Requirement));
                            if !ok {
                                findings.push(error("E859", &file, &format!(
                                    "AssumptionOfUse.appliesTo '{}' must be a SafetyGoal, CybersecurityGoal, Argument, or Requirement", r)));
                            }
                        }
                    }
                }
            }
        }

        // ── Tier 2: DamageScenario (E807-E810) ───────────────────────────────
        if matches!(fm.element_type, Some(ElementType::DamageScenario)) {
            if fm.id.is_none() { findings.push(error("E807", &file, "`id` is required on DamageScenario")); }
            if fm.name.is_none() { findings.push(error("E807", &file, "`name` is required on DamageScenario")); }
            if fm.status.is_none() { findings.push(error("E807", &file, "`status` is required on DamageScenario")); }
            if let Some(ref id) = fm.id {
                if !is_ds_id(id) {
                    findings.push(error("E808", &file, &format!("`id` '{}' does not match DS-* pattern", id)));
                }
            }
            // E809: damageSeverity enum
            if let Some(ref s) = fm.damage_severity {
                if !["severe","major","moderate","negligible"].contains(&s.as_str()) {
                    findings.push(error("E809", &file, &format!("DamageScenario.damageSeverity '{}' must be severe, major, moderate, or negligible", s)));
                }
            }
            // E810: impactCategories enum
            if let Some(ref cats) = fm.impact_categories {
                for cat in cats {
                    if !["safety","financial","operational","privacy"].contains(&cat.as_str()) {
                        findings.push(error("E810", &file, &format!("DamageScenario.impactCategories '{}' must be safety, financial, operational, or privacy", cat)));
                    }
                }
            }
            // E844: hazardRef must resolve to a HazardousEvent or SafetyGoal
            // (§T4 safety↔security co-engineering, ISO 26262 ⇄ ISO/SAE 21434).
            if let Some(ref refs) = fm.hazard_ref {
                for r in refs {
                    match resolver.resolve_ref(elements, r) {
                        None => findings.push(error("E844", &file, &format!("DamageScenario.hazardRef '{}' does not resolve to any model element", r))),
                        Some(target) => {
                            if !matches!(target.frontmatter.element_type, Some(ElementType::HazardousEvent) | Some(ElementType::SafetyGoal)) {
                                findings.push(error("E844", &file, &format!("DamageScenario.hazardRef '{}' must reference a HazardousEvent or SafetyGoal", r)));
                            }
                        }
                    }
                }
            }
            // W030: a safety-tagged DamageScenario with no hazardRef (the
            // cross-domain gap an FS+CS assessor looks for first). Opt-in:
            // only fires when impactCategories includes `safety`.
            let safety_tagged = fm.impact_categories.as_ref()
                .map(|c| c.iter().any(|x| x == "safety")).unwrap_or(false);
            let has_hazard_ref = fm.hazard_ref.as_ref().map(|r| !r.is_empty()).unwrap_or(false);
            if safety_tagged && !has_hazard_ref {
                findings.push(warning("W030", &file, "DamageScenario has impactCategories: safety but no hazardRef — link it to the HazardousEvent/SafetyGoal it endangers (ISO 26262 ⇄ ISO/SAE 21434 co-analysis)"));
            }
        }

        // ── Tier 2: ThreatScenario (E811-E814) ───────────────────────────────
        if matches!(fm.element_type, Some(ElementType::ThreatScenario)) {
            if fm.id.is_none() { findings.push(error("E811", &file, "`id` is required on ThreatScenario")); }
            if fm.name.is_none() { findings.push(error("E811", &file, "`name` is required on ThreatScenario")); }
            if fm.status.is_none() { findings.push(error("E811", &file, "`status` is required on ThreatScenario")); }
            if let Some(ref id) = fm.id {
                if !is_ts_id(id) {
                    findings.push(error("E812", &file, &format!("`id` '{}' does not match TS-* pattern", id)));
                }
            }
            // E813: attackFeasibility enum
            if let Some(ref f) = fm.attack_feasibility {
                if !["high","medium","low","very_low"].contains(&f.as_str()) {
                    findings.push(error("E813", &file, &format!("ThreatScenario.attackFeasibility '{}' must be high, medium, low, or very_low", f)));
                }
            }
            // E814: attackVector enum
            if let Some(ref v) = fm.attack_vector {
                if !["network","adjacent","local","physical"].contains(&v.as_str()) {
                    findings.push(error("E814", &file, &format!("ThreatScenario.attackVector '{}' must be network, adjacent, local, or physical", v)));
                }
            }
            // E845: riskTreatment enum (ISO/SAE 21434 §9 / §15.9 risk treatment).
            if let Some(ref rt) = fm.risk_treatment {
                if !["avoid","reduce","share","retain"].contains(&rt.as_str()) {
                    findings.push(error("E845", &file, &format!("ThreatScenario.riskTreatment '{}' must be avoid, reduce, share, or retain", rt)));
                }
            }
            // E844: a ThreatScenario's own direct hazardRef must resolve to a
            // HazardousEvent or SafetyGoal (§T4 safety↔security co-engineering).
            if let Some(ref refs) = fm.hazard_ref {
                for r in refs {
                    match resolver.resolve_ref(elements, r) {
                        None => findings.push(error("E844", &file, &format!("ThreatScenario.hazardRef '{}' does not resolve to any model element", r))),
                        Some(target) => {
                            if !matches!(target.frontmatter.element_type, Some(ElementType::HazardousEvent) | Some(ElementType::SafetyGoal)) {
                                findings.push(error("E844", &file, &format!("ThreatScenario.hazardRef '{}' must reference a HazardousEvent or SafetyGoal", r)));
                            }
                        }
                    }
                }
            }
        }

        // ── Tier 2: CybersecurityGoal (E815-E818) ────────────────────────────
        if matches!(fm.element_type, Some(ElementType::CybersecurityGoal)) {
            if fm.id.is_none() { findings.push(error("E815", &file, "`id` is required on CybersecurityGoal")); }
            if fm.name.is_none() { findings.push(error("E815", &file, "`name` is required on CybersecurityGoal")); }
            if fm.status.is_none() { findings.push(error("E815", &file, "`status` is required on CybersecurityGoal")); }
            if let Some(ref id) = fm.id {
                if !is_csg_id(id) {
                    findings.push(error("E816", &file, &format!("`id` '{}' does not match CSG-* pattern", id)));
                }
            }
            // E817: securityProperty enum
            if let Some(ref sp) = fm.security_property {
                if !["confidentiality","integrity","availability","authenticity"].contains(&sp.as_str()) {
                    findings.push(error("E817", &file, &format!("CybersecurityGoal.securityProperty '{}' must be confidentiality, integrity, availability, or authenticity", sp)));
                }
            }
            // E818: calLevel enum
            if let Some(ref cl) = fm.cal_level {
                if !["CAL1","CAL2","CAL3","CAL4"].contains(&cl.as_str()) {
                    findings.push(error("E818", &file, &format!("CybersecurityGoal.calLevel '{}' must be CAL1, CAL2, CAL3, or CAL4", cl)));
                }
            }
        }

        // ── ConfirmationMeasure (E847-E851) ──────────────────────────────────
        // REQ-TRS-SAFE-007 (ISO 26262-2 §6 confirmation measures / -8 §5 DIA).
        if matches!(fm.element_type, Some(ElementType::ConfirmationMeasure)) {
            if fm.id.is_none() { findings.push(error("E847", &file, "`id` is required on ConfirmationMeasure")); }
            if fm.name.is_none() { findings.push(error("E847", &file, "`name` is required on ConfirmationMeasure")); }
            if fm.status.is_none() { findings.push(error("E847", &file, "`status` is required on ConfirmationMeasure")); }
            // E848: id pattern (CM-*)
            if let Some(ref id) = fm.id {
                if !is_cm_id(id) {
                    findings.push(error("E848", &file, &format!("`id` '{}' does not match CM-* pattern", id)));
                }
            }
            // E849: measureType enum
            if let Some(ref mt) = fm.measure_type {
                if !["confirmation_review","functional_safety_audit","functional_safety_assessment","cybersecurity_assessment"].contains(&mt.as_str()) {
                    findings.push(error("E849", &file, &format!("ConfirmationMeasure.measureType '{}' must be confirmation_review, functional_safety_audit, functional_safety_assessment, or cybersecurity_assessment", mt)));
                }
            }
            // E850: independenceLevel enum
            if let Some(ref il) = fm.independence_level {
                if !["I1","I2","I3"].contains(&il.as_str()) {
                    findings.push(error("E850", &file, &format!("ConfirmationMeasure.independenceLevel '{}' must be I1, I2, or I3", il)));
                }
            }
            // E851: confirms refs must resolve.
            // E860: target must be SafetyGoal, CybersecurityGoal, HazardousEvent, or native Requirement (REQ-TRS-SEC-005).
            if let Some(ref refs) = fm.confirms {
                for r in refs {
                    match resolver.resolve_ref(elements, r) {
                        None => findings.push(error("E851", &file, &format!("ConfirmationMeasure.confirms '{}' does not resolve to any model element", r))),
                        Some(target) => {
                            let ok = Resolver::is_safety_goal(target)
                                || Resolver::is_cybersecurity_goal(target)
                                || Resolver::is_hazardous_event(target)
                                || Resolver::is_native_requirement(target)
                                || matches!(target.frontmatter.element_type, Some(ElementType::Requirement));
                            if !ok {
                                findings.push(error("E860", &file, &format!(
                                    "ConfirmationMeasure.confirms '{}' is not a valid confirmation target type (expected SafetyGoal, CybersecurityGoal, HazardousEvent, or Requirement)", r)));
                            }
                        }
                    }
                }
            }
        }

        // ── Asset (E861-E864; REQ-TRS-TYPE-017; ISO/SAE 21434 §15.3) ─────────
        if matches!(fm.element_type, Some(ElementType::Asset)) {
            if fm.id.is_none()     { findings.push(error("E861", &file, "`id` is required on Asset")); }
            if fm.name.is_none()   { findings.push(error("E861", &file, "`name` is required on Asset")); }
            if fm.status.is_none() { findings.push(error("E861", &file, "`status` is required on Asset")); }
            if let Some(ref id) = fm.id {
                if !is_asset_id(id) {
                    findings.push(error("E862", &file, &format!(
                        "`id` '{}' does not match ASSET-* pattern (^ASSET(-[A-Z0-9]{{2,12}})+-[0-9]{{3,}}$)", id)));
                }
            }
            const VALID_CP: &[&str] = &["confidentiality", "integrity", "availability", "authenticity"];
            if let Some(ref props) = fm.cybersecurity_properties {
                for p in props {
                    if !VALID_CP.contains(&p.as_str()) {
                        findings.push(error("E863", &file, &format!(
                            "Asset.cybersecurityProperties '{}' is not valid — expected confidentiality, integrity, availability, or authenticity", p)));
                    }
                }
            }
        }

        // E864: DamageScenario.assets refs must resolve to Asset elements (REQ-TRS-TYPE-017).
        if matches!(fm.element_type, Some(ElementType::DamageScenario)) {
            if let Some(ref asset_refs) = fm.assets {
                for r in asset_refs {
                    match resolver.resolve_ref(elements, r) {
                        None => findings.push(error("E864", &file, &format!(
                            "DamageScenario.assets '{}' does not resolve to any model element", r))),
                        Some(target) if !Resolver::is_asset(target) => findings.push(error("E864", &file, &format!(
                            "DamageScenario.assets '{}' does not resolve to an Asset element", r))),
                        _ => {}
                    }
                }
            }
        }

        // ── Tier 2: SecurityControl (E819-E821) ──────────────────────────────
        if matches!(fm.element_type, Some(ElementType::SecurityControl)) {
            if fm.id.is_none() { findings.push(error("E819", &file, "`id` is required on SecurityControl")); }
            if fm.name.is_none() { findings.push(error("E819", &file, "`name` is required on SecurityControl")); }
            if fm.status.is_none() { findings.push(error("E819", &file, "`status` is required on SecurityControl")); }
            if let Some(ref id) = fm.id {
                if !is_sc_id(id) {
                    findings.push(error("E820", &file, &format!("`id` '{}' does not match SC-* pattern", id)));
                }
            }
            // E821: controlType enum
            if let Some(ref ct) = fm.control_type {
                if !["prevention","detection","response","recovery"].contains(&ct.as_str()) {
                    findings.push(error("E821", &file, &format!("SecurityControl.controlType '{}' must be prevention, detection, response, or recovery", ct)));
                }
            }
        }

        // ── Tier 2: VulnerabilityReport (E822-E824) ──────────────────────────
        if matches!(fm.element_type, Some(ElementType::VulnerabilityReport)) {
            if fm.id.is_none() { findings.push(error("E822", &file, "`id` is required on VulnerabilityReport")); }
            if fm.name.is_none() { findings.push(error("E822", &file, "`name` is required on VulnerabilityReport")); }
            if fm.status.is_none() { findings.push(error("E822", &file, "`status` is required on VulnerabilityReport")); }
            if let Some(ref id) = fm.id {
                if !is_vr_id(id) {
                    findings.push(error("E823", &file, &format!("`id` '{}' does not match VR-* pattern", id)));
                }
            }
            // E824: cvssScore 0.0-10.0
            if let Some(score) = fm.cvss_score {
                if !(0.0..=10.0).contains(&score) {
                    findings.push(error("E824", &file, &format!("VulnerabilityReport.cvssScore {} is out of range 0.0–10.0", score)));
                }
            }
            // W803: open vulnerability reports draw attention
            if fm.status.as_deref() == Some("open") {
                findings.push(warning("W803", &file, "VulnerabilityReport has status: open — ensure it is being tracked and mitigated"));
            }
        }

        // E011: TestCase must have a gherkin block
        if matches!(fm.element_type, Some(ElementType::TestCase)) {
            if !elem.doc.contains("```gherkin") {
                findings.push(error("E011", &file, "TestCase body has no ```gherkin fenced block"));
            }
        }

        // E012: native Requirement normative text must be non-empty
        if let Some(ElementType::Requirement) = &fm.element_type {
            if fm.id.as_deref().map(is_req_id).unwrap_or(false) {
                let normative = normative_text(&elem.doc);
                if normative.trim().is_empty() {
                    findings.push(error("E012", &file, "Requirement normative text is empty"));
                }
            }
        }

        // E014: Scenario Outline without Examples table
        if matches!(fm.element_type, Some(ElementType::TestCase)) {
            check_scenario_outline_has_examples(&elem.doc, &file, &mut findings);
        }

        // E015: first gherkin block must have Feature: line
        if matches!(fm.element_type, Some(ElementType::TestCase)) {
            if !first_gherkin_has_feature(&elem.doc) {
                findings.push(error("E015", &file, "first ```gherkin block has no Feature: line"));
            }
        }

        // W001: normative text should contain "shall"
        if let Some(ElementType::Requirement) = &fm.element_type {
            if fm.id.as_deref().map(is_req_id).unwrap_or(false) {
                let normative = normative_text(&elem.doc);
                if !normative.contains("shall") {
                    findings.push(warning("W001", &file, "normative text contains no 'shall'"));
                }
            }
        }

        // W006: silLevel and asilLevel both set — incompatible standards
        if fm.sil_level.is_some() && fm.asil_level.is_some() {
            findings.push(warning("W006", &file,
                "both silLevel (IEC 61508) and asilLevel (ISO 26262) are set — these are incompatible standards; use only one"));
        }

        // Source-drift checks (W004/W009) are scoped to TestCase status (issue #6):
        //   active           -> "live": drift is a real defect, emit W004/W009.
        //   draft|review|approved -> "planned": sources may not exist yet, emit
        //                            informational I010 instead.
        //   retired (or unknown)  -> suppress entirely.
        // Non-TestCase elements with a sourceFile are always checked (W004).
        let is_tc = matches!(fm.element_type, Some(ElementType::TestCase));
        let tc_status = fm.status.as_deref().unwrap_or("");
        let drift_live = !is_tc || tc_status == "active";
        let drift_planned = is_tc && matches!(tc_status, "draft" | "review" | "approved");
        let drift_relevant = drift_live || drift_planned;

        // W004: sourceFile must exist. Local paths (model-/repo-relative, absolute,
        // or file://) are checked on disk. Remote URIs are accepted as external
        // and not verified locally — unless a download hook is enabled
        // (`--fetch-remote`), in which case a fetch failure is flagged (§11.12).
        if let Some(ref sf) = fm.source_file {
            if drift_relevant {
                let (missing, w004_msg, i010_msg) = match config.classify_source(sf) {
                    crate::config::SourceLocation::Local(p) => (
                        !p.exists(),
                        format!("sourceFile '{}' does not exist on disk", sf),
                        format!("planned TestCase (status: {}): sourceFile '{}' is not present yet", tc_status, sf),
                    ),
                    crate::config::SourceLocation::Remote(uri) => {
                        let miss = match &config.remote_hook {
                            Some(hook) => !hook.fetch(&uri).map_or(false, |p| p.exists()),
                            None => false, // remote, no hook: accepted, not checked
                        };
                        (
                            miss,
                            format!("remote sourceFile '{}' could not be retrieved via the configured download hook", sf),
                            format!("planned TestCase (status: {}): remote sourceFile '{}' could not be retrieved", tc_status, sf),
                        )
                    }
                };
                if missing {
                    if drift_live {
                        findings.push(warning("W004", &file, &w004_msg));
                    } else {
                        findings.push(info("I010", &file, &i010_msg));
                    }
                }
            }
        }

        // W023: implementedBy paths must exist (§12.7). The implementation trace
        // links an architecture element (Part/PartDef) to its source artifact(s).
        // Opt-in: only checked when implementedBy is present. Draft elements are
        // suppressed (the implementation may not exist yet). Local paths
        // (model-/repo-relative, absolute, or file://) are checked on disk; remote
        // URIs are accepted as external and not verified locally.
        if let Some(ref impls) = fm.implemented_by {
            let is_arch = matches!(
                fm.element_type,
                Some(ElementType::Part) | Some(ElementType::PartDef)
            );
            let is_draft = fm.status.as_deref() == Some("draft");
            if is_arch && !is_draft {
                for path in impls {
                    if let crate::config::SourceLocation::Local(p) = config.classify_source(path) {
                        if !p.exists() {
                            findings.push(warning("W023", &file, &format!(
                                "implementedBy path '{}' does not exist on disk", path,
                            )));
                        }
                    }
                }
            }
        }

        // W009: every testFunctions[].function must resolve to a definition in
        // sourceFile (function-level traceability — catches renamed/deleted tests
        // that W004's file-level check cannot see). Live TestCases drift to W009;
        // planned TestCases surface I010; remote (un-fetched) and retired are skipped.
        if let (Some(sf), Some(fns)) = (&fm.source_file, &fm.test_functions) {
            if drift_relevant {
                if let Some(src_path) = config.resolve_source_local(sf) {
                    if src_path.exists() {
                        use crate::matchers::FnResolution;
                        let func_key = serde_yaml::Value::String("function".into());
                        for tf in fns {
                            if let serde_yaml::Value::Mapping(map) = tf {
                                if let Some(serde_yaml::Value::String(func)) = map.get(&func_key) {
                                    if config.matchers.resolve(&src_path, func) == FnResolution::NotFound {
                                        if drift_live {
                                            findings.push(warning("W009", &file, &format!(
                                                "testFunction '{}' not found in sourceFile '{}'", func, sf,
                                            )));
                                        } else {
                                            findings.push(info("I010", &file, &format!(
                                                "planned TestCase (status: {}): testFunction '{}' not present in sourceFile '{}'",
                                                tc_status, func, sf,
                                            )));
                                        }
                                    }
                                    // Found / Unreadable: nothing.
                                }
                            }
                        }
                    }
                }
            }
        }

        // W010: ingested test results — an active/verified TestCase whose mapped
        // test function last failed or was absent from the run (issue #4).
        if let (Some(results), Some(ElementType::TestCase), Some(fns)) =
            (&config.results, &fm.element_type, &fm.test_functions)
        {
            let tc_status = fm.status.as_deref().unwrap_or("");
            if tc_status == "active" {
                use crate::results::FnVerdict;
                let func_key = serde_yaml::Value::String("function".into());
                for tf in fns {
                    if let serde_yaml::Value::Mapping(map) = tf {
                        if let Some(serde_yaml::Value::String(func)) = map.get(&func_key) {
                            match results.verdict_for(func) {
                                FnVerdict::Pass => {}
                                FnVerdict::Ignored => findings.push(warning(
                                    "W010",
                                    &file,
                                    &format!(
                                        "{} TestCase: test function '{}' was ignored/skipped in the ingested results",
                                        tc_status, func
                                    ),
                                )),
                                FnVerdict::Fail => findings.push(warning(
                                    "W010",
                                    &file,
                                    &format!(
                                        "{} TestCase: test function '{}' FAILED in the ingested results",
                                        tc_status, func
                                    ),
                                )),
                                FnVerdict::Missing => findings.push(warning(
                                    "W010",
                                    &file,
                                    &format!(
                                        "{} TestCase: test function '{}' was not present in the ingested results",
                                        tc_status, func
                                    ),
                                )),
                            }
                        }
                    }
                }
            }
        }

        // E200: Configuration id must match CONF-* pattern
        if matches!(fm.element_type, Some(ElementType::Configuration)) {
            if let Some(ref id) = fm.id {
                if !is_conf_id(id) {
                    findings.push(error("E200", &file, &format!("`id` '{}' does not match CONF-* pattern", id)));
                }
            }
        }

        // E201: Configuration required fields
        if matches!(fm.element_type, Some(ElementType::Configuration)) {
            if fm.id.is_none() {
                findings.push(error("E201", &file, "`id` is required on Configuration"));
            }
            if fm.name.is_none() {
                findings.push(error("E201", &file, "`name` is required on Configuration"));
            }
            if fm.status.is_none() {
                findings.push(error("E201", &file, "`status` is required on Configuration"));
            }
            // REQ-TRS-MG-011 — a Configuration opting in as a MagicGrid parametric
            // variant (`custom_fields: { mg_variant: true }`) may omit `featureModel:`.
            // Such a Configuration denotes the empty feature selection (identity
            // projection); its differentiator is `parameterBindings`. The relaxation
            // is scoped to the marker, so every non-MagicGrid model is unchanged.
            let is_mg_variant = fm.mg_bool("mg_variant") == Some(true);
            if fm.feature_model.is_none() && !is_mg_variant {
                findings.push(error("E201", &file, "`featureModel` is required on Configuration"));
            }
        }

        // E201: FeatureDef requires a stable `id` (REQ-TRS-ID-006). Every feature
        // carries a `FEAT-*` short-name so it can be referenced by a stable id
        // independent of its path/name. The id's pattern is checked by E006; its
        // presence is required here. (The feature is still name-identified — its label
        // and qname segment are `name`.)
        if matches!(fm.element_type, Some(ElementType::FeatureDef)) && fm.id.is_none() {
            findings.push(error("E201", &file, "`id` (a FEAT-* stable id) is required on FeatureDef"));
        }

        // E300: ADR.id must match ADR-* pattern
        if matches!(fm.element_type, Some(ElementType::ADR)) {
            if let Some(ref id) = fm.id {
                if !is_adr_id(id) {
                    findings.push(error("E300", &file, &format!("`id` '{}' does not match ADR-* pattern", id)));
                }
            }
        }

        // E301: ADR required fields
        if matches!(fm.element_type, Some(ElementType::ADR)) {
            if fm.id.is_none() {
                findings.push(error("E301", &file, "`id` is required on ADR"));
            }
            if fm.name.is_none() {
                findings.push(error("E301", &file, "`name` is required on ADR"));
            }
            if fm.status.is_none() {
                findings.push(error("E301", &file, "`status` is required on ADR"));
            }
        }

        // E302: reqDomain enum validation
        if let Some(ref rd) = fm.req_domain {
            const DOMAINS: &[&str] = &["system", "hardware", "software"];
            if !DOMAINS.contains(&rd.as_str()) {
                findings.push(error("E302", &file, &format!("unknown reqDomain value '{}'", rd)));
            }
        }

        // E303: domain enum validation
        if let Some(ref d) = fm.domain {
            const DOMAINS: &[&str] = &["system", "hardware", "software"];
            if !DOMAINS.contains(&d.as_str()) {
                findings.push(error("E303", &file, &format!("unknown domain value '{}'", d)));
            }
        }

        // E304: ADR.status enum validation
        if matches!(fm.element_type, Some(ElementType::ADR)) {
            if let Some(ref status) = fm.status {
                const ADR_STATUSES: &[&str] = &["proposed", "accepted", "deprecated", "superseded"];
                if !ADR_STATUSES.contains(&status.as_str()) {
                    findings.push(error("E304", &file, &format!("unknown ADR status '{}'", status)));
                }
            }
        }

        // ── ReviewRecord (§19, GH #71) ───────────────────────────────────────
        if matches!(fm.element_type, Some(ElementType::ReviewRecord)) {
            // E700: required fields.
            if fm.id.is_none() {
                findings.push(error("E700", &file, "`id` is required on ReviewRecord"));
            }
            if fm.name.is_none() {
                findings.push(error("E700", &file, "`name` is required on ReviewRecord"));
            }
            if fm.status.is_none() {
                findings.push(error("E700", &file, "`status` is required on ReviewRecord"));
            }
            if fm.review_type.is_none() {
                findings.push(error("E700", &file, "`reviewType` is required on ReviewRecord"));
            }
            if fm.reviews.as_ref().map_or(true, |r| r.is_empty()) {
                findings.push(error("E700", &file, "`reviews` (at least one covered element) is required on ReviewRecord"));
            }
            // E701: id pattern.
            if let Some(ref id) = fm.id {
                if !is_rr_id(id) {
                    findings.push(error("E701", &file, &format!("`id` '{}' does not match RR-* pattern", id)));
                }
            }
            // E702: status enum.
            if let Some(ref s) = fm.status {
                const RR_STATUSES: &[&str] = &["open", "closed", "waived"];
                if !RR_STATUSES.contains(&s.as_str()) {
                    findings.push(error("E702", &file, &format!("ReviewRecord.status '{}' must be open, closed, or waived", s)));
                }
            }
            // E703: reviewType enum.
            if let Some(ref rt) = fm.review_type {
                const REVIEW_TYPES: &[&str] = &[
                    "design_review", "requirements_review", "hazard_review",
                    "test_readiness_review", "inspection", "walk_through",
                ];
                if !REVIEW_TYPES.contains(&rt.as_str()) {
                    findings.push(error("E703", &file, &format!("ReviewRecord.reviewType '{}' is not a recognised review type", rt)));
                }
            }
            // E704: each reviews entry must resolve.
            if let Some(ref reviews) = fm.reviews {
                for r in reviews {
                    if resolver.resolve_ref(elements, r).is_none() {
                        findings.push(error("E704", &file, &format!("ReviewRecord.reviews entry '{}' does not resolve to a known element", r)));
                    }
                }
            }
            // E705: items[].disposition enum; track open items for W700.
            let mut has_open_item = false;
            if let Some(ref items) = fm.items {
                for it in items {
                    if let Some(m) = it.as_mapping() {
                        match yaml_field(m, "disposition").and_then(|v| v.as_str()) {
                            Some("open") => has_open_item = true,
                            Some("closed") | Some("not_applicable") => {}
                            other => findings.push(error(
                                "E705",
                                &file,
                                &format!("ReviewRecord items[].disposition '{}' must be open, closed, or not_applicable", other.unwrap_or("<missing>")),
                            )),
                        }
                    }
                }
            }
            // W700: a closed review with an unresolved (open) action item.
            if fm.status.as_deref() == Some("closed") && has_open_item {
                findings.push(warning("W700", &file, "ReviewRecord is `status: closed` but has an action item with `disposition: open`"));
            }
        }

        // ── TradeStudy (§15, GH #63) ─────────────────────────────────────────
        // Codes drafted in the spec as E400–E408 / W400–W403 collide with the Diagram
        // codes; reassigned to E869–E877 / W061–W064 (see release notes / §15.5).
        if matches!(fm.element_type, Some(ElementType::TradeStudy)) {
            let is_draft = fm.status.as_deref() == Some("draft");
            // E869: required fields.
            for (present, label) in [
                (fm.id.is_some(), "id"),
                (fm.name.is_some(), "name"),
                (fm.status.is_some(), "status"),
                (fm.criteria.is_some(), "criteria"),
                (fm.alternatives.is_some(), "alternatives"),
                (fm.scores.is_some(), "scores"),
            ] {
                if !present {
                    findings.push(error("E869", &file, &format!("`{}` is required on TradeStudy", label)));
                }
            }
            // E870: id pattern.
            if let Some(ref id) = fm.id {
                if !is_trd_id(id) {
                    findings.push(error("E870", &file, &format!("`id` '{}' does not match TRD-* pattern", id)));
                }
            }
            // Criteria — E871/E872/E873, collect names.
            let mut crit_names: HashSet<String> = HashSet::new();
            let mut any_positive_weight = false;
            let mut saw_criterion = false;
            if let Some(ref crits) = fm.criteria {
                for c in crits {
                    let Some(m) = c.as_mapping() else { continue };
                    saw_criterion = true;
                    let name = yaml_field(m, "name").and_then(|v| v.as_str());
                    let weight = yaml_field(m, "weight");
                    let dir = yaml_field(m, "direction").and_then(|v| v.as_str());
                    if name.is_none() || weight.is_none() || dir.is_none() {
                        findings.push(error("E871", &file, "TradeStudy criteria entry is missing `name`, `weight`, or `direction`"));
                    }
                    if let Some(n) = name {
                        crit_names.insert(n.to_string());
                    }
                    if let Some(w) = weight.and_then(yaml_num) {
                        if !(0.0..=1.0).contains(&w) {
                            findings.push(error("E872", &file, &format!("TradeStudy criterion weight {} is not in [0.0, 1.0]", w)));
                        }
                        // A positive weight (even if out of range) means weights are not all zero.
                        any_positive_weight |= w > 0.0;
                    }
                    if let Some(d) = dir {
                        if d != "maximize" && d != "minimize" {
                            findings.push(error("E873", &file, &format!("TradeStudy criterion direction '{}' must be maximize or minimize", d)));
                        }
                    }
                }
                if saw_criterion && !any_positive_weight {
                    findings.push(error("E872", &file, "TradeStudy criteria weights are all zero"));
                }
            }
            // Alternatives — E874/E875, W064, collect names.
            let mut alt_names: HashSet<String> = HashSet::new();
            if let Some(ref alts) = fm.alternatives {
                if alts.is_empty() {
                    findings.push(error("E874", &file, "TradeStudy `alternatives` is empty"));
                }
                for a in alts {
                    let Some(m) = a.as_mapping() else { continue };
                    match yaml_field(m, "name").and_then(|v| v.as_str()) {
                        Some(n) => {
                            alt_names.insert(n.to_string());
                        }
                        None => findings.push(error("E875", &file, "TradeStudy alternatives entry is missing `name`")),
                    }
                    if !is_draft {
                        if let Some(el) = yaml_field(m, "element").and_then(|v| v.as_str()) {
                            if resolver.resolve_ref(elements, el).is_none() {
                                findings.push(warning("W064", &file, &format!("TradeStudy alternative `element` '{}' does not resolve", el)));
                            }
                        }
                    }
                }
            }
            // Scores — E876/E877, collect (alt, crit) coverage.
            let mut have: HashSet<(String, String)> = HashSet::new();
            if let Some(ref scores) = fm.scores {
                for s in scores {
                    let Some(m) = s.as_mapping() else { continue };
                    let alt = yaml_field(m, "alternative").and_then(|v| v.as_str());
                    let crit = yaml_field(m, "criterion").and_then(|v| v.as_str());
                    if let Some(a) = alt {
                        if !alt_names.contains(a) {
                            findings.push(error("E876", &file, &format!("TradeStudy score references unknown alternative '{}'", a)));
                        }
                    }
                    if let Some(c) = crit {
                        if !crit_names.contains(c) {
                            findings.push(error("E876", &file, &format!("TradeStudy score references unknown criterion '{}'", c)));
                        }
                    }
                    match yaml_field(m, "score") {
                        Some(v) if yaml_num(v).is_some() => {}
                        _ => findings.push(error("E877", &file, "TradeStudy score `score` is not a number")),
                    }
                    if let (Some(a), Some(c)) = (alt, crit) {
                        have.insert((a.to_string(), c.to_string()));
                    }
                }
            }
            // W063: incomplete score matrix (some alternative×criterion pair has no entry).
            if !is_draft && !alt_names.is_empty() && !crit_names.is_empty() {
                let missing = alt_names
                    .iter()
                    .flat_map(|a| crit_names.iter().map(move |c| (a.clone(), c.clone())))
                    .filter(|p| !have.contains(p))
                    .count();
                if missing > 0 {
                    findings.push(warning("W063", &file, &format!("TradeStudy score matrix is incomplete — {} alternative×criterion pair(s) have no score", missing)));
                }
            }
            // W061: complete study without a decision ADR.
            if fm.status.as_deref() == Some("complete") && fm.decision.is_none() {
                findings.push(warning("W061", &file, "TradeStudy is `status: complete` but has no `decision:` ADR recording the outcome"));
            }
            // W062: objective present but unresolved.
            if !is_draft {
                if let Some(ref obj) = fm.objective {
                    if resolver.resolve_ref(elements, obj).is_none() {
                        findings.push(warning("W062", &file, &format!("TradeStudy `objective` '{}' does not resolve", obj)));
                    }
                }
            }
        }

        // W304: isDeploymentPackage: true combined with domain: hardware
        if fm.is_deployment_package == Some(true) {
            if fm.domain.as_deref() == Some("hardware") {
                findings.push(warning("W304", &file, "`isDeploymentPackage: true` combined with `domain: hardware` — deployment packages must be software"));
            }
        }

        // ── Diagram checks (E4xx / W4xx) ─────────────────────────────────────

        if matches!(fm.element_type, Some(ElementType::Diagram)) {
            // W400: no diagramKind — rendering mode is ambiguous
            // Suppressed for companion SVGs: svgMode: companion already specifies how to display the diagram.
            if fm.diagram_kind.is_none() && fm.svg_mode.as_deref() != Some("companion") {
                findings.push(warning("W400", &file, "Diagram element has no `diagramKind` — rendering mode ambiguous"));
            }
            // E400: Mermaid diagrams require a ```mermaid fenced block in the body
            if fm.diagram_kind.as_deref() == Some("Mermaid") && !elem.doc.contains("```mermaid") {
                findings.push(error("E400", &file, "`diagramKind: Mermaid` but body has no ```mermaid fenced block"));
            }
            // E401: PlantUML diagrams require a ```plantuml fenced block in the body
            if fm.diagram_kind.as_deref() == Some("PlantUML") && !elem.doc.contains("```plantuml") {
                findings.push(error("E401", &file, "`diagramKind: PlantUML` but body has no ```plantuml fenced block"));
            }
            // W408–W410: validate %% annotations inside Mermaid blocks.
            //   W408: `%% ref: QN` — QN doesn't resolve
            //   W409: no `%% ref:` annotations at all
            //   W410: `%% link: NodeId QN` — QN doesn't resolve
            if fm.diagram_kind.as_deref() == Some("Mermaid") {
                let mermaid_block = elem.doc.find("```mermaid").and_then(|start| {
                    let after_fence = start + "```mermaid".len();
                    elem.doc[after_fence..].find("```").map(|end| &elem.doc[after_fence..after_fence + end])
                });
                if let Some(block) = mermaid_block {
                    let mut ref_count = 0usize;
                    for line in block.lines() {
                        let trimmed = line.trim();
                        if let Some(ref_str) = trimmed.strip_prefix("%% ref:") {
                            let ref_str = ref_str.trim();
                            if !ref_str.is_empty() {
                                ref_count += 1;
                                if resolver.resolve_ref(elements, ref_str).is_none() {
                                    findings.push(warning(
                                        "W408",
                                        &file,
                                        &format!("Mermaid `%% ref:` annotation '{}' does not resolve to a known element", ref_str),
                                    ));
                                }
                            }
                        } else if let Some(rest) = trimmed.strip_prefix("%% link:") {
                            // Format: %% link: NodeId QualifiedName
                            let qn = rest.trim().splitn(2, ' ').nth(1).map(|s| s.trim()).unwrap_or("");
                            if !qn.is_empty() && resolver.resolve_ref(elements, qn).is_none() {
                                findings.push(warning(
                                    "W410",
                                    &file,
                                    &format!("Mermaid `%% link:` '{}' does not resolve to a known element", qn),
                                ));
                            }
                        }
                    }
                    if ref_count == 0 {
                        findings.push(warning(
                            "W409",
                            &file,
                            "Mermaid diagram has no `%% ref:` annotations — add at least one to link diagram nodes to model elements",
                        ));
                    }
                }
            }
            // W411: shapes `link:` must resolve to a known element.
            // Accepts `link: QualifiedName` (string) or `link: true` (reuses the shape's ref: value).
            // W412: href="..." attributes found directly in an SVG body must resolve to model elements.
            // Both prevent links rotting silently when elements are renamed or deleted.
            // W401: subject must resolve to a known element
            if let Some(ref subj) = fm.subject {
                if resolver.resolve_ref(elements, subj).is_none() {
                    findings.push(warning(
                        "W401",
                        &file,
                        &format!("`subject` '{}' does not resolve to a known element", subj),
                    ));
                }
            }
            // W402: shapes ref must resolve; refs where any ancestor resolves are suppressed
            // (covers inline features at any depth, e.g. System::part::port::subport)
            let validate_shape_ref = |ref_str: &str, findings: &mut Vec<Finding>| {
                if resolver.resolve_ref(elements, ref_str).is_some() {
                    return;
                }
                let has_resolvable_ancestor = {
                    let mut seg = ref_str;
                    let mut found = false;
                    while let Some(pos) = seg.rfind("::") {
                        seg = &seg[..pos];
                        if resolver.resolve_ref(elements, seg).is_some() {
                            found = true;
                            break;
                        }
                    }
                    found
                };
                if !has_resolvable_ancestor {
                    findings.push(warning(
                        "W402",
                        &file,
                        &format!("shapes `ref` '{}' does not resolve to a known element", ref_str),
                    ));
                }
            };
            let validate_shape_link = |attrs: &serde_yaml::Mapping, findings: &mut Vec<Finding>| {
                let link_qn: Option<&str> = match attrs.get(&serde_yaml::Value::String("link".into())) {
                    Some(serde_yaml::Value::String(s)) if !s.is_empty() => Some(s.as_str()),
                    Some(serde_yaml::Value::Bool(true)) => attrs
                        .get(&serde_yaml::Value::String("ref".into()))
                        .and_then(|v| v.as_str()),
                    _ => None,
                };
                if let Some(qn) = link_qn {
                    if resolver.resolve_ref(elements, qn).is_none() {
                        findings.push(warning(
                            "W411",
                            &file,
                            &format!("shapes `link` '{}' does not resolve to a known element", qn),
                        ));
                    }
                }
            };
            match fm.shapes.as_ref() {
                Some(serde_yaml::Value::Mapping(shapes_map)) => {
                    for shape_val in shapes_map.values() {
                        if let serde_yaml::Value::Mapping(attrs) = shape_val {
                            if let Some(serde_yaml::Value::String(ref_str)) =
                                attrs.get(&serde_yaml::Value::String("ref".into()))
                            {
                                validate_shape_ref(ref_str, &mut findings);
                            }
                            validate_shape_link(attrs, &mut findings);
                        }
                    }
                }
                Some(serde_yaml::Value::Sequence(shapes_seq)) => {
                    for shape_val in shapes_seq {
                        if let serde_yaml::Value::Mapping(attrs) = shape_val {
                            if let Some(serde_yaml::Value::String(ref_str)) =
                                attrs.get(&serde_yaml::Value::String("ref".into()))
                            {
                                validate_shape_ref(ref_str, &mut findings);
                            }
                            validate_shape_link(attrs, &mut findings);
                        }
                    }
                }
                _ => {}
            }
            // W412: href="..." in the SVG fenced block must resolve to a known model element.
            // Only relative paths (not http/https/# anchors) are checked.
            if elem.doc.contains("```svg") {
                let svg_block = elem.doc.find("```svg").and_then(|start| {
                    let after = start + "```svg".len();
                    elem.doc[after..].find("```").map(|end| &elem.doc[after..after + end])
                });
                if let Some(svg) = svg_block {
                    let diagram_dir = std::path::Path::new(&file)
                        .parent()
                        .unwrap_or(std::path::Path::new("."))
                        .to_string_lossy()
                        .into_owned();
                    let href_re = regex::Regex::new(r#"href="([^"]+)""#).unwrap();
                    for cap in href_re.captures_iter(svg) {
                        let href = &cap[1];
                        // Skip external and anchor-only links
                        if href.starts_with("http://")
                            || href.starts_with("https://")
                            || href.starts_with('#')
                            || href.starts_with('/')
                        {
                            continue;
                        }
                        let resolved = normalize_relative_path(&diagram_dir, href);
                        if !elements.iter().any(|e| e.file_path == resolved) {
                            findings.push(warning(
                                "W412",
                                &file,
                                &format!("SVG `href` '{}' (resolved: '{}') does not match any model element file", href, resolved),
                            ));
                        }
                    }
                }
            }
            // W403: edge source/target must reference a shape id defined in this diagram's shapes
            let shape_ids: HashSet<String> = match fm.shapes.as_ref() {
                Some(serde_yaml::Value::Mapping(map)) => {
                    map.keys().filter_map(|k| k.as_str().map(|s| s.to_string())).collect()
                }
                Some(serde_yaml::Value::Sequence(seq)) => seq
                    .iter()
                    .filter_map(|sh| {
                        if let serde_yaml::Value::Mapping(m) = sh {
                            m.get(&serde_yaml::Value::String("id".into()))
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string())
                        } else {
                            None
                        }
                    })
                    .collect(),
                _ => HashSet::new(),
            };
            if !shape_ids.is_empty() {
                let validate_edge = |edge_attrs: &serde_yaml::Mapping, findings: &mut Vec<Finding>| {
                    for field in &["source", "target"] {
                        if let Some(serde_yaml::Value::String(ref_str)) =
                            edge_attrs.get(&serde_yaml::Value::String((*field).into()))
                        {
                            if !shape_ids.contains(ref_str.as_str()) {
                                findings.push(warning(
                                    "W403",
                                    &file,
                                    &format!(
                                        "edge `{}` '{}' is not a defined shape id in this diagram",
                                        field, ref_str
                                    ),
                                ));
                            }
                        }
                    }
                };
                match fm.edges.as_ref() {
                    Some(serde_yaml::Value::Mapping(edges_map)) => {
                        for edge_val in edges_map.values() {
                            if let serde_yaml::Value::Mapping(attrs) = edge_val {
                                validate_edge(attrs, &mut findings);
                            }
                        }
                    }
                    Some(serde_yaml::Value::Sequence(edges_seq)) => {
                        for edge_val in edges_seq {
                            if let serde_yaml::Value::Mapping(attrs) = edge_val {
                                validate_edge(attrs, &mut findings);
                            }
                        }
                    }
                    _ => {}
                }
            }

            // W080 (§22.4): a `Sequence` diagram must include an edge for every
            // SendAction/AcceptAction reachable through its subject ActionDef's
            // sub-action tree. Draft-suppressed; gateable with `--deny W080`.
            if fm.diagram_kind.as_deref() == Some("Sequence")
                && fm.status.as_deref() != Some("draft")
            {
                if let Some(subj_qn) = fm.subject.as_deref() {
                    if let Some(subj_el) = resolver.resolve_ref(elements, subj_qn) {
                        if matches!(
                            subj_el.frontmatter.element_type,
                            Some(ElementType::ActionDef)
                        ) {
                            // Collect every SendAction/AcceptAction name in the
                            // subject's sub-action tree (recursing into IfAction
                            // then/else branches and nested subActions).
                            let mut msg_actions: Vec<String> = Vec::new();
                            if let Some(subs) = subj_el.frontmatter.sub_actions.as_ref() {
                                collect_message_actions(subs, &mut msg_actions);
                            }
                            if !msg_actions.is_empty() {
                                // Set of qnames/short-names referenced by any edge `ref:`.
                                let edge_vals: Vec<&serde_yaml::Value> = match fm.edges.as_ref() {
                                    Some(serde_yaml::Value::Mapping(m)) => m.values().collect(),
                                    Some(serde_yaml::Value::Sequence(s)) => s.iter().collect(),
                                    _ => Vec::new(),
                                };
                                let mut edge_refs: HashSet<&str> = HashSet::new();
                                for v in edge_vals {
                                    if let serde_yaml::Value::Mapping(a) = v {
                                        if let Some(r) = a
                                            .get(&serde_yaml::Value::String("ref".into()))
                                            .and_then(|x| x.as_str())
                                        {
                                            edge_refs.insert(r);
                                        }
                                    }
                                }
                                for action in &msg_actions {
                                    let qn = format!("{}::{}", subj_el.qualified_name, action);
                                    if !edge_refs.contains(qn.as_str())
                                        && !edge_refs.contains(action.as_str())
                                    {
                                        findings.push(warning(
                                            "W080",
                                            &file,
                                            &format!(
                                                "Sequence diagram has no `edges` entry for message action '{}' of subject `{}` — add an edge or remove the action",
                                                qn, subj_qn
                                            ),
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // ── State machine checks (W07x, §22.1) ───────────────────────────────
        // Phase A: W075 — flag the deprecated `from`/`to`/`trigger` transition
        // spelling (accepted as aliases, but the canonical schema is
        // `source`/`target`/`accept`, §8.8.3). Draft-suppressed; `--deny W075`.
        if matches!(
            fm.element_type,
            Some(ElementType::StateDef) | Some(ElementType::State)
        ) && fm.status.as_deref() != Some("draft")
        {
            let subs_opt = fm.sub_states.as_deref().filter(|s| !s.is_empty());

            // Recursive name + edge collection across every nesting level.
            let mut universe: HashSet<String> = HashSet::new();
            let mut all_edges: Vec<StateEdge> = Vec::new();
            collect_machine(subs_opt.unwrap_or(&[]), fm.transitions.as_deref(), &mut universe, &mut all_edges);

            // W075 — deprecated `from`/`to`/`trigger` keys anywhere in the machine.
            if all_edges.iter().any(|e| e.legacy) {
                findings.push(warning(
                    "W075",
                    &file,
                    "state-machine transition uses deprecated keys `from`/`to`/`trigger` — migrate to canonical `source`/`target`/`accept` (§8.8.3)",
                ));
            }

            if let Some(subs) = subs_opt {
                // W070–W074 / W077 / W078 — recursive completeness over the state
                // hierarchy (flat regions, parallel regions, and composite substates).
                check_state_node(
                    None,
                    subs,
                    fm.transitions.as_deref(),
                    fm.is_parallel == Some(true),
                    &file,
                    &mut findings,
                );

                // W076 — a transition endpoint that is neither a state anywhere in this
                // machine nor a model element resolvable by qualified name.
                let mut unresolved: std::collections::BTreeSet<String> = Default::default();
                for e in &all_edges {
                    for ep in [e.source.as_deref(), e.target.as_deref()].into_iter().flatten() {
                        if !universe.contains(ep) && resolver.resolve_ref(elements, ep).is_none() {
                            unresolved.insert(ep.to_string());
                        }
                    }
                }
                for ep in unresolved {
                    findings.push(warning(
                        "W076",
                        &file,
                        &format!("transition endpoint '{}' does not resolve to a state in scope", ep),
                    ));
                }

                // W079 — entry/do/exit and transition `effect` behavior references that
                // resolve to no model element.
                let mut refs = Vec::new();
                collect_state_refs(subs, &mut refs);
                let mut unresolved_refs: std::collections::BTreeSet<String> = Default::default();
                for r in &refs {
                    if resolver.resolve_ref(elements, r).is_none() {
                        unresolved_refs.insert(r.clone());
                    }
                }
                for r in unresolved_refs {
                    findings.push(warning(
                        "W079",
                        &file,
                        &format!("state-machine behavior reference '{}' (entry/do/exit/effect) does not resolve to a known element", r),
                    ));
                }
            }
        }

        // ── Budget expression language (§22.2) ───────────────────────────────
        // A CalculationDef with `bodyLanguage: budget` evaluates a restricted arithmetic
        // `body:` over inline attribute values, optionally bounded by an `evaluate:`
        // ConstraintDef. E866 (bad evaluate target), E867 (syntax), E868 (unresolved
        // operand), W060 (value violates the constraint bound).
        if matches!(fm.element_type, Some(ElementType::CalculationDef))
            && fm.body_language.as_deref() == Some("budget")
        {
            if let Some(ref ev) = fm.evaluate {
                let is_constraint = resolver
                    .resolve_ref(elements, ev)
                    .map(|e| matches!(e.frontmatter.element_type, Some(ElementType::ConstraintDef)))
                    .unwrap_or(false);
                if !is_constraint {
                    findings.push(error(
                        "E866",
                        &file,
                        &format!("`evaluate: {}` does not resolve to a ConstraintDef", ev),
                    ));
                }
            }
            if let Some(ref body) = fm.body {
                let res = |r: &str| resolve_budget_operand(r, fm, elements, &resolver);
                match eval_budget(body, &res) {
                    Err(msg) => findings.push(error(
                        "E867",
                        &file,
                        &format!("budget expression syntax error: {}", msg),
                    )),
                    Ok((value, unresolved)) => {
                        for u in &unresolved {
                            findings.push(error(
                                "E868",
                                &file,
                                &format!("budget operand '{}' resolves to no numeric attribute in scope", u),
                            ));
                        }
                        // W060 — best-effort bound check (draft-suppressed, opt-in via --deny).
                        if unresolved.is_empty() && fm.status.as_deref() != Some("draft") {
                            if let Some(ref ev) = fm.evaluate {
                                if let Some(c) = resolver.resolve_ref(elements, ev) {
                                    if matches!(c.frontmatter.element_type, Some(ElementType::ConstraintDef)) {
                                        if let Some((op, bound)) = c
                                            .frontmatter
                                            .expression
                                            .as_deref()
                                            .and_then(constraint_simple_bound)
                                        {
                                            let ok = match op {
                                                "<=" => value <= bound,
                                                ">=" => value >= bound,
                                                "<" => value < bound,
                                                ">" => value > bound,
                                                "==" => (value - bound).abs() < 1e-9,
                                                _ => true,
                                            };
                                            if !ok {
                                                findings.push(warning(
                                                    "W060",
                                                    &file,
                                                    &format!("budget evaluates to {} which violates constraint `{}` ({} {} {})", value, ev, value, op, bound),
                                                ));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // E402: companion SVG file must exist on disk
        // All paths are resolved relative to the .md file's parent directory.
        let md_dir = std::path::Path::new(&file)
            .parent()
            .unwrap_or(std::path::Path::new("."));
        if fm.svg_mode.as_deref() == Some("companion") {
            let companion_path = if let Some(ref sf) = fm.svg_file {
                md_dir.join(sf)
            } else {
                // Default: same stem as the .md file, .svg extension
                std::path::Path::new(&file).with_extension("svg")
            };
            if !companion_path.exists() {
                findings.push(error(
                    "E402",
                    &file,
                    &format!("companion SVG file '{}' does not exist on disk", companion_path.display()),
                ));
            }
        } else if let Some(ref svg_file) = fm.svg_file {
            // svgFile set without svgMode: companion — still validate existence
            if !md_dir.join(svg_file).exists() {
                findings.push(error(
                    "E402",
                    &file,
                    &format!("`svgFile` '{}' does not exist on disk", svg_file),
                ));
            }
        }

        // W405: body must be consistent with svgMode
        if let Some(ref mode) = fm.svg_mode {
            match mode.as_str() {
                "companion" => {
                    if !elem.doc.contains("<img") {
                        findings.push(warning(
                            "W405",
                            &file,
                            "`svgMode: companion` but body contains no `<img` tag pointing to the SVG file",
                        ));
                    }
                }
                "inline" => {
                    if !elem.doc.contains("```svg") {
                        findings.push(warning(
                            "W405",
                            &file,
                            "`svgMode: inline` but body contains no fenced ```svg block",
                        ));
                    }
                }
                _ => {}
            }
        }

        // W406/W407: SVG id consistency — frontmatter shape/edge ids vs inline SVG
        // Only checked for inline mode (companion SVG is not loaded by the validator)
        if fm.svg_mode.as_deref().unwrap_or("inline") == "inline" {
            // Collect ids declared in shapes: and edges: frontmatter
            let fm_ids: HashSet<String> = {
                let mut ids = HashSet::new();
                let collect_map_keys = |map: &serde_yaml::Mapping, ids: &mut HashSet<String>| {
                    for k in map.keys() {
                        if let Some(s) = k.as_str() {
                            ids.insert(s.to_string());
                        }
                    }
                };
                let collect_seq_ids = |seq: &[serde_yaml::Value], ids: &mut HashSet<String>| {
                    for v in seq {
                        if let serde_yaml::Value::Mapping(m) = v {
                            if let Some(serde_yaml::Value::String(id)) =
                                m.get(&serde_yaml::Value::String("id".into()))
                            {
                                ids.insert(id.clone());
                            }
                        }
                    }
                };
                if let Some(s) = &fm.shapes {
                    match s {
                        serde_yaml::Value::Mapping(m) => collect_map_keys(m, &mut ids),
                        serde_yaml::Value::Sequence(seq) => collect_seq_ids(seq, &mut ids),
                        _ => {}
                    }
                }
                if let Some(e) = &fm.edges {
                    match e {
                        serde_yaml::Value::Mapping(m) => collect_map_keys(m, &mut ids),
                        serde_yaml::Value::Sequence(seq) => collect_seq_ids(seq, &mut ids),
                        _ => {}
                    }
                }
                ids
            };

            if !fm_ids.is_empty() || elem.doc.contains("```svg") {
                // Extract id="..." values from the inline SVG block
                let svg_ids: HashSet<String> = {
                    let mut ids = HashSet::new();
                    let mut remaining = elem.doc.as_str();
                    while let Some(pos) = remaining.find("id=\"") {
                        remaining = &remaining[pos + 4..];
                        if let Some(end) = remaining.find('"') {
                            ids.insert(remaining[..end].to_string());
                            remaining = &remaining[end + 1..];
                        } else {
                            break;
                        }
                    }
                    ids
                };

                // Remove SVG-internal ids (markers, gradients, filters, symbols)
                // that are referenced via url(#id) — they are never model element shapes.
                let svg_ids: HashSet<String> = {
                    let mut url_refs: HashSet<String> = HashSet::new();
                    let mut rem = elem.doc.as_str();
                    while let Some(pos) = rem.find("url(#") {
                        rem = &rem[pos + 5..];
                        if let Some(end) = rem.find(')') {
                            url_refs.insert(rem[..end].to_string());
                            rem = &rem[end + 1..];
                        } else {
                            break;
                        }
                    }
                    svg_ids.into_iter().filter(|id| !url_refs.contains(id.as_str())).collect()
                };

                // W406: frontmatter id with no matching SVG element
                for id in &fm_ids {
                    if !svg_ids.contains(id.as_str()) {
                        findings.push(warning(
                            "W406",
                            &file,
                            &format!("frontmatter shape/edge id '{}' has no matching `id` attribute in the inline SVG", id),
                        ));
                    }
                }
                // W407: SVG element id with no matching frontmatter entry
                for id in &svg_ids {
                    if !fm_ids.contains(id.as_str()) {
                        findings.push(warning(
                            "W407",
                            &file,
                            &format!("SVG element id '{}' has no matching entry in frontmatter `shapes`/`edges`", id),
                        ));
                    }
                }
            }
        }

        // ── Allocation cross-reference checks (E5xx) ─────────────────────────

        // E500/E501: features with type: Allocation must have resolvable allocatedFrom/allocatedTo
        if let Some(ref feats) = fm.features {
            for feat_val in feats {
                if let serde_yaml::Value::Mapping(ref feat) = *feat_val {
                    let feat_type = feat
                        .get(&serde_yaml::Value::String("type".into()))
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    if feat_type == "Allocation" {
                        if let Some(serde_yaml::Value::String(ref from_str)) =
                            feat.get(&serde_yaml::Value::String("allocatedFrom".into()))
                        {
                            if resolver.resolve_ref(elements, from_str).is_none() {
                                findings.push(error(
                                    "E500",
                                    &file,
                                    &format!("Allocation feature `allocatedFrom` '{}' does not resolve", from_str),
                                ));
                            }
                        }
                        if let Some(serde_yaml::Value::String(ref to_str)) =
                            feat.get(&serde_yaml::Value::String("allocatedTo".into()))
                        {
                            if resolver.resolve_ref(elements, to_str).is_none() {
                                findings.push(error(
                                    "E501",
                                    &file,
                                    &format!("Allocation feature `allocatedTo` '{}' does not resolve", to_str),
                                ));
                            }
                        }
                    }
                }
            }
        }

        // E502/E503: allocatedFrom/allocatedTo must each resolve on any element that sets them
        if let Some(ref afs) = fm.allocated_from {
            for af in afs {
                if resolver.resolve_ref(elements, af).is_none() {
                    findings.push(error(
                        "E502",
                        &file,
                        &format!("`allocatedFrom` '{}' does not resolve to a known element", af),
                    ));
                }
            }
        }
        if let Some(ref ats) = fm.allocated_to {
            for at_ref in ats {
                if resolver.resolve_ref(elements, at_ref).is_none() {
                    if config.peer_resolves(at_ref) {
                        // §14.4 — valid cross-repo allocation target.
                    } else if config.has_repos() {
                        findings.push(error(
                            "E512",
                            &file,
                            &format!(
                                "cross-repo allocatedTo reference '{}' resolves neither locally nor in any loaded repo",
                                at_ref
                            ),
                        ));
                    } else {
                        findings.push(error(
                            "E503",
                            &file,
                            &format!("`allocatedTo` '{}' does not resolve to a known element", at_ref),
                        ));
                    }
                }
            }
        }

        // ── Structural cross-reference warnings (W5xx) ───────────────────────

        // W500: viewpoint on View must resolve to a ViewpointDef
        if matches!(fm.element_type, Some(ElementType::View)) {
            if let Some(ref vp) = fm.viewpoint {
                match resolver.resolve_ref(elements, vp) {
                    None => findings.push(warning(
                        "W500",
                        &file,
                        &format!("`viewpoint` '{}' does not resolve to any element", vp),
                    )),
                    Some(target)
                        if !matches!(
                            target.frontmatter.element_type,
                            Some(ElementType::ViewpointDef)
                        ) =>
                    {
                        findings.push(warning(
                            "W500",
                            &file,
                            &format!("`viewpoint` '{}' does not resolve to a ViewpointDef", vp),
                        ));
                    }
                    _ => {}
                }
            }
        }

        // W501: exhibitsStates entries must resolve to known elements
        if let Some(ref states) = fm.exhibits_states {
            for st in states {
                if resolver.resolve_ref(elements, st).is_none() {
                    findings.push(warning(
                        "W501",
                        &file,
                        &format!("`exhibitsStates` entry '{}' does not resolve to any known element", st),
                    ));
                }
            }
        }

        // W502: expose entries on View must resolve to known elements
        if matches!(fm.element_type, Some(ElementType::View)) {
            if let Some(ref expose_vals) = fm.expose {
                for exp_val in expose_vals {
                    let ref_str = match exp_val {
                        serde_yaml::Value::String(s) => Some(s.as_str()),
                        serde_yaml::Value::Mapping(map) => map
                            .get(&serde_yaml::Value::String("ref".into()))
                            .and_then(|v| v.as_str()),
                        _ => None,
                    };
                    if let Some(r) = ref_str {
                        if resolver.resolve_ref(elements, r).is_none() {
                            findings.push(warning(
                                "W502",
                                &file,
                                &format!("`expose` entry '{}' does not resolve to any known element", r),
                            ));
                        }
                    }
                }
            }
        }

        // W404: operation parameter typedBy / returnType doesn't resolve to a known element
        if let Some(ref ops) = fm.operations {
            for op_val in ops {
                if let serde_yaml::Value::Mapping(ref op) = *op_val {
                    if let Some(serde_yaml::Value::Sequence(ref params)) =
                        op.get(&serde_yaml::Value::String("parameters".into()))
                    {
                        for param_val in params {
                            if let serde_yaml::Value::Mapping(ref param) = *param_val {
                                if let Some(serde_yaml::Value::String(ref typed_by)) =
                                    param.get(&serde_yaml::Value::String("typedBy".into()))
                                {
                                    if resolver.resolve_ref(elements, typed_by).is_none()
                                        && matches!(
                                            crate::resolver::builtin_type_kind(typed_by),
                                            crate::resolver::BuiltinType::NotBuiltin
                                        )
                                        && !crate::units::is_recognised_type_ref(typed_by)
                                    {
                                        findings.push(warning(
                                            "W404",
                                            &file,
                                            &format!(
                                                "operation parameter `typedBy` '{}' does not resolve to a known element",
                                                typed_by
                                            ),
                                        ));
                                    }
                                }
                            }
                        }
                    }
                    // also check returnType
                    if let Some(serde_yaml::Value::String(ref ret)) =
                        op.get(&serde_yaml::Value::String("returnType".into()))
                    {
                        if resolver.resolve_ref(elements, ret).is_none()
                            && matches!(
                                crate::resolver::builtin_type_kind(ret),
                                crate::resolver::BuiltinType::NotBuiltin
                            )
                            && !crate::units::is_recognised_type_ref(ret)
                        {
                            findings.push(warning(
                                "W404",
                                &file,
                                &format!(
                                    "operation `returnType` '{}' does not resolve to a known element",
                                    ret
                                ),
                            ));
                        }
                    }
                }
            }
        }

        // ── Documentation completeness (W6xx) ─────────────────────────────────

        // ── Tier 4: FaultTree (E900-E902) ────────────────────────────────────
        if matches!(fm.element_type, Some(ElementType::FaultTree)) {
            if fm.id.is_none() { findings.push(error("E900", &file, "`id` is required on FaultTree")); }
            if fm.name.is_none() { findings.push(error("E900", &file, "`name` is required on FaultTree")); }
            if fm.status.is_none() { findings.push(error("E900", &file, "`status` is required on FaultTree")); }
            if fm.top_event.is_none() { findings.push(error("E900", &file, "`topEvent` is required on FaultTree — reference a SafetyGoal")); }
            if let Some(ref id) = fm.id {
                if !is_ft_id(id) {
                    findings.push(error("E901", &file, &format!("`id` '{}' does not match FT-* pattern", id)));
                }
            }
        }

        // ── Tier 4: FaultTreeGate (E903-E906) ────────────────────────────────
        if matches!(fm.element_type, Some(ElementType::FaultTreeGate)) {
            if fm.id.is_none() { findings.push(error("E903", &file, "`id` is required on FaultTreeGate")); }
            if fm.name.is_none() { findings.push(error("E903", &file, "`name` is required on FaultTreeGate")); }
            if fm.gate_type.is_none() { findings.push(error("E903", &file, "`gateType` is required on FaultTreeGate")); }
            if let Some(ref id) = fm.id {
                if !is_ftg_id(id) {
                    findings.push(error("E904", &file, &format!("`id` '{}' does not match FTG-* pattern", id)));
                }
            }
            // E905: gateType enum
            if let Some(ref gt) = fm.gate_type {
                if !["AND","OR","XOR","NOT","inhibit"].contains(&gt.as_str()) {
                    findings.push(error("E905", &file, &format!("FaultTreeGate.gateType '{}' must be AND, OR, XOR, NOT, or inhibit", gt)));
                }
            }
            // W901: gate with no inputs is a dead end
            if fm.inputs.as_ref().map_or(true, |v| v.is_empty()) {
                findings.push(warning("W901", &file, "FaultTreeGate has no `inputs` — it contributes nothing to the fault tree"));
            }
        }

        // ── Tier 4: FaultTreeEvent (E907-E910) ───────────────────────────────
        if matches!(fm.element_type, Some(ElementType::FaultTreeEvent)) {
            if fm.id.is_none() { findings.push(error("E907", &file, "`id` is required on FaultTreeEvent")); }
            if fm.name.is_none() { findings.push(error("E907", &file, "`name` is required on FaultTreeEvent")); }
            if fm.event_kind.is_none() { findings.push(error("E907", &file, "`eventKind` is required on FaultTreeEvent")); }
            if let Some(ref id) = fm.id {
                if !is_fte_id(id) {
                    findings.push(error("E908", &file, &format!("`id` '{}' does not match FTE-* pattern", id)));
                }
            }
            // E909: eventKind enum
            if let Some(ref ek) = fm.event_kind {
                if !["basic","undeveloped","house"].contains(&ek.as_str()) {
                    findings.push(error("E909", &file, &format!("FaultTreeEvent.eventKind '{}' must be basic, undeveloped, or house", ek)));
                }
            }
        }

        // ── Tier 4: AttackTree (E915-E917) — ISO/SAE 21434 §15.7 ─────────────
        if matches!(fm.element_type, Some(ElementType::AttackTree)) {
            if fm.id.is_none() { findings.push(error("E915", &file, "`id` is required on AttackTree")); }
            if fm.name.is_none() { findings.push(error("E915", &file, "`name` is required on AttackTree")); }
            if fm.status.is_none() { findings.push(error("E915", &file, "`status` is required on AttackTree")); }
            if fm.threat_ref.is_none() { findings.push(error("E915", &file, "`threatRef` is required on AttackTree — reference a ThreatScenario")); }
            if let Some(ref id) = fm.id {
                if !is_at_id(id) {
                    findings.push(error("E916", &file, &format!("`id` '{}' does not match AT-* pattern", id)));
                }
            }
        }

        // ── Tier 4: AttackTreeGate (E918-E920) ───────────────────────────────
        if matches!(fm.element_type, Some(ElementType::AttackTreeGate)) {
            if fm.id.is_none() { findings.push(error("E918", &file, "`id` is required on AttackTreeGate")); }
            if fm.name.is_none() { findings.push(error("E918", &file, "`name` is required on AttackTreeGate")); }
            if fm.gate_type.is_none() { findings.push(error("E918", &file, "`gateType` is required on AttackTreeGate")); }
            if let Some(ref id) = fm.id {
                if !is_atg_id(id) {
                    findings.push(error("E918", &file, &format!("`id` '{}' does not match ATG-* pattern", id)));
                }
            }
            // E919: gateType enum (AND = sequential path, OR = alternatives)
            if let Some(ref gt) = fm.gate_type {
                if !["AND","OR"].contains(&gt.as_str()) {
                    findings.push(error("E919", &file, &format!("AttackTreeGate.gateType '{}' must be AND (sequential path) or OR (alternatives)", gt)));
                }
            }
            // W901-analog: gate with no inputs is a dead end (reuses W036 family? no —
            // mirror FTA's W901 shape but keep W036 for the empty-tree warning). A
            // gate with no inputs contributes nothing.
            if fm.inputs.as_ref().map_or(true, |v| v.is_empty()) {
                findings.push(warning("W037", &file, "AttackTreeGate has no `inputs` — it contributes nothing to the attack tree"));
            }
        }

        // ── Tier 4: AttackStep (E921) ────────────────────────────────────────
        if matches!(fm.element_type, Some(ElementType::AttackStep)) {
            if fm.id.is_none() { findings.push(error("E921", &file, "`id` is required on AttackStep")); }
            if fm.name.is_none() { findings.push(error("E921", &file, "`name` is required on AttackStep")); }
            if let Some(ref id) = fm.id {
                if !is_ats_id(id) {
                    findings.push(error("E921", &file, &format!("`id` '{}' does not match ATS-* pattern", id)));
                }
            }
            // attackFeasibility enum (high|medium|low|very_low)
            if let Some(ref f) = fm.attack_feasibility {
                if !["high","medium","low","very_low"].contains(&f.as_str()) {
                    findings.push(error("E921", &file, &format!("AttackStep.attackFeasibility '{}' must be high, medium, low, or very_low", f)));
                }
            }
        }

        // ── Tier 4: FMEASheet (E911-E912) ────────────────────────────────────
        if matches!(fm.element_type, Some(ElementType::FMEASheet)) {
            if fm.id.is_none() { findings.push(error("E911", &file, "`id` is required on FMEASheet")); }
            if fm.name.is_none() { findings.push(error("E911", &file, "`name` is required on FMEASheet")); }
            if fm.status.is_none() { findings.push(error("E911", &file, "`status` is required on FMEASheet")); }
            if let Some(ref id) = fm.id {
                if !is_fmea_id(id) {
                    findings.push(error("E912", &file, &format!("`id` '{}' does not match FMEA-* pattern", id)));
                }
            }
            // W902: empty sheet
            if fm.entries.as_ref().map_or(true, |v| v.is_empty()) {
                findings.push(warning("W902", &file, "FMEASheet has no `entries` — add at least one failure mode row"));
            }
        }

        // ── Tier 4: FMEAEntry (E913-E914, W903-W904) — synthesised by walker ─
        if matches!(fm.element_type, Some(ElementType::FMEAEntry)) {
            if let Some(ref id) = fm.id {
                if !is_fm_id(id) {
                    findings.push(error("E913", &file, &format!("FMEAEntry `id` '{}' does not match FM-* pattern", id)));
                }
            }
            // E914: severity / occurrence / detection range 1–10
            for (label, val) in [
                ("fmeaSeverity", fm.fmea_severity),
                ("occurrence", fm.occurrence),
                ("detection", fm.detection),
            ] {
                if let Some(v) = val {
                    if !(1..=10).contains(&v) {
                        findings.push(error("E914", &file, &format!("FMEAEntry.{} {} is out of range 1–10", label, v)));
                    }
                }
            }
            // W903: high-RPN entry without a recommended action
            if let Some(rpn) = fm.rpn {
                if rpn > 100 && fm.recommended_action.is_none() {
                    findings.push(warning("W903", &file, &format!("FMEAEntry RPN {} > 100 but has no `recommendedAction`", rpn)));
                }
            }
            // E922: unknown key in FMEA entry — silent drops in a safety analysis are errors
            for key in &fm.unknown_fmea_keys {
                findings.push(error("E922", &file, &format!(
                    "FMEAEntry has unknown key '{}' — this field is silently ignored (recognised: failureMode, effect, cause, fmeaSeverity, occurrence, detection, rpn, recommendedAction, satisfies)", key)));
            }
        }

        // ── Tier 4: TARASheet (E940-E941, W905) ─────────────────────────────────
        if matches!(fm.element_type, Some(ElementType::TARASheet)) {
            if fm.id.is_none() { findings.push(error("E940", &file, "`id` is required on TARASheet")); }
            if fm.name.is_none() { findings.push(error("E940", &file, "`name` is required on TARASheet")); }
            if fm.status.is_none() { findings.push(error("E940", &file, "`status` is required on TARASheet")); }
            if let Some(ref id) = fm.id {
                if !is_tara_id(id) {
                    findings.push(error("E941", &file, &format!("`id` '{}' does not match TARA-* pattern", id)));
                }
            }
            // W905: empty sheet — all four tables absent or empty
            let all_empty = fm.damage_table.as_ref().map_or(true, |v| v.is_empty())
                && fm.threat_table.as_ref().map_or(true, |v| v.is_empty())
                && fm.goal_table.as_ref().map_or(true, |v| v.is_empty())
                && fm.control_table.as_ref().map_or(true, |v| v.is_empty());
            if all_empty {
                findings.push(warning("W905", &file, "TARASheet has no rows in any section table — add damageTable, threatTable, goalTable, or controlTable entries"));
            }
        }

        // W600: PartDef and Part elements should have non-empty documentation
        if matches!(
            fm.element_type,
            Some(ElementType::PartDef) | Some(ElementType::Part)
        ) && elem.doc.trim().is_empty()
        {
            findings.push(warning("W600", &file, "PartDef/Part has an empty documentation body"));
        }

        // W601: ActionDef and Action elements should have non-empty documentation
        if matches!(
            fm.element_type,
            Some(ElementType::ActionDef) | Some(ElementType::Action)
        ) && elem.doc.trim().is_empty()
        {
            findings.push(warning("W601", &file, "ActionDef/Action has an empty documentation body"));
        }
    }

    // ── Model-time checks (cross-element) ────────────────────────────────────

    // E101: duplicate id
    {
        let mut seen_ids: HashMap<&str, &str> = HashMap::new();
        for elem in elements {
            if let Some(ref id) = elem.frontmatter.id {
                if let Some(prev_file) = seen_ids.insert(id.as_str(), elem.file_path.as_str()) {
                    findings.push(error(
                        "E101",
                        &elem.file_path,
                        &format!("duplicate id '{}' (first seen in {})", id, prev_file),
                    ));
                }
            }
        }
    }

    // W616: two TestPlans with an identical (configurations, scope) pair.
    // The config set is the resolved/declared bound configurations (qualified
    // names, order-independent); absent `configurations:` (config-agnostic) is its
    // own distinct key so two config-agnostic plans at the same scope also collide.
    {
        let mut seen: HashMap<(Vec<String>, Option<String>), &str> = HashMap::new();
        for elem in elements {
            if !matches!(elem.frontmatter.element_type, Some(ElementType::TestPlan)) {
                continue;
            }
            let mut cfgs: Vec<String> = crate::testplan::plan_configs(elem, elements, &resolver)
                .iter()
                .map(|c| c.qualified_name.clone())
                .collect();
            cfgs.sort();
            cfgs.dedup();
            let key = (cfgs, elem.frontmatter.scope.clone());
            if let Some(prev) = seen.insert(key, elem.file_path.as_str()) {
                findings.push(warning(
                    "W616",
                    &elem.file_path,
                    &format!(
                        "TestPlan shares an identical (configurations, scope) pair with {} (likely redundant or duplicated plans)",
                        prev
                    ),
                ));
            }
        }
    }

    // Build verified_by and derived_children reverse indices, and check E102–E105
    let mut verified_by: HashMap<String, Vec<String>> = HashMap::new();
    let mut derived_children: HashMap<String, Vec<String>> = HashMap::new();

    for elem in elements {
        let fm = &elem.frontmatter;

        // verifies: cross-reference check
        if let Some(ref vs) = fm.verifies {
            for v in vs {
                match resolver.resolve_ref(elements, v) {
                    None if config.peer_resolves(v) => { /* §14.4 valid cross-repo reference */ }
                    None if config.has_repos() => findings.push(error(
                        "E512",
                        &elem.file_path,
                        &format!(
                            "cross-repo verifies reference '{}' resolves neither locally nor in any loaded repo",
                            v
                        ),
                    )),
                    None => findings.push(error(
                        "E102",
                        &elem.file_path,
                        &format!("unresolved verifies reference '{}'", v),
                    )),
                    Some(target) => {
                        // E104: target must be a native Requirement
                        if !Resolver::is_native_requirement(target) {
                            findings.push(error(
                                "E104",
                                &elem.file_path,
                                &format!("'{}' does not resolve to a native Requirement", v),
                            ));
                        } else if let Some(ref req_id) = target.frontmatter.id {
                            // Build reverse index
                            if let Some(ref tc_id) = elem.frontmatter.id {
                                verified_by
                                    .entry(req_id.clone())
                                    .or_default()
                                    .push(tc_id.clone());
                            }
                        }
                    }
                }
            }
        }

        // derivedFrom: cross-reference check
        if let Some(ref dfs) = fm.derived_from {
            for df in dfs {
                match resolver.resolve_ref(elements, df) {
                    None if config.peer_resolves(df) => { /* §14.4 valid cross-repo reference */ }
                    None if config.has_repos() => findings.push(error(
                        "E512",
                        &elem.file_path,
                        &format!(
                            "cross-repo derivedFrom reference '{}' resolves neither locally nor in any loaded repo",
                            df
                        ),
                    )),
                    None => findings.push(error(
                        "E103",
                        &elem.file_path,
                        &format!("unresolved derivedFrom reference '{}'", df),
                    )),
                    Some(target) => {
                        // E105: target must be a native Requirement
                        if !Resolver::is_native_requirement(target) {
                            findings.push(error(
                                "E105",
                                &elem.file_path,
                                &format!("'{}' does not resolve to a native Requirement", df),
                            ));
                        } else if let Some(ref parent_id) = target.frontmatter.id {
                            if let Some(ref child_id) = elem.frontmatter.id {
                                derived_children
                                    .entry(parent_id.clone())
                                    .or_default()
                                    .push(child_id.clone());
                            }
                        }
                    }
                }
            }
        }

        // E106: testFunctions[].scenario must match a Gherkin scenario title
        if let Some(ref fns) = fm.test_functions {
            let scenarios = extract_gherkin_scenarios(&elem.doc);
            for tf in fns {
                if let Some(serde_yaml::Value::Mapping(map)) = Some(tf) {
                    if let Some(serde_yaml::Value::String(scenario)) =
                        map.get(&serde_yaml::Value::String("scenario".into()))
                    {
                        if !scenarios.contains(scenario.as_str()) {
                            findings.push(error(
                                "E106",
                                &elem.file_path,
                                &format!(
                                    "testFunctions scenario '{}' not found in Gherkin blocks — add to a ```gherkin block: `Scenario: {}` (or run `syscribe scaffold-gherkin {} --fix`)",
                                    scenario,
                                    scenario,
                                    fm.id.as_deref().unwrap_or("<TC>")
                                ),
                            ));
                        }
                    }
                }
            }
        }
    }

    // W002/W003: coverage checks for native Requirements
    for elem in elements {
        if !Resolver::is_native_requirement(elem) {
            continue;
        }
        let req_id = elem.frontmatter.id.as_deref().unwrap_or("");
        let status = elem.frontmatter.status.as_deref().unwrap_or("");
        let active_tcs: Vec<_> = verified_by
            .get(req_id)
            .map(|tcs| {
                tcs.iter()
                    .filter(|tc_id| {
                        resolver
                            .get_by_id(elements, tc_id)
                            .and_then(|e| e.frontmatter.status.as_deref())
                            == Some("active")
                    })
                    .cloned()
                    .collect()
            })
            .unwrap_or_default();

        let is_parent = derived_children.get(req_id).map_or(false, |v| !v.is_empty());
        match status {
            // W002: leaf requirements at approved/implemented need an active TestCase.
            // Parent requirements (those with derivedChildren) are verified by
            // decomposition — all their leaf descendants carry the test coverage —
            // so W002 is suppressed for them.
            "approved" | "implemented" if active_tcs.is_empty() && !is_parent => {
                findings.push(warning(
                    "W002",
                    &elem.file_path,
                    &format!("Requirement '{}' (status: {}) has no active TestCase", req_id, status),
                ));
            }
            "verified" if active_tcs.is_empty() => {
                findings.push(warning(
                    "W003",
                    &elem.file_path,
                    &format!("Requirement '{}' has status: verified but no active TestCase covers it", req_id),
                ));
            }
            _ => {}
        }

        // W702: asilLevel: D requirement must have at least one active L5 (HIL) TestCase
        if elem.frontmatter.asil_level.as_deref() == Some("D") && !active_tcs.is_empty() {
            let has_l5 = active_tcs.iter().any(|tc_id| {
                resolver
                    .get_by_id(elements, tc_id)
                    .and_then(|e| e.frontmatter.test_level.as_deref())
                    == Some("L5")
            });
            if !has_l5 {
                findings.push(warning(
                    "W702",
                    &elem.file_path,
                    &format!("Requirement '{}' has asilLevel: D but no active TestCase at testLevel: L5 (HIL) — ISO 26262-6 §9 requires hardware-in-the-loop testing for ASIL D", req_id),
                ));
            }
        }

        // E865 / W860 (§22.3, R-007b): ASIL D / SIL 4 decomposition pair completeness.
        // When an ASIL D / SIL 4 requirement's integrity-bearing children are *all*
        // strictly lower (a decomposition claim per ISO 26262-9 §5 / IEC 61508-2 §7.4.9),
        // the channels must be ≥2 and satisfy architecturally distinct elements.
        {
            let pfm = &elem.frontmatter;
            let parent_is_d4 =
                pfm.asil_level.as_deref() == Some("D") || pfm.sil_level == Some(4);
            if parent_is_d4 {
                let leveled: Vec<&RawElement> = derived_children
                    .get(req_id)
                    .map(|ids| {
                        ids.iter()
                            .filter_map(|cid| resolver.get_by_id(elements, cid))
                            .filter(|c| {
                                c.frontmatter.asil_level.is_some()
                                    || c.frontmatter.sil_level.is_some()
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                let all_lower = !leveled.is_empty()
                    && leveled.iter().all(|c| {
                        integrity_is_lower(
                            c.frontmatter.asil_level.as_deref(),
                            c.frontmatter.sil_level,
                            pfm.asil_level.as_deref(),
                            pfm.sil_level,
                        )
                    });
                if all_lower {
                    if leveled.len() < 2 {
                        // W860 — single-channel decomposition (draft-suppressed).
                        if pfm.status.as_deref() != Some("draft") {
                            findings.push(warning(
                                "W860",
                                &elem.file_path,
                                &format!("ASIL D / SIL 4 requirement '{}' has a single lower-level decomposition child — a decomposition needs at least two independent channels", req_id),
                            ));
                        }
                    } else {
                        // E865 — two siblings share a `satisfies:` target.
                        let mut by_target: std::collections::BTreeMap<&str, Vec<&str>> =
                            std::collections::BTreeMap::new();
                        for c in &leveled {
                            let cid = c.frontmatter.id.as_deref().unwrap_or("");
                            if let Some(sat) = &c.frontmatter.satisfies {
                                for s in sat {
                                    by_target.entry(s.as_str()).or_default().push(cid);
                                }
                            }
                        }
                        for (target, kids) in &by_target {
                            if kids.len() >= 2 {
                                findings.push(error(
                                    "E865",
                                    &elem.file_path,
                                    &format!("ASIL/SIL decomposition of '{}' is not architecturally independent — siblings {} all satisfy the same element '{}'; decomposed channels must satisfy distinct elements", req_id, kids.join(", "), target),
                                ));
                            }
                        }
                    }
                }
            }
        }

        // W029 (REQ-TRS-VAL-016, GH #22): a non-draft requirement carrying an
        // integrity level and a `wcet:` claim must be backed by a *measuring*
        // test — an active TestCase verifying it at testLevel L5 (HIL) or tagged
        // `timing`/`wcet`. Opt-in (needs both wcet and SIL/ASIL), draft-suppressed.
        let has_integrity =
            elem.frontmatter.sil_level.is_some() || elem.frontmatter.asil_level.is_some();
        let wcet_claim = elem.frontmatter.wcet.as_deref().filter(|w| !w.trim().is_empty());
        if let Some(wcet) = wcet_claim {
            if has_integrity && status != "draft" {
                let measured = active_tcs.iter().any(|tc_id| {
                    resolver.get_by_id(elements, tc_id).is_some_and(|tc| {
                        tc.frontmatter.test_level.as_deref() == Some("L5")
                            || tc.frontmatter.tags.as_ref().is_some_and(|ts| {
                                ts.iter().any(|t| t == "timing" || t == "wcet")
                            })
                    })
                });
                if !measured {
                    findings.push(warning(
                        "W029",
                        &elem.file_path,
                        &format!("Requirement '{}' declares wcet: '{}' but no active measuring TestCase (testLevel L5 or timing/wcet-tagged) verifies it", req_id, wcet),
                    ));
                }
            }
        }

        // W305: parent requirement must have at least one active integration-level TestCase
        // (L3 system test, L4 system integration test, or L5 HIL/acceptance).
        // Leaf-level test cases (L1/L2) on derived requirements are not sufficient to
        // verify the emergent, composed behaviour expressed by the parent.
        if is_parent && matches!(status, "approved" | "implemented" | "verified") {
            let has_integration_tc = active_tcs.iter().any(|tc_id| {
                resolver
                    .get_by_id(elements, tc_id)
                    .and_then(|e| e.frontmatter.test_level.as_deref())
                    .map_or(false, |lvl| matches!(lvl, "L3" | "L4" | "L5"))
            });
            if !has_integration_tc {
                findings.push(warning(
                    "W305",
                    &elem.file_path,
                    &format!(
                        "parent Requirement '{}' (status: {}) has no active system integration TestCase (testLevel: L3, L4, or L5)",
                        req_id, status
                    ),
                ));
            }
        }

        // W005: orphan (no derivedFrom and no derivedChildren)
        let has_parent = elem.frontmatter.derived_from.as_ref().map_or(false, |v| !v.is_empty());
        let has_children = derived_children.get(req_id).map_or(false, |v| !v.is_empty());
        if !has_parent && !has_children {
            findings.push(warning(
                "W005",
                &elem.file_path,
                &format!(
                    "Requirement '{}' has no derivedFrom and no derivedChildren — possible orphan",
                    req_id
                ),
            ));
        }
    }

    // W704 (§19, GH #71): review coverage gap. Scoped to non-draft native Requirements and
    // dormant unless the model uses ReviewRecords — a Requirement covered by no
    // `ReviewRecord.reviews:` list. Opt-in; gateable with `--deny W704`.
    {
        let review_records: Vec<&RawElement> = elements
            .iter()
            .filter(|e| matches!(e.frontmatter.element_type, Some(ElementType::ReviewRecord)))
            .collect();
        if !review_records.is_empty() {
            let mut covered: HashSet<String> = HashSet::new();
            for rr in &review_records {
                if let Some(reviews) = &rr.frontmatter.reviews {
                    for r in reviews {
                        covered.insert(r.clone());
                        if let Some(t) = resolver.resolve_ref(elements, r) {
                            if let Some(id) = &t.frontmatter.id {
                                covered.insert(id.clone());
                            }
                            covered.insert(t.qualified_name.clone());
                        }
                    }
                }
            }
            for el in elements {
                if !Resolver::is_native_requirement(el)
                    || el.frontmatter.status.as_deref() == Some("draft")
                {
                    continue;
                }
                let id = el.frontmatter.id.as_deref().unwrap_or("");
                if !covered.contains(id) && !covered.contains(&el.qualified_name) {
                    findings.push(warning(
                        "W704",
                        &el.file_path,
                        &format!("Requirement '{}' appears in no ReviewRecord.reviews list — no review evidence", id),
                    ));
                }
            }
        }
    }

    // ── IEC 62443 Zone / Conduit (§13, GH #61) ──────────────────────────────
    {
        let is_zone = |e: &RawElement| matches!(e.frontmatter.element_type, Some(ElementType::Zone));
        let is_part = |e: &RawElement| {
            matches!(e.frontmatter.element_type, Some(ElementType::PartDef) | Some(ElementType::Part))
        };
        // Zones referenced by any conduit (fromZone/toZone) → for W953.
        let mut conduit_zone_refs: HashSet<String> = HashSet::new();
        for c in elements.iter().filter(|e| matches!(e.frontmatter.element_type, Some(ElementType::Conduit))) {
            for z in [&c.frontmatter.from_zone, &c.frontmatter.to_zone].into_iter().flatten() {
                if let Some(t) = resolver.resolve_ref(elements, z) {
                    conduit_zone_refs.insert(t.qualified_name.clone());
                }
            }
        }
        // Parts referenced by any Zone.members → for W952.
        let mut member_parts: HashSet<String> = HashSet::new();
        for z in elements.iter().filter(|e| is_zone(e)) {
            for m in z.frontmatter.members.as_deref().unwrap_or(&[]) {
                if let Some(t) = resolver.resolve_ref(elements, m) {
                    member_parts.insert(t.qualified_name.clone());
                }
            }
        }

        for z in elements.iter().filter(|e| is_zone(e)) {
            let fm = &z.frontmatter;
            let f = &z.file_path;
            // E950 required fields.
            for (present, label) in [(fm.id.is_some(), "id"), (fm.name.is_some(), "name"), (fm.status.is_some(), "status"), (fm.target_sl.is_some(), "targetSL")] {
                if !present {
                    findings.push(error("E950", f, &format!("`{}` is required on Zone", label)));
                }
            }
            // E951 id pattern.
            if let Some(id) = &fm.id {
                if !is_zn_id(id) {
                    findings.push(error("E951", f, &format!("`id` '{}' does not match ZN-* pattern", id)));
                }
            }
            // E955 members must resolve to PartDef/Part.
            for m in fm.members.as_deref().unwrap_or(&[]) {
                match resolver.resolve_ref(elements, m) {
                    Some(t) if is_part(t) => {}
                    _ => findings.push(error("E955", f, &format!("Zone member '{}' does not resolve to a PartDef/Part", m))),
                }
            }
            // W950 achievedSL < targetSL.
            if let (Some(a), Some(t)) = (fm.achieved_sl, fm.target_sl) {
                if a < t {
                    findings.push(warning("W950", f, &format!("Zone achievedSL {} is below targetSL {} — security level not yet achieved", a, t)));
                }
            }
            // W953 approved zone (targetSL>=2) with no referencing conduit.
            if fm.status.as_deref() == Some("approved") && fm.target_sl.unwrap_or(0) >= 2 && !conduit_zone_refs.contains(&z.qualified_name) {
                findings.push(warning("W953", f, &format!("approved Zone '{}' (targetSL {}) is referenced by no Conduit", fm.id.as_deref().unwrap_or(&z.qualified_name), fm.target_sl.unwrap_or(0))));
            }
        }

        for c in elements.iter().filter(|e| matches!(e.frontmatter.element_type, Some(ElementType::Conduit))) {
            let fm = &c.frontmatter;
            let f = &c.file_path;
            for (present, label) in [(fm.id.is_some(), "id"), (fm.name.is_some(), "name"), (fm.status.is_some(), "status"), (fm.from_zone.is_some(), "fromZone"), (fm.to_zone.is_some(), "toZone")] {
                if !present {
                    findings.push(error("E952", f, &format!("`{}` is required on Conduit", label)));
                }
            }
            if let Some(id) = &fm.id {
                if !is_cd_id(id) {
                    findings.push(error("E953", f, &format!("`id` '{}' does not match CD-* pattern", id)));
                }
            }
            // E954 from/to must resolve to a Zone; collect connected targetSLs for W951.
            let mut zone_target_sls: Vec<u8> = Vec::new();
            for (z, which) in [(&fm.from_zone, "fromZone"), (&fm.to_zone, "toZone")] {
                if let Some(zref) = z {
                    match resolver.resolve_ref(elements, zref) {
                        Some(t) if is_zone(t) => {
                            if let Some(sl) = t.frontmatter.target_sl {
                                zone_target_sls.push(sl);
                            }
                        }
                        _ => findings.push(error("E954", f, &format!("Conduit {} '{}' does not resolve to a Zone", which, zref))),
                    }
                }
            }
            // W951 conduit achievedSL below either connected zone's targetSL (opt-in).
            if let Some(a) = fm.achieved_sl {
                if let Some(&max_req) = zone_target_sls.iter().max() {
                    if a < max_req {
                        findings.push(warning("W951", f, &format!("Conduit achievedSL {} is below a connected zone targetSL {} — boundary weaker than the zones it connects", a, max_req)));
                    }
                }
            }
        }

        for p in elements.iter().filter(|e| is_part(e)) {
            let fm = &p.frontmatter;
            let f = &p.file_path;
            // E956 inZone must resolve to a Zone.
            if let Some(z) = &fm.in_zone {
                match resolver.resolve_ref(elements, z) {
                    Some(t) if is_zone(t) => {}
                    _ => findings.push(error("E956", f, &format!("`inZone` '{}' does not resolve to a Zone", z))),
                }
            }
            // W952 targetSL claim with no zone membership (opt-in).
            if fm.target_sl.is_some() && fm.in_zone.is_none() && !member_parts.contains(&p.qualified_name) {
                findings.push(warning("W952", f, &format!("'{}' declares targetSL but belongs to no Zone (no inZone, no Zone.members entry)", p.qualified_name)));
            }
        }
    }

    // ── §14 Multi-repository composition (E510–E515, W510, GH #62) ───────────
    // Active only when `[repos]` is configured (`config.repos` non-empty); the
    // bare `validate` entry point and every single-repo model are unaffected.
    if config.has_repos() {
        let cfg_file = ".syscribe.toml";
        let repo_by_alias: HashMap<&str, &crate::config::LoadedRepo> =
            config.repos.iter().map(|r| (r.alias.as_str(), r)).collect();

        // Per configured repo: E510 (circular), E511 (missing path, no ref), W510 (no ref).
        for repo in &config.repos {
            if repo.circular {
                findings.push(error(
                    "E510",
                    cfg_file,
                    &format!(
                        "circular repo import: repo '{}' transitively imports back into this model",
                        repo.alias
                    ),
                ));
            }
            if !repo.exists && repo.config.git_ref.is_none() {
                findings.push(error(
                    "E511",
                    cfg_file,
                    &format!(
                        "repos.{}.path '{}' does not exist on disk and no `ref` is configured",
                        repo.alias, repo.config.path
                    ),
                ));
            }
            if repo.config.git_ref.is_none() {
                findings.push(warning(
                    "W510",
                    cfg_file,
                    &format!(
                        "repo '{}' has no `ref:` — composition is not pinned to a reproducible snapshot",
                        repo.alias
                    ),
                ));
            }
        }

        // E515: a stable ID exported by both the local model and a peer repo.
        let local_ids: HashSet<&str> = elements
            .iter()
            .filter_map(|e| e.frontmatter.id.as_deref())
            .filter(|id| is_stable_id(id))
            .collect();
        for repo in &config.repos {
            let mut dup: Vec<&str> = repo
                .stable_ids
                .iter()
                .map(String::as_str)
                .filter(|id| local_ids.contains(id))
                .collect();
            dup.sort_unstable();
            for id in dup {
                findings.push(error(
                    "E515",
                    cfg_file,
                    &format!(
                        "stable ID '{}' is exported by both the local model and repo '{}' — the id namespace is global across the composition",
                        id, repo.alias
                    ),
                ));
            }
        }

        // E513 / E514: `repoImports:` declarations on Package elements.
        for elem in elements {
            let Some(imports) = &elem.frontmatter.repo_imports else {
                continue;
            };
            for imp in imports {
                let get = |k: &str| {
                    imp.get(serde_yaml::Value::String(k.to_string()))
                        .and_then(|v| v.as_str())
                        .map(str::to_string)
                };
                let Some(alias) = get("repo") else {
                    findings.push(error(
                        "E513",
                        &elem.file_path,
                        "`repoImports` entry is missing the required `repo` alias",
                    ));
                    continue;
                };
                match repo_by_alias.get(alias.as_str()) {
                    None => findings.push(error(
                        "E513",
                        &elem.file_path,
                        &format!("`repoImports` names alias '{}', absent from the `[repos]` config", alias),
                    )),
                    Some(repo) if repo.exists => {
                        if let Some(qname) = get("qname") {
                            let suffix = format!("::{qname}");
                            let found = repo.qnames.contains(&qname)
                                || repo.qnames.iter().any(|q| q.ends_with(&suffix));
                            if !found {
                                findings.push(error(
                                    "E514",
                                    &elem.file_path,
                                    &format!(
                                        "`repoImports` qname '{}' does not resolve to any element in repo '{}'",
                                        qname, alias
                                    ),
                                ));
                            }
                        } else {
                            findings.push(error(
                                "E514",
                                &elem.file_path,
                                &format!("`repoImports` from '{}' is missing the required `qname`", alias),
                            ));
                        }
                    }
                    // Repo configured but absent on disk: E511 already reported it.
                    Some(_) => {}
                }
            }
        }
    }

    // W007: *Def element never used as supertype: or typedBy: anywhere in the model.
    // Scans top-level fields AND typedBy inside features/connections/performs sub-objects
    // and exhibitsStates lists, so that elements referenced only in those positions are
    // not incorrectly flagged.
    {
        let mut referenced_defs: HashSet<String> = HashSet::new();
        for elem in elements.iter() {
            let fm = &elem.frontmatter;

            // Top-level supertype and typedBy
            for field in [fm.supertype.as_ref(), fm.typed_by.as_ref()].into_iter().flatten() {
                for s in yaml_strings(field) {
                    if let Some(target) = resolver.resolve_ref(elements, s) {
                        referenced_defs.insert(target.qualified_name.clone());
                    }
                }
            }

            // exhibitsStates: Vec<String> — direct qualified name references
            for s in fm.exhibits_states.iter().flatten() {
                if let Some(target) = resolver.resolve_ref(elements, s) {
                    referenced_defs.insert(target.qualified_name.clone());
                }
            }

            // features, connections, performs, flow_connections, etc. —
            // scan typedBy inside each mapping entry (and nested ports sub-key)
            for list in [
                fm.features.as_deref(),
                fm.connections.as_deref(),
                fm.flow_connections.as_deref(),
                fm.binding_connections.as_deref(),
                fm.succession_connections.as_deref(),
                fm.performs.as_deref(),
            ]
            .into_iter()
            .flatten()
            {
                collect_typed_by_refs(list, elements, &resolver, &mut referenced_defs);
            }
        }
        for elem in elements {
            if is_type_def(elem) {
                if !referenced_defs.contains(&elem.qualified_name) {
                    findings.push(warning(
                        "W007",
                        &elem.file_path,
                        &format!(
                            "'{}' is defined but never used as a supertype or type",
                            elem.qualified_name
                        ),
                    ));
                }
            }
        }
    }

    // E001 / E002 / E005 / W008: parse-time issues and missing/unknown type fields
    for elem in elements {
        match &elem.parse_issue {
            Some(ParseIssue::NoFrontmatter) => {
                findings.push(error(
                    "E001",
                    &elem.file_path,
                    "file does not begin with '---' (missing frontmatter delimiter)",
                ));
            }
            Some(ParseIssue::YamlError(msg)) => {
                findings.push(error(
                    "E002",
                    &elem.file_path,
                    &format!("frontmatter is not valid YAML 1.2: {}", msg),
                ));
            }
            None => {
                // E005: type: value present but not in the element type inventory
                if matches!(elem.frontmatter.element_type, Some(ElementType::Unknown)) {
                    findings.push(error(
                        "E005",
                        &elem.file_path,
                        &format!(
                            "'{}' has an unrecognised `type:` value — not in the element type inventory",
                            elem.qualified_name
                        ),
                    ));
                } else if elem.frontmatter.element_type.is_none() {
                    // W008: no type: field at all
                    findings.push(warning(
                        "W008",
                        &elem.file_path,
                        &format!(
                            "'{}' has no type: field — element will be ignored by most commands",
                            elem.qualified_name
                        ),
                    ));
                }
            }
        }
    }

    // E209: appliesWhen must parse as a boolean expression over FeatureDefs and
    // every operand must resolve to a FeatureDef. A bare QName or a list (legacy
    // AND) are the trivial cases; `and`/`or`/`not`/parentheses are also accepted.
    for elem in elements {
        if let Some(ref aw) = elem.frontmatter.applies_when {
            match crate::variability::applies_when_expr(aw) {
                Err(msg) => findings.push(error(
                    "E209",
                    &elem.file_path,
                    &format!("invalid appliesWhen expression: {}", msg),
                )),
                Ok(None) => {}
                Ok(Some(expr)) => {
                    for r in expr.operands() {
                        match resolver.resolve_ref(elements, &r) {
                            None => findings.push(error(
                                "E209",
                                &elem.file_path,
                                &format!("unresolved appliesWhen reference '{}'", r),
                            )),
                            Some(target) if !Resolver::is_feature_def(target) => {
                                findings.push(error(
                                    "E209",
                                    &elem.file_path,
                                    &format!("'{}' does not resolve to a FeatureDef", r),
                                ));
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    // W016: a Configuration that parsed no feature selections while a feature
    // model exists. Catches the legacy `selections:` footgun (issue #12) — an
    // ignored selection block silently yields an all-N/A matrix — by surfacing
    // it as a local warning instead of a confusing downstream symptom.
    {
        let has_feature_def = elements
            .iter()
            .any(|e| matches!(e.frontmatter.element_type, Some(ElementType::FeatureDef)));
        if has_feature_def {
            for cfg in elements
                .iter()
                .filter(|e| matches!(e.frontmatter.element_type, Some(ElementType::Configuration)))
            {
                if cfg.frontmatter.feature_selections().is_empty() {
                    let msg = if cfg.frontmatter.extra.contains_key("selections") {
                        "Configuration declares a `selections:` key, which is ignored — feature selections must be a `features:` map of `<FeatureDef>: true/false` (§9.8); this configuration currently selects no features"
                    } else {
                        "Configuration parsed no feature selections — add a `features:` map of `<FeatureDef>: true/false` (§9.8), otherwise its appliesWhen-conditioned elements all evaluate N/A"
                    };
                    findings.push(warning("W016", &cfg.file_path, msg));
                }
            }
        }
    }

    // E203–E206 / E222 / W017: FeatureDef parameter binding validation (§9.7).
    // Shared with `feature-check` so a product line validated holistically gets
    // the same binding/range enforcement (GH #14).
    findings.extend(parameter_binding_findings(elements));

    // W028: duplicate external references (§3, REQ-TRS-EXTREF-001).
    findings.extend(ext_ref_duplicate_findings(elements));


    // ── E228 / W026: transitive package appliesWhen (REQ-TRS-VAR-006) ────────
    // A package may carry appliesWhen to gate its whole subtree, with at most one
    // declaration per root-to-leaf path. Dormant unless a FeatureDef exists.
    if elements
        .iter()
        .any(|e| matches!(e.frontmatter.element_type, Some(ElementType::FeatureDef)))
    {
        let pkg = crate::variability::package_conditions(elements);
        let is_pkg = |t: &Option<ElementType>| {
            matches!(
                t,
                Some(ElementType::Package) | Some(ElementType::LibraryPackage) | Some(ElementType::Namespace)
            )
        };
        let kind_label = |t: &Option<ElementType>| match t {
            Some(ElementType::FeatureDef) => "FeatureDef",
            Some(ElementType::Configuration) => "Configuration",
            _ => "element",
        };
        for e in elements {
            let own = e.frontmatter.applies_when.is_some();
            let et = &e.frontmatter.element_type;
            let qn = if e.qualified_name.is_empty() { "<root>" } else { &e.qualified_name };
            // (a) forbidden target — own appliesWhen on a FeatureDef / Configuration
            if own && matches!(et, Some(ElementType::FeatureDef) | Some(ElementType::Configuration)) {
                findings.push(error("E228", &e.file_path, &format!(
                    "appliesWhen is not permitted on a {} ('{}')", kind_label(et), qn)));
            }
            // (b) forbidden target — own appliesWhen on the model-root package
            if own && is_pkg(et) && e.qualified_name.is_empty() {
                findings.push(error("E228", &e.file_path,
                    "appliesWhen is not permitted on the model-root package (it would project the whole model to empty)"));
            }
            // (c) nested — own appliesWhen with an ancestor package that also declares one
            if own {
                if let Some(anc) = crate::variability::ancestor_package_with_aw(e, &pkg) {
                    findings.push(error("E228", &e.file_path, &format!(
                        "appliesWhen on '{}' is nested under package '{}', which already declares appliesWhen — at most one declaration per path", qn, anc)));
                }
            }
            // (d) a gated package's subtree may not contain a FeatureDef / Configuration
            if matches!(et, Some(ElementType::FeatureDef) | Some(ElementType::Configuration)) {
                if let Some(anc) = crate::variability::ancestor_package_with_aw(e, &pkg) {
                    findings.push(error("E228", &e.file_path, &format!(
                        "package '{}' declares appliesWhen but its subtree contains a {} ('{}') — the feature model / configurations may not be gated", anc, kind_label(et), qn)));
                }
            }
        }
        // W026: a package declares appliesWhen but gates no element.
        for pq in pkg.keys() {
            let prefix = format!("{}::", pq);
            let gates = elements.iter().any(|e| e.qualified_name.starts_with(&prefix));
            if !gates {
                let file = elements
                    .iter()
                    .find(|e| &e.qualified_name == pq && is_pkg(&e.frontmatter.element_type))
                    .map(|e| e.file_path.clone())
                    .unwrap_or_default();
                findings.push(warning("W026", &file, &format!(
                    "package '{}' declares appliesWhen but gates no element (empty subtree)", pq)));
            }
        }
    }

    // W015: per-Configuration coverage (variant-aware uncovered requirement).
    // Only active when the variability dimension is on (REQ-TRS-VAR-001). For
    // each Configuration C and each non-draft requirement R that is *active* in
    // C, require a non-draft TestCase that runs in C and verifies R; otherwise
    // emit W015 on C's file. Dormant models keep the flat uncovered check.
    if crate::variability::is_active(elements) {
        use crate::variability::FeatureExpr;
        // Feature id→qname alias: appliesWhen operands and Configuration selections
        // keyed by a FeatureDef's FEAT-* id are normalized to the qname so they
        // share one key space (REQ-TRS-ID-006).
        let feat_alias = crate::variability::feature_id_to_qname(elements);
        let parse_aw = |elem: &RawElement| -> Option<FeatureExpr> {
            elem.frontmatter
                .applies_when
                .as_ref()
                .and_then(|aw| crate::variability::applies_when_expr(aw).ok().flatten())
                .map(|e| {
                    e.canonicalize(&|q: &str| crate::variability::canon_feature_ref(q, &feat_alias))
                })
        };
        let is_draft = |elem: &RawElement| elem.frontmatter.status.as_deref() == Some("draft");

        // Non-draft requirements: (display id, applies_when, identity keys).
        let reqs: Vec<(String, Option<FeatureExpr>, Vec<String>)> = elements
            .iter()
            .filter(|e| {
                matches!(e.frontmatter.element_type, Some(ElementType::Requirement)) && !is_draft(e)
            })
            .map(|e| {
                let id = e
                    .frontmatter
                    .id
                    .clone()
                    .unwrap_or_else(|| e.qualified_name.clone());
                let mut keys = vec![e.qualified_name.clone()];
                if let Some(i) = &e.frontmatter.id {
                    keys.push(i.clone());
                }
                (id, parse_aw(e), keys)
            })
            .collect();

        // Non-draft TestCases: (applies_when, verifies entries).
        let tcs: Vec<(Option<FeatureExpr>, Vec<String>)> = elements
            .iter()
            .filter(|e| {
                matches!(e.frontmatter.element_type, Some(ElementType::TestCase)) && !is_draft(e)
            })
            .map(|e| (parse_aw(e), e.frontmatter.verifies.clone().unwrap_or_default()))
            .collect();

        for cfg in elements
            .iter()
            .filter(|e| matches!(e.frontmatter.element_type, Some(ElementType::Configuration)))
        {
            let sel = crate::variability::canon_selection(
                &cfg.frontmatter.feature_selections(),
                &feat_alias,
            );
            let selected = |q: &str| sel.get(q).copied().unwrap_or(false);
            let cfg_id = cfg
                .frontmatter
                .id
                .clone()
                .unwrap_or_else(|| cfg.qualified_name.clone());
            for (rid, rexpr, rkeys) in &reqs {
                let active = rexpr.as_ref().map_or(true, |e| e.eval(&selected));
                if !active {
                    continue;
                }
                let covered = tcs.iter().any(|(texpr, verifies)| {
                    let runs = texpr.as_ref().map_or(true, |e| e.eval(&selected));
                    runs && verifies.iter().any(|v| rkeys.iter().any(|k| k == v))
                });
                if !covered {
                    findings.push(warning(
                        "W015",
                        &cfg.file_path,
                        &format!(
                            "requirement '{}' is active in configuration '{}' but no TestCase covering it runs in {}",
                            rid, cfg_id, cfg_id
                        ),
                    ));
                }
            }
        }
    }

    // ── Tier 2 cross-reference checks (E825-E830) ────────────────────────────

    // Build reverse index: csg_implemented_by[csg_id_or_qn] — used for W802
    let mut csg_implemented: HashSet<String> = HashSet::new();
    // Build reverse index: he_referenced_by[he_id_or_qn] — used for W800
    let mut he_referenced: HashSet<String> = HashSet::new();
    // Build reverse index: csg_derived_reqs[csg_id_or_qn] — used for W804
    let mut csg_derived_reqs: HashSet<String> = HashSet::new();
    // Build reverse index: sg_derived_reqs[sg_id_or_qn] — used for W805
    let mut sg_derived_reqs: HashSet<String> = HashSet::new();
    // Build reverse index: asset_referenced — used for W810
    let mut asset_referenced: HashSet<String> = HashSet::new();

    for elem in elements {
        let fm = &elem.frontmatter;

        // E825: SafetyGoal.hazardousEvents must each resolve to a HazardousEvent
        if matches!(fm.element_type, Some(ElementType::SafetyGoal)) {
            if let Some(ref refs) = fm.hazardous_events {
                for r in refs {
                    match resolver.resolve_ref(elements, r) {
                        None => findings.push(error("E825", &elem.file_path,
                            &format!("`hazardousEvents` '{}' does not resolve to any element", r))),
                        Some(target) if !Resolver::is_hazardous_event(target) => {
                            findings.push(error("E825", &elem.file_path,
                                &format!("`hazardousEvents` '{}' does not resolve to a HazardousEvent", r)));
                        }
                        Some(target) => {
                            he_referenced.insert(target.qualified_name.clone());
                            if let Some(ref id) = target.frontmatter.id { he_referenced.insert(id.clone()); }
                        }
                    }
                }
            }
        }

        // E826: ThreatScenario.damageScenarios must each resolve to a DamageScenario
        if matches!(fm.element_type, Some(ElementType::ThreatScenario)) {
            if let Some(ref refs) = fm.damage_scenarios {
                for r in refs {
                    match resolver.resolve_ref(elements, r) {
                        None => findings.push(error("E826", &elem.file_path,
                            &format!("`damageScenarios` '{}' does not resolve to any element", r))),
                        Some(target) if !Resolver::is_damage_scenario(target) => {
                            findings.push(error("E826", &elem.file_path,
                                &format!("`damageScenarios` '{}' does not resolve to a DamageScenario", r)));
                        }
                        _ => {}
                    }
                }
            }
        }

        // E827: CybersecurityGoal.threatScenarios must each resolve to a ThreatScenario
        if matches!(fm.element_type, Some(ElementType::CybersecurityGoal)) {
            if let Some(ref refs) = fm.threat_scenarios {
                for r in refs {
                    match resolver.resolve_ref(elements, r) {
                        None => findings.push(error("E827", &elem.file_path,
                            &format!("`threatScenarios` '{}' does not resolve to any element", r))),
                        Some(target) if !Resolver::is_threat_scenario(target) => {
                            findings.push(error("E827", &elem.file_path,
                                &format!("`threatScenarios` '{}' does not resolve to a ThreatScenario", r)));
                        }
                        _ => {}
                    }
                }
            }
        }

        // E828: SecurityControl.implementsGoals must each resolve to a CybersecurityGoal
        if matches!(fm.element_type, Some(ElementType::SecurityControl)) {
            if let Some(ref refs) = fm.implements_goals {
                for r in refs {
                    match resolver.resolve_ref(elements, r) {
                        None => findings.push(error("E828", &elem.file_path,
                            &format!("`implementsGoals` '{}' does not resolve to any element", r))),
                        Some(target) if !Resolver::is_cybersecurity_goal(target) => {
                            findings.push(error("E828", &elem.file_path,
                                &format!("`implementsGoals` '{}' does not resolve to a CybersecurityGoal", r)));
                        }
                        Some(target) => {
                            csg_implemented.insert(target.qualified_name.clone());
                            if let Some(ref id) = target.frontmatter.id { csg_implemented.insert(id.clone()); }
                        }
                    }
                }
            }
        }

        // E829: VulnerabilityReport.mitigatedBy must each resolve to a SecurityControl
        if matches!(fm.element_type, Some(ElementType::VulnerabilityReport)) {
            if let Some(ref refs) = fm.mitigated_by {
                for r in refs {
                    match resolver.resolve_ref(elements, r) {
                        None => findings.push(error("E829", &elem.file_path,
                            &format!("`mitigatedBy` '{}' does not resolve to any element", r))),
                        Some(target) if !Resolver::is_security_control(target) => {
                            findings.push(error("E829", &elem.file_path,
                                &format!("`mitigatedBy` '{}' does not resolve to a SecurityControl", r)));
                        }
                        _ => {}
                    }
                }
            }
            // E830: affectedElements must resolve to known model elements
            if let Some(ref refs) = fm.affected_elements {
                for r in refs {
                    if resolver.resolve_ref(elements, r).is_none() {
                        findings.push(error("E830", &elem.file_path,
                            &format!("`affectedElements` '{}' does not resolve to any element", r)));
                    }
                }
            }
        }

        // E831: derivedFromCybersecurityGoal must resolve to a CybersecurityGoal
        if let Some(ref goal_ref) = fm.derived_from_cybersecurity_goal {
            match resolver.resolve_ref(elements, goal_ref) {
                None => findings.push(error("E831", &elem.file_path,
                    &format!("`derivedFromCybersecurityGoal` '{}' does not resolve to any element", goal_ref))),
                Some(target) if !Resolver::is_cybersecurity_goal(target) => {
                    findings.push(error("E831", &elem.file_path,
                        &format!("`derivedFromCybersecurityGoal` '{}' does not resolve to a CybersecurityGoal", goal_ref)));
                }
                Some(target) => {
                    csg_derived_reqs.insert(target.qualified_name.clone());
                    if let Some(ref id) = target.frontmatter.id { csg_derived_reqs.insert(id.clone()); }
                }
            }
        }

        // E832: derivedFromSafetyGoal must resolve to a SafetyGoal
        if let Some(ref goal_ref) = fm.derived_from_safety_goal {
            match resolver.resolve_ref(elements, goal_ref) {
                None => findings.push(error("E832", &elem.file_path,
                    &format!("`derivedFromSafetyGoal` '{}' does not resolve to any element", goal_ref))),
                Some(target) if !Resolver::is_safety_goal(target) => {
                    findings.push(error("E832", &elem.file_path,
                        &format!("`derivedFromSafetyGoal` '{}' does not resolve to a SafetyGoal", goal_ref)));
                }
                Some(target) => {
                    sg_derived_reqs.insert(target.qualified_name.clone());
                    if let Some(ref id) = target.frontmatter.id { sg_derived_reqs.insert(id.clone()); }
                }
            }
        }

        // Populate asset_referenced from DamageScenario.assets (used for W810).
        if matches!(fm.element_type, Some(ElementType::DamageScenario)) {
            if let Some(ref asset_refs) = fm.assets {
                for r in asset_refs {
                    if let Some(target) = resolver.resolve_ref(elements, r) {
                        asset_referenced.insert(target.qualified_name.clone());
                        if let Some(ref id) = target.frontmatter.id { asset_referenced.insert(id.clone()); }
                    }
                }
            }
        }

    }

    // ── Tier 4 cross-reference checks ────────────────────────────────────────

    for elem in elements {
        let fm = &elem.frontmatter;

        // E902: FaultTree.topEvent must resolve to a SafetyGoal
        if matches!(fm.element_type, Some(ElementType::FaultTree)) {
            if let Some(ref te) = fm.top_event {
                match resolver.resolve_ref(elements, te) {
                    None => findings.push(error("E902", &elem.file_path,
                        &format!("`topEvent` '{}' does not resolve to any element", te))),
                    Some(target) if !Resolver::is_safety_goal(target) => {
                        findings.push(error("E902", &elem.file_path,
                            &format!("`topEvent` '{}' does not resolve to a SafetyGoal", te)));
                    }
                    _ => {}
                }
            }
        }

        // E906: FaultTreeGate.inputs must each resolve to a FaultTreeGate or FaultTreeEvent
        if matches!(fm.element_type, Some(ElementType::FaultTreeGate)) {
            if let Some(ref inputs) = fm.inputs {
                for r in inputs {
                    match resolver.resolve_ref(elements, r) {
                        None => findings.push(error("E906", &elem.file_path,
                            &format!("`inputs` '{}' does not resolve to any element", r))),
                        Some(target)
                            if !Resolver::is_fault_tree_gate(target)
                                && !Resolver::is_fault_tree_event(target) =>
                        {
                            findings.push(error("E906", &elem.file_path,
                                &format!("`inputs` '{}' is not a FaultTreeGate or FaultTreeEvent", r)));
                        }
                        _ => {}
                    }
                }
            }
        }

        // E917: AttackTree.threatRef must resolve to a ThreatScenario
        if matches!(fm.element_type, Some(ElementType::AttackTree)) {
            if let Some(ref tr) = fm.threat_ref {
                match resolver.resolve_ref(elements, tr) {
                    None => findings.push(error("E917", &elem.file_path,
                        &format!("`threatRef` '{}' does not resolve to any element", tr))),
                    Some(target) if !Resolver::is_threat_scenario(target) => {
                        findings.push(error("E917", &elem.file_path,
                            &format!("`threatRef` '{}' does not resolve to a ThreatScenario", tr)));
                    }
                    _ => {}
                }
            }
        }

        // E920: AttackTreeGate.inputs must each resolve to an AttackTreeGate or AttackStep
        if matches!(fm.element_type, Some(ElementType::AttackTreeGate)) {
            if let Some(ref inputs) = fm.inputs {
                for r in inputs {
                    match resolver.resolve_ref(elements, r) {
                        None => findings.push(error("E920", &elem.file_path,
                            &format!("`inputs` '{}' does not resolve to any element", r))),
                        Some(target)
                            if !Resolver::is_attack_tree_gate(target)
                                && !Resolver::is_attack_step(target) =>
                        {
                            findings.push(error("E920", &elem.file_path,
                                &format!("`inputs` '{}' is not an AttackTreeGate or AttackStep", r)));
                        }
                        _ => {}
                    }
                }
            }
        }

        // W904: FMEAEntry.ref (subject) should resolve to a known element
        if matches!(fm.element_type, Some(ElementType::FMEAEntry)) {
            if let Some(ref r) = fm.subject {
                if resolver.resolve_ref(elements, r).is_none() {
                    findings.push(warning("W904", &elem.file_path,
                        &format!("FMEAEntry `ref` '{}' does not resolve to a known element", r)));
                }
            }
            // W927: FMEAEntry.ftaRef should resolve to a known FaultTreeEvent
            if let Some(ref r) = fm.fta_ref {
                match resolver.resolve_ref(elements, r) {
                    None => findings.push(warning("W927", &elem.file_path,
                        &format!("FMEAEntry `ftaRef` '{}' does not resolve to a known element", r))),
                    Some(target) if !Resolver::is_fault_tree_event(target) => {
                        findings.push(warning("W927", &elem.file_path,
                            &format!("FMEAEntry `ftaRef` '{}' does not resolve to a FaultTreeEvent", r)));
                    }
                    _ => {}
                }
            }
        }

        // W926: FaultTreeEvent.fmeaRef should resolve to a known FMEAEntry
        if matches!(fm.element_type, Some(ElementType::FaultTreeEvent)) {
            if let Some(ref r) = fm.fmea_ref {
                match resolver.resolve_ref(elements, r) {
                    None => findings.push(warning("W926", &elem.file_path,
                        &format!("FaultTreeEvent `fmeaRef` '{}' does not resolve to a known element", r))),
                    Some(target) if !matches!(target.frontmatter.element_type, Some(ElementType::FMEAEntry)) => {
                        findings.push(warning("W926", &elem.file_path,
                            &format!("FaultTreeEvent `fmeaRef` '{}' does not resolve to a FMEAEntry", r)));
                    }
                    _ => {}
                }
            }
        }
    }

    // W900: FaultTree with no FaultTreeGate or FaultTreeEvent children
    for elem in elements {
        if !matches!(elem.frontmatter.element_type, Some(ElementType::FaultTree)) {
            continue;
        }
        let prefix = format!("{}::", elem.qualified_name);
        let has_children = elements.iter().any(|e| {
            e.qualified_name.starts_with(&prefix)
                && matches!(
                    e.frontmatter.element_type,
                    Some(ElementType::FaultTreeGate) | Some(ElementType::FaultTreeEvent)
                )
        });
        if !has_children {
            let id = elem.frontmatter.id.as_deref().unwrap_or(&elem.qualified_name);
            findings.push(warning("W900", &elem.file_path,
                &format!("FaultTree '{}' has no FaultTreeGate or FaultTreeEvent children", id)));
        }
    }

    // W036: AttackTree with no AttackTreeGate or AttackStep children (ISO/SAE
    // 21434 §15.7) — the empty-tree warning, analog of FTA's W900.
    for elem in elements {
        if !matches!(elem.frontmatter.element_type, Some(ElementType::AttackTree)) {
            continue;
        }
        let prefix = format!("{}::", elem.qualified_name);
        let has_children = elements.iter().any(|e| {
            e.qualified_name.starts_with(&prefix)
                && matches!(
                    e.frontmatter.element_type,
                    Some(ElementType::AttackTreeGate) | Some(ElementType::AttackStep)
                )
        });
        if !has_children {
            let id = elem.frontmatter.id.as_deref().unwrap_or(&elem.qualified_name);
            findings.push(warning("W036", &elem.file_path,
                &format!("AttackTree '{}' has no AttackTreeGate or AttackStep children", id)));
        }
    }

    // W035: AttackTree computed feasibility (weakest-link roll-up) does not match
    // the linked ThreatScenario.attackFeasibility (ISO/SAE 21434 §15.7
    // reconciliation). Fires only when the threat resolves and both feasibilities
    // are computable. Gateable via --deny W035; promotable via [profiles].
    for elem in elements {
        if !matches!(elem.frontmatter.element_type, Some(ElementType::AttackTree)) {
            continue;
        }
        let Some(ref tr) = elem.frontmatter.threat_ref else { continue };
        let Some(threat) = resolver.resolve_ref(elements, tr) else { continue };
        if !Resolver::is_threat_scenario(threat) { continue; }
        let Some(declared) = threat.frontmatter.attack_feasibility.as_deref() else { continue };
        let Some(computed) = crate::attack_tree::tree_feasibility(elem, elements, &resolver) else { continue };
        if computed != declared {
            let id = elem.frontmatter.id.as_deref().unwrap_or(&elem.qualified_name);
            findings.push(warning("W035", &elem.file_path,
                &format!("AttackTree '{}' computed feasibility '{}' does not match linked ThreatScenario '{}' declared attackFeasibility '{}'",
                    id, computed, tr, declared)));
        }
    }

    // W800: HazardousEvent not referenced by any SafetyGoal
    for elem in elements {
        if Resolver::is_hazardous_event(elem) {
            let referenced = he_referenced.contains(&elem.qualified_name)
                || elem.frontmatter.id.as_ref().map_or(false, |id| he_referenced.contains(id));
            if !referenced {
                let id = elem.frontmatter.id.as_deref().unwrap_or(&elem.qualified_name);
                findings.push(warning("W800", &elem.file_path,
                    &format!("HazardousEvent '{}' is not referenced by any SafetyGoal.hazardousEvents", id)));
            }
        }
    }

    // W806: SafetyGoal with no hazardousEvents reference — not grounded in a hazard analysis
    for elem in elements {
        if Resolver::is_safety_goal(elem) {
            let has_he = elem.frontmatter.hazardous_events
                .as_ref()
                .map_or(false, |v| !v.is_empty());
            if !has_he {
                let id = elem.frontmatter.id.as_deref().unwrap_or(&elem.qualified_name);
                findings.push(warning("W806", &elem.file_path,
                    &format!("SafetyGoal '{}' has no `hazardousEvents` — it is not grounded in any hazard analysis", id)));
            }
        }
    }

    // W033: quantitative HW safety metric below ASIL/SIL target (ISO 26262-5
    // §8-9, GH #29). Opt-in: computed and gated ONLY for SafetyGoals whose
    // contributing FaultTreeEvents declare diagnosticCoverage — goals without DC
    // data produce no metrics and no finding, keeping unannotated models silent.
    // Warning (not error) by codebase convention; gateable via `--deny W033` and
    // profile-promotable. Shares the formula module with the `metrics` command.
    for report in crate::metrics::report_all(elements, &resolver) {
        let (Some(_metrics), Some(gate)) = (report.metrics.as_ref(), report.gate.as_ref())
        else {
            continue;
        };
        if gate.passed() {
            continue;
        }
        let detail = gate
            .misses
            .iter()
            .map(|m| {
                if m.metric == "PMHF" {
                    format!("{} {:.3e} ≥ target {:.0e} /h", m.metric, m.actual, m.target)
                } else {
                    format!("{} {:.4} < target {:.2}", m.metric, m.actual, m.target)
                }
            })
            .collect::<Vec<_>>()
            .join("; ");
        let level = report
            .asil
            .clone()
            .map(|a| format!("ASIL {}", a))
            .or_else(|| report.sil.map(|s| format!("SIL {}", s)))
            .unwrap_or_else(|| "—".to_string());
        findings.push(warning(
            "W033",
            &report.file_path,
            &format!(
                "SafetyGoal '{}' ({}) misses its hardware safety target: {} (ISO 26262-5 §8-9)",
                report.id, level, detail
            ),
        ));
    }

    // W802: CybersecurityGoal not implemented by any SecurityControl
    for elem in elements {
        if Resolver::is_cybersecurity_goal(elem) {
            let implemented = csg_implemented.contains(&elem.qualified_name)
                || elem.frontmatter.id.as_ref().map_or(false, |id| csg_implemented.contains(id));
            if !implemented {
                let id = elem.frontmatter.id.as_deref().unwrap_or(&elem.qualified_name);
                findings.push(warning("W802", &elem.file_path,
                    &format!("CybersecurityGoal '{}' is not implemented by any SecurityControl.implementsGoals", id)));
            }
        }
    }

    // W804: CybersecurityGoal not referenced by any Requirement via derivedFromCybersecurityGoal
    for elem in elements {
        if Resolver::is_cybersecurity_goal(elem) {
            let has_req = csg_derived_reqs.contains(&elem.qualified_name)
                || elem.frontmatter.id.as_ref().map_or(false, |id| csg_derived_reqs.contains(id));
            if !has_req {
                let id = elem.frontmatter.id.as_deref().unwrap_or(&elem.qualified_name);
                findings.push(warning("W804", &elem.file_path,
                    &format!("CybersecurityGoal '{}' has no Requirement with `derivedFromCybersecurityGoal` pointing to it", id)));
            }
        }
    }

    // W810: Asset not referenced by any DamageScenario.assets (REQ-TRS-TYPE-017).
    for elem in elements {
        if Resolver::is_asset(elem) {
            let referenced = asset_referenced.contains(&elem.qualified_name)
                || elem.frontmatter.id.as_ref().map_or(false, |id| asset_referenced.contains(id));
            if !referenced {
                let id = elem.frontmatter.id.as_deref().unwrap_or(&elem.qualified_name);
                findings.push(warning("W810", &elem.file_path,
                    &format!("Asset '{}' is not referenced by any DamageScenario.assets (ISO/SAE 21434 §15.3 asset identification gap)", id)));
            }
        }
    }

    // ── ISO/SAE 21434 risk determination (GH #30): W031 + W032 ────────────────
    // Warnings (not errors) by codebase convention: completeness gaps are
    // warnings (cf. W306/W029/W030) so bundled-model exit codes stay 0; both are
    // gateable via `--deny` and promotable via [profiles].
    {
        use crate::risk::{self, threat_risk_level, RiskLevel};

        // Set of threat keys (qname + id) addressed by some CybersecurityGoal.
        let mut addressed: std::collections::HashSet<String> = std::collections::HashSet::new();
        for csg in elements.iter().filter(|e| Resolver::is_cybersecurity_goal(e)) {
            if let Some(ref refs) = csg.frontmatter.threat_scenarios {
                for r in refs {
                    if let Some(ts) = resolver.resolve_ref(elements, r) {
                        if Resolver::is_threat_scenario(ts) {
                            addressed.insert(ts.qualified_name.clone());
                            if let Some(ref id) = ts.frontmatter.id {
                                addressed.insert(id.clone());
                            }
                        }
                    }
                }
            }
        }

        // W031: untreated high/critical-risk ThreatScenario.
        for ts in elements.iter().filter(|e| Resolver::is_threat_scenario(e)) {
            let level = match threat_risk_level(ts, elements, &resolver) {
                Some(l) => l,
                None => continue, // unknown risk → listed, not gated
            };
            if level != RiskLevel::High && level != RiskLevel::Critical {
                continue;
            }
            let has_treatment = ts.frontmatter.risk_treatment.is_some();
            let is_addressed = addressed.contains(&ts.qualified_name)
                || ts
                    .frontmatter
                    .id
                    .as_ref()
                    .map_or(false, |id| addressed.contains(id));
            if !has_treatment && !is_addressed {
                let id = ts.frontmatter.id.as_deref().unwrap_or(&ts.qualified_name);
                findings.push(warning(
                    "W031",
                    &ts.file_path,
                    &format!(
                        "ThreatScenario '{}' has {} computed risk but no riskTreatment and is not addressed by any CybersecurityGoal (ISO/SAE 21434 §15.9)",
                        id,
                        level.as_str()
                    ),
                ));
            }
        }

        // W032: CAL inconsistency — a CybersecurityGoal's calLevel is below the
        // expected minimum CAL for the max risk over the threats it lists.
        for csg in elements.iter().filter(|e| Resolver::is_cybersecurity_goal(e)) {
            let Some(ref refs) = csg.frontmatter.threat_scenarios else { continue };
            let mut max_level: Option<RiskLevel> = None;
            for r in refs {
                if let Some(ts) = resolver.resolve_ref(elements, r) {
                    if Resolver::is_threat_scenario(ts) {
                        if let Some(l) = threat_risk_level(ts, elements, &resolver) {
                            max_level = Some(max_level.map_or(l, |m| m.max(l)));
                        }
                    }
                }
            }
            let Some(level) = max_level else { continue }; // no computable risk
            let expected = risk::expected_cal_rank(level);
            // calLevel absent → rank 0, treated as below any expected rank ≥1.
            let actual = csg
                .frontmatter
                .cal_level
                .as_deref()
                .and_then(risk::cal_rank)
                .unwrap_or(0);
            if actual < expected {
                let id = csg.frontmatter.id.as_deref().unwrap_or(&csg.qualified_name);
                let actual_label = csg
                    .frontmatter
                    .cal_level
                    .as_deref()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "(none)".to_string());
                findings.push(warning(
                    "W032",
                    &csg.file_path,
                    &format!(
                        "CybersecurityGoal '{}' declares calLevel {} but its {}-risk threats require at least {} (ISO/SAE 21434 §15.8)",
                        id,
                        actual_label,
                        level.as_str(),
                        risk::cal_label(expected)
                    ),
                ));
            }
        }
    }

    // W805: SafetyGoal not referenced by any Requirement via derivedFromSafetyGoal
    for elem in elements {
        if Resolver::is_safety_goal(elem) {
            let has_req = sg_derived_reqs.contains(&elem.qualified_name)
                || elem.frontmatter.id.as_ref().map_or(false, |id| sg_derived_reqs.contains(id));
            if !has_req {
                let id = elem.frontmatter.id.as_deref().unwrap_or(&elem.qualified_name);
                findings.push(warning("W805", &elem.file_path,
                    &format!("SafetyGoal '{}' has no Requirement with `derivedFromSafetyGoal` pointing to it", id)));
            }
        }
    }

    // ── DIA/CIA responsibility + confirmation measures (REQ-TRS-SAFE-007) ─────
    // W038: a non-draft work product with no `responsibility:`. Opt-in: dormant
    // unless at least one element in the model declares `responsibility:`.
    {
        let responsibility_adopted = elements.iter().any(|e| {
            e.frontmatter.responsibility.as_deref().map(|s| !s.trim().is_empty()).unwrap_or(false)
        });
        if responsibility_adopted {
            for elem in elements {
                let fm = &elem.frontmatter;
                let is_work_product = matches!(
                    fm.element_type,
                    Some(ElementType::Requirement)
                        | Some(ElementType::PartDef)
                        | Some(ElementType::Part)
                        | Some(ElementType::SafetyGoal)
                        | Some(ElementType::CybersecurityGoal)
                );
                if !is_work_product {
                    continue;
                }
                if fm.status.as_deref() == Some("draft") {
                    continue;
                }
                let has_resp = fm.responsibility.as_deref().map(|s| !s.trim().is_empty()).unwrap_or(false);
                if !has_resp {
                    let id = fm.id.as_deref().unwrap_or(&elem.qualified_name);
                    findings.push(warning("W038", &elem.file_path, &format!(
                        "work product '{}' declares no `responsibility:` — assign the accountable party/organisation (ISO 26262-8 §5 DIA / ISO/SAE 21434 §7 CIA)", id)));
                }
            }
        }
    }

    // W039: a high-integrity item lacking its required independent assessment.
    // Opt-in: dormant unless at least one ConfirmationMeasure exists in the model.
    {
        let has_confirmation_measure = elements.iter().any(Resolver::is_confirmation_measure);
        if has_confirmation_measure {
            let mut fs_assessed: HashSet<String> = HashSet::new();    // functional_safety_assessment @ I3
            let mut cs_assessed_i3: HashSet<String> = HashSet::new(); // cybersecurity_assessment @ I3
            let mut cs_assessed_i2: HashSet<String> = HashSet::new(); // cybersecurity_assessment @ I2 or I3 (REQ-TRS-SEC-007)
            for cm in elements {
                if !Resolver::is_confirmation_measure(cm) { continue; }
                let fm = &cm.frontmatter;
                let il = fm.independence_level.as_deref().unwrap_or("");
                let mt = fm.measure_type.as_deref().unwrap_or("");
                let refs = fm.confirms.as_deref().unwrap_or(&[]);
                let insert_to = |set: &mut HashSet<String>| {
                    for r in refs {
                        if let Some(target) = resolver.resolve_ref(elements, r) {
                            set.insert(target.qualified_name.clone());
                            if let Some(id) = &target.frontmatter.id { set.insert(id.clone()); }
                        }
                    }
                };
                if il == "I3" && mt == "functional_safety_assessment" { insert_to(&mut fs_assessed); }
                if il == "I3" && mt == "cybersecurity_assessment" { insert_to(&mut cs_assessed_i3); }
                if (il == "I2" || il == "I3") && mt == "cybersecurity_assessment" { insert_to(&mut cs_assessed_i2); }
            }

            let is_assessed = |elem: &RawElement, set: &HashSet<String>| -> bool {
                set.contains(&elem.qualified_name)
                    || elem.frontmatter.id.as_deref().map(|id| set.contains(id)).unwrap_or(false)
            };

            for elem in elements {
                let fm = &elem.frontmatter;
                // ASIL D or SIL 3/4 SafetyGoal or native Requirement → I3 functional_safety_assessment.
                let is_safety_item = Resolver::is_safety_goal(elem)
                    || Resolver::is_native_requirement(elem);
                let needs_fs_assessment = fm.asil_level.as_deref() == Some("D")
                    || matches!(fm.sil_level, Some(3) | Some(4));
                if is_safety_item && needs_fs_assessment && !is_assessed(elem, &fs_assessed) {
                    let id = fm.id.as_deref().unwrap_or(&elem.qualified_name);
                    let integrity = if let Some(sil) = fm.sil_level {
                        format!("SIL {}", sil)
                    } else {
                        "ASIL D".to_string()
                    };
                    findings.push(warning("W039", &elem.file_path, &format!(
                        "{} item '{}' has no independent (I3) functional_safety_assessment ConfirmationMeasure confirming it (ISO 26262-2 §6 / IEC 61508-1 §8)", integrity, id)));
                }
                if Resolver::is_cybersecurity_goal(elem) {
                    let id = fm.id.as_deref().unwrap_or(&elem.qualified_name);
                    // CAL4 → I3 cybersecurity_assessment required (ISO/SAE 21434 §7).
                    if fm.cal_level.as_deref() == Some("CAL4") && !is_assessed(elem, &cs_assessed_i3) {
                        findings.push(warning("W039", &elem.file_path, &format!(
                            "CAL4 item '{}' has no independent (I3) cybersecurity_assessment ConfirmationMeasure confirming it (ISO/SAE 21434 §7)", id)));
                    }
                    // CAL3 → I2 (or higher) cybersecurity_assessment required (REQ-TRS-SEC-007).
                    if fm.cal_level.as_deref() == Some("CAL3") && !is_assessed(elem, &cs_assessed_i2) {
                        findings.push(warning("W039", &elem.file_path, &format!(
                            "CAL3 item '{}' has no I2 cybersecurity_assessment ConfirmationMeasure confirming it (ISO/SAE 21434 §7)", id)));
                    }
                }
            }
        }
    }

    // ── Traceability checks (§12) ─────────────────────────────────────────────

    // Build reverse index: satisfied_reqs[req_qname_or_id] = list of satisfying element qnames
    let mut satisfied_reqs: HashMap<String, Vec<String>> = HashMap::new();
    for elem in elements {
        if let Some(ref sat) = elem.frontmatter.satisfies {
            for s in sat {
                if let Some(target) = resolver.resolve_ref(elements, s) {
                    satisfied_reqs
                        .entry(target.qualified_name.clone())
                        .or_default()
                        .push(elem.qualified_name.clone());
                } else if config.has_repos() && !config.peer_resolves(s) {
                    // §14.4 — a satisfies target resolving in no repo is E512.
                    findings.push(error(
                        "E512",
                        &elem.file_path,
                        &format!(
                            "cross-repo satisfies reference '{}' resolves neither locally nor in any loaded repo",
                            s
                        ),
                    ));
                }
            }
        }
    }

    // W306 (REQ-TRS-TRACE-010, GH #17): a high-integrity requirement that is not
    // a fully integrated safety mechanism — draft, unsatisfied, or active in no
    // configuration. Default threshold silLevel>=4 / asilLevel D (per-profile
    // configurability rides with the severity-profile work, GH #18).
    {
        let var_active = crate::variability::is_active(elements);
        let pkg = crate::variability::package_conditions(elements);
        let feat_alias = crate::variability::feature_id_to_qname(elements);
        let configs: Vec<std::collections::BTreeMap<String, bool>> = if var_active {
            elements
                .iter()
                .filter(|e| matches!(e.frontmatter.element_type, Some(ElementType::Configuration)))
                .map(|c| {
                    crate::variability::canon_selection(
                        &c.frontmatter.feature_selections(),
                        &feat_alias,
                    )
                })
                .collect()
        } else {
            Vec::new()
        };

        for elem in elements {
            if !Resolver::is_native_requirement(elem) {
                continue;
            }
            let fm = &elem.frontmatter;
            let high_integrity =
                fm.sil_level.is_some_and(|n| n >= 4) || fm.asil_level.as_deref() == Some("D");
            if !high_integrity {
                continue;
            }

            // A parent requirement (one with derivedChildren) is satisfied
            // transitively through its leaves — E312 forbids satisfying it
            // directly — so the "no satisfier" sub-condition applies to leaf
            // requirements only (GH #34; mirrors the W002 parent suppression).
            let req_id = fm.id.as_deref().unwrap_or(&elem.qualified_name);
            let is_parent = derived_children.get(req_id).map_or(false, |v| !v.is_empty());

            let mut reasons: Vec<&str> = Vec::new();
            if fm.status.as_deref() == Some("draft") {
                reasons.push("status: draft");
            }
            if !is_parent && !satisfied_reqs.contains_key(&elem.qualified_name) {
                reasons.push("no element satisfies it");
            }
            // all-N/A: a feature model is active, configurations exist, and the
            // requirement's effective appliesWhen is false in every one of them.
            if var_active && !configs.is_empty() {
                if let Some(expr) = crate::variability::effective_expr_canon(elem, &pkg, &feat_alias) {
                    let active_somewhere = configs
                        .iter()
                        .any(|sel| expr.eval(&|q: &str| sel.get(q).copied().unwrap_or(false)));
                    if !active_somewhere {
                        reasons.push("active in no configuration");
                    }
                }
            }

            if !reasons.is_empty() {
                findings.push(warning(
                    "W306",
                    &elem.file_path,
                    &format!(
                        "high-integrity Requirement '{}' is not a fully integrated safety mechanism: {}",
                        req_id,
                        reasons.join("; ")
                    ),
                ));
            }
        }
    }

    for elem in elements {
        let fm = &elem.frontmatter;

        // E310: native Requirement with derivedFrom must have breakdownAdr
        if Resolver::is_native_requirement(elem) {
            if fm.derived_from.as_ref().map_or(false, |v| !v.is_empty()) {
                if fm.breakdown_adr.is_none() {
                    findings.push(error(
                        "E310",
                        &elem.file_path,
                        "Requirement has `derivedFrom` but no `breakdownAdr`",
                    ));
                }
            }
        }

        // E311: breakdownAdr must resolve to an ADR
        if let Some(ref adr_ref) = fm.breakdown_adr {
            match resolver.resolve_ref(elements, adr_ref) {
                None => findings.push(error(
                    "E311",
                    &elem.file_path,
                    &format!("`breakdownAdr` '{}' cannot be resolved", adr_ref),
                )),
                Some(target) if !Resolver::is_adr(target) => {
                    findings.push(error(
                        "E311",
                        &elem.file_path,
                        &format!("`breakdownAdr` '{}' does not resolve to an ADR", adr_ref),
                    ));
                }
                // W303: breakdownAdr references a proposed ADR but requirement is approved or higher
                Some(target) => {
                    let req_status = fm.status.as_deref().unwrap_or("");
                    let adr_status = target.frontmatter.status.as_deref().unwrap_or("");
                    const APPROVED_OR_HIGHER: &[&str] = &["approved", "implemented", "verified"];
                    if adr_status == "proposed" && APPROVED_OR_HIGHER.contains(&req_status) {
                        findings.push(warning(
                            "W303",
                            &elem.file_path,
                            &format!(
                                "`breakdownAdr` '{}' is still `proposed` but Requirement has status '{}'",
                                adr_ref, req_status
                            ),
                        ));
                    }
                }
            }
        }

        // E312: a parent requirement (has derivedChildren) must not appear in any satisfies list
        if Resolver::is_native_requirement(elem) {
            let req_id = fm.id.as_deref().unwrap_or("");
            let is_parent = derived_children.get(req_id).map_or(false, |c| !c.is_empty());
            if is_parent {
                let qn = &elem.qualified_name;
                let in_satisfies = satisfied_reqs.contains_key(qn.as_str())
                    || (req_id != "" && satisfied_reqs.contains_key(req_id));
                if in_satisfies {
                    findings.push(error(
                        "E312",
                        &elem.file_path,
                        &format!("parent Requirement '{}' appears in a `satisfies:` list — only leaf requirements may be assigned", req_id),
                    ));
                }
            }
        }

        // E313: satisfies domain mismatch — architecture element domain vs requirement reqDomain
        if let Some(ref sat) = fm.satisfies {
            let elem_domain = fm.domain.as_deref().unwrap_or("system");
            for s in sat {
                if let Some(target) = resolver.resolve_ref(elements, s) {
                    if Resolver::is_native_requirement(target) {
                        let req_domain = target.frontmatter.req_domain.as_deref().unwrap_or("system");
                        if elem_domain != "system" && req_domain != "system" && elem_domain != req_domain {
                            findings.push(error(
                                "E313",
                                &elem.file_path,
                                &format!(
                                    "`satisfies` domain mismatch: element has `domain: {}` but requirement '{}' has `reqDomain: {}`",
                                    elem_domain, s, req_domain
                                ),
                            ));
                        }
                    }
                }
            }
        }

        // E841 / W808: derivedFromSafetyGoal — integrity level must propagate downstream
        if let Some(ref goal_ref) = fm.derived_from_safety_goal {
            if let Some(goal) = resolver.resolve_ref(elements, goal_ref) {
                let gfm = &goal.frontmatter;
                let child_has = fm.asil_level.is_some() || fm.sil_level.is_some();
                let src_has   = gfm.asil_level.is_some() || gfm.sil_level.is_some();
                if src_has && !child_has {
                    findings.push(error(
                        "E841",
                        &elem.file_path,
                        &format!(
                            "SafetyGoal '{}' carries an integrity level — this element must also set asilLevel or silLevel",
                            goal_ref
                        ),
                    ));
                } else if src_has && child_has
                    && integrity_is_lower(
                        fm.asil_level.as_deref(), fm.sil_level,
                        gfm.asil_level.as_deref(), gfm.sil_level,
                    )
                    && fm.breakdown_adr.is_none()
                {
                    findings.push(warning(
                        "W808",
                        &elem.file_path,
                        &format!(
                            "integrity level is lower than SafetyGoal '{}' — add `breakdownAdr` to justify the ASIL/SIL decomposition",
                            goal_ref
                        ),
                    ));
                }
            }
        }

        // E842 / W808: derivedFrom — integrity level must propagate through requirement chains
        if let Some(ref dfs) = fm.derived_from {
            for df in dfs {
                if let Some(parent) = resolver.resolve_ref(elements, df) {
                    let pfm = &parent.frontmatter;
                    let child_has = fm.asil_level.is_some() || fm.sil_level.is_some();
                    let src_has   = pfm.asil_level.is_some() || pfm.sil_level.is_some();
                    if src_has && !child_has {
                        findings.push(error(
                            "E842",
                            &elem.file_path,
                            &format!(
                                "parent element '{}' carries an integrity level — derived element must also set asilLevel or silLevel",
                                df
                            ),
                        ));
                    } else if src_has && child_has
                        && integrity_is_lower(
                            fm.asil_level.as_deref(), fm.sil_level,
                            pfm.asil_level.as_deref(), pfm.sil_level,
                        )
                        && fm.breakdown_adr.is_none()
                    {
                        findings.push(warning(
                            "W808",
                            &elem.file_path,
                            &format!(
                                "integrity level is lower than parent '{}' — add `breakdownAdr` to justify the ASIL/SIL decomposition",
                                df
                            ),
                        ));
                    }
                }
            }
        }

        // E843 / W808: satisfies — architecture element must inherit integrity level
        if let Some(ref sat) = fm.satisfies {
            for s in sat {
                if let Some(target) = resolver.resolve_ref(elements, s) {
                    let tfm = &target.frontmatter;
                    let child_has = fm.asil_level.is_some() || fm.sil_level.is_some();
                    let src_has   = tfm.asil_level.is_some() || tfm.sil_level.is_some();
                    if src_has && !child_has {
                        findings.push(error(
                            "E843",
                            &elem.file_path,
                            &format!(
                                "requirement '{}' carries an integrity level — satisfying element must also set asilLevel or silLevel",
                                s
                            ),
                        ));
                    } else if src_has && child_has
                        && integrity_is_lower(
                            fm.asil_level.as_deref(), fm.sil_level,
                            tfm.asil_level.as_deref(), tfm.sil_level,
                        )
                        && fm.breakdown_adr.is_none()
                    {
                        findings.push(warning(
                            "W808",
                            &elem.file_path,
                            &format!(
                                "integrity level is lower than satisfied requirement '{}' — add `breakdownAdr` to justify the ASIL/SIL decomposition",
                                s
                            ),
                        ));
                    }
                }
            }
        }

        // E315: cross-domain direct supertype/typedBy references
        let elem_domain = fm.domain.as_deref().unwrap_or("system");
        if elem_domain != "system" {
            for field_val in [fm.supertype.as_ref(), fm.typed_by.as_ref()].into_iter().flatten() {
                for r in yaml_strings(field_val) {
                    if let Some(target) = resolver.resolve_ref(elements, r) {
                        let target_domain = target.frontmatter.domain.as_deref().unwrap_or("system");
                        if target_domain != "system" && elem_domain != target_domain {
                            findings.push(error(
                                "E315",
                                &elem.file_path,
                                &format!(
                                    "cross-domain reference: `domain: {}` element references `domain: {}` element '{}' — use Allocation instead",
                                    elem_domain, target_domain, r
                                ),
                            ));
                        }
                    }
                }
            }
        }
    }

    // E314: deployment packages must have at least one Allocation to a hardware element
    {
        // Build a set of (allocateFrom qname) → target domain for all Allocation elements
        let mut hw_alloc_targets: HashSet<String> = HashSet::new();
        for elem in elements {
            if !matches!(elem.frontmatter.element_type, Some(ElementType::Allocation)) {
                continue;
            }
            // allocated_from is the software side; allocated_to is the hardware side
            if let Some(ref to_refs) = elem.frontmatter.allocated_to {
                for to_ref in to_refs {
                    if let Some(target) = resolver.get(elements, to_ref) {
                        if target.frontmatter.domain.as_deref() == Some("hardware") {
                            if let Some(ref from_refs) = elem.frontmatter.allocated_from {
                                for from_ref in from_refs {
                                    hw_alloc_targets.insert(from_ref.clone());
                                }
                            }
                        }
                    }
                }
            }
        }
        for elem in elements {
            if elem.frontmatter.is_deployment_package == Some(true) {
                if !hw_alloc_targets.contains(&elem.qualified_name) {
                    findings.push(error(
                        "E314",
                        &elem.file_path,
                        &format!(
                            "`isDeploymentPackage: true` element '{}' has no Allocation to a hardware element",
                            elem.qualified_name
                        ),
                    ));
                }
            }
        }
    }

    // W034: Freedom From Interference / dependent-failure analysis (ISO 26262-9 §7,
    // REQ-TRS-SAFE-006). Flag mixed-criticality elements that share an allocation
    // target without a freedom-from-interference / partitioning argument.
    //
    // Deferred: the issue's "cross-domain bonus" (surfacing the shared resources as
    // candidate cybersecurity attack surfaces for the co-analysis view) is NOT done here.
    {
        use std::collections::{BTreeMap, BTreeSet};

        // Opt-in / dormant: do nothing unless at least one element carries an ASIL or
        // SIL classification. A non-safety model emits zero W034.
        let safety_active = elements.iter().any(|e| {
            e.frontmatter.asil_level.is_some() || e.frontmatter.sil_level.is_some()
        });

        if safety_active {
            // Integrity tag for an element: asilLevel, else silLevel ("SIL<n>"), else "QM".
            let integrity_tag = |fm: &crate::element::RawFrontmatter| -> String {
                if let Some(ref a) = fm.asil_level {
                    a.clone()
                } else if let Some(s) = fm.sil_level {
                    format!("SIL{}", s)
                } else {
                    "QM".to_string()
                }
            };

            // True if an element carries an FFI argument: a non-empty ffiRationale, OR a
            // breakdownAdr resolving to an `accepted` ADR.
            let has_ffi_arg = |elem: &RawElement| -> bool {
                if elem
                    .frontmatter
                    .ffi_rationale
                    .as_deref()
                    .map_or(false, |s| !s.trim().is_empty())
                {
                    return true;
                }
                if let Some(ref adr_ref) = elem.frontmatter.breakdown_adr {
                    if let Some(target) = resolver.resolve_ref(elements, adr_ref) {
                        if Resolver::is_adr(target)
                            && target.frontmatter.status.as_deref() == Some("accepted")
                        {
                            return true;
                        }
                    }
                }
                false
            };

            // Collect allocation edges source -> target from every form, resolving
            // references via the Resolver; invert into target qname -> { source qnames }.
            // (Allocation elements use the same allocatedFrom/allocatedTo fields.)
            let mut targets: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
            for elem in elements {
                let qn = &elem.qualified_name;
                // element with allocatedTo: source = this element, target = each T
                if let Some(ref tos) = elem.frontmatter.allocated_to {
                    for t in tos {
                        if let Some(target) = resolver.resolve_ref(elements, t) {
                            targets
                                .entry(target.qualified_name.clone())
                                .or_default()
                                .insert(qn.clone());
                        }
                    }
                }
                // element with allocatedFrom: target = this element, source = each S
                if let Some(ref froms) = elem.frontmatter.allocated_from {
                    for s in froms {
                        if let Some(source) = resolver.resolve_ref(elements, s) {
                            targets
                                .entry(qn.clone())
                                .or_default()
                                .insert(source.qualified_name.clone());
                        }
                    }
                }
            }

            for (target_qn, sources) in &targets {
                if sources.len() < 2 {
                    continue; // a target with <2 sources cannot host a sharing
                }
                let target_elem = resolver.get(elements, target_qn);
                let target_has_ffi = target_elem.map_or(false, has_ffi_arg);

                let srcs: Vec<&String> = sources.iter().collect();
                for i in 0..srcs.len() {
                    for j in (i + 1)..srcs.len() {
                        let a = match resolver.get(elements, srcs[i]) {
                            Some(e) => e,
                            None => continue,
                        };
                        let b = match resolver.get(elements, srcs[j]) {
                            Some(e) => e,
                            None => continue,
                        };
                        let tag_a = integrity_tag(&a.frontmatter);
                        let tag_b = integrity_tag(&b.frontmatter);
                        if tag_a == tag_b {
                            continue; // same tag → not mixed-criticality
                        }
                        // Excused when the target OR at least one source carries an FFI arg.
                        if target_has_ffi || has_ffi_arg(a) || has_ffi_arg(b) {
                            continue;
                        }
                        findings.push(warning(
                            "W034",
                            &a.file_path,
                            &format!(
                                "mixed-criticality sharing on allocation target '{}': '{}' ({}) and '{}' ({}) have no freedom-from-interference argument (add `ffiRationale:` or an `accepted` `breakdownAdr:`)",
                                target_qn, srcs[i], tag_a, srcs[j], tag_b
                            ),
                        ));
                    }
                }
            }
        }
    }

    // W300/W301: leaf requirement coverage by satisfying architecture elements
    for elem in elements {
        if !Resolver::is_native_requirement(elem) {
            continue;
        }
        let req_id = elem.frontmatter.id.as_deref().unwrap_or("");
        let is_parent = derived_children.get(req_id).map_or(false, |c| !c.is_empty());
        if is_parent {
            continue; // only check leaf requirements
        }
        let status = elem.frontmatter.status.as_deref().unwrap_or("");
        let satisfiers = satisfied_reqs.get(&elem.qualified_name).map(|v| v.len()).unwrap_or(0);

        if matches!(status, "approved" | "implemented") && satisfiers == 0 {
            findings.push(warning(
                "W300",
                &elem.file_path,
                &format!("leaf Requirement '{}' (status: {}) has no satisfying architecture element", req_id, status),
            ));
        } else if satisfiers > 1 {
            findings.push(warning(
                "W301",
                &elem.file_path,
                &format!("leaf Requirement '{}' is satisfied by {} elements — only one expected", req_id, satisfiers),
            ));
        }

        // W302: leaf requirement still has reqDomain: system at implemented/verified
        if matches!(status, "implemented" | "verified") {
            let req_domain = elem.frontmatter.req_domain.as_deref().unwrap_or("system");
            if req_domain == "system" {
                findings.push(warning(
                    "W302",
                    &elem.file_path,
                    &format!("leaf Requirement '{}' (status: {}) still has `reqDomain: system` — refine to `hardware` or `software`", req_id, status),
                ));
            }
        }
    }

    // E016/E017/E018: cycle detection in supertype, derivedFrom, and subsets graphs
    {
        let (full_graph, node_idx) = crate::graph::build_graph(elements);
        // Map NodeIndex back to file path for error reporting.
        let idx_to_file: HashMap<petgraph::graph::NodeIndex, &str> = node_idx
            .iter()
            .map(|(qn, &ni)| {
                let file = elements
                    .iter()
                    .find(|e| &e.qualified_name == qn)
                    .map(|e| e.file_path.as_str())
                    .unwrap_or(qn.as_str());
                (ni, file)
            })
            .collect();

        let checks: &[(&str, EdgeKind, &str)] = &[
            ("E016", EdgeKind::Supertype, "supertype cycle detected"),
            ("E017", EdgeKind::DerivedFrom, "derivedFrom cycle detected"),
            ("E018", EdgeKind::Subsets, "subsets cycle detected"),
            // E107 (GH #25): typedBy was previously excluded — a usage typed by
            // itself (length-1 cycle) or a typedBy cycle was silently accepted.
            ("E107", EdgeKind::TypedBy, "typedBy cycle detected (a usage cannot be typed by itself, directly or transitively)"),
        ];

        for (code, kind, label) in checks {
            let mut sub: DiGraph<petgraph::graph::NodeIndex, ()> = DiGraph::new();
            let mut sub_nodes: HashMap<petgraph::graph::NodeIndex, petgraph::graph::NodeIndex> =
                HashMap::new();

            for edge in full_graph.edge_references() {
                if edge.weight() == kind {
                    let src_orig = edge.source();
                    let dst_orig = edge.target();
                    let src = *sub_nodes
                        .entry(src_orig)
                        .or_insert_with(|| sub.add_node(src_orig));
                    let dst = *sub_nodes
                        .entry(dst_orig)
                        .or_insert_with(|| sub.add_node(dst_orig));
                    sub.add_edge(src, dst, ());
                }
            }

            if let Err(cycle) = toposort(&sub, None) {
                let orig_ni = sub[cycle.node_id()];
                let file = idx_to_file.get(&orig_ni).copied().unwrap_or("unknown");
                let qname = &full_graph[orig_ni];
                findings.push(error(
                    code,
                    file,
                    &format!("{} involving '{}'", label, qname),
                ));
            }
        }
    }

    // ── MagicGrid (REQ-TRS-MG-001..005) ──────────────────────────────────────
    //
    // The `refines:` base-format checks (E316/W307) and the `refinedBy` reverse
    // index always run. Everything in the `MG###` namespace is gated behind the
    // MagicGrid profile (`config.magicgrid`), so a model that does not opt in
    // sees none of those findings.
    let mut refined_by: HashMap<String, Vec<String>> = HashMap::new();
    let mut actor_in: HashMap<String, Vec<String>> = HashMap::new();
    let mut mop_refined_by: HashMap<String, Vec<String>> = HashMap::new();

    // Key a target element by its stable id when present, else its qualified name —
    // matching the keying of verifiedBy/derivedChildren and the show/trace lookups.
    let index_key = |t: &RawElement| -> String {
        t.frontmatter
            .id
            .clone()
            .unwrap_or_else(|| t.qualified_name.clone())
    };
    // Display label for a use case in a reverse index (id if present, else qname).
    let elem_label = |e: &RawElement| -> String {
        e.frontmatter
            .id
            .clone()
            .unwrap_or_else(|| e.qualified_name.clone())
    };

    let is_use_case_def = |e: &RawElement| {
        matches!(e.frontmatter.element_type, Some(ElementType::UseCaseDef))
    };
    let is_use_case_usage = |e: &RawElement| {
        matches!(e.frontmatter.element_type, Some(ElementType::UseCase))
    };

    // BASE: refines on UseCaseDef / UseCase (E316), refinedBy index, W307.
    // REQ-TRS-MG-010 — the `refines:` link is also honoured on behavioral defs
    // (ActionDef/Action/StateDef/State): white-box functional analysis (W2)
    // refines the system requirements it realises. The E316 resolve/non-Requirement
    // check and the `refinedBy` index apply identically; only the W307
    // "missing refines" warning stays scoped to UseCaseDef.
    let is_behavioral_refiner = |e: &RawElement| {
        matches!(
            e.frontmatter.element_type,
            Some(ElementType::ActionDef)
                | Some(ElementType::Action)
                | Some(ElementType::StateDef)
                | Some(ElementType::State)
        )
    };
    for elem in elements {
        let fm = &elem.frontmatter;
        let is_uc = is_use_case_def(elem) || is_use_case_usage(elem);
        if !(is_uc || is_behavioral_refiner(elem)) {
            continue;
        }
        // Display noun for the diagnostic ("use case" vs "behavioral element").
        let noun = if is_uc { "use case" } else { "behavioral element" };
        let refines = fm.refines.as_deref().unwrap_or(&[]);
        for r in refines {
            match resolver.resolve_ref(elements, r) {
                None => findings.push(error(
                    "E316",
                    &elem.file_path,
                    &format!(
                        "{} '{}' refines '{}' which resolves to nothing",
                        noun, elem.qualified_name, r
                    ),
                )),
                Some(target) => {
                    let is_req = matches!(
                        target.frontmatter.element_type,
                        Some(ElementType::Requirement) | Some(ElementType::RequirementDef)
                    );
                    if !is_req {
                        findings.push(error(
                            "E316",
                            &elem.file_path,
                            &format!(
                                "{} '{}' refines '{}' which resolves to a {:?}, not a Requirement/RequirementDef",
                                noun,
                                elem.qualified_name,
                                r,
                                target.frontmatter.element_type.clone().unwrap_or(ElementType::Unknown)
                            ),
                        ));
                    } else {
                        refined_by
                            .entry(index_key(target))
                            .or_default()
                            .push(elem_label(elem));
                    }
                }
            }
        }

        // W307: a non-draft UseCaseDef with no refines link (absent or empty).
        if is_use_case_def(elem) {
            let status = fm.status.as_deref().unwrap_or("");
            if status != "draft" && refines.is_empty() {
                findings.push(warning(
                    "W307",
                    &elem.file_path,
                    &format!(
                        "UseCaseDef '{}' has no refines link to a requirement",
                        elem.qualified_name
                    ),
                ));
            }
        }
    }

    // GATED: the MG### checks fire only under the MagicGrid profile.
    if config.magicgrid {
        // True if a type may carry use-case-style `actors:` for MG-002.
        let carries_actors = |e: &RawElement| {
            matches!(
                e.frontmatter.element_type,
                Some(ElementType::UseCaseDef)
                    | Some(ElementType::UseCase)
                    | Some(ElementType::RequirementDef)
                    | Some(ElementType::Requirement)
            )
        };
        let is_part = |e: &RawElement| {
            matches!(
                e.frontmatter.element_type,
                Some(ElementType::Part) | Some(ElementType::PartDef)
            )
        };

        // MG-002: actor validation + actorIn reverse index.
        for elem in elements {
            let fm = &elem.frontmatter;
            if !carries_actors(elem) {
                continue;
            }
            let actors = fm.actors.as_deref().unwrap_or(&[]);
            for a in actors {
                match resolver.resolve_ref(elements, a) {
                    None => findings.push(error(
                        "MG010",
                        &elem.file_path,
                        &format!(
                            "use case '{}' actor '{}' resolves to nothing",
                            elem.qualified_name, a
                        ),
                    )),
                    Some(target) => {
                        if !is_part(target) {
                            findings.push(error(
                                "MG011",
                                &elem.file_path,
                                &format!(
                                    "use case '{}' actor '{}' resolves to a {:?}, not a Part/PartDef",
                                    elem.qualified_name,
                                    a,
                                    target.frontmatter.element_type.clone().unwrap_or(ElementType::Unknown)
                                ),
                            ));
                        } else {
                            if target.frontmatter.mg_bool("mg_external") != Some(true) {
                                findings.push(error(
                                    "MG012",
                                    &elem.file_path,
                                    &format!(
                                        "actor '{}' referenced by use case '{}' is not marked custom_fields.mg_external: true",
                                        target.qualified_name, elem.qualified_name
                                    ),
                                ));
                            }
                            actor_in
                                .entry(index_key(target))
                                .or_default()
                                .push(elem_label(elem));
                        }
                    }
                }
            }
            // MG013: a non-draft UseCaseDef with empty/absent actors.
            if is_use_case_def(elem) {
                let status = fm.status.as_deref().unwrap_or("");
                if status != "draft" && actors.is_empty() {
                    findings.push(error(
                        "MG013",
                        &elem.file_path,
                        &format!(
                            "UseCaseDef '{}' declares no actors (every black-box use case must name at least one actor)",
                            elem.qualified_name
                        ),
                    ));
                }
            }
        }

        // MG-003: mg_cell coordinate + type/column validation.
        const CELLS: [&str; 12] = [
            "B1", "B2", "B3", "B4", "W1", "W2", "W3", "W4", "S1", "S2", "S3", "S4",
        ];
        for elem in elements {
            let fm = &elem.frontmatter;
            let Some(cell) = fm.mg_str("mg_cell") else { continue };
            let cell = cell.trim().to_string();
            if !CELLS.contains(&cell.as_str()) {
                findings.push(error(
                    "MG020",
                    &elem.file_path,
                    &format!(
                        "element '{}' has invalid mg_cell '{}' (expected one of B1-B4, W1-W4, S1-S4)",
                        elem.qualified_name, cell
                    ),
                ));
                continue;
            }
            // Column number is the trailing digit; check type/pillar compatibility.
            let col = cell.chars().last().unwrap_or('0');
            let ty = fm.element_type.clone().unwrap_or(ElementType::Unknown);
            let ok = match col {
                '1' => matches!(ty, ElementType::Requirement | ElementType::RequirementDef),
                '2' => matches!(
                    ty,
                    ElementType::UseCaseDef
                        | ElementType::UseCase
                        | ElementType::ActionDef
                        | ElementType::Action
                        | ElementType::StateDef
                        | ElementType::State
                ),
                '3' => matches!(
                    ty,
                    ElementType::Part
                        | ElementType::PartDef
                        | ElementType::Port
                        | ElementType::PortDef
                        | ElementType::Interface
                        | ElementType::InterfaceDef
                        | ElementType::Connection
                        | ElementType::ConnectionDef
                ),
                '4' => matches!(
                    ty,
                    ElementType::ConstraintDef
                        | ElementType::Constraint
                        | ElementType::CalculationDef
                        | ElementType::Calculation
                        | ElementType::AnalysisCase
                ),
                _ => true,
            };
            if !ok {
                findings.push(error(
                    "MG021",
                    &elem.file_path,
                    &format!(
                        "element '{}' of type {:?} is incompatible with mg_cell '{}' (column {})",
                        elem.qualified_name, ty, cell, col
                    ),
                ));
            }
        }

        // MG-004: Measures of Effectiveness (mg_moe).
        for elem in elements {
            let fm = &elem.frontmatter;
            if fm.mg_bool("mg_moe") != Some(true) {
                continue;
            }
            // MG030: host must be a CalculationDef or AnalysisCase.
            let host_ok = matches!(
                fm.element_type,
                Some(ElementType::CalculationDef) | Some(ElementType::AnalysisCase)
            );
            if !host_ok {
                findings.push(error(
                    "MG030",
                    &elem.file_path,
                    &format!(
                        "mg_moe: true on '{}' of type {:?} — an MoE must be a CalculationDef or AnalysisCase",
                        elem.qualified_name,
                        fm.element_type.clone().unwrap_or(ElementType::Unknown)
                    ),
                ));
            }
            // MG031: mg_moe_measures must resolve to a Requirement/RequirementDef.
            match fm.mg_str("mg_moe_measures") {
                None => findings.push(error(
                    "MG031",
                    &elem.file_path,
                    &format!("MoE '{}' has no mg_moe_measures", elem.qualified_name),
                )),
                Some(m) => match resolver.resolve_ref(elements, &m) {
                    Some(t)
                        if matches!(
                            t.frontmatter.element_type,
                            Some(ElementType::Requirement) | Some(ElementType::RequirementDef)
                        ) => {}
                    _ => findings.push(error(
                        "MG031",
                        &elem.file_path,
                        &format!(
                            "MoE '{}' mg_moe_measures '{}' does not resolve to a Requirement/RequirementDef",
                            elem.qualified_name, m
                        ),
                    )),
                },
            }
            // MG032: direction must be maximize or minimize.
            let direction = fm.mg_str("mg_moe_direction");
            let dir_ok = matches!(direction.as_deref(), Some("maximize") | Some("minimize"));
            if !dir_ok {
                findings.push(error(
                    "MG032",
                    &elem.file_path,
                    &format!(
                        "MoE '{}' mg_moe_direction is {} — expected 'maximize' or 'minimize'",
                        elem.qualified_name,
                        direction.as_deref().map(|d| format!("'{}'", d)).unwrap_or_else(|| "absent".into())
                    ),
                ));
            }
            // MG033: numeric/consistent bounds + optional weight in [0,1].
            let threshold_raw = fm.custom_fields.get("mg_moe_threshold");
            let objective_raw = fm.custom_fields.get("mg_moe_objective");
            let threshold = fm.mg_f64("mg_moe_threshold");
            let objective = fm.mg_f64("mg_moe_objective");
            if (threshold_raw.is_some() && threshold.is_none())
                || (objective_raw.is_some() && objective.is_none())
            {
                findings.push(error(
                    "MG033",
                    &elem.file_path,
                    &format!(
                        "MoE '{}' mg_moe_threshold/mg_moe_objective must be numeric",
                        elem.qualified_name
                    ),
                ));
            } else if let (Some(th), Some(ob)) = (threshold, objective) {
                let consistent = match direction.as_deref() {
                    Some("maximize") => ob >= th,
                    Some("minimize") => ob <= th,
                    _ => true,
                };
                if !consistent {
                    findings.push(error(
                        "MG033",
                        &elem.file_path,
                        &format!(
                            "MoE '{}' bounds inconsistent with direction '{}': threshold {} objective {}",
                            elem.qualified_name,
                            direction.as_deref().unwrap_or("?"),
                            th,
                            ob
                        ),
                    ));
                }
            }
            if let Some(w_raw) = fm.custom_fields.get("mg_moe_weight") {
                match fm.mg_f64("mg_moe_weight") {
                    Some(w) if (0.0..=1.0).contains(&w) => {}
                    _ => {
                        let _ = w_raw;
                        findings.push(error(
                            "MG033",
                            &elem.file_path,
                            &format!(
                                "MoE '{}' mg_moe_weight must be numeric in [0, 1]",
                                elem.qualified_name
                            ),
                        ));
                    }
                }
            }
        }

        // MG-008: Measurements of Performance (mg_mop) + mopRefinedBy index.
        // An MoP is a CalculationDef/ConstraintDef/AnalysisCase marked mg_mop:true
        // that refines (mg_mop_refines) the black-box MoE it supports.
        for elem in elements {
            let fm = &elem.frontmatter;
            if fm.mg_bool("mg_mop") != Some(true) {
                continue;
            }
            // MG050: host must be a CalculationDef, ConstraintDef, or AnalysisCase.
            let host_ok = matches!(
                fm.element_type,
                Some(ElementType::CalculationDef)
                    | Some(ElementType::ConstraintDef)
                    | Some(ElementType::AnalysisCase)
            );
            if !host_ok {
                findings.push(error(
                    "MG050",
                    &elem.file_path,
                    &format!(
                        "mg_mop: true on '{}' of type {:?} — an MoP must be a CalculationDef, ConstraintDef, or AnalysisCase",
                        elem.qualified_name,
                        fm.element_type.clone().unwrap_or(ElementType::Unknown)
                    ),
                ));
            }
            // MG051: mg_mop_refines must be present and resolve (by qname or id).
            // MG052: the resolved target must be marked mg_moe: true.
            match fm.mg_str("mg_mop_refines") {
                None => findings.push(error(
                    "MG051",
                    &elem.file_path,
                    &format!("MoP '{}' has no mg_mop_refines", elem.qualified_name),
                )),
                Some(m) => match resolver.resolve_ref(elements, &m) {
                    None => findings.push(error(
                        "MG051",
                        &elem.file_path,
                        &format!(
                            "MoP '{}' mg_mop_refines '{}' does not resolve to a model element",
                            elem.qualified_name, m
                        ),
                    )),
                    Some(target) => {
                        if target.frontmatter.mg_bool("mg_moe") != Some(true) {
                            findings.push(error(
                                "MG052",
                                &elem.file_path,
                                &format!(
                                    "MoP '{}' mg_mop_refines '{}' resolves to an element that is not marked mg_moe: true",
                                    elem.qualified_name, m
                                ),
                            ));
                        } else {
                            // mopRefinedBy reverse index on the MoE, keyed like the
                            // other reverse indices (id when present, else qname).
                            mop_refined_by
                                .entry(index_key(target))
                                .or_default()
                                .push(elem_label(elem));
                        }
                    }
                },
            }
        }

        // MG-009: System-of-Interest boundary marker (mg_soi).
        // MG060 wrong host; MG062 also mg_external; MG061 more than one SoI in model.
        let mut soi_count = 0usize;
        for elem in elements {
            let fm = &elem.frontmatter;
            if fm.mg_bool("mg_soi") != Some(true) {
                continue;
            }
            soi_count += 1;
            // MG060: host must be a Part/PartDef.
            if !is_part(elem) {
                findings.push(error(
                    "MG060",
                    &elem.file_path,
                    &format!(
                        "mg_soi: true on '{}' of type {:?} — the system of interest must be a Part/PartDef",
                        elem.qualified_name,
                        fm.element_type.clone().unwrap_or(ElementType::Unknown)
                    ),
                ));
            }
            // MG062: an element cannot be both the SoI and external to it.
            if fm.mg_bool("mg_external") == Some(true) {
                findings.push(error(
                    "MG062",
                    &elem.file_path,
                    &format!(
                        "'{}' is marked both mg_soi: true and mg_external: true — the system of interest cannot also be external",
                        elem.qualified_name
                    ),
                ));
            }
        }
        // MG061: ambiguous boundary — more than one SoI. Emit on every SoI element.
        if soi_count > 1 {
            for elem in elements {
                if elem.frontmatter.mg_bool("mg_soi") == Some(true) {
                    findings.push(error(
                        "MG061",
                        &elem.file_path,
                        &format!(
                            "'{}' is one of {} elements marked mg_soi: true — a MagicGrid model has a single system of interest",
                            elem.qualified_name, soi_count
                        ),
                    ));
                }
            }
        }

        // MG-011: mg_variant is only meaningful on a Configuration.
        for elem in elements {
            if elem.frontmatter.mg_bool("mg_variant") != Some(true) {
                continue;
            }
            if !matches!(elem.frontmatter.element_type, Some(ElementType::Configuration)) {
                findings.push(error(
                    "MG070",
                    &elem.file_path,
                    &format!(
                        "mg_variant: true on '{}' of type {:?} — mg_variant marks a parametric-variant Configuration",
                        elem.qualified_name,
                        elem.frontmatter.element_type.clone().unwrap_or(ElementType::Unknown)
                    ),
                ));
            }
        }

        // MG-005: logical/physical layering (mg_layer on a Part/PartDef).
        // Pre-index each part's layer and build the set of logical parts realised by
        // an Allocation to a physical part.
        let layer_of = |e: &RawElement| -> Option<String> {
            if !is_part(e) {
                return None;
            }
            e.frontmatter.mg_str("mg_layer")
        };
        // qname -> layer, for resolving Allocation/supertype targets.
        let mut part_layer: HashMap<String, String> = HashMap::new();
        for e in elements {
            if let Some(l) = layer_of(e) {
                part_layer.insert(e.qualified_name.clone(), l);
            }
        }
        // Logical parts that have an allocation to a physical part (source qname
        // set). Both authoring forms feed one unified edge set (REQ-TRS-ALLOC-001).
        let alloc_edges = allocation_edges(elements, &resolver);
        let mut logical_realised: HashSet<String> = HashSet::new();
        for (from, to) in &alloc_edges {
            if part_layer.get(to).map(|l| l == "physical").unwrap_or(false) {
                logical_realised.insert(from.clone());
            }
        }
        for elem in elements {
            if !is_part(elem) {
                continue;
            }
            let Some(layer) = elem.frontmatter.mg_str("mg_layer") else { continue };
            // MG040: layer must be logical or physical.
            if layer != "logical" && layer != "physical" {
                findings.push(error(
                    "MG040",
                    &elem.file_path,
                    &format!(
                        "part '{}' has invalid mg_layer '{}' (expected 'logical' or 'physical')",
                        elem.qualified_name, layer
                    ),
                ));
                continue;
            }
            // MG041: a logical part with no Allocation to a physical part.
            if layer == "logical" && !logical_realised.contains(&elem.qualified_name) {
                findings.push(error(
                    "MG041",
                    &elem.file_path,
                    &format!(
                        "logical part '{}' has no Allocation to a physical element",
                        elem.qualified_name
                    ),
                ));
            }
            // MG042: a logical part directly sharing supertype/typedBy with a physical
            // part (or vice versa). Mirror the E315 walk over supertype + typedBy.
            for field_val in [elem.frontmatter.supertype.as_ref(), elem.frontmatter.typed_by.as_ref()]
                .into_iter()
                .flatten()
            {
                for r in yaml_strings(field_val) {
                    if let Some(target) = resolver.resolve_ref(elements, r) {
                        if let Some(other_layer) = part_layer.get(&target.qualified_name) {
                            if (layer == "logical" && other_layer == "physical")
                                || (layer == "physical" && other_layer == "logical")
                            {
                                findings.push(error(
                                    "MG042",
                                    &elem.file_path,
                                    &format!(
                                        "cross-layer coupling: {} part '{}' references {} part '{}' via supertype/typedBy — relate logical and physical only through an Allocation",
                                        layer, elem.qualified_name, other_layer, r
                                    ),
                                ));
                            }
                        }
                    }
                }
            }
        }

        // ── MagicGrid completeness / coverage warnings (REQ-TRS-MG-014) ──────
        //
        // The *gap-analysis* half of MagicGrid validation: each link of the trace
        // chain (needs → use cases → context → MoEs → requirements → architecture)
        // must actually be present. These are advisory warnings, surfaced by
        // `magicgrid --audit`. They run at the tail of the gated pass so the reverse
        // indices (`refined_by`, `derived_children`, `mop_refined_by`) are fully
        // built before MG080/MG083 consult them.

        // MG080 — orphan stakeholder need. A non-draft B1 Requirement that is
        // neither refined by any behavioral element (empty `refined_by`) nor derived
        // into any requirement (empty `derived_children`). Both indices are keyed by
        // the target's stable id else qname (`index_key`), matching the B1 need's key.
        for elem in elements {
            let fm = &elem.frontmatter;
            if !matches!(fm.element_type, Some(ElementType::Requirement)) {
                continue;
            }
            if fm.status.as_deref() == Some("draft") {
                continue;
            }
            if fm.mg_str("mg_cell").as_deref() != Some("B1") {
                continue;
            }
            let key = index_key(elem);
            let is_refined = refined_by.get(&key).map_or(false, |v| !v.is_empty());
            let is_derived = derived_children.get(&key).map_or(false, |v| !v.is_empty());
            if !is_refined && !is_derived {
                findings.push(warning(
                    "MG080",
                    &elem.file_path,
                    &format!(
                        "orphan stakeholder need: B1 requirement '{}' is neither refined by a use case nor derived into a system requirement",
                        elem.qualified_name
                    ),
                ));
            }
        }

        // MG081 — unallocated functional-analysis element. A W2 behavioral element
        // (ActionDef/Action/StateDef/State) that is the `allocatedFrom` of no
        // Allocation edge whose target resolves to a logical (W3) Part/PartDef.
        // Reuses the Allocation-edge extraction of MG041 (features[].allocatedFrom/
        // allocatedTo + top-level allocated_from/allocated_to), but matches targets
        // marked `mg_layer: logical` rather than physical.
        let is_w2_function = |e: &RawElement| {
            matches!(
                e.frontmatter.element_type,
                Some(ElementType::ActionDef)
                    | Some(ElementType::Action)
                    | Some(ElementType::StateDef)
                    | Some(ElementType::State)
            )
        };
        // Source qnames allocated to a logical part, from the same unified edge
        // set as MG041 (REQ-TRS-ALLOC-001) — matching `mg_layer: logical` targets.
        let mut allocated_to_logical: HashSet<String> = HashSet::new();
        for (from, to) in &alloc_edges {
            if part_layer.get(to).map(|l| l == "logical").unwrap_or(false) {
                allocated_to_logical.insert(from.clone());
            }
        }
        for elem in elements {
            if !is_w2_function(elem) {
                continue;
            }
            if elem.frontmatter.mg_str("mg_cell").as_deref() != Some("W2") {
                continue;
            }
            if !allocated_to_logical.contains(&elem.qualified_name) {
                findings.push(warning(
                    "MG081",
                    &elem.file_path,
                    &format!(
                        "unallocated functional-analysis element: W2 '{}' is allocated to no logical (W3) Part/PartDef",
                        elem.qualified_name
                    ),
                ));
            }
        }

        // MG082 — missing System of Interest. Emitted once, model-level: the model
        // declares a System Context (at least one `mg_external: true` element) but no
        // element is marked `mg_soi: true`. Attach the finding to the first external
        // element's file (else the model root).
        let externals: Vec<&RawElement> = elements
            .iter()
            .filter(|e| e.frontmatter.mg_bool("mg_external") == Some(true))
            .collect();
        let has_soi = elements
            .iter()
            .any(|e| e.frontmatter.mg_bool("mg_soi") == Some(true));
        if !externals.is_empty() && !has_soi {
            let file = externals
                .first()
                .map(|e| e.file_path.as_str())
                .unwrap_or("");
            findings.push(warning(
                "MG082",
                file,
                "missing System of Interest: the model has an mg_external element but no element is marked mg_soi: true",
            ));
        }

        // MG083 — MoE without a MoP. An `mg_moe` element with an empty `mop_refined_by`
        // entry (keyed by stable id else qname, the MG-008 keying): no Measurement of
        // Performance refines it.
        for elem in elements {
            if elem.frontmatter.mg_bool("mg_moe") != Some(true) {
                continue;
            }
            let key = index_key(elem);
            let refined = mop_refined_by.get(&key).map_or(false, |v| !v.is_empty());
            if !refined {
                findings.push(warning(
                    "MG083",
                    &elem.file_path,
                    &format!(
                        "MoE '{}' has no Measurement of Performance refining it (empty mopRefinedBy)",
                        elem.qualified_name
                    ),
                ));
            }
        }
    }

    // ── Derived allocatedFrom index + W503 redundancy (REQ-TRS-ALLOC-001) ────
    //
    // The derived reverse index and the redundancy warning both run for every
    // model (not gated on the MagicGrid profile), over the unified, form-tagged
    // edge set. `allocated_from[target_key]` lists the sources allocated to the
    // target, keyed by the target's stable id else qname (matching the other
    // reverse indices); each source is labelled by its stable id else qname.
    let tagged_edges = allocation_edges_tagged(elements, &resolver);
    // qname → label (stable id when present, else qname) for both endpoints.
    let label_of: HashMap<String, String> = elements
        .iter()
        .map(|e| {
            let label = e
                .frontmatter
                .id
                .clone()
                .unwrap_or_else(|| e.qualified_name.clone());
            (e.qualified_name.clone(), label)
        })
        .collect();
    let mut allocated_from: HashMap<String, Vec<String>> = HashMap::new();
    // Per edge, the set of forms that produced it (for W503), in encounter order.
    let mut edge_forms: HashMap<(String, String), (bool, bool)> = HashMap::new();
    let mut edge_order: Vec<(String, String)> = Vec::new();
    for (from, to, form) in &tagged_edges {
        let entry = edge_forms.entry((from.clone(), to.clone()));
        if matches!(entry, std::collections::hash_map::Entry::Vacant(_)) {
            edge_order.push((from.clone(), to.clone()));
        }
        let flags = entry.or_insert((false, false));
        match form {
            AllocForm::AllocatedTo => flags.0 = true,
            AllocForm::Element => flags.1 = true,
        }
    }
    // Build allocated_from from the de-duplicated edge set.
    for (from, to) in &edge_order {
        let target_key = label_of.get(to).cloned().unwrap_or_else(|| to.clone());
        let source_label = label_of.get(from).cloned().unwrap_or_else(|| from.clone());
        let bucket = allocated_from.entry(target_key).or_default();
        if !bucket.contains(&source_label) {
            bucket.push(source_label);
        }
    }
    // W503 — the same source → target edge declared by BOTH forms.
    for (from, to) in &edge_order {
        if let Some((has_allocated_to, has_element)) = edge_forms.get(&(from.clone(), to.clone())) {
            if *has_allocated_to && *has_element {
                let file = resolver
                    .resolve_ref(elements, from)
                    .map(|e| e.file_path.clone())
                    .unwrap_or_default();
                findings.push(warning(
                    "W503",
                    &file,
                    &format!(
                        "redundant allocation: {} → {} is declared by both an allocatedTo and an Allocation element — use one form",
                        from, to
                    ),
                ));
            }
        }
    }

    // REQ-TRS-XREF-006 — root-package-name hint. A qualified name is path-relative
    // and the model-root package (`_index.md`, empty qname) contributes no segment,
    // so the root package's `name:` is never part of a qualified name. When an
    // unresolved-reference finding quotes a reference that begins with the root
    // package name followed by `::` and the stripped remainder resolves, append a
    // diagnostic hint naming the corrected reference. This changes nothing about
    // resolution — the original error still fires.
    annotate_root_name_hints(&mut findings, elements, &resolver);

    ValidationResult {
        findings,
        verified_by,
        derived_children,
        refined_by,
        actor_in,
        mop_refined_by,
        allocated_from,
    }
}

/// The finding codes that quote a single cross-reference and to which the
/// REQ-TRS-XREF-006 root-name hint applies (the generic unresolved-reference
/// findings: traceability, refinement, allocation, and the structural
/// supertype/typedBy/subsets/redefines/connection resolution errors).
const ROOT_HINT_CODES: &[&str] = &["E102", "E103", "E311", "E316", "E502", "E503"];

/// REQ-TRS-XREF-006 — append a "did you mean" hint to any unresolved-reference
/// finding whose quoted reference wrongly includes the model-root package name.
fn annotate_root_name_hints(
    findings: &mut [Finding],
    elements: &[RawElement],
    resolver: &Resolver,
) {
    // The model-root package is the element with an empty qualified name (the
    // root `_index.md`). Its `name:` is the offending prefix authors wrongly add.
    let root_name = match elements
        .iter()
        .find(|e| e.qualified_name.is_empty())
        .and_then(|e| e.frontmatter.name.as_deref())
    {
        Some(n) if !n.is_empty() => n.to_string(),
        _ => return, // no named root package — never fire (REQ-TRS-XREF-006).
    };
    let prefix = format!("{}::", root_name);

    for f in findings.iter_mut() {
        if !ROOT_HINT_CODES.contains(&f.code) {
            continue;
        }
        if f.message.contains("hint: the model-root package name") {
            continue; // already annotated
        }
        // Extract every single-quoted token from the message and test each one.
        let mut hint: Option<String> = None;
        for token in single_quoted_tokens(&f.message) {
            if let Some(stripped) = token.strip_prefix(&prefix) {
                if !stripped.is_empty() && resolver.resolve_ref(elements, stripped).is_some() {
                    hint = Some(stripped.to_string());
                    break;
                }
            }
        }
        if let Some(stripped) = hint {
            f.message.push_str(&format!(
                " (hint: the model-root package name is not part of qualified names; did you mean '{}'?)",
                stripped
            ));
        }
    }
}

/// Extract the contents of every `'…'` single-quoted span in a string.
fn single_quoted_tokens(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut chars = s.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        if c == '\'' {
            let start = i + 1;
            let mut end = None;
            for (j, c2) in s[start..].char_indices() {
                if c2 == '\'' {
                    end = Some(start + j);
                    break;
                }
            }
            if let Some(e) = end {
                out.push(s[start..e].to_string());
                // Skip past the closing quote.
                while let Some(&(k, _)) = chars.peek() {
                    if k <= e {
                        chars.next();
                    } else {
                        break;
                    }
                }
            }
        }
    }
    out
}

/// Recursively scan a list of YAML mappings for `typedBy:` string values and resolve them
/// into qualified names added to `out`. Also descends into `ports:` sub-lists.
fn collect_typed_by_refs(
    list: &[serde_yaml::Value],
    elements: &[RawElement],
    resolver: &Resolver,
    out: &mut HashSet<String>,
) {
    let key_typed_by = serde_yaml::Value::String("typedBy".into());
    let key_ports = serde_yaml::Value::String("ports".into());
    for item in list {
        if let serde_yaml::Value::Mapping(map) = item {
            if let Some(v) = map.get(&key_typed_by) {
                for s in yaml_strings(v) {
                    if let Some(target) = resolver.resolve_ref(elements, s) {
                        out.insert(target.qualified_name.clone());
                    }
                }
            }
            // Recurse into nested ports: sub-key
            if let Some(serde_yaml::Value::Sequence(ports)) = map.get(&key_ports) {
                collect_typed_by_refs(ports, elements, resolver, out);
            }
        }
    }
}

/// Returns true for element types that are definitions and must be used by at least one usage.
fn is_type_def(elem: &RawElement) -> bool {
    matches!(
        elem.frontmatter.element_type,
        Some(
            ElementType::PartDef
            | ElementType::ItemDef
            | ElementType::AttributeDef
            | ElementType::PortDef
            | ElementType::ConnectionDef
            | ElementType::InterfaceDef
            | ElementType::ActionDef
            | ElementType::ConstraintDef
            | ElementType::RequirementDef
            | ElementType::CalculationDef
            | ElementType::StateDef
            | ElementType::FlowDef
            | ElementType::UseCaseDef
            | ElementType::ViewpointDef
            | ElementType::ViewDef
            | ElementType::AllocationDef
        )
    )
}

/// FeatureDef parameter-binding validation (§9.7): E203–E206, E222, W017.
/// Shared by the main `validate` pass and by `feature-check`, so a product line
/// checked holistically gets the same binding/range enforcement (GH #14).
/// Dormant unless at least one `FeatureDef` exists.
pub fn parameter_binding_findings(elements: &[RawElement]) -> Vec<Finding> {
    let mut findings: Vec<Finding> = Vec::new();
    let has_feature_def = elements
        .iter()
        .any(|e| matches!(e.frontmatter.element_type, Some(ElementType::FeatureDef)));
    if !has_feature_def {
        return findings;
    }
    struct ParamMeta {
        is_fixed: bool,
        range: Option<(f64, f64)>,
        enum_values: Option<Vec<String>>,
        is_required: bool,
        has_default: bool,
        /// Binding-time rank (compile=0, load=1, runtime=2); `None` when `bindingTime:`
        /// is absent (unspecified — the parameter opts out of binding-time checks).
        binding_time: Option<u8>,
    }
    let parse_range = |s: &str| -> Option<(f64, f64)> {
        // Accept both "min..max" and inclusive "min..=max".
        let (lo, hi) = s.split_once("..")?;
        let hi = hi.trim();
        let hi = hi.strip_prefix('=').unwrap_or(hi).trim();
        Some((lo.trim().parse().ok()?, hi.parse().ok()?))
    };
    let num = |v: &serde_yaml::Value| v.as_f64().or_else(|| v.as_i64().map(|i| i as f64));

    let mut feature_params: HashMap<String, HashMap<String, ParamMeta>> = HashMap::new();
    for fd in elements
        .iter()
        .filter(|e| matches!(e.frontmatter.element_type, Some(ElementType::FeatureDef)))
    {
        let mut params: HashMap<String, ParamMeta> = HashMap::new();
        if let Some(list) = &fd.frontmatter.parameters {
            for p in list {
                let serde_yaml::Value::Mapping(m) = p else { continue };
                let get = |k: &str| m.get(serde_yaml::Value::String(k.to_string()));
                let Some(name) = get("name").and_then(|v| v.as_str()) else { continue };
                let is_fixed = get("isFixed").and_then(|v| v.as_bool()).unwrap_or(false)
                    || get("derivedFrom").is_some()
                    || get("value").is_some();
                let range = get("range").and_then(|v| v.as_str()).and_then(parse_range);
                let enum_values = get("enumValues")
                    .map(|v| yaml_strings(v).into_iter().map(|s| s.to_string()).collect::<Vec<_>>());
                let is_required = get("isRequired").and_then(|v| v.as_bool()).unwrap_or(false);
                let has_default = get("default").is_some() || get("value").is_some();
                // bindingTime: PLE triad (compile<load<runtime); E230 on an unknown value.
                let binding_time = match get("bindingTime").and_then(|v| v.as_str()) {
                    None => None,
                    Some("compile") => Some(0),
                    Some("load") => Some(1),
                    Some("runtime") => Some(2),
                    Some(other) => {
                        findings.push(error("E230", &fd.file_path, &format!(
                            "parameter '{}.{}' has bindingTime '{}' which is not one of compile/load/runtime",
                            fd.qualified_name, name, other)));
                        None
                    }
                };
                params.insert(
                    name.to_string(),
                    ParamMeta { is_fixed, range, enum_values, is_required, has_default, binding_time },
                );
            }
        }
        feature_params.insert(fd.qualified_name.clone(), params);
    }

    for cfg in elements
        .iter()
        .filter(|e| matches!(e.frontmatter.element_type, Some(ElementType::Configuration)))
    {
        let sel = crate::variability::canon_selection(
            &cfg.frontmatter.feature_selections(),
            &crate::variability::feature_id_to_qname(elements),
        );
        let is_selected = |feat: &str| sel.get(feat).copied().unwrap_or(false);
        let file = &cfg.file_path;
        let mut bound: HashSet<String> = HashSet::new();

        if let Some(serde_yaml::Value::Mapping(bindings)) = &cfg.frontmatter.parameter_bindings {
            for (k, val) in bindings {
                let Some(path) = k.as_str() else { continue };
                let Some((feat, pname)) = path.rsplit_once('.') else {
                    findings.push(error("E222", file, &format!(
                        "parameterBindings key '{}' is not a '<FeatureDef>.<param>' path (the parameter member is separated by '.')", path)));
                    continue;
                };
                bound.insert(path.to_string());
                let Some(params) = feature_params.get(feat) else {
                    findings.push(error("E222", file, &format!(
                        "parameterBindings path '{}' references unknown FeatureDef '{}'", path, feat)));
                    continue;
                };
                let Some(meta) = params.get(pname) else {
                    findings.push(error("E222", file, &format!(
                        "parameterBindings path '{}' references undeclared parameter '{}' on '{}'", path, pname, feat)));
                    continue;
                };
                if !is_selected(feat) {
                    findings.push(error("E203", file, &format!(
                        "parameterBindings binds '{}' but feature '{}' is not selected", path, feat)));
                }
                if meta.is_fixed {
                    findings.push(error("E204", file, &format!(
                        "parameterBindings binds '{}' which is fixed (isFixed/value/derivedFrom) and may not be overridden", path)));
                }
                if let Some((lo, hi)) = meta.range {
                    if let Some(n) = num(val) {
                        if n < lo || n > hi {
                            findings.push(error("E205", file, &format!(
                                "parameterBindings '{}' = {} is outside range {}..{}", path, n, lo, hi)));
                        }
                    }
                }
                if let Some(allowed) = &meta.enum_values {
                    if let Some(s) = val.as_str() {
                        if !allowed.iter().any(|a| a == s) {
                            findings.push(error("E206", file, &format!(
                                "parameterBindings '{}' = '{}' is not in enumValues {:?}", path, s, allowed)));
                        }
                    }
                }
                if meta.binding_time == Some(2) {
                    findings.push(warning("W027", file, &format!(
                        "parameterBindings binds '{}' which has bindingTime: runtime (resolved by the running system, not at configuration time)", path)));
                }
            }
        }

        for (feat, params) in &feature_params {
            if !is_selected(feat) {
                continue;
            }
            for (pname, meta) in params {
                // A runtime parameter is legitimately unbound by a Configuration —
                // the running system supplies its value (REQ-TRS-PARAM-004).
                if meta.is_required && !meta.is_fixed && !meta.has_default && meta.binding_time != Some(2) {
                    let path = format!("{}.{}", feat, pname);
                    if !bound.contains(&path) {
                        findings.push(warning("W017", file, &format!(
                            "required parameter '{}' of selected feature '{}' is not bound (and has no default)", path, feat)));
                    }
                }
            }
        }
    }
    findings
}

/// W028 (§3, REQ-TRS-EXTREF-001): an `extRef` value declared by two or more
/// elements. Opt-in (dormant unless some element declares `extRef`); one finding
/// per duplicated value, naming the sharing elements. Lookup still returns all.
pub fn ext_ref_duplicate_findings(elements: &[RawElement]) -> Vec<Finding> {
    // Map each external reference to the elements (qnames) that declare it.
    let mut owners: HashMap<&str, Vec<&str>> = HashMap::new();
    let mut order: Vec<&str> = Vec::new();
    for e in elements {
        let Some(refs) = &e.frontmatter.ext_ref else { continue };
        for r in refs {
            let r = r.trim();
            if r.is_empty() {
                continue;
            }
            let entry = owners.entry(r).or_default();
            if entry.is_empty() {
                order.push(r);
            }
            entry.push(e.qualified_name.as_str());
        }
    }
    let mut findings = Vec::new();
    for r in order {
        let owners = &owners[r];
        if owners.len() > 1 {
            // Anchor the finding at the first declaring element's file.
            let file = elements
                .iter()
                .find(|e| e.qualified_name == owners[0])
                .map(|e| e.file_path.as_str())
                .unwrap_or("");
            findings.push(warning("W028", file, &format!(
                "extRef '{}' is declared by {} elements ({})",
                r, owners.len(), owners.join(", "))));
        }
    }
    findings
}

/// True if `req` is `target`, or a transitive `derivedFrom` descendant of it —
/// i.e. `req` lies in the goal-closure of `target`. Used by the TestPlan W614
/// check so a plan that demonstrates a parent goal whose leaves are tested is
/// not flagged (the parent is demonstrated through its leaves). Cycle-guarded.
fn req_self_or_descendant_of(
    req: &RawElement,
    target: &RawElement,
    elements: &[RawElement],
    resolver: &Resolver,
) -> bool {
    let mut stack = vec![req];
    let mut seen: HashSet<String> = HashSet::new();
    while let Some(cur) = stack.pop() {
        if !seen.insert(cur.qualified_name.clone()) {
            continue;
        }
        if cur.qualified_name == target.qualified_name {
            return true;
        }
        if let Some(parents) = &cur.frontmatter.derived_from {
            for p in parents {
                if let Some(parent) = resolver.resolve_ref(elements, p) {
                    stack.push(parent);
                }
            }
        }
    }
    false
}

/// Which authoring form produced an allocation edge.
///
/// `AllocatedTo` is form 1 — an `allocatedTo:` on the source element (the
/// OSLC-canonical default; the source *is* the derived `allocatedFrom`).
/// `Element` is form 2 — a standalone `type: Allocation` element naming both
/// `allocatedFrom` and `allocatedTo`, top-level or per `features:` entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllocForm {
    AllocatedTo,
    Element,
}

/// Raw, form-tagged allocation edges before de-duplication, with endpoints
/// already resolved to qualified names. Skips operands that do not resolve
/// (E502/E503 report those separately). This is the single source of truth
/// shared by [`allocation_edges`], `MG041`, `MG081`, `matrix --allocations`,
/// the derived `allocatedFrom` index, and the W503 redundancy check.
pub fn allocation_edges_tagged(
    elements: &[RawElement],
    resolver: &Resolver,
) -> Vec<(String, String, AllocForm)> {
    let mut edges: Vec<(String, String, AllocForm)> = Vec::new();
    let resolve = |r: &str| resolver.resolve_ref(elements, r).map(|e| e.qualified_name.clone());

    for elem in elements {
        let is_allocation =
            matches!(elem.frontmatter.element_type, Some(ElementType::Allocation));

        // Form 1 — `allocatedTo:` on a non-Allocation source element. The element
        // itself is the derived `allocatedFrom`.
        if !is_allocation {
            if let Some(ref tos) = elem.frontmatter.allocated_to {
                for to in tos {
                    if let Some(to_qn) = resolve(to) {
                        edges.push((
                            elem.qualified_name.clone(),
                            to_qn,
                            AllocForm::AllocatedTo,
                        ));
                    }
                }
            }
            continue;
        }

        // Form 2 — a standalone `type: Allocation` element.
        // Top-level allocatedFrom × allocatedTo cartesian.
        let froms = elem.frontmatter.allocated_from.as_deref().unwrap_or(&[]);
        let tos = elem.frontmatter.allocated_to.as_deref().unwrap_or(&[]);
        for to in tos {
            for from in froms {
                if let (Some(f), Some(t)) = (resolve(from), resolve(to)) {
                    edges.push((f, t, AllocForm::Element));
                }
            }
        }
        // Each `features:` entry carrying BOTH allocatedFrom and allocatedTo —
        // regardless of whether the entry also declares a feature-level
        // `type: Allocation` (the per-feature type requirement is dropped).
        if let Some(ref feats) = elem.frontmatter.features {
            for feat_val in feats {
                if let serde_yaml::Value::Mapping(ref feat) = *feat_val {
                    let from = feat
                        .get(&serde_yaml::Value::String("allocatedFrom".into()))
                        .and_then(|v| v.as_str());
                    let to = feat
                        .get(&serde_yaml::Value::String("allocatedTo".into()))
                        .and_then(|v| v.as_str());
                    if let (Some(from), Some(to)) = (from, to) {
                        if let (Some(f), Some(t)) = (resolve(from), resolve(to)) {
                            edges.push((f, t, AllocForm::Element));
                        }
                    }
                }
            }
        }
    }
    edges
}

/// The unified, de-duplicated set of resolved allocation edges
/// `(from_qname, to_qname)` from BOTH authoring forms. Consumed by `MG041`,
/// `MG081`, `matrix --allocations`, and the derived `allocatedFrom` index so
/// the gate and the matrix can never disagree.
pub fn allocation_edges(elements: &[RawElement], resolver: &Resolver) -> Vec<(String, String)> {
    let mut seen: HashSet<(String, String)> = HashSet::new();
    let mut out: Vec<(String, String)> = Vec::new();
    for (from, to, _form) in allocation_edges_tagged(elements, resolver) {
        if seen.insert((from.clone(), to.clone())) {
            out.push((from, to));
        }
    }
    out
}

fn error(code: &'static str, file: &str, msg: &str) -> Finding {
    Finding { code, file: file.to_string(), message: msg.to_string(), severity: Severity::Error }
}

fn warning(code: &'static str, file: &str, msg: &str) -> Finding {
    Finding { code, file: file.to_string(), message: msg.to_string(), severity: Severity::Warning }
}

fn info(code: &'static str, file: &str, msg: &str) -> Finding {
    Finding { code, file: file.to_string(), message: msg.to_string(), severity: Severity::Info }
}

/// True when a YAML value is a *scalar* — string, number, bool, or null.
/// Mappings and sequences are not scalars.
fn is_yaml_scalar(v: &serde_yaml::Value) -> bool {
    matches!(
        v,
        serde_yaml::Value::Null
            | serde_yaml::Value::Bool(_)
            | serde_yaml::Value::Number(_)
            | serde_yaml::Value::String(_)
    )
}

/// W041 shape predicate (GH #39): a `custom_fields` value is well-shaped when it is
/// a scalar, or a list whose every element is a scalar. A nested map, or a list
/// containing a map/list, is rejected.
fn is_custom_field_shape_ok(v: &serde_yaml::Value) -> bool {
    match v {
        serde_yaml::Value::Sequence(items) => items.iter().all(is_yaml_scalar),
        other => is_yaml_scalar(other),
    }
}

/// Extract the normative text: everything before the first `##` heading.
fn normative_text(doc: &str) -> &str {
    doc.find("\n## ")
        .or_else(|| doc.find("\n# "))
        .map(|pos| &doc[..pos])
        .unwrap_or(doc)
}

/// Extract all scenario titles (Scenario: / Scenario Outline:) from Gherkin blocks.
fn extract_gherkin_scenarios(doc: &str) -> HashSet<&str> {
    let mut titles = HashSet::new();
    let mut in_gherkin = false;
    for line in doc.lines() {
        let trimmed = line.trim();
        if trimmed == "```gherkin" {
            in_gherkin = true;
            continue;
        }
        if in_gherkin && trimmed == "```" {
            in_gherkin = false;
            continue;
        }
        if in_gherkin {
            if let Some(rest) = trimmed.strip_prefix("Scenario:").or_else(|| {
                trimmed
                    .strip_prefix("Scenario Outline:")
                    .or_else(|| trimmed.strip_prefix("Scenario outline:"))
            }) {
                titles.insert(rest.trim());
            }
        }
    }
    titles
}

fn check_scenario_outline_has_examples(doc: &str, file: &str, findings: &mut Vec<Finding>) {
    let mut in_gherkin = false;
    let mut in_outline = false;
    for line in doc.lines() {
        let trimmed = line.trim();
        if trimmed == "```gherkin" {
            in_gherkin = true;
            continue;
        }
        if in_gherkin && trimmed == "```" {
            if in_outline {
                findings.push(error("E014", file, "Scenario Outline has no Examples: table"));
            }
            in_gherkin = false;
            in_outline = false;
            continue;
        }
        if in_gherkin {
            if trimmed.starts_with("Scenario Outline:") || trimmed.starts_with("Scenario outline:") {
                in_outline = true;
            } else if trimmed.starts_with("Examples:") {
                in_outline = false;
            } else if in_outline
                && (trimmed.starts_with("Scenario:")
                    || trimmed.starts_with("Scenario Outline:")
                    || trimmed == "```")
            {
                findings.push(error("E014", file, "Scenario Outline has no Examples: table"));
                in_outline = false;
            }
        }
    }
    if in_outline {
        findings.push(error("E014", file, "Scenario Outline has no Examples: table"));
    }
}

fn first_gherkin_has_feature(doc: &str) -> bool {
    let mut in_first = false;
    let mut found = false;
    for line in doc.lines() {
        let trimmed = line.trim();
        if !in_first && trimmed == "```gherkin" {
            in_first = true;
            continue;
        }
        if in_first {
            if trimmed == "```" {
                break;
            }
            if trimmed.starts_with("Feature:") {
                found = true;
                break;
            }
        }
    }
    !in_first || found // if no gherkin block, E011 will fire; don't double-report
}

#[cfg(test)]
mod custom_field_shape_tests {
    use super::*;

    fn scalar(s: &str) -> serde_yaml::Value {
        serde_yaml::Value::String(s.to_string())
    }

    #[test]
    fn scalars_are_ok() {
        assert!(is_custom_field_shape_ok(&scalar("Bosch")));
        assert!(is_custom_field_shape_ok(&serde_yaml::Value::Number(3.into())));
        assert!(is_custom_field_shape_ok(&serde_yaml::Value::Bool(true)));
        assert!(is_custom_field_shape_ok(&serde_yaml::Value::Null));
    }

    #[test]
    fn list_of_scalars_is_ok() {
        let v = serde_yaml::Value::Sequence(vec![scalar("A-1"), scalar("A-2")]);
        assert!(is_custom_field_shape_ok(&v));
    }

    #[test]
    fn nested_map_is_rejected() {
        let mut m = serde_yaml::Mapping::new();
        m.insert(scalar("k"), scalar("v"));
        assert!(!is_custom_field_shape_ok(&serde_yaml::Value::Mapping(m)));
    }

    #[test]
    fn list_with_nonscalar_is_rejected() {
        let inner = serde_yaml::Value::Sequence(vec![scalar("x")]);
        let v = serde_yaml::Value::Sequence(vec![scalar("ok"), inner]);
        assert!(!is_custom_field_shape_ok(&v));
    }
}

#[cfg(test)]
mod custom_field_roundtrip_tests {
    #[test]
    fn custom_fields_serialize_sorted() {
        let yaml = "type: PartDef\ncustom_fields:\n  zeta: 1\n  alpha: 2\n  middle: [c, a, b]\n";
        let fm: crate::element::RawFrontmatter = serde_yaml::from_str(yaml).unwrap();
        let out = serde_yaml::to_string(&fm).unwrap();
        let a = out.find("alpha").unwrap();
        let m = out.find("middle").unwrap();
        let z = out.find("zeta").unwrap();
        assert!(a < m && m < z, "custom_fields keys not sorted:\n{out}");
        // list element order is preserved verbatim
        assert!(out.contains("- c"));
    }
}
