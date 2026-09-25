use std::collections::{HashMap, HashSet};
use petgraph::algo::toposort;
use petgraph::graph::DiGraph;
use petgraph::visit::EdgeRef;
use crate::config::{load_plantuml_config, ValidateConfig};
use crate::element::{ElementType, ParseIssue, RawElement};
use crate::graph::EdgeKind;
use crate::resolver::{
    is_adr_id, is_asset_id, is_aou_id, is_arg_id, is_at_id, is_atg_id, is_ats_id, is_basic_name, is_cm_id,
    is_cd_id, is_conf_id, is_csg_id, is_ds_id, is_fm_id, is_fmea_id, is_ft_id, is_fte_id, is_ftg_id, is_he_id,
    is_zn_id,
    is_pi_id,
    is_req_id, is_rr_id, is_sc_id, is_sg_id, is_stable_id, is_tara_id, is_tc_id, is_test_plan_id, is_trd_id, is_ts_id,
    is_vr_id, Resolver,
};

/// Matches `href="..."` attributes in an embedded SVG block. Compiled once
/// (clippy::regex_creation_in_loops — it is consulted inside the per-element loop).
static HREF_RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();

fn href_re() -> &'static regex::Regex {
    HREF_RE.get_or_init(|| regex::Regex::new(r#"href="([^"]+)""#).unwrap())
}

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
    /// verifiedBy[target_id_or_qname] = ids of the (native TestCase) elements
    /// whose `verifies:` resolves to that target. Only recorded when the
    /// verifying element itself carries a stable id — in practice always a
    /// native TestCase, the only source type `REQ-TRS-SYSMLV2-004` concerns.
    /// The target key is the target's stable id when present, else its
    /// qualified name (`REQ-TRS-SYSMLV2-004`: a SysMLv2-mapped target has no
    /// id) — matching the id-else-qname convention `refined_by`/
    /// `allocated_from`/etc. already use. Consumers that specifically want
    /// "active TestCase" coverage (W002/W003) filter this list themselves by
    /// looking up each id's `status`.
    pub verified_by: HashMap<String, Vec<String>>,
    /// derived_children[req_id] = list of child req ids
    pub derived_children: HashMap<String, Vec<String>>,
    /// REQ-TRS-PLANITEM-002 — planning_children[pi_id] = ids of the PlanningItems
    /// naming it via `parent:`. A PlanningItem with an empty (absent-key) entry
    /// here is a **leaf** (REQ-TRS-PLANITEM-002's definition, consumed by the
    /// evidence rule in REQ-TRS-PLANITEM-006); one with no `parent:` at all is
    /// **top-level**. The two are independent: a lone PlanningItem is both.
    pub planning_children: HashMap<String, Vec<String>>,
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
            .get(serde_yaml::Value::String("kind".into()))
            .and_then(|v| v.as_str());
        let name = m
            .get(serde_yaml::Value::String("name".into()))
            .and_then(|v| v.as_str());
        if let (Some(k), Some(n)) = (kind, name) {
            if k == "SendAction" || k == "AcceptAction" {
                out.push(n.to_string());
            }
        }
        for branch in ["then", "else", "subActions"] {
            if let Some(serde_yaml::Value::Sequence(seq)) =
                m.get(serde_yaml::Value::String(branch.into()))
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
/// Whether a `Diagram`'s rendering path carries its SVG inline in the body —
/// the only case the SVG id-consistency rules `W406`/`W407` apply to
/// (§8.16.7 step 3, GH #158). Paths with no inline SVG by design:
/// PlantUML companion (`pumlMode: companion` — the `.puml`/`.svg` companions
/// are the source of truth), companion SVG (`svgMode: companion` or
/// `svgFile:`), Mermaid / inline PlantUML (`diagramKind: Mermaid|PlantUML`),
/// and a structured `layout:` diagram whose body has no ` ```svg ` block
/// (the server renders its SVG from the manifest).
fn diagram_has_inline_svg(fm: &crate::element::RawFrontmatter, doc: &str) -> bool {
    if fm.puml_mode.as_deref() == Some("companion") {
        return false;
    }
    if fm.svg_mode.as_deref().unwrap_or("inline") != "inline" {
        return false;
    }
    if fm.svg_mode.is_none() && fm.svg_file.is_some() {
        return false;
    }
    if matches!(fm.diagram_kind.as_deref(), Some("Mermaid") | Some("PlantUML")) {
        return false;
    }
    if fm.layout.is_some() && !doc.contains("```svg") {
        return false;
    }
    true
}

fn yaml_field<'a>(m: &'a serde_yaml::Mapping, k: &str) -> Option<&'a serde_yaml::Value> {
    m.get(serde_yaml::Value::String(k.to_string()))
}

// ── PlanningItem evidence: resolution primitives (REQ-TRS-PLANITEM-005/006) ──
//
// Single source of truth for "does this evidence: entry's own target resolve"
// so the per-entry E716/E717 checks and REQ-TRS-PLANITEM-006's leaf-evidence
// rule (E719) never disagree — the latter is built directly on top of the
// former rather than re-deriving resolution logic.

/// True when a PlanningItem evidence: `ref:` value resolves to any known
/// element — unrestricted by kind (ADR-SYS-PLANITEM-001 Decision 3).
fn evidence_ref_resolves(elements: &[RawElement], resolver: &Resolver, r: &str) -> bool {
    resolver.resolve_ref(elements, r).is_some()
}

/// True when a PlanningItem evidence: `path:` value resolves, reusing
/// `implementedBy:`'s exact `classify_source` semantics: a local path must
/// exist on disk, a remote URI is accepted as external with no local check.
fn evidence_path_resolves(config: &ValidateConfig, p: &str) -> bool {
    match config.classify_source(p) {
        crate::config::SourceLocation::Local(pb) => pb.exists(),
        crate::config::SourceLocation::Remote(_) => true,
    }
}

/// Whether a single evidence: entry's own `ref:`/`path:` target resolves,
/// ignoring any `rationale:` waiver entirely (waiver semantics are the
/// caller's concern — E716/E717 use a waiver to skip *flagging* an entry;
/// REQ-TRS-PLANITEM-006's leaf rule uses it to exclude the entry from
/// *counting* as proof — two different questions over the same primitive).
fn evidence_entry_target_resolves(
    entry: &serde_yaml::Value,
    elements: &[RawElement],
    resolver: &Resolver,
    config: &ValidateConfig,
) -> bool {
    let Some(m) = entry.as_mapping() else { return false };
    if let Some(r) = yaml_field(m, "ref").and_then(|v| v.as_str()) {
        if evidence_ref_resolves(elements, resolver, r) {
            return true;
        }
    }
    if let Some(p) = yaml_field(m, "path").and_then(|v| v.as_str()) {
        if evidence_path_resolves(config, p) {
            return true;
        }
    }
    false
}

/// True when an evidence: entry's own `rationale:` is a non-empty string,
/// mirroring `ffi_rationale`'s co-located waiver pattern (documents *and*
/// waives that one entry).
fn evidence_entry_is_waived(entry: &serde_yaml::Value) -> bool {
    entry
        .as_mapping()
        .and_then(|m| yaml_field(m, "rationale"))
        .and_then(|v| v.as_str())
        .is_some_and(|r| !r.trim().is_empty())
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

/// Collect `W929` messages for transitions missing a required endpoint (§8.8.3): a
/// **top-level** transition (the machine's own `transitions:`) needs `source:`/`from:`,
/// and **every** transition — top-level or nested at any depth under `subStates:` — needs
/// `target:`/`to:`. A nested transition's `source` is implicit (its enclosing substate).
/// Such a transition yields no usable `(source → target)` edge, so without this check the
/// §22.1 completeness rules would silently ignore it (REQ-TRS-SM-009, GH #136).
fn incomplete_transitions(
    sub_states: &[serde_yaml::Value],
    top: Option<&[serde_yaml::Value]>,
    out: &mut Vec<String>,
) {
    fn describe(m: &serde_yaml::Mapping, known: Option<&str>, which: &str) -> String {
        match yaml_field(m, "name").and_then(|v| v.as_str()) {
            Some(n) => format!("transition '{}'", n),
            None => match known {
                Some(k) => format!("transition {} '{}'", which, k),
                None => "transition".to_string(),
            },
        }
    }
    fn endpoint<'a>(m: &'a serde_yaml::Mapping, canon: &str, alias: &str) -> Option<&'a str> {
        yaml_field(m, canon).or_else(|| yaml_field(m, alias)).and_then(|v| v.as_str())
    }
    for t in top.unwrap_or(&[]) {
        let Some(m) = t.as_mapping() else { continue };
        let (src, tgt) = (endpoint(m, "source", "from"), endpoint(m, "target", "to"));
        if src.is_none() {
            out.push(format!(
                "top-level {} has no `source:` — a transition not nested under its source substate must name it (§8.8.3)",
                describe(m, tgt, "to")
            ));
        }
        if tgt.is_none() {
            out.push(format!(
                "top-level {} has no `target:` — every transition must name its target state (§8.8.3)",
                describe(m, src, "from")
            ));
        }
    }
    for s in sub_states {
        let Some(sm) = s.as_mapping() else { continue };
        let state = yaml_field(sm, "name").and_then(|v| v.as_str());
        if let Some(serde_yaml::Value::Sequence(ts)) = yaml_field(sm, "transitions") {
            for t in ts {
                let Some(m) = t.as_mapping() else { continue };
                if endpoint(m, "target", "to").is_none() {
                    let src = endpoint(m, "source", "from").or(state);
                    out.push(format!(
                        "{} has no `target:` — every transition must name its target state (§8.8.3)",
                        describe(m, src, "from")
                    ));
                }
            }
        }
        if let Some(serde_yaml::Value::Sequence(inner)) = yaml_field(sm, "subStates") {
            incomplete_transitions(inner, None, out);
        }
    }
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
    // §14.3/§14.4 (GH #138): with `[repos]` configured, install the local
    // `repoImports:` mount points so a `<package>::<as>::X` reference resolves
    // to the peer's `<qname>::X` everywhere `peer_resolves` is consulted.
    if config.has_repos() && config.repo_mounts.is_empty() {
        let mounts = crate::config::repo_mounts(elements, &config.repos);
        if !mounts.is_empty() {
            let mut mounted = config.clone();
            mounted.repo_mounts = mounts;
            return validate_with_config(elements, &mounted);
        }
    }
    let mut findings: Vec<Finding> = Vec::new();

    // Collect findings stashed on `RawElement.derive_findings` by more than one
    // walker post-processing pass: the derive pass (E504-E506) and native
    // SysMLv2 submodel ingestion (W540) both share this one vector — see that
    // field's doc comment in element.rs.
    for elem in elements {
        for (code, file, message) in &elem.derive_findings {
            let sev = if code.starts_with('E') { Severity::Error } else { Severity::Warning };
            let static_code: &'static str = match code.as_str() {
                // Declarative derive pass (GH #127): E504 cycle (reserved), E505
                // formula parse error, E506 unknown `elements["QName"]` reference —
                // disjoint from the Allocation resolution codes E500–E503.
                "E504" => "E504", "E505" => "E505", "E506" => "E506",
                // Native SysML v2/KerML submodel ingestion (ADR-SYS-SYSMLV2-001,
                // REQ-TRS-SYSMLV2-006) — its own code range, distinct from the
                // WASM-plugin family. W541 is a placeholder pending REQ-TRS-SYSMLV2-006's
                // formal dedicated range.
                "W540" => "W540",
                "W541" => "W541",
                // REQ-TRS-SYSMLV2-015: a connect endpoint's genuinely
                // two-segment chain fell back to a head-only edge because
                // the tail wasn't a locally-redeclared feature -- same
                // dedicated code range as W540/W541.
                "W542" => "W542",
                // REQ-TRS-FM-005: single-file `featureTree:` sheet explosion
                // (`walker::explode_feature_model_trees`) — a node with no
                // `name:` (E231), a qname collision (E232), or `featureTree:`
                // declared on a non-`FeatureModel` element (W048).
                "E231" => "E231",
                "E232" => "E232",
                "E233" => "E233",
                "W048" => "W048",
                // Foreign-format ingestion via stdio-subprocess plugins
                // (ADR-SYS-PLUGIN-002) — its own dedicated code range,
                // distinct from the temporary SysMLv2 placeholder range
                // (W540-W542) and from the never-shipped WASM-plugin family
                // (reserved E530-E532/W530-W534 per ADR-SYS-PLUGIN-002,
                // `feat/wasm-plugins`, unmerged).
                "E550" => "E550",
                "E551" => "E551",
                "W550" => "W550",
                "W551" => "W551",
                "W552" => "W552",
                "W553" => "W553",
                // Annotated-source ingestion (ADR-SYS-ANNOTATE-001) — its own
                // dedicated code range, distinct from the stdio-plugin family
                // (E550/E551/W550-553) above.
                "E560" => "E560",
                "E561" => "E561",
                "W560" => "W560",
                "W561" => "W561",
                "W562" => "W562",
                "W563" => "W563",
                // §3.10 locale documentation variants (walker
                // `attach_locale_variants`, REQ-TRS-PARSE-010): E026 dangling
                // `qualifiedName:` target, W051 duplicate locale / type
                // mismatch / ignored structural field.
                "E026" => "E026",
                "W051" => "W051",
                _ => "E000",
            };
            findings.push(Finding { code: static_code, file: file.clone(), message: message.clone(), severity: sev });
        }
    }

    // W415: [plantuml] style_file path does not exist (REQ-TRS-PUML-042)
    if let Some(ref root) = config.model_root {
        let pcfg = load_plantuml_config(root);
        if let Some(ref sf) = pcfg.style_file {
            if !sf.exists() {
                findings.push(warning(
                    "W415",
                    root.to_str().unwrap_or(""),
                    &format!(
                        "[plantuml] style_file '{}' does not exist — fix the path in .syscribe.toml",
                        sf.display()
                    ),
                ));
            }
        }
    }

    // W046: malformed `[ids.prefixes]` config (REQ-TRS-ID-007). Reported once, before
    // any per-element id check. An unknown element-type key or a prefix not matching
    // `^[A-Z][A-Z0-9]{1,11}$` is skipped by the resolver; surface it so the author
    // knows the intended prefix is not in effect. Keys are sorted for stable output.
    if !config.id_extra_prefixes.is_empty() {
        let cfg_file = config
            .model_root
            .as_ref()
            .map(|r| r.join(".syscribe.toml").display().to_string())
            .unwrap_or_else(|| ".syscribe.toml".to_string());
        let mut types: Vec<&String> = config.id_extra_prefixes.keys().collect();
        types.sort();
        for type_name in types {
            if !crate::resolver::is_stable_id_type_name(type_name) {
                findings.push(warning(
                    "W046",
                    &cfg_file,
                    &format!(
                        "[ids.prefixes] key '{}' is not an id-identified element type — entry ignored",
                        type_name
                    ),
                ));
                continue;
            }
            for prefix in &config.id_extra_prefixes[type_name] {
                if !crate::resolver::is_valid_id_prefix(prefix) {
                    findings.push(warning(
                        "W046",
                        &cfg_file,
                        &format!(
                            "[ids.prefixes] prefix '{}' for type '{}' is malformed (expected uppercase, 2–12 chars, starting with a letter) — prefix ignored",
                            prefix, type_name
                        ),
                    ));
                }
            }
        }
    }

    // W309: malformed `[users]` key (REQ-TRS-PLANITEM-008). A key that isn't a
    // well-formed username is ignored by the E722 roster check (it can never
    // match a format-valid `assignedTo:`, which E723 already governs
    // separately) — surface it so the author knows the entry isn't in effect.
    // Keys sorted for stable output, same posture as the W046 block above.
    if !config.users.is_empty() {
        let cfg_file = config
            .model_root
            .as_ref()
            .map(|r| r.join(".syscribe.toml").display().to_string())
            .unwrap_or_else(|| ".syscribe.toml".to_string());
        let mut names: Vec<&String> = config.users.keys().collect();
        names.sort();
        for name in names {
            if !crate::resolver::is_valid_username(name) {
                findings.push(warning(
                    "W309",
                    &cfg_file,
                    &format!(
                        "[users] key '{}' is not a valid username (expected: lowercase, starting with a letter or underscore, then lowercase letters/digits/underscore/hyphen, max 32 chars) — entry ignored",
                        name
                    ),
                ));
            }
        }
    }

    // W630: malformed `[linkTypes]` config (REQ-TRS-LINKTYPE-001) — reported once
    // against `.syscribe.toml`, same posture as W046/W309 above. The registry has
    // already dropped structurally invalid entries (so their uses surface as E630).
    if !config.link_types.defects().is_empty() {
        let cfg_file = config
            .model_root
            .as_ref()
            .map(|r| r.join(".syscribe.toml").display().to_string())
            .unwrap_or_else(|| ".syscribe.toml".to_string());
        for msg in config.link_types.defects() {
            findings.push(warning("W630", &cfg_file, msg));
        }
    }

    // E630–E636/W631: `links:` instances against their declarations
    // (REQ-TRS-LINKTYPE-002..005). Always computed on the authored elements.
    findings.extend(link_type_findings(elements, config));

    // REQ-TRS-LINKTYPE-006 — `extends`: every rule and reverse index below sees an
    // instance of a type extending `satisfies`/`verifies`/`derivedFrom`/`refines`
    // as an entry of that base field. `effective_view` builds a private clone for
    // this run only (the caller's elements are never mutated and nothing here is
    // ever written back), and `link_prov` answers, per `(source, entry, code)`,
    // whether a code is relaxed or the entry is withheld from the reverse index.
    // With no extending type in use this is `None` and every rule runs on the
    // authored elements exactly as before. Content hashing (baselines, suspect
    // links) keeps using `authored_elements` so the projection is never skewed.
    let authored_elements = elements;
    let effective = crate::link_types::effective_view(elements, &config.link_types);
    let default_prov = crate::link_types::Provenance::default();
    let (elements, link_prov): (&[RawElement], &crate::link_types::Provenance) = match &effective {
        Some(view) => (view.elements.as_slice(), &view.provenance),
        None => (authored_elements, &default_prov),
    };

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

        // W048 (REQ-TRS-FM-005 review): `parameterConstraints:` is a typed
        // field (promoted off the `extra` catch-all so declaring it no longer
        // falsely raises W047 on the element that hosts it — see
        // `RawFrontmatter::parameter_constraints`), which means a misplaced
        // block is otherwise now completely silent instead of at least
        // getting the old, if confusingly-worded, W047. `feature_model.rs`
        // only ever reads it off Package/LibraryPackage/Namespace/
        // FeatureModel; flag it here on any other type.
        if fm.parameter_constraints.is_some()
            && !matches!(
                fm.element_type,
                Some(ElementType::Package)
                    | Some(ElementType::LibraryPackage)
                    | Some(ElementType::Namespace)
                    | Some(ElementType::FeatureModel)
            )
        {
            findings.push(warning(
                "W048",
                &file,
                "'parameterConstraints:' is only recognized on a Package/LibraryPackage/Namespace or a FeatureModel sheet; ignored here",
            ));
        }

        // W047 (REQ-TRS-SCHEMA-001): unrecognised top-level frontmatter field. Any key
        // not bound to a recognised schema field lands in the `extra` catch-all and is
        // otherwise silently discarded — a hazard for typos (`reqDomian`, `verifis`).
        // Advisory severity; gate with `--deny W047`. Author-defined data belongs under
        // `custom_fields:` (§3.15), which is exempt. Keys sorted for deterministic order.
        let mut unknown_keys: Vec<&str> = fm.extra.keys().map(String::as_str).collect();
        // `ref:` is a schema field only on a FaultTreeEvent (REQ-TRS-FTA-002); on
        // any other type it is still unrecognized, exactly as before it was bound.
        if fm.event_ref.is_some() && !matches!(fm.element_type, Some(ElementType::FaultTreeEvent)) {
            unknown_keys.push("ref");
        }
        // W049 (REQ-TRS-QNAME-005, GH #160): the qualified name is purely
        // path-derived (§4.2/§4.5/§11.3), so `qualifiedName:` is not an identity
        // override. It is meaningful only as a §3.10 locale variant's target
        // (with `locale:`); anywhere else a value that differs from the
        // path-derived name is reported and ignored.
        if let (Some(q), None) = (&fm.qualified_name, &fm.locale) {
            if q.trim() != elem.qualified_name {
                findings.push(warning(
                    "W049",
                    &file,
                    &format!(
                        "`qualifiedName: {}` is not supported as an identity override — the qualified name is \
                         path-derived ('{}', §4.5/§11.3); the field is ignored. Move or rename the file to \
                         change the qualified name (`qualifiedName:` is only meaningful on a `locale:` \
                         documentation variant, §3.10)",
                        q.trim(),
                        elem.qualified_name
                    ),
                ));
            }
        }

        // `deciders:` is a schema field only on an ADR (§8.17.1, REQ-TRS-ADR-001).
        if fm.deciders.is_some() && !matches!(fm.element_type, Some(ElementType::ADR)) {
            unknown_keys.push("deciders");
        }
        unknown_keys.sort_unstable();
        for key in unknown_keys {
            findings.push(warning(
                "W047",
                &file,
                &format!(
                    "unrecognized frontmatter field '{}' — not a recognized schema field; \
                     move author-defined data under `custom_fields:` (§3.15)",
                    key
                ),
            ));
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
            if fm.verifies.as_ref().is_none_or(|v| v.is_empty()) {
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
                    if let Some(tb) = m.get(k("typedBy")) {
                        type_refs.extend(yaml_strings(tb));
                    }
                }
            }
            for v in fm.parameters.iter().flatten() {
                if let serde_yaml::Value::Mapping(m) = v {
                    if let Some(t) = m.get(k("type")) {
                        type_refs.extend(yaml_strings(t));
                    }
                }
            }
            for op in fm.operations.iter().flatten() {
                if let serde_yaml::Value::Mapping(m) = op {
                    if let Some(rt) = m.get(k("returnType")) {
                        type_refs.extend(yaml_strings(rt));
                    }
                    if let Some(serde_yaml::Value::Sequence(params)) = m.get(k("parameters")) {
                        for p in params {
                            if let serde_yaml::Value::Mapping(pm) = p {
                                if let Some(tb) = pm.get(k("typedBy")) {
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
                        .get(kk("typedBy"))
                        .or_else(|| m.get(kk("type")))
                        .and_then(|x| x.as_str());
                    let u = m.get(kk("unit")).and_then(|x| x.as_str());
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
                        // REQ-TRS-LINKTYPE-006 — only entries in the reverse index
                        // credit coverage: a `coverage = false` extending
                        // verifies/derivedFrom link traces but covers nothing.
                        let covered = members.iter().any(|tc| {
                            tc.frontmatter.verifies.as_ref().is_some_and(|vs| {
                                vs.iter().enumerate().any(|(vi, v)| {
                                    link_prov.in_reverse_index(
                                        &tc.qualified_name,
                                        crate::link_types::BaseLink::Verifies,
                                        vi,
                                    ) && resolver.resolve_ref(elements, v).is_some_and(|rt| {
                                        req_self_or_descendant_of(rt, target, elements, &resolver, link_prov)
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
                // Argument.evidence is a flat list of scalar element refs. A
                // non-scalar entry (e.g. a PlanningItem-shaped `ref:`/`path:`
                // mapping, which cannot appear on an Argument) previously failed
                // the whole file's YAML parse outright (a loud E002) back when
                // this field was Vec<String>; broadening it to Vec<Value> for
                // PlanningItem's benefit means such an entry now parses cleanly
                // but silently contributes nothing anywhere (validate,
                // safety-case, suspect list) unless flagged here explicitly —
                // E718 keeps this a loud, caught error again instead of a
                // silent data-loss regression.
                for r in refs {
                    match r.as_str() {
                        Some(s) => {
                            if resolver.resolve_ref(elements, s).is_none() {
                                findings.push(error("E855", &file, &format!("Argument.evidence '{}' does not resolve to any model element", s)));
                            }
                        }
                        None => {
                            findings.push(error(
                                "E718",
                                &file,
                                "Argument evidence entry is not a scalar reference (expected a string id/qname)",
                            ));
                        }
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
            // E924: status enum (§8.18.2; REQ-TRS-VAL-018, GH #136).
            if let Some(ref s) = fm.status {
                if !["planned", "in_progress", "completed"].contains(&s.as_str()) {
                    findings.push(error("E924", &file, &format!("ConfirmationMeasure.status '{}' must be planned, in_progress, or completed", s)));
                }
            }
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
        if matches!(fm.element_type, Some(ElementType::TestCase))
            && !elem.doc.contains("```gherkin") {
                findings.push(error("E011", &file, "TestCase body has no ```gherkin fenced block"));
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
        if matches!(fm.element_type, Some(ElementType::TestCase))
            && !first_gherkin_has_feature(&elem.doc) {
                findings.push(error("E015", &file, "first ```gherkin block has no Feature: line"));
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
                            Some(hook) => !hook.fetch(&uri).is_some_and(|p| p.exists()),
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
        // links an architecture element (Part/PartDef/Interface/InterfaceDef) to
        // its source artifact(s). Opt-in: only checked when implementedBy is
        // present. Draft elements are suppressed (the implementation may not exist
        // yet). Local paths (model-/repo-relative, absolute, or file://) are
        // checked on disk; remote URIs are accepted as external and not verified.
        if let Some(ref impls) = fm.implemented_by {
            let is_arch = matches!(
                fm.element_type,
                Some(ElementType::Part)
                    | Some(ElementType::PartDef)
                    | Some(ElementType::Interface)
                    | Some(ElementType::InterfaceDef)
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

        // ── PlanningItem (ADR-SYS-PLANITEM-001, REQ-TRS-PLANITEM-001) ────────
        // Schema-only scope for REQ-TRS-PLANITEM-001: stable id, required
        // `name`/`status`, and an optional `itemType`. Hierarchy (`parent:`) is
        // checked in the cross-reference pass below (REQ-TRS-PLANITEM-002); the
        // structural half of `achieves:` (required on a top-level item) is
        // checked here since it needs no cross-reference resolution, while its
        // resolve/type check lives in the cross-reference pass alongside
        // `parent:` (REQ-TRS-PLANITEM-003). Evidence/appliesWhen remain separate
        // follow-on requirements (REQ-TRS-PLANITEM-004..006), not checked here.
        if matches!(fm.element_type, Some(ElementType::PlanningItem)) {
            // E706: id pattern.
            if let Some(ref id) = fm.id {
                if !is_pi_id(id) {
                    findings.push(error("E706", &file, &format!("`id` '{}' does not match PI-* pattern", id)));
                }
            }
            // E707: required fields.
            if fm.id.is_none() {
                findings.push(error("E707", &file, "`id` is required on PlanningItem"));
            }
            if fm.name.is_none() {
                findings.push(error("E707", &file, "`name` is required on PlanningItem"));
            }
            if fm.status.is_none() {
                findings.push(error("E707", &file, "`status` is required on PlanningItem"));
            }
            // E708: status enum — GitHub Projects' three defaults plus `blocked`
            // (ADR-SYS-PLANITEM-001 decision 4).
            if let Some(ref status) = fm.status {
                const PI_STATUSES: &[&str] = &["todo", "in_progress", "blocked", "done"];
                if !PI_STATUSES.contains(&status.as_str()) {
                    findings.push(error("E708", &file, &format!("unknown PlanningItem status '{}'", status)));
                }
            }
            // E709: itemType enum (optional field) — GitHub's default Issue Types
            // (ADR-SYS-PLANITEM-001 decision 4). No inheritance/matching constraint
            // against a parent's itemType is imposed — there is no hierarchy check
            // here at all (REQ-TRS-PLANITEM-001 scope note).
            if let Some(ref item_type) = fm.item_type {
                const PI_ITEM_TYPES: &[&str] = &["bug", "task", "feature"];
                if !PI_ITEM_TYPES.contains(&item_type.as_str()) {
                    findings.push(error("E709", &file, &format!("unknown PlanningItem itemType '{}'", item_type)));
                }
            }
            // E713: a top-level PlanningItem (no `parent:`) must set at least one
            // `achieves:` entry — its reason for existing (REQ-TRS-PLANITEM-003).
            // A non-top-level item is not required to (its purpose is inherited
            // in spirit from its ancestry, per the requirement's rationale).
            if fm.parent.is_none() && fm.achieves.as_ref().is_none_or(|a| a.is_empty()) {
                findings.push(error(
                    "E713",
                    &file,
                    "top-level PlanningItem (no `parent`) must set at least one `achieves` entry",
                ));
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
            if fm.reviews.as_ref().is_none_or(|r| r.is_empty()) {
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
        if fm.is_deployment_package == Some(true)
            && fm.domain.as_deref() == Some("hardware") {
                findings.push(warning("W304", &file, "`isDeploymentPackage: true` combined with `domain: hardware` — deployment packages must be software"));
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
                            let qn = rest.trim().split_once(' ').map(|x| x.1).map(|s| s.trim()).unwrap_or("");
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
                if !resolver.resolves_shape_ref(elements, ref_str) {
                    findings.push(warning(
                        "W402",
                        &file,
                        &format!("shapes `ref` '{}' does not resolve to a known element", ref_str),
                    ));
                }
            };
            let validate_shape_link = |attrs: &serde_yaml::Mapping, findings: &mut Vec<Finding>| {
                let link_qn: Option<&str> = match attrs.get(serde_yaml::Value::String("link".into())) {
                    Some(serde_yaml::Value::String(s)) if !s.is_empty() => Some(s.as_str()),
                    Some(serde_yaml::Value::Bool(true)) => attrs
                        .get(serde_yaml::Value::String("ref".into()))
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
                                attrs.get(serde_yaml::Value::String("ref".into()))
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
                                attrs.get(serde_yaml::Value::String("ref".into()))
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
                    let href_re = href_re();
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
                            m.get(serde_yaml::Value::String("id".into()))
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
                            edge_attrs.get(serde_yaml::Value::String((*field).into()))
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
                                            .get(serde_yaml::Value::String("ref".into()))
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

            // W929 — a transition missing a required endpoint (REQ-TRS-SM-009).
            let mut incomplete = Vec::new();
            incomplete_transitions(subs_opt.unwrap_or(&[]), fm.transitions.as_deref(), &mut incomplete);
            for msg in incomplete {
                findings.push(warning("W929", &file, &msg));
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

        // E403: unrecognized pumlMode value (REQ-TRS-PUML-032)
        if let Some(ref mode) = fm.puml_mode {
            if mode != "companion" {
                findings.push(error(
                    "E403",
                    &file,
                    &format!(
                        "unrecognized `pumlMode` value '{}' — only `companion` is supported",
                        mode
                    ),
                ));
            }
        }

        // E404: pumlMode: companion requires diagramKind (REQ-TRS-PUML-033)
        if fm.puml_mode.as_deref() == Some("companion") && fm.diagram_kind.is_none() {
            findings.push(error(
                "E404",
                &file,
                "`pumlMode: companion` requires `diagramKind` to be set (e.g. `diagramKind: BDD`)",
            ));
        }

        // W413: pumlMode: companion body must contain an image reference (REQ-TRS-PUML-030)
        if fm.puml_mode.as_deref() == Some("companion")
            && !elem.doc.contains("![")
            && !elem.doc.contains("<img")
        {
            findings.push(warning(
                "W413",
                &file,
                "`pumlMode: companion` but body contains no image reference (`![...](.svg)` or `<img`) pointing to the anticipated SVG output",
            ));
        }

        // W414: pumlMode: companion .puml file not yet generated (REQ-TRS-PUML-031)
        if fm.puml_mode.as_deref() == Some("companion") {
            let puml_path = if let Some(ref pf) = fm.puml_file {
                md_dir.join(pf.trim_start_matches("./"))
            } else {
                std::path::Path::new(&file).with_extension("puml")
            };
            if !puml_path.exists() {
                findings.push(warning(
                    "W414",
                    &file,
                    &format!(
                        "companion `.puml` file '{}' not found — run `syscribe plantuml` to generate it",
                        puml_path.display()
                    ),
                ));
            }
        }

        // W406/W407: SVG id consistency — frontmatter shape/edge ids vs inline SVG.
        // §8.16.7 step 3 scopes the check to *inline* SVG, so it is skipped for
        // every rendering path that carries no inline SVG by design (GH #158,
        // REQ-TRS-DIAG-003).
        if diagram_has_inline_svg(fm, &elem.doc) {
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
                                m.get(serde_yaml::Value::String("id".into()))
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
                        .get(serde_yaml::Value::String("type".into()))
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    if feat_type == "Allocation" {
                        if let Some(serde_yaml::Value::String(ref from_str)) =
                            feat.get(serde_yaml::Value::String("allocatedFrom".into()))
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
                            feat.get(serde_yaml::Value::String("allocatedTo".into()))
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

        // W930 (GH #142, §12.9): the `features:`-entry allocation form belongs to
        // form 2 — a standalone `type: Allocation` element. On any other element an
        // entry declaring an allocation (feature-level `type: Allocation`, or an
        // `allocatedFrom:`/`allocatedTo:` key) is not an allocation edge
        // (`allocation_edges_tagged` never reads it), so flag it rather than let
        // it be silently ignored.
        if !matches!(fm.element_type, Some(ElementType::Allocation)) {
            for feat in fm.features.iter().flatten() {
                let serde_yaml::Value::Mapping(m) = feat else { continue };
                let key = |k: &str| m.get(serde_yaml::Value::String(k.into()));
                let declares_allocation = key("type").and_then(|v| v.as_str()) == Some("Allocation")
                    || key("allocatedFrom").is_some()
                    || key("allocatedTo").is_some();
                if declares_allocation {
                    let fname = key("name").and_then(|v| v.as_str()).unwrap_or("?");
                    findings.push(warning(
                        "W930",
                        &file,
                        &format!(
                            "features: entry '{}' declares an allocation, but only a `type: Allocation` element carries features-form allocations (§12.9) — it contributes no allocation edge; use `allocatedTo:` on the source or a standalone `Allocation` element",
                            fname
                        ),
                    ));
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
                            .get(serde_yaml::Value::String("ref".into()))
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
                        op.get(serde_yaml::Value::String("parameters".into()))
                    {
                        for param_val in params {
                            if let serde_yaml::Value::Mapping(ref param) = *param_val {
                                if let Some(serde_yaml::Value::String(ref typed_by)) =
                                    param.get(serde_yaml::Value::String("typedBy".into()))
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
                        op.get(serde_yaml::Value::String("returnType".into()))
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
            if fm.inputs.as_ref().is_none_or(|v| v.is_empty()) {
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
            if fm.inputs.as_ref().is_none_or(|v| v.is_empty()) {
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
            if fm.entries.as_ref().is_none_or(|v| v.is_empty()) {
                findings.push(warning("W902", &file, "FMEASheet has no `entries` — add at least one failure mode row"));
            }
            // E923 / W928 (GH #132): per-row integrity the walker's explosion
            // (`walker::explode_fmea_entries`) cannot report itself.
            for (idx, row) in fm.entries.iter().flatten().enumerate() {
                findings.extend(fmea_row_findings(&file, idx + 1, row));
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
            let all_empty = fm.damage_table.as_ref().is_none_or(|v| v.is_empty())
                && fm.threat_table.as_ref().is_none_or(|v| v.is_empty())
                && fm.goal_table.as_ref().is_none_or(|v| v.is_empty())
                && fm.control_table.as_ref().is_none_or(|v| v.is_empty());
            if all_empty {
                findings.push(warning("W905", &file, "TARASheet has no rows in any section table — add damageTable, threatTable, goalTable, or controlTable entries"));
            }
        }

        // W600: PartDef and Part elements should have non-empty documentation.
        // REQ-TRS-VAL-017: suppressed on a Part usage whose typedBy: target
        // itself carries non-empty documentation -- a bare `part x :
        // SomeDocumentedPartDef;` isn't actually missing documentation, it's
        // one lookup away. A PartDef itself always fires regardless (it's
        // the type being referenced, nothing further to fall back to); a
        // Part usage whose typedBy: doesn't resolve, or resolves to an
        // equally-undocumented target, still fires exactly as before.
        // REQ-TRS-SYSMLV2-016: the typedBy: lookup goes through
        // resolve_scoped_ref, not the plain resolve_ref this check used at
        // first -- a SysMLv2-authored typedBy: is frequently a *relative*
        // name (e.g. "Services::Documented" written inside `package System`)
        // that only resolves once searched outward through the referencing
        // element's own enclosing-package scope chain; a hand-authored
        // typedBy: (always written fully qualified from the model root, by
        // this format's own convention) resolves identically either way.
        if matches!(
            fm.element_type,
            Some(ElementType::PartDef) | Some(ElementType::Part)
        ) && elem.doc.trim().is_empty()
        {
            let documented_via_type = matches!(fm.element_type, Some(ElementType::Part))
                && fm
                    .typed_by
                    .as_ref()
                    .map(yaml_strings)
                    .into_iter()
                    .flatten()
                    .filter_map(|tb| resolver.resolve_scoped_ref(elements, &elem.qualified_name, tb))
                    .any(|target| !target.doc.trim().is_empty());
            if !documented_via_type {
                findings.push(warning("W600", &file, "PartDef/Part has an empty documentation body"));
            }
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

    // E108: duplicate qualified name, any origin (hand-authored, FMEA/TARA row
    // explosion, SysMLv2 ingestion, or a stdio plugin's envelope) — the
    // origin-agnostic sibling of E101 (duplicate id). `Resolver::by_qname`
    // still silently keeps the last-inserted element on a collision
    // (unchanged here, a separate concern); this only adds the missing
    // diagnostic so the collision itself isn't invisible.
    {
        let mut seen_qnames: HashMap<&str, &str> = HashMap::new();
        for elem in elements {
            if let Some(prev_file) = seen_qnames.insert(elem.qualified_name.as_str(), elem.file_path.as_str()) {
                findings.push(error(
                    "E108",
                    &elem.file_path,
                    &format!(
                        "duplicate qualified name '{}' (first seen in {})",
                        elem.qualified_name, prev_file
                    ),
                ));
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
    // REQ-TRS-PLANITEM-002 — children[parent_id] = ids of PlanningItems naming it
    // via `parent:`. Keyed by the parent's stable id (PlanningItem is always
    // id-identified, so unlike derived_children/verified_by there is no
    // id-else-qname fallback needed here).
    let mut planning_children: HashMap<String, Vec<String>> = HashMap::new();
    // REQ-TRS-SYSMLV2-004 / ADR-SYS-PLUGIN-002 / ADR-SYS-ANNOTATE-001 —
    // side-channel provenance sets: which qnames were actually synthesized by
    // SysMLv2 ingestion vs. a stdio plugin vs. annotated-source scanning,
    // consulted by `Resolver::is_verify_target` so the E104 widening only
    // ever applies to real synthesized targets of the matching origin, never
    // to hand-authored native elements of the same kind. See
    // `sysmlv2::synthesized_qnames`'s, `plugins::synthesized_qnames`'s, and
    // `annotations::synthesized_qnames`'s doc comments.
    let sysmlv2_qnames = crate::sysmlv2::synthesized_qnames(elements);
    let plugin_qnames = crate::plugins::synthesized_qnames(elements);
    let annotation_qnames = crate::annotations::synthesized_qnames(elements);

    for elem in elements {
        let fm = &elem.frontmatter;

        // verifies: cross-reference check
        if let Some(ref vs) = fm.verifies {
            for (vi, v) in vs.iter().enumerate() {
                // REQ-TRS-LINKTYPE-006 — an entry contributed by a type extending
                // `verifies` may relax E104 or stay out of `verifiedBy`.
                let e104_relaxed = link_prov.relaxed(&elem.qualified_name, crate::link_types::BaseLink::Verifies, vi, "E104");
                let indexed = link_prov.in_reverse_index(&elem.qualified_name, crate::link_types::BaseLink::Verifies, vi);
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
                        // E104: target must be a native Requirement, or a *bona fide*
                        // SysMLv2-synthesized (REQ-TRS-SYSMLV2-004), plugin-synthesized
                        // (ADR-SYS-PLUGIN-002), or annotation-synthesized
                        // (ADR-SYS-ANNOTATE-001) element of one of the fixed mapped kinds —
                        // gated on `sysmlv2_qnames`/`plugin_qnames`/`annotation_qnames`, not
                        // kind alone, so a hand-authored native Part/etc. is never rescued by
                        // this widening.
                        if !e104_relaxed
                            && !Resolver::is_verify_target(target, &sysmlv2_qnames, &plugin_qnames, &annotation_qnames)
                        {
                            findings.push(error(
                                "E104",
                                &elem.file_path,
                                &format!(
                                    "'{}' does not resolve to a native Requirement{}",
                                    v,
                                    link_prov.via_suffix(&elem.qualified_name, crate::link_types::BaseLink::Verifies, vi)
                                ),
                            ));
                        } else if !indexed {
                            // `coverage = false` (REQ-TRS-LINKTYPE-006): checked, not credited.
                        } else if let Some(ref tc_id) = elem.frontmatter.id {
                            // Build reverse index — keyed by the target's stable id when
                            // present, else its qualified name (REQ-TRS-SYSMLV2-004: a
                            // SysMLv2-mapped target has no id, matching the id-else-qname
                            // convention `refined_by`/`allocated_from`/etc. already use).
                            //
                            // The verifying element's own label is deliberately NOT given the
                            // same id-else-qname fallback: only a `verifies:`-carrying element
                            // that itself has a stable id (in practice, always a native
                            // TestCase — the only source type REQ-TRS-SYSMLV2-004 is about)
                            // is recorded here, exactly as before this feature touched this
                            // code. Falling back to qname on this side previously let ANY
                            // `verifies:`-carrying, id-less element in (including a SysMLv2
                            // `RequirementUsage`'s own `verify:` from REQ-TRS-SYSMLV2-003),
                            // polluting `verified_by` for the server's `/api/validation`,
                            // `syscribe export`/`export-html`, `query`, the LSP CodeLens count,
                            // the Rhai `verified_by` property, and — worst — the MCP `evidence`
                            // tool's verification-chain output, none of which expect (or
                            // filter for) a non-TestCase entry here.
                            let target_key = target
                                .frontmatter
                                .id
                                .clone()
                                .unwrap_or_else(|| target.qualified_name.clone());
                            let list = verified_by.entry(target_key).or_default();
                            // A contributed entry duplicating a verifier already
                            // listed (a built-in `verifies:` to the same target) is
                            // not listed twice (REQ-TRS-LINKTYPE-006).
                            let dup = link_prov.is_contributed(&elem.qualified_name, crate::link_types::BaseLink::Verifies, vi)
                                && list.contains(tc_id);
                            if !dup {
                                list.push(tc_id.clone());
                            }
                        }
                    }
                }
            }
        }

        // derivedFrom: cross-reference check. A Configuration's `derivedFrom:`
        // is configuration inheritance (§9.8), not a requirement derivation — it
        // is checked by `configuration_inheritance_findings` (E215/E234–E237)
        // and never enters the `derivedChildren` index.
        let is_configuration = matches!(fm.element_type, Some(ElementType::Configuration));
        if let (Some(dfs), false) = (fm.derived_from.as_ref(), is_configuration) {
            for (di, df) in dfs.iter().enumerate() {
                // REQ-TRS-LINKTYPE-006 — per-entry E105 relaxation / coverage.
                let e105_relaxed = link_prov.relaxed(&elem.qualified_name, crate::link_types::BaseLink::DerivedFrom, di, "E105");
                let indexed = link_prov.in_reverse_index(&elem.qualified_name, crate::link_types::BaseLink::DerivedFrom, di);
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
                        if !e105_relaxed && !Resolver::is_native_requirement(target) {
                            findings.push(error(
                                "E105",
                                &elem.file_path,
                                &format!(
                                    "'{}' does not resolve to a native Requirement{}",
                                    df,
                                    link_prov.via_suffix(&elem.qualified_name, crate::link_types::BaseLink::DerivedFrom, di)
                                ),
                            ));
                        } else if !indexed {
                            // `coverage = false`: not a parent/child edge for coverage.
                        } else if let Some(ref parent_id) = target.frontmatter.id {
                            if let Some(ref child_id) = elem.frontmatter.id {
                                // As for verifiedBy: a contributed entry duplicating
                                // an existing child edge is not listed twice.
                                let list = derived_children.entry(parent_id.clone()).or_default();
                                let dup = link_prov.is_contributed(
                                    &elem.qualified_name,
                                    crate::link_types::BaseLink::DerivedFrom,
                                    di,
                                ) && list.contains(child_id);
                                if !dup {
                                    list.push(child_id.clone());
                                }
                            }
                        }
                    }
                }
            }
        }

        // parent: cross-reference check (REQ-TRS-PLANITEM-002). Only meaningful on
        // a PlanningItem; a `parent:` value on any other type is simply an
        // unrecognized field for that type (rejected earlier, E003-style) and never
        // reaches here. Cycle detection is a separate, graph-wide pass below (E712).
        if matches!(fm.element_type, Some(ElementType::PlanningItem)) {
            if let Some(ref p) = fm.parent {
                match resolver.resolve_ref(elements, p) {
                    None => findings.push(error(
                        "E710",
                        &elem.file_path,
                        &format!("unresolved PlanningItem parent reference '{}'", p),
                    )),
                    Some(target) => {
                        if !Resolver::is_planning_item(target) {
                            findings.push(error(
                                "E711",
                                &elem.file_path,
                                &format!("PlanningItem `parent` '{}' does not resolve to a PlanningItem", p),
                            ));
                        } else if let Some(ref parent_id) = target.frontmatter.id {
                            if let Some(ref child_id) = fm.id {
                                planning_children
                                    .entry(parent_id.clone())
                                    .or_default()
                                    .push(child_id.clone());
                            }
                        }
                    }
                }
            }
        }

        // achieves: cross-reference check (REQ-TRS-PLANITEM-003). Empirically
        // confirmed (not assumed — cf. the SysMLv2 satisfy/verify asymmetry) that
        // no generic cross-reference-resolution infrastructure catches a dangling
        // `achieves:` target on its own; unlike verifies/derivedFrom (E102/E103)
        // it needed its own explicit dangling check here. Deliberately separate
        // from `satisfies:`'s machinery (E312/W300 stay scoped to architecture
        // satisfies, never see `achieves:`).
        if matches!(fm.element_type, Some(ElementType::PlanningItem)) {
            if let Some(ref ach) = fm.achieves {
                for a in ach {
                    match resolver.resolve_ref(elements, a) {
                        None => findings.push(error(
                            "E714",
                            &elem.file_path,
                            &format!("unresolved PlanningItem achieves reference '{}'", a),
                        )),
                        Some(target) if !Resolver::is_native_requirement(target) => {
                            findings.push(error(
                                "E715",
                                &elem.file_path,
                                &format!("PlanningItem `achieves` '{}' does not resolve to a native Requirement", a),
                            ));
                        }
                        Some(_) => {}
                    }
                }
            }
        }

        // blockedBy: cross-reference check (REQ-TRS-PLANITEM-007). Permissive,
        // unrestricted by kind — same posture as evidence.ref: (REQ-TRS-PLANITEM-005):
        // an undecided ADR or any other unmet model dependency is as legitimate a
        // blocker as another PlanningItem, so there is no "must resolve to a
        // PlanningItem" analogue of E715 here. Cycle detection is a separate,
        // graph-wide pass below (E721), mirroring parent:'s E712.
        if matches!(fm.element_type, Some(ElementType::PlanningItem)) {
            if let Some(ref bs) = fm.blocked_by {
                for b in bs {
                    if resolver.resolve_ref(elements, b).is_none() {
                        findings.push(error(
                            "E720",
                            &elem.file_path,
                            &format!("unresolved PlanningItem blockedBy reference '{}'", b),
                        ));
                    }
                }
                // W308: a non-empty blockedBy: on an item whose status isn't
                // `blocked` is likely stale (the blocker was resolved and
                // `status:` was never updated) — a warning, not an error, since
                // the two fields are independently author-maintained and neither
                // is computed from the other.
                if !bs.is_empty() && fm.status.as_deref() != Some("blocked") {
                    findings.push(warning(
                        "W308",
                        &elem.file_path,
                        &format!(
                            "PlanningItem has a non-empty `blockedBy:` but `status` is '{}', not 'blocked' — likely stale",
                            fm.status.as_deref().unwrap_or("(unset)")
                        ),
                    ));
                }
            }
        }

        // assignedTo: format + roster check (REQ-TRS-PLANITEM-008). Not a
        // cross-reference (users are declared strings, not model elements) —
        // checked against `config.users` instead of the resolver.
        //
        // E723 (format) is always checked, independent of whether any roster
        // is configured — a Unix-style username shape (REQ-TRS-PLANITEM-008)
        // is an intrinsic constraint on the field, like an id pattern, not a
        // roster-membership rule. E722 (declared-in-roster) stays dormant,
        // like every other config-gated check, when `[users]` is empty; when
        // it's configured, an already-format-valid value is checked against
        // it — a malformed value is only reported once (E723), not doubled up
        // with a redundant "not declared" finding for the same defect.
        if matches!(fm.element_type, Some(ElementType::PlanningItem)) {
            if let Some(ref who) = fm.assigned_to {
                if !crate::resolver::is_valid_username(who) {
                    findings.push(error(
                        "E723",
                        &elem.file_path,
                        &format!(
                            "PlanningItem assignedTo '{}' is not a valid username (expected: lowercase, starting with a letter or underscore, then lowercase letters/digits/underscore/hyphen, max 32 chars)",
                            who
                        ),
                    ));
                } else if !config.users.is_empty() && !config.users.contains_key(who) {
                    findings.push(error(
                        "E722",
                        &elem.file_path,
                        &format!(
                            "PlanningItem assignedTo '{}' is not a declared user — add it to [users] in .syscribe.toml",
                            who
                        ),
                    ));
                }
            }
        }

        // evidence: cross-reference / path-existence check (REQ-TRS-PLANITEM-005).
        // Each entry is duck-typed — recognised by which key it carries, not a
        // `type:` tag, the same idiom the Allocation `features:`-list convention
        // already establishes:
        //   - `ref:` — any resolvable element, unrestricted by kind (no
        //     Resolver::is_native_requirement-style gate — ADR-SYS-PLANITEM-001
        //     Decision 3 is explicit that this is NOT a fixed allowed-kind list).
        //   - `path:` — resolved exactly like `implementedBy:` (reusing
        //     config.classify_source, not reimplementing it): a local path is
        //     checked to exist, a remote URI is accepted as external.
        // Either form's own `rationale:` (a non-empty string) waives that one
        // entry's check — mirroring ffi_rationale's co-located waiver pattern.
        // Every other entry in the same list is still checked normally (no
        // blanket suppression). This task authors/validates evidence: in
        // isolation only; the "leaf needs evidence" rule is REQ-TRS-PLANITEM-006.
        if matches!(fm.element_type, Some(ElementType::PlanningItem)) {
            if let Some(ref ev) = fm.evidence {
                for entry in ev {
                    let Some(m) = entry.as_mapping() else { continue };
                    let waived = evidence_entry_is_waived(entry);

                    if let Some(r) = yaml_field(m, "ref").and_then(|v| v.as_str()) {
                        if !waived && !evidence_ref_resolves(elements, &resolver, r) {
                            findings.push(error(
                                "E716",
                                &elem.file_path,
                                &format!("PlanningItem evidence `ref` '{}' does not resolve to any model element", r),
                            ));
                        }
                    }
                    if let Some(p) = yaml_field(m, "path").and_then(|v| v.as_str()) {
                        if !waived && !evidence_path_resolves(config, p) {
                            findings.push(error(
                                "E717",
                                &elem.file_path,
                                &format!("PlanningItem evidence `path` '{}' does not exist on disk", p),
                            ));
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
                        map.get(serde_yaml::Value::String("scenario".into()))
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

    // E719: a leaf PlanningItem (REQ-TRS-PLANITEM-002: empty computed
    // `children`) at `status: done` must have at least one non-waived,
    // resolving `evidence:` entry (REQ-TRS-PLANITEM-006). Reuses task #15's
    // `planning_children` (now fully populated — this runs after the loop
    // that built it) for leaf detection and task #18's
    // `evidence_entry_target_resolves`/`evidence_entry_is_waived` primitives
    // for per-entry resolution, rather than re-deriving either. Harder
    // severity than the analogous Requirement rule (`W300`, a warning):
    // claiming `done` with zero resolvable evidence is a correctness defect,
    // not a time-bound gap. A non-leaf (has children) is not constrained by
    // this rule at all, regardless of its own status/evidence — its
    // completion is a function of its children, not its own evidence list.
    // A waived-only list still fails this: a rationale excuses one entry's
    // *check*, it does not manufacture a passing entry.
    for elem in elements {
        if !matches!(elem.frontmatter.element_type, Some(ElementType::PlanningItem)) {
            continue;
        }
        if elem.frontmatter.status.as_deref() != Some("done") {
            continue;
        }
        let pi_id = elem.frontmatter.id.as_deref().unwrap_or("");
        let is_leaf = planning_children.get(pi_id).is_none_or(|c| c.is_empty());
        if !is_leaf {
            continue;
        }
        let has_resolving_evidence = elem.frontmatter.evidence.as_ref().is_some_and(|ev| {
            ev.iter().any(|entry| {
                !evidence_entry_is_waived(entry)
                    && evidence_entry_target_resolves(entry, elements, &resolver, config)
            })
        });
        if !has_resolving_evidence {
            findings.push(error(
                "E719",
                &elem.file_path,
                "leaf PlanningItem is `status: done` but has no non-waived, resolving `evidence:` entry",
            ));
        }
    }

    // W310 (issue #114): PlanningItem-scoped completion check. W002/W003/W305
    // already warn from the Requirement's own file when it lacks verification
    // coverage, but nothing ties "this specific PlanningItem you're about to
    // mark done" to "here specifically are the achieves: requirements that
    // aren't actually backed by evidence yet" — the moment that distinction
    // matters most. Deliberately reuses the *exact* bar W002/W305 already
    // apply per requirement kind (any active TestCase for a leaf requirement;
    // an active *integration-level* L3/L4/L5 TestCase for a parent one, since
    // a parent's leaf descendants carrying coverage doesn't change what W305
    // requires directly on the parent itself) so this never disagrees with
    // what `validate` already says about the achieved Requirement on its own
    // file — it just re-surfaces the same fact scoped to the PlanningItem,
    // on the PlanningItem's own file (distinct finding, not a duplicate of
    // W002/W305: different code, different file). Applies at any tree
    // position, not just leaves (unlike E719): an achieves: claim's
    // verification state doesn't depend on whether this item has children.
    // A dangling (E714) or wrong-kind (E715) achieves: target is skipped
    // here — those are reported once, already, by their own checks above.
    for elem in elements {
        if !matches!(elem.frontmatter.element_type, Some(ElementType::PlanningItem)) {
            continue;
        }
        if elem.frontmatter.status.as_deref() != Some("done") {
            continue;
        }
        let pi_id = elem.frontmatter.id.as_deref().unwrap_or("");
        let Some(ach) = elem.frontmatter.achieves.as_ref() else { continue };
        for a in ach {
            let Some(target) = resolver.resolve_ref(elements, a) else { continue };
            if !Resolver::is_native_requirement(target) {
                continue;
            }
            let req_id = target.frontmatter.id.as_deref().unwrap_or(a.as_str());
            let active_tcs: Vec<&String> = verified_by
                .get(req_id)
                .map(|tcs| {
                    tcs.iter()
                        .filter(|tc_id| {
                            resolver
                                .get_by_id(elements, tc_id)
                                .and_then(|e| e.frontmatter.status.as_deref())
                                == Some("active")
                        })
                        .collect()
                })
                .unwrap_or_default();
            let is_parent = derived_children.get(req_id).is_some_and(|v| !v.is_empty());
            let covered = if is_parent {
                active_tcs.iter().any(|tc_id| {
                    resolver
                        .get_by_id(elements, tc_id)
                        .and_then(|e| e.frontmatter.test_level.as_deref())
                        .is_some_and(|lvl| matches!(lvl, "L3" | "L4" | "L5"))
                })
            } else {
                !active_tcs.is_empty()
            };
            if !covered {
                let need = if is_parent {
                    "active system-integration TestCase (testLevel L3, L4, or L5)"
                } else {
                    "active TestCase"
                };
                findings.push(warning(
                    "W310",
                    &elem.file_path,
                    &format!(
                        "PlanningItem '{}' is 'done', but achieves '{}' which has no {} (see also W002/W305 on the requirement itself)",
                        pi_id, req_id, need
                    ),
                ));
            }
        }
    }

    // W311 (issue #115): claim-overlap check. Two PlanningItems that are both
    // "active" (`status: in_progress`, or explicitly claimed via `claimedBy:`)
    // and share either an `achieves:` Requirement or an `evidence[].path`
    // resolving to the same repo-relative path are very likely two agents
    // about to (or already) step on each other's work -- the two concrete
    // overlap shapes that come up running several LLM agents against one
    // model concurrently. Advisory only (a warning, not a filesystem lock):
    // fires once per overlapping pair, per overlap kind, attached to the
    // lexically-first item's file (by stable id) so re-running `validate`
    // doesn't double up the same pair from each side.
    {
        let is_active = |e: &&RawElement| -> bool {
            matches!(e.frontmatter.element_type, Some(ElementType::PlanningItem))
                && (e.frontmatter.status.as_deref() == Some("in_progress")
                    || e.frontmatter.claimed_by.as_deref().is_some_and(|s| !s.trim().is_empty()))
        };
        let active_items: Vec<&RawElement> = elements.iter().filter(is_active).collect();

        let evidence_paths = |e: &RawElement| -> HashSet<String> {
            e.frontmatter
                .evidence
                .as_ref()
                .map(|ev| {
                    ev.iter()
                        .filter_map(|entry| {
                            let serde_yaml::Value::Mapping(m) = entry else { return None };
                            yaml_field(m, "path").and_then(|v| v.as_str()).map(String::from)
                        })
                        .collect()
                })
                .unwrap_or_default()
        };

        for i in 0..active_items.len() {
            for j in (i + 1)..active_items.len() {
                let (a, b) = (active_items[i], active_items[j]);
                let a_id = a.frontmatter.id.as_deref().unwrap_or("");
                let b_id = b.frontmatter.id.as_deref().unwrap_or("");
                if a_id.is_empty() || b_id.is_empty() || a_id == b_id {
                    continue;
                }
                let (first, second, first_id, second_id) =
                    if a_id <= b_id { (a, b, a_id, b_id) } else { (b, a, b_id, a_id) };

                let a_ach: HashSet<&str> =
                    first.frontmatter.achieves.as_deref().unwrap_or(&[]).iter().map(String::as_str).collect();
                let b_ach: HashSet<&str> =
                    second.frontmatter.achieves.as_deref().unwrap_or(&[]).iter().map(String::as_str).collect();
                let mut shared_achieves: Vec<&str> = a_ach.intersection(&b_ach).copied().collect();
                shared_achieves.sort_unstable();
                for shared in shared_achieves {
                    findings.push(warning(
                        "W311",
                        &first.file_path,
                        &format!(
                            "PlanningItem '{first_id}' and '{second_id}' are both active (in_progress/claimed) and overlap by achieves '{shared}' — possible duplicate work"
                        ),
                    ));
                }

                let a_paths = evidence_paths(first);
                let b_paths = evidence_paths(second);
                let mut shared_paths: Vec<&String> = a_paths.intersection(&b_paths).collect();
                shared_paths.sort_unstable();
                for shared in shared_paths {
                    findings.push(warning(
                        "W311",
                        &first.file_path,
                        &format!(
                            "PlanningItem '{first_id}' and '{second_id}' are both active (in_progress/claimed) and overlap by evidence.path '{shared}' — possible duplicate work"
                        ),
                    ));
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

        let is_parent = derived_children.get(req_id).is_some_and(|v| !v.is_empty());
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
                                for (si, s) in sat.iter().enumerate() {
                                    let kids = by_target.entry(s.as_str()).or_default();
                                    // A contributed entry duplicating the child's own
                                    // satisfies of the same element is one channel,
                                    // not two (REQ-TRS-LINKTYPE-006).
                                    if link_prov.is_contributed(&c.qualified_name, crate::link_types::BaseLink::Satisfies, si)
                                        && kids.contains(&cid)
                                    {
                                        continue;
                                    }
                                    kids.push(cid);
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
                    .is_some_and(|lvl| matches!(lvl, "L3" | "L4" | "L5"))
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

        // W005: orphan (no upstream link and no derivedChildren). A requirement
        // derived from a SafetyGoal/CybersecurityGoal is traced upstream just as
        // much as one with `derivedFrom:` (GH #151) — the goal is its parent.
        let non_empty = |s: &Option<String>| s.as_deref().is_some_and(|v| !v.trim().is_empty());
        let has_parent = elem.frontmatter.derived_from.as_ref().is_some_and(|v| !v.is_empty())
            || non_empty(&elem.frontmatter.derived_from_safety_goal)
            || non_empty(&elem.frontmatter.derived_from_cybersecurity_goal);
        let has_children = derived_children.get(req_id).is_some_and(|v| !v.is_empty());
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
        let is_conduit = |e: &RawElement| matches!(e.frontmatter.element_type, Some(ElementType::Conduit));
        // E925 / E926 — documented value sets (§13.2–§13.4; REQ-TRS-VAL-018, GH #136).
        for e in elements.iter().filter(|e| is_zone(e) || is_conduit(e) || is_part(e)) {
            let fm = &e.frontmatter;
            for (v, label) in [(fm.target_sl, "targetSL"), (fm.achieved_sl, "achievedSL")] {
                if let Some(sl) = v {
                    if !(1..=4).contains(&sl) {
                        findings.push(error("E925", &e.file_path, &format!(
                            "{} {} is outside the IEC 62443 Security Level range 1–4", label, sl)));
                    }
                }
            }
            if is_zone(e) || is_conduit(e) {
                if let Some(s) = fm.status.as_deref() {
                    if !["draft", "review", "approved", "deprecated"].contains(&s) {
                        let kind = if is_zone(e) { "Zone" } else { "Conduit" };
                        findings.push(error("E926", &e.file_path, &format!(
                            "{}.status '{}' must be draft, review, approved, or deprecated", kind, s)));
                    }
                }
            }
        }
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
            // W511: peer work tree has drifted from its configured ref (REQ-TRS-TYPE-022).
            // Gate it to a hard failure in CI with `--deny W511`.
            if repo.ref_state == crate::config::RefState::Drift {
                let want = repo.config.git_ref.as_deref().unwrap_or("");
                let detail = repo
                    .head_sha
                    .as_deref()
                    .map(|h| format!(" (HEAD {} ≠ {})", &h[..h.len().min(8)], want))
                    .unwrap_or_default();
                findings.push(warning(
                    "W511",
                    cfg_file,
                    &format!(
                        "repo '{}' is not at its configured ref '{}'{} — run `repos sync {}`",
                        repo.alias, want, detail, repo.alias
                    ),
                ));
            }
            // W512: the parent's submodule gitlink for `path` disagrees with the
            // commit the `ref:` resolves to (REQ-TRS-TYPE-023). Only when `path`
            // is a submodule and both commits are known. Gate with `--deny W512`.
            if let (Some(gitlink), Some(ref_commit)) = (&repo.gitlink_sha, &repo.ref_commit) {
                if gitlink != ref_commit {
                    let short = |s: &str| s[..s.len().min(8)].to_string();
                    findings.push(warning(
                        "W512",
                        cfg_file,
                        &format!(
                            "repo '{}' ref '{}' resolves to {} but the git submodule gitlink for '{}' pins {} — `.syscribe.toml` disagrees with `.gitmodules`",
                            repo.alias,
                            repo.config.git_ref.as_deref().unwrap_or(""),
                            short(ref_commit),
                            repo.config.path,
                            short(gitlink),
                        ),
                    ));
                }
            }
        }

        // E515 (peer vs peer, GH #138): a stable ID exported by two different
        // peer repos. Two aliases naming the same peer model root are one repo.
        {
            let loaded: Vec<&crate::config::LoadedRepo> = config.repos.iter().filter(|r| r.exists).collect();
            let canon = |p: &std::path::Path| std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf());
            for (i, a) in loaded.iter().enumerate() {
                for b in loaded.iter().skip(i + 1) {
                    if canon(&a.model_root) == canon(&b.model_root) {
                        continue;
                    }
                    let mut dup: Vec<&str> = a
                        .stable_ids
                        .iter()
                        .map(String::as_str)
                        .filter(|id| b.stable_ids.contains(*id))
                        .collect();
                    dup.sort_unstable();
                    for id in dup {
                        findings.push(error(
                            "E515",
                            cfg_file,
                            &format!(
                                "stable ID '{}' is exported by both repo '{}' and repo '{}' — the id namespace is global across the composition",
                                id, a.alias, b.alias
                            ),
                        ));
                    }
                }
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
    // REQ-TRS-SYSMLV2-017: the top-level supertype:/typedBy: lookup and the nested
    // typedBy: lookup (features/connections/performs/ports) go through
    // resolve_scoped_ref, not the plain resolve_ref this check used at first — the
    // same widening REQ-TRS-SYSMLV2-016 made for W600's suppression check, applied
    // to the other call site its own Scope bullet named as sharing the identical
    // root cause: a SysMLv2-authored, package-relative supertype:/typedBy: value
    // only resolves once searched outward through the referencing element's own
    // enclosing-package scope chain. exhibitsStates is left on plain resolve_ref —
    // it is never synthesized by SysMLv2 ingestion, so it is always written fully
    // qualified from the model root by this format's own hand-authored convention.
    {
        let mut referenced_defs: HashSet<String> = HashSet::new();
        for elem in elements.iter() {
            let fm = &elem.frontmatter;

            // Top-level supertype and typedBy
            for field in [fm.supertype.as_ref(), fm.typed_by.as_ref()].into_iter().flatten() {
                for s in yaml_strings(field) {
                    if let Some(target) = resolver.resolve_scoped_ref(elements, &elem.qualified_name, s) {
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
                collect_typed_by_refs(list, elements, &resolver, &elem.qualified_name, &mut referenced_defs);
            }
        }
        for elem in elements {
            if is_type_def(elem)
                && !referenced_defs.contains(&elem.qualified_name) {
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
    findings.extend(parameter_binding_findings(elements, config, &resolver));

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
            .map(|e| {
                // Only entries in the reverse index credit coverage: a `coverage =
                // false` extending verifies link is withheld (REQ-TRS-LINKTYPE-006).
                let credited: Vec<String> = e
                    .frontmatter
                    .verifies
                    .as_deref()
                    .unwrap_or_default()
                    .iter()
                    .enumerate()
                    .filter(|(vi, _)| {
                        link_prov.in_reverse_index(&e.qualified_name, crate::link_types::BaseLink::Verifies, *vi)
                    })
                    .map(|(_, v)| v.clone())
                    .collect();
                (parse_aw(e), credited)
            })
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
                let active = rexpr.as_ref().is_none_or(|e| e.eval(&selected));
                if !active {
                    continue;
                }
                let covered = tcs.iter().any(|(texpr, verifies)| {
                    let runs = texpr.as_ref().is_none_or(|e| e.eval(&selected));
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
            // E927 (REQ-TRS-FTA-002): FaultTreeEvent.ref must resolve to a model element
            if let Some(ref r) = fm.event_ref {
                if resolver.resolve_ref(elements, r).is_none() {
                    findings.push(error("E927", &elem.file_path,
                        &format!("FaultTreeEvent `ref` '{}' does not resolve to a known element", r)));
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
                || elem.frontmatter.id.as_ref().is_some_and(|id| he_referenced.contains(id));
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
                .is_some_and(|v| !v.is_empty());
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
                || elem.frontmatter.id.as_ref().is_some_and(|id| csg_implemented.contains(id));
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
                || elem.frontmatter.id.as_ref().is_some_and(|id| csg_derived_reqs.contains(id));
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
                || elem.frontmatter.id.as_ref().is_some_and(|id| asset_referenced.contains(id));
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
                    .is_some_and(|id| addressed.contains(id));
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
                || elem.frontmatter.id.as_ref().is_some_and(|id| sg_derived_reqs.contains(id));
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

    // Build reverse index: satisfied_reqs[req_qname_or_id] = list of satisfying element qnames.
    // Deliberately built from every element's `satisfies:`, not filtered to
    // Part/PartDef — feeds W300's leaf-assignment coverage count, which
    // must credit any endorsed satisfying shape (Part/PartDef; a behavioral
    // definition StateDef/ActionDef; or one of Connection/ConnectionDef,
    // Interface/InterfaceDef, Item/ItemDef, Attribute/AttributeDef,
    // Port/PortDef, Allocation, Flow/FlowDef) exactly like a Part/PartDef
    // satisfier (see the E313 comment below for the full rationale).
    let mut satisfied_reqs: HashMap<String, Vec<String>> = HashMap::new();
    // REQ-TRS-LINKTYPE-006 — the targets E312 ("parent in a satisfies list") is
    // checked against: every satisfies-like entry, credited or not, except those
    // of a type relaxing E312. Without extending types it equals the keys of
    // `satisfied_reqs`.
    let mut e312_satisfied: HashSet<String> = HashSet::new();
    // Targets held by at least one authored satisfies entry, and the link types
    // behind contributed ones — to name the type in E312 when only a
    // `links:` entry makes the requirement "appear in a satisfies list".
    let mut e312_authored: HashSet<String> = HashSet::new();
    let mut e312_via: HashMap<String, std::collections::BTreeSet<String>> = HashMap::new();
    for elem in elements {
        if let Some(ref sat) = elem.frontmatter.satisfies {
            for (si, s) in sat.iter().enumerate() {
                if let Some(target) = resolver.resolve_ref(elements, s) {
                    let qn = &elem.qualified_name;
                    let base = crate::link_types::BaseLink::Satisfies;
                    if !link_prov.relaxed(qn, base, si, "E312") {
                        e312_satisfied.insert(target.qualified_name.clone());
                        match link_prov.via(qn, base, si) {
                            Some(n) => {
                                e312_via.entry(target.qualified_name.clone()).or_default().insert(n.to_string());
                            }
                            None => {
                                e312_authored.insert(target.qualified_name.clone());
                            }
                        }
                    }
                    // `coverage = false` withholds the entry from the reverse index;
                    // a contributed entry duplicating one the element already holds
                    // is not counted twice in `satisfiedBy`.
                    if !link_prov.in_reverse_index(qn, base, si) {
                        continue;
                    }
                    let entry = satisfied_reqs.entry(target.qualified_name.clone()).or_default();
                    if link_prov.is_contributed(qn, base, si) && entry.contains(qn) {
                        continue;
                    }
                    entry.push(qn.clone());
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
            let is_parent = derived_children.get(req_id).is_some_and(|v| !v.is_empty());

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

        // E310: native Requirement with derivedFrom must have breakdownAdr.
        // REQ-TRS-LINKTYPE-006 — an element-level rule: relaxed only when *every*
        // derivedFrom-like entry comes from a type relaxing E310.
        let derived_len = fm.derived_from.as_ref().map_or(0, |v| v.len());
        if Resolver::is_native_requirement(elem)
            && derived_len > 0
                && fm.breakdown_adr.is_none()
                && !link_prov.all_relaxed(&elem.qualified_name, crate::link_types::BaseLink::DerivedFrom, derived_len, "E310") {
                    findings.push(error(
                        "E310",
                        &elem.file_path,
                        &format!(
                            "Requirement has `derivedFrom` but no `breakdownAdr`{}",
                            link_prov.via_suffix_all(&elem.qualified_name, crate::link_types::BaseLink::DerivedFrom, derived_len)
                        ),
                    ));
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
                    // REQ-TRS-LINKTYPE-006 — element-level, like E310 above.
                    let w303_relaxed = link_prov.all_relaxed(
                        &elem.qualified_name,
                        crate::link_types::BaseLink::DerivedFrom,
                        derived_len,
                        "W303",
                    );
                    if adr_status == "proposed" && APPROVED_OR_HIGHER.contains(&req_status) && !w303_relaxed {
                        findings.push(warning(
                            "W303",
                            &elem.file_path,
                            &format!(
                                "`breakdownAdr` '{}' is still `proposed` but Requirement has status '{}'{}",
                                adr_ref,
                                req_status,
                                link_prov.via_suffix_all(&elem.qualified_name, crate::link_types::BaseLink::DerivedFrom, derived_len)
                            ),
                        ));
                    }
                }
            }
        }

        // E312: a parent requirement (has derivedChildren) must not appear in any satisfies list
        if Resolver::is_native_requirement(elem) {
            let req_id = fm.id.as_deref().unwrap_or("");
            let is_parent = derived_children.get(req_id).is_some_and(|c| !c.is_empty());
            if is_parent {
                let qn = &elem.qualified_name;
                let in_satisfies = e312_satisfied.contains(qn.as_str())
                    || (!req_id.is_empty() && e312_satisfied.contains(req_id));
                if in_satisfies {
                    let via = match e312_via.get(qn.as_str()) {
                        Some(names) if !e312_authored.contains(qn.as_str()) => {
                            let list: Vec<String> = names.iter().map(|n| format!("links.{n}")).collect();
                            format!(" (via {})", list.join(", "))
                        }
                        _ => String::new(),
                    };
                    findings.push(error(
                        "E312",
                        &elem.file_path,
                        &format!("parent Requirement '{}' appears in a `satisfies:` list — only leaf requirements may be assigned{}", req_id, via),
                    ));
                }
            }
        }

        // E313: satisfies domain mismatch — element domain vs requirement reqDomain.
        // Deliberately not gated on `elem`'s type: `satisfies:` isn't restricted to
        // Part/PartDef (§12.5's own wording only names them as the common case).
        // A behavioral definition (StateDef/ActionDef) satisfying a requirement
        // is an equally legitimate claim — the same reasoning `refines:`'s E316
        // already made explicit for behavioral definitions vs UseCaseDef
        // (REQ-TRS-MG-010). Extended for consistency to every other kind
        // `Resolver::is_verify_target`/E104 already treats as
        // requirement/architecture-shaped for SysMLv2/plugin-synthesized
        // elements — Connection/ConnectionDef, Interface/InterfaceDef,
        // Item/ItemDef, Attribute/AttributeDef, Port/PortDef, Allocation — plus
        // Flow/FlowDef (not in E104's list, but the same "artifact that can
        // fulfil a requirement" reasoning applies to item flow). See
        // `satisfies_shape_tests`.
        if let Some(ref sat) = fm.satisfies {
            let elem_domain = fm.domain.as_deref().unwrap_or("system");
            for (si, s) in sat.iter().enumerate() {
                // REQ-TRS-LINKTYPE-006 — E313 relaxed for this entry only.
                if link_prov.relaxed(&elem.qualified_name, crate::link_types::BaseLink::Satisfies, si, "E313") {
                    continue;
                }
                if let Some(target) = resolver.resolve_ref(elements, s) {
                    if Resolver::is_native_requirement(target) {
                        let req_domain = target.frontmatter.req_domain.as_deref().unwrap_or("system");
                        if elem_domain != "system" && req_domain != "system" && elem_domain != req_domain {
                            findings.push(error(
                                "E313",
                                &elem.file_path,
                                &format!(
                                    "`satisfies` domain mismatch: element has `domain: {}` but requirement '{}' has `reqDomain: {}`{}",
                                    elem_domain,
                                    s,
                                    req_domain,
                                    link_prov.via_suffix(&elem.qualified_name, crate::link_types::BaseLink::Satisfies, si)
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
            for (di, df) in dfs.iter().enumerate() {
                let via = link_prov.via_suffix(&elem.qualified_name, crate::link_types::BaseLink::DerivedFrom, di);
                if let Some(parent) = resolver.resolve_ref(elements, df) {
                    let pfm = &parent.frontmatter;
                    let child_has = fm.asil_level.is_some() || fm.sil_level.is_some();
                    let src_has   = pfm.asil_level.is_some() || pfm.sil_level.is_some();
                    if src_has && !child_has {
                        findings.push(error(
                            "E842",
                            &elem.file_path,
                            &format!(
                                "parent element '{}' carries an integrity level — derived element must also set asilLevel or silLevel{}",
                                df, via
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
                                "integrity level is lower than parent '{}' — add `breakdownAdr` to justify the ASIL/SIL decomposition{}",
                                df, via
                            ),
                        ));
                    }
                }
            }
        }

        // E843 / W808: satisfies — satisfying element must inherit integrity level.
        // Same deliberately type-agnostic posture as E313/W300 above — any
        // endorsed satisfying shape is held to the same rule as a Part/PartDef
        // would be.
        if let Some(ref sat) = fm.satisfies {
            for (si, s) in sat.iter().enumerate() {
                let via = link_prov.via_suffix(&elem.qualified_name, crate::link_types::BaseLink::Satisfies, si);
                if let Some(target) = resolver.resolve_ref(elements, s) {
                    let tfm = &target.frontmatter;
                    let child_has = fm.asil_level.is_some() || fm.sil_level.is_some();
                    let src_has   = tfm.asil_level.is_some() || tfm.sil_level.is_some();
                    if src_has && !child_has {
                        findings.push(error(
                            "E843",
                            &elem.file_path,
                            &format!(
                                "requirement '{}' carries an integrity level — satisfying element must also set asilLevel or silLevel{}",
                                s, via
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
                                "integrity level is lower than satisfied requirement '{}' — add `breakdownAdr` to justify the ASIL/SIL decomposition{}",
                                s, via
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

    // E314: deployment packages must have at least one allocation to a hardware
    // element. Every form of the §12.9 unified edge set counts (GH #131): a
    // top-level or `features:`-form `Allocation` element, an `allocatedTo:` on
    // the package itself, and a legacy authored `allocatedFrom:` on the target.
    {
        // Sources (qnames) with at least one allocation edge to a hardware element.
        let mut hw_alloc_targets: HashSet<String> = HashSet::new();
        for (from, to) in allocation_edges(elements, &resolver) {
            if resolver
                .get(elements, &to)
                .is_some_and(|t| t.frontmatter.domain.as_deref() == Some("hardware"))
            {
                hw_alloc_targets.insert(from);
            }
        }
        for elem in elements {
            if elem.frontmatter.is_deployment_package == Some(true)
                && !hw_alloc_targets.contains(&elem.qualified_name) {
                    findings.push(error(
                        "E314",
                        &elem.file_path,
                        &format!(
                            "`isDeploymentPackage: true` element '{}' has no allocation to a hardware element (an `Allocation` element, or `allocatedTo:` on the element)",
                            elem.qualified_name
                        ),
                    ));
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
                    .is_some_and(|s| !s.trim().is_empty())
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

            // Allocation edges source -> target from the §12.9 unified edge set
            // (the same extractor E314, MG041/MG081 and `matrix --allocations`
            // use — GH #131), inverted into target qname -> { source qnames }. A
            // standalone `Allocation` element contributes its allocatedFrom ->
            // allocatedTo edge, never itself as an endpoint.
            let mut targets: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
            for (from, to) in allocation_edges(elements, &resolver) {
                targets.entry(to).or_default().insert(from);
            }

            for (target_qn, sources) in &targets {
                if sources.len() < 2 {
                    continue; // a target with <2 sources cannot host a sharing
                }
                let target_elem = resolver.get(elements, target_qn);
                let target_has_ffi = target_elem.is_some_and(has_ffi_arg);

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

    // W300: leaf requirement coverage by satisfying architecture elements. `W301`
    // (more than one satisfier) is RETIRED (GH #121): a leaf may be jointly
    // satisfied by several elements — structural, behavioural or a mix — and
    // leaf/parent is decided by `derivedChildren` alone, never by satisfier count.
    for elem in elements {
        if !Resolver::is_native_requirement(elem) {
            continue;
        }
        let req_id = elem.frontmatter.id.as_deref().unwrap_or("");
        let is_parent = derived_children.get(req_id).is_some_and(|c| !c.is_empty());
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
        // Configuration inheritance cycles are E236 (§9.8), not E017.
        let configuration_qnames: HashSet<&str> = elements
            .iter()
            .filter(|e| matches!(e.frontmatter.element_type, Some(ElementType::Configuration)))
            .map(|e| e.qualified_name.as_str())
            .collect();

        let checks: &[(&str, EdgeKind, &str)] = &[
            ("E016", EdgeKind::Supertype, "supertype cycle detected"),
            ("E017", EdgeKind::DerivedFrom, "derivedFrom cycle detected"),
            ("E018", EdgeKind::Subsets, "subsets cycle detected"),
            // E107 (GH #25): typedBy was previously excluded — a usage typed by
            // itself (length-1 cycle) or a typedBy cycle was silently accepted.
            ("E107", EdgeKind::TypedBy, "typedBy cycle detected (a usage cannot be typed by itself, directly or transitively)"),
            // E712 (REQ-TRS-PLANITEM-002): a PlanningItem `parent:` chain that
            // cycles back to itself. Same posture as E017 (derivedFrom cycle) —
            // reported gracefully via toposort, never a panic or infinite loop.
            ("E712", EdgeKind::PlanningParent, "PlanningItem parent cycle detected"),
            // E721 (REQ-TRS-PLANITEM-007): a PlanningItem blockedBy chain that
            // cycles back to itself, directly or through other PlanningItems.
            // Same posture as E712.
            ("E721", EdgeKind::PlanningBlockedBy, "PlanningItem blockedBy cycle detected"),
        ];

        for (code, kind, label) in checks {
            let mut sub: DiGraph<petgraph::graph::NodeIndex, ()> = DiGraph::new();
            let mut sub_nodes: HashMap<petgraph::graph::NodeIndex, petgraph::graph::NodeIndex> =
                HashMap::new();

            for edge in full_graph.edge_references() {
                if *kind == EdgeKind::DerivedFrom
                    && configuration_qnames.contains(full_graph[edge.source()].as_str())
                {
                    continue;
                }
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
        for (ri, r) in refines.iter().enumerate() {
            // REQ-TRS-LINKTYPE-006 — per-entry E316 relaxation / coverage.
            let e316_relaxed = link_prov.relaxed(&elem.qualified_name, crate::link_types::BaseLink::Refines, ri, "E316");
            let indexed = link_prov.in_reverse_index(&elem.qualified_name, crate::link_types::BaseLink::Refines, ri);
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
                    if !is_req && !e316_relaxed {
                        findings.push(error(
                            "E316",
                            &elem.file_path,
                            &format!(
                                "{} '{}' refines '{}' which resolves to a {:?}, not a Requirement/RequirementDef{}",
                                noun,
                                elem.qualified_name,
                                r,
                                target.frontmatter.element_type.clone().unwrap_or(ElementType::Unknown),
                                link_prov.via_suffix(&elem.qualified_name, crate::link_types::BaseLink::Refines, ri)
                            ),
                        ));
                    } else if indexed {
                        // A contributed entry duplicating an existing refiner is
                        // not listed twice (REQ-TRS-LINKTYPE-006).
                        let list = refined_by.entry(index_key(target)).or_default();
                        let label = elem_label(elem);
                        let dup = link_prov.is_contributed(&elem.qualified_name, crate::link_types::BaseLink::Refines, ri)
                            && list.contains(&label);
                        if !dup {
                            list.push(label);
                        }
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
            let is_refined = refined_by.get(&key).is_some_and(|v| !v.is_empty());
            let is_derived = derived_children.get(&key).is_some_and(|v| !v.is_empty());
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
            let refined = mop_refined_by.get(&key).is_some_and(|v| !v.is_empty());
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
    // Per edge, the forms that produced it (for W503), in encounter order.
    let mut edge_forms: HashMap<(String, String), Vec<AllocForm>> = HashMap::new();
    let mut edge_order: Vec<(String, String)> = Vec::new();
    for (from, to, form) in &tagged_edges {
        let entry = edge_forms.entry((from.clone(), to.clone()));
        if matches!(entry, std::collections::hash_map::Entry::Vacant(_)) {
            edge_order.push((from.clone(), to.clone()));
        }
        let forms = entry.or_default();
        if !forms.contains(form) {
            forms.push(*form);
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
    // W503 — the same source → target edge declared by more than one form.
    for (from, to) in &edge_order {
        if let Some(forms) = edge_forms.get(&(from.clone(), to.clone())) {
            if forms.len() >= 2 {
                // Stable, form-ordered wording (AllocatedTo, Element, AuthoredFrom).
                let mut sorted = forms.clone();
                sorted.sort_by_key(|f| match f {
                    AllocForm::AllocatedTo => 0,
                    AllocForm::Element => 1,
                    AllocForm::AuthoredFrom => 2,
                });
                let labels: Vec<&str> = sorted.iter().map(|f| f.label()).collect();
                let file = resolver
                    .resolve_ref(elements, from)
                    .map(|e| e.file_path.clone())
                    .unwrap_or_default();
                findings.push(warning(
                    "W503",
                    &file,
                    &format!(
                        "redundant allocation: {} → {} is declared by both {} — use one form",
                        from,
                        to,
                        labels.join(" and ")
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
    // E110–E114 (REQ-TRS-XREF-007, GH #125): unresolved supertype/typedBy/
    // subsets/redefines/satisfies references. Before the root-name hint so the
    // hint annotates them too.
    findings.extend(crate::structural_refs::unresolved_structural_ref_findings(elements, &resolver, config));

    annotate_root_name_hints(&mut findings, elements, &resolver);

    // §9.9 — Build-system integration: E050 (conflicting buildExports var names
    // across selected features, not resolved by buildOverrides).
    // Gate: only run when at least one element opts in to the build-config feature
    // so the pass is zero-cost for models that do not use it.
    let has_build_fields = elements.iter().any(|e| {
        e.frontmatter.build_exports.is_some() || e.frontmatter.build_overrides.is_some()
    });
    if has_build_fields {
        findings.extend(crate::build_config::validate_build_exports(elements));
    }

    // REQ-TRS-BL-005 — baseline drift / freeze pass. E520 released-drift, W520
    // approved-drift, E521 seal-vs-manifest tamper, E522 unresolved supersedes.
    if let Some(root) = config.model_root.as_deref() {
        if authored_elements.iter().any(|e| crate::baseline::is_baseline(&e.frontmatter)) {
            for (code, file, msg) in crate::baseline::scan(authored_elements, &resolver, root) {
                let sev = if code.starts_with('E') { Severity::Error } else { Severity::Warning };
                findings.push(Finding { code, file, message: msg, severity: sev });
            }
        }
    }

    // REQ-TRS-SUS-LINKS-004 — W090 suspect-link detection. For each trace link that
    // carries a stored baseline, recompute the target's projection hash and compare;
    // a mismatch is suspect. Unbaselined links stay silent (opt-in/additive) and an
    // unresolvable baselined target is left to the unresolved-cross-reference checks.
    // Hashes are taken over the authored elements (never the effective view), and
    // custom `links:` targets count as trace links unless their type opts out
    // (`suspect = false`, REQ-TRS-LINKTYPE-011).
    for link in crate::suspect::scan_with(authored_elements, &resolver, &config.link_types) {
        if matches!(link.state, crate::suspect::LinkState::Suspect) {
            findings.push(warning(
                "W090",
                &link.source_file,
                &format!(
                    "suspect link: `{}` {} `{}` — the target's content changed since the \
                     baseline was captured; review and re-run `suspect accept {} {}`",
                    link.source_label(),
                    link.kind,
                    link.target_ref,
                    link.source_label(),
                    link.target_ref,
                ),
            ));
        }
    }

    // ── Configuration inheritance through `derivedFrom:` (§9.8, GH #137) ─────
    // Before the HPLE pass so a local sub-configuration target with a broken
    // inheritance chain counts as not internally valid (E518).
    findings.extend(configuration_inheritance_findings(elements));

    // ── HPLE `subConfigurations:` (REQ-TRS-HPLE-001, ADR-SYS-HPLE-001) ───────
    // Active whenever any Configuration declares `subConfigurations:`, whether
    // or not `[repos]` is configured — a purely-local hierarchy is valid too.
    // Runs last, after every ordinary per-element check above has populated
    // `findings`, so a local target's validity can be read off of what this
    // same pass already computed for it instead of re-validating (see the
    // function doc comment for why that matters).
    findings.extend(sub_configuration_findings(elements, config, &resolver, &findings));

    // ── HPLE open-parameter completeness (REQ-TRS-HPLE-004, `W513`) ──────────
    // Dormant unless some Configuration declares `subConfigurations:`, same as
    // the checks above.
    findings.extend(open_parameter_findings(elements, config, &resolver));

    ValidationResult {
        findings,
        verified_by,
        derived_children,
        planning_children,
        refined_by,
        actor_in,
        mop_refined_by,
        allocated_from,
    }
}

/// User-defined link instances against their declarations (ADR-SYS-LINKTYPE-001):
///
/// * `E630` — a `links:` key that is not a valid declared link type; the message
///   lists the declared types, or says none are and how to declare one
///   (REQ-TRS-LINKTYPE-002/012);
/// * `E631` — `links:` is not a mapping, or a key's value is not a reference or a
///   list of references;
/// * `E632` — a target that does not resolve (like `satisfies:`; a valid
///   cross-repo reference under `[repos]` is accepted);
/// * `E633`/`E634` — source/target type outside the declared `sourceTypes`/
///   `targetTypes` (REQ-TRS-LINKTYPE-003);
/// * `E635`/`W631` — more targets than the cardinality's upper bound / fewer than
///   its lower bound on a non-draft element of a declared source type
///   (REQ-TRS-LINKTYPE-004);
/// * `E636` — a cycle formed solely by links of an `acyclic` type, self-links
///   included, once per cycle (strongly connected component) naming its members
///   (REQ-TRS-LINKTYPE-005).
///
/// Dormant for a model with no `[linkTypes]` table and no `links:` field.
fn link_type_findings(elements: &[RawElement], config: &ValidateConfig) -> Vec<Finding> {
    use crate::link_types::{element_type_name, label, parse_links, LinksField};

    let reg = &config.link_types;
    let mut findings = Vec::new();
    let any_links = elements.iter().any(|e| e.frontmatter.links.is_some());
    if !any_links && reg.is_empty() {
        return findings;
    }
    let resolver = Resolver::new(elements);

    for elem in elements {
        let fm = &elem.frontmatter;
        let file = elem.file_path.as_str();
        // Authored target count per declared type on this element (for E635/W631).
        let mut counts: HashMap<usize, usize> = HashMap::new();
        match parse_links(fm) {
            LinksField::Absent => {}
            LinksField::NotAMapping => findings.push(error(
                "E631",
                file,
                "`links:` must be a mapping of link-type name → reference or list of references",
            )),
            LinksField::Entries(entries) => {
                for entry in entries {
                    let Some(ti) = entry.key_ok.then(|| reg.index_of(&entry.key)).flatten() else {
                        findings.push(error(
                            "E630",
                            file,
                            &format!(
                                "`links:` key '{}' is not a declared link type — {}",
                                entry.key,
                                reg.declared_names_hint()
                            ),
                        ));
                        continue;
                    };
                    let decl = &reg.types()[ti];
                    let Some(targets) = entry.targets else {
                        findings.push(error(
                            "E631",
                            file,
                            &format!("`links.{}` must be a reference or a list of references", entry.key),
                        ));
                        continue;
                    };
                    // A target listed twice (`[A, A]`) is one link (REQ-TRS-LINKTYPE-004).
                    let distinct: HashSet<&String> = targets.iter().collect();
                    *counts.entry(ti).or_default() += distinct.len();
                    if !decl.permits_source(fm.element_type.as_ref()) {
                        findings.push(error(
                            "E633",
                            file,
                            &format!(
                                "link type '{}' does not permit a source of type '{}' (sourceTypes: {})",
                                decl.name,
                                element_type_name(fm.element_type.as_ref()),
                                decl.source_types.as_deref().unwrap_or_default().join(", ")
                            ),
                        ));
                    }
                    for t in &targets {
                        match resolver.resolve_ref(elements, t) {
                            None if config.peer_resolves(t) => { /* §14.4 valid cross-repo reference */ }
                            None => findings.push(error(
                                "E632",
                                file,
                                &format!("unresolved `links.{}` reference '{}'", decl.name, t),
                            )),
                            Some(target) => {
                                if !decl.permits_target(target.frontmatter.element_type.as_ref()) {
                                    findings.push(error(
                                        "E634",
                                        file,
                                        &format!(
                                            "`links.{}` target '{}' is a '{}', not one of the permitted targetTypes ({})",
                                            decl.name,
                                            t,
                                            element_type_name(target.frontmatter.element_type.as_ref()),
                                            decl.target_types.as_deref().unwrap_or_default().join(", ")
                                        ),
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }

        // Cardinality: the upper bound applies to any holder; the lower bound only
        // to non-draft elements of a declared source type (even with no `links:`).
        let draft = fm.status.as_deref() == Some("draft");
        for (ti, decl) in reg.types().iter().enumerate() {
            let n = counts.get(&ti).copied().unwrap_or(0);
            if let Some(max) = decl.cardinality.max {
                if n > max {
                    findings.push(error(
                        "E635",
                        file,
                        &format!(
                            "`links.{}` has {} target(s) but its cardinality '{}' allows at most {}",
                            decl.name, n, decl.cardinality, max
                        ),
                    ));
                }
            }
            if !draft && n < decl.cardinality.min && decl.requires_links_on(fm.element_type.as_ref()) {
                findings.push(warning(
                    "W631",
                    file,
                    &format!(
                        "{} '{}' has {} `{}` link(s) but the link type's cardinality '{}' requires at least {}",
                        element_type_name(fm.element_type.as_ref()),
                        label(elem),
                        n,
                        decl.name,
                        decl.cardinality,
                        decl.cardinality.min
                    ),
                ));
            }
        }
    }

    // E636: cycles in acyclic types — one finding per strongly connected component
    // with more than one member, or a single member linking to itself.
    let edges = crate::link_types::resolved_edges(elements, &resolver, reg);
    for (ti, decl) in reg.types().iter().enumerate() {
        if !decl.acyclic {
            continue;
        }
        let mut g: DiGraph<usize, ()> = DiGraph::new();
        let mut node_of: HashMap<usize, petgraph::graph::NodeIndex> = HashMap::new();
        let mut self_loops: HashSet<usize> = HashSet::new();
        for e in edges.iter().filter(|e| e.type_idx == ti) {
            if e.source == e.target {
                self_loops.insert(e.source);
            }
            let a = *node_of.entry(e.source).or_insert_with(|| g.add_node(e.source));
            let b = *node_of.entry(e.target).or_insert_with(|| g.add_node(e.target));
            g.add_edge(a, b, ());
        }
        let mut cycles: Vec<Vec<usize>> = petgraph::algo::tarjan_scc(&g)
            .into_iter()
            .map(|scc| scc.into_iter().map(|n| g[n]).collect::<Vec<usize>>())
            .filter(|m| m.len() > 1 || m.first().is_some_and(|x| self_loops.contains(x)))
            .collect();
        for members in &mut cycles {
            members.sort_by_key(|&i| label(&elements[i]));
        }
        cycles.sort_by_key(|m| label(&elements[m[0]]));
        for members in cycles {
            let names: Vec<String> = members.iter().map(|&i| label(&elements[i])).collect();
            let msg = if names.len() == 1 {
                format!("acyclic link type '{}' has a self-link on '{}'", decl.name, names[0])
            } else {
                format!("acyclic link type '{}' forms a cycle among {}", decl.name, names.join(", "))
            };
            findings.push(error("E636", &elements[members[0]].file_path, &msg));
        }
    }

    findings
}

/// The finding codes that quote a single cross-reference and to which the
/// REQ-TRS-XREF-006 root-name hint applies (the generic unresolved-reference
/// findings: traceability, refinement, allocation, and the structural
/// supertype/typedBy/subsets/redefines/connection resolution errors).
const ROOT_HINT_CODES: &[&str] = &[
    "E102", "E103", "E110", "E111", "E112", "E113", "E114", "E311", "E316", "E502", "E503", "E506", "E632",
];

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
    from_qname: &str,
    out: &mut HashSet<String>,
) {
    let key_typed_by = serde_yaml::Value::String("typedBy".into());
    let key_ports = serde_yaml::Value::String("ports".into());
    for item in list {
        if let serde_yaml::Value::Mapping(map) = item {
            if let Some(v) = map.get(&key_typed_by) {
                for s in yaml_strings(v) {
                    if let Some(target) = resolver.resolve_scoped_ref(elements, from_qname, s) {
                        out.insert(target.qualified_name.clone());
                    }
                }
            }
            // Recurse into nested ports: sub-key
            if let Some(serde_yaml::Value::Sequence(ports)) = map.get(&key_ports) {
                collect_typed_by_refs(ports, elements, resolver, from_qname, out);
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

/// Bounded recursion depth for the `subConfigurations:` peer-validity gate
/// (below). Validating a peer re-enters `validate_with_config` on the peer's
/// own elements, which — if that peer's own `Configuration` also declares
/// `subConfigurations:` — re-enters [`sub_configuration_findings`] again for
/// the next tier down. A genuine multi-tier hierarchy is shallow (a handful of
/// tiers at most); this guard only exists so an authoring mistake that forms
/// an actual cycle across repos degrades to a reported `E518` rather than
/// recursing forever, matching this codebase's standing "report, never panic"
/// rule for circular references.
const HPLE_MAX_DEPTH: u32 = 16;

/// Stack size for the dedicated thread each peer-validity recursion runs on
/// (below). `validate_with_config` is a large function; re-entering it
/// recursively (peer → its own `subConfigurations` → its peer → …) on
/// whatever stack the *caller* happens to have is not safe — measured at
/// 150+ KB per level, even a handful of levels can exceed a constrained
/// stack (notably Tokio's 2 MiB default worker-thread stack, which both the
/// MCP server and the LSP server validate from). Each recursive step
/// therefore runs on a freshly spawned thread sized generously enough that
/// the full `HPLE_MAX_DEPTH` bound is comfortably safe regardless of the
/// calling thread's own stack size.
const HPLE_PEER_STACK_SIZE: usize = 32 * 1024 * 1024;

thread_local! {
    static HPLE_DEPTH: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

/// Human-readable label for an element's `type:` in a diagnostic message.
fn element_type_label(t: &Option<ElementType>) -> String {
    match t {
        Some(ty) => format!("{ty:?}"),
        None => "untyped element".to_string(),
    }
}

/// `config` with `elements`' own `repoImports:` mount points installed
/// (§14.3/§14.4) — what `validate_with_config` does on entry, repeated here so
/// the HPLE entry points below also resolve `<package>::<as>::X` mount paths
/// when called directly (e.g. `feature-check` calls
/// [`parameter_binding_findings`] without going through `validate_with_config`)
/// and for each peer tier they recurse into (GH #146). Borrows `config`
/// unchanged when no repo is configured or the mounts are already installed.
fn with_repo_mounts<'a>(elements: &[RawElement], config: &'a ValidateConfig) -> std::borrow::Cow<'a, ValidateConfig> {
    if config.has_repos() && config.repo_mounts.is_empty() {
        let mounts = crate::config::repo_mounts(elements, &config.repos);
        if !mounts.is_empty() {
            let mut mounted = config.clone();
            mounted.repo_mounts = mounts;
            return std::borrow::Cow::Owned(mounted);
        }
    }
    std::borrow::Cow::Borrowed(config)
}

/// The loaded peer repo a `subConfigurations:` entry `sc` that did not resolve
/// locally should be looked up in, with the name to resolve it by there
/// (REQ-TRS-HPLE-001, §14.4 order). A `repoImports:` mount path
/// (`<package>::<as>::X`) names exactly one repo and translates to that
/// peer's native qualified name (GH #146); otherwise the first loaded repo
/// whose qname/stable-id index knows `sc` (exact, or as a trailing `::`
/// segment) is used with `sc` itself. `None` when no loaded repo can hold it.
fn sub_configuration_peer<'a>(config: &'a ValidateConfig, sc: &str) -> Option<(&'a crate::config::LoadedRepo, String)> {
    if let Some((repo, native)) = config.unmount(sc) {
        return repo.exists.then_some((repo, native));
    }
    let suffix = format!("::{sc}");
    config
        .repos
        .iter()
        .filter(|repo| repo.exists)
        .find(|repo| {
            repo.stable_ids.contains(sc) || repo.qnames.contains(sc) || repo.qnames.iter().any(|q| q.ends_with(&suffix))
        })
        .map(|repo| (repo, sc.to_string()))
}

/// A `parameterBindings:` key with its `<FeatureDef>` part translated through
/// `config`'s `repoImports:` mounts to the peer-native qualified name (GH
/// #146), or `None` when the key is not a dotted path under a mount.
fn unmount_binding_key(config: &ValidateConfig, key: &str) -> Option<String> {
    let (feat, pname) = key.rsplit_once('.')?;
    config.unmount(feat).map(|(_, native)| format!("{native}.{pname}"))
}

/// Run `validate_with_config` + `check_feature_model_deep` against a peer's
/// elements on a dedicated thread carrying a generous, fixed-size stack
/// ([`HPLE_PEER_STACK_SIZE`]) rather than whatever stack the caller happens
/// to have — see that constant's doc comment. `next_depth` seeds the new
/// thread's *own* [`HPLE_DEPTH`] (thread-locals do not propagate across a
/// thread spawn) so the bounded-recursion guard still applies however many
/// tiers deep this goes.
///
/// Returns `(error_findings, void, invalid_configs)` — plain owned data, not
/// a borrowed `ValidationResult`/`DeepReport`, so nothing here needs to
/// outlive the joined thread. Two distinct failure modes are both handled as
/// `Err(())` rather than ever propagating a panic to the calling thread — a
/// validity *gate* must fail closed either way:
///   - OS thread creation itself fails (`spawn_scoped` returns `Err`) —
///     plausible resource exhaustion under concurrent load in a long-running
///     MCP/LSP server recursing through several 32 MiB-stack levels at once.
///   - The spawned thread panics (should not happen: the stack is sized well
///     beyond the bounded depth) — caught via `join()`.
fn run_peer_validation_on_dedicated_thread(
    peer_elements: &[RawElement],
    peer_config: &ValidateConfig,
    next_depth: u32,
) -> Result<(Vec<Finding>, bool, Vec<String>), ()> {
    std::thread::scope(|scope| {
        let spawned = std::thread::Builder::new()
            .stack_size(HPLE_PEER_STACK_SIZE)
            .spawn_scoped(scope, move || {
                HPLE_DEPTH.with(|d| d.set(next_depth));
                let peer_result = validate_with_config(peer_elements, peer_config);
                let errors: Vec<Finding> = peer_result.errors().cloned().collect();
                let peer_deep = crate::feature_model::check_feature_model_deep(peer_elements);
                (errors, peer_deep.void, peer_deep.invalid_configs)
            });
        let handle = spawned.map_err(|_| ())?;
        handle.join().map_err(|_| ())
    })
}

/// `subConfigurations:` resolution + peer-validity gate (REQ-TRS-HPLE-001,
/// `ADR-SYS-HPLE-001`). A `Configuration` may consolidate one or more other
/// `Configuration` elements, reachable locally or via a loaded peer repo
/// (§14) — the local model is searched first, then each loaded repo in
/// declaration order, mirroring the existing `verifies:`/`derivedFrom:`
/// cross-repo resolution order (§14.4).
///
/// Each entry must resolve to a real element (else `E516`, dangling) that is
/// a `Configuration` (else `E517`, wrong type) which is itself internally
/// valid — SAT-satisfiable and error-free (else `E518`). For a peer entry
/// this genuinely loads and parses the peer's model via [`crate::walker::walk_model`]
/// rather than trusting the existence-only qname/id `HashSet`s on
/// [`crate::config::LoadedRepo`] — confirming a peer `Configuration`'s
/// validity requires reading its real feature/parameter structure, which
/// those sets do not carry (`ADR-SYS-HPLE-001` Decision 1's "correction found
/// during implementation scoping").
///
/// `prior_findings` is the full `findings` vec already accumulated by every
/// *other* check in this same `validate_with_config` pass (this function is
/// called last, precisely so this is complete). A **local** target's
/// validity is read off of that plus every HPLE finding already produced
/// *within this same call* for other local configs — see "processing order"
/// below — rather than re-validating it, which would mean recursing into
/// `validate_with_config` on the very same `elements` slice this call is
/// itself part of. This gives a local target the same "clean and
/// error-free" bar a peer target is held to (peer targets get a genuine,
/// independent `validate_with_config` run; a local target's independent run
/// is simply this one, already done) without the self-recursion that would
/// otherwise risk. `check_feature_model_deep` is still consulted directly
/// for the SAT-semantics half, since deep feature-model analysis is not
/// part of the ordinary per-element passes.
///
/// **Processing order.** Local configs are *not* walked in `elements` order
/// against one frozen snapshot — a single fixed snapshot can only ever
/// propagate a freshly-found problem one level up a chain, no matter how
/// that walk is ordered (confirmed: a 3-tier chain A → B → C with a plain
/// structural error on C correctly flagged B, since B's own error came from
/// the snapshot, but never flagged A, since B's *freshly generated* finding
/// never fed back into anything). Instead, local configs are processed in
/// dependency order — a topological sort over the local `subConfigurations`
/// graph, leaves (configs with no further local targets) first — threading
/// one single, *growing* findings accumulator through the walk. By the time
/// a config is processed, every local config it depends on has already been
/// processed and its findings are already in the accumulator, however many
/// local tiers deep that chain goes. A local cycle has no topological order;
/// the configs involved are processed last, in a deterministic order, and
/// any entry naming a target that is itself part of the unresolved cycle is
/// reported directly (its accumulated findings cannot be trusted — no valid
/// processing order put them there first) rather than silently read off of
/// an order-dependent partial result — degrading gracefully, never looping
/// or panicking, matching this codebase's standing rule for circular
/// references.
///
/// Dormant (returns empty) unless some `Configuration` actually declares
/// `subConfigurations:`, so a model with none anywhere is unaffected.
pub fn sub_configuration_findings(
    elements: &[RawElement],
    config: &ValidateConfig,
    resolver: &Resolver,
    prior_findings: &[Finding],
) -> Vec<Finding> {
    let config = &*with_repo_mounts(elements, config);
    let mut own_findings: Vec<Finding> = Vec::new();

    let configs: Vec<&RawElement> = elements
        .iter()
        .filter(|e| matches!(e.frontmatter.element_type, Some(ElementType::Configuration)))
        .collect();
    if configs
        .iter()
        .all(|c| c.frontmatter.sub_configurations.is_none())
    {
        return own_findings;
    }

    // ── Dependency order: topological sort over the *local* subConfigurations
    // graph (edges: config i -> local Configuration target j it names), leaves
    // first, so a chain of any local depth propagates in one pass. ──────────
    let index_of: HashMap<&str, usize> = configs
        .iter()
        .enumerate()
        .map(|(i, c)| (c.qualified_name.as_str(), i))
        .collect();
    let mut depends_on: Vec<HashSet<usize>> = vec![HashSet::new(); configs.len()];
    for (i, cfg) in configs.iter().enumerate() {
        let Some(entries) = &cfg.frontmatter.sub_configurations else {
            continue;
        };
        for raw_sc in entries {
            let sc = raw_sc.trim();
            if sc.is_empty() {
                continue;
            }
            if let Some(target) = resolver.resolve_ref(elements, sc) {
                if matches!(target.frontmatter.element_type, Some(ElementType::Configuration)) {
                    if let Some(&j) = index_of.get(target.qualified_name.as_str()) {
                        // Self-references (j == i) are kept, not skipped: a
                        // config cannot depend on its own not-yet-computed
                        // result, so this correctly forces it into the
                        // leftover/cycle set below rather than pretending it
                        // has a valid position.
                        depends_on[i].insert(j);
                    }
                }
            }
        }
    }
    let mut dependents: Vec<Vec<usize>> = vec![Vec::new(); configs.len()];
    for (i, deps) in depends_on.iter().enumerate() {
        for &j in deps {
            dependents[j].push(i);
        }
    }
    let mut remaining: Vec<HashSet<usize>> = depends_on.clone();
    let mut queue: std::collections::VecDeque<usize> = (0..configs.len())
        .filter(|&i| remaining[i].is_empty())
        .collect();
    let mut processed = vec![false; configs.len()];
    let mut order: Vec<usize> = Vec::with_capacity(configs.len());
    while let Some(i) = queue.pop_front() {
        if processed[i] {
            continue;
        }
        processed[i] = true;
        order.push(i);
        for &d in &dependents[i] {
            remaining[d].remove(&i);
            if !processed[d] && remaining[d].is_empty() {
                queue.push_back(d);
            }
        }
    }
    // Leftover: part of (or depends on) a local subConfigurations cycle, for
    // which no topological order exists. Processed last, in a deterministic
    // (qname-sorted) order, so this never loops or panics — see the function
    // doc comment.
    let mut leftover: Vec<usize> = (0..configs.len()).filter(|&i| !processed[i]).collect();
    leftover.sort_by_key(|&i| configs[i].qualified_name.clone());
    let in_cycle: HashSet<usize> = leftover.iter().copied().collect();
    order.extend(leftover);

    // This model's own deep report, computed at most once and only if some
    // entry actually resolves locally (a purely-peer hierarchy never needs it).
    let mut local_deep: Option<crate::feature_model::DeepReport> = None;
    // Peer walks are cached per repo model root so N entries into the same
    // repo only walk it once.
    let mut peer_cache: HashMap<std::path::PathBuf, Option<Vec<RawElement>>> = HashMap::new();

    // The growing accumulator: `prior_findings` plus every HPLE finding
    // produced so far *in this call*, for local configs already processed
    // (dependency order guarantees a local target is always processed before
    // whatever consolidates it, so its entry here is always final by then).
    let mut effective: Vec<Finding> = prior_findings.to_vec();

    for &ci in &order {
        let cfg = configs[ci];
        let Some(entries) = &cfg.frontmatter.sub_configurations else {
            continue;
        };
        for raw_sc in entries {
            let sc = raw_sc.trim();
            if sc.is_empty() {
                continue;
            }

            // 1. Local resolution first (§14.4 order).
            if let Some(target) = resolver.resolve_ref(elements, sc) {
                if !matches!(target.frontmatter.element_type, Some(ElementType::Configuration)) {
                    let f = error(
                        "E517",
                        &cfg.file_path,
                        &format!(
                            "subConfigurations '{}' resolves to a {}, not a Configuration",
                            sc,
                            element_type_label(&target.frontmatter.element_type)
                        ),
                    );
                    effective.push(f.clone());
                    own_findings.push(f);
                    continue;
                }
                let tgt_id = target
                    .frontmatter
                    .id
                    .clone()
                    .unwrap_or_else(|| target.qualified_name.clone());

                // A target that is itself part of an unresolved local cycle
                // has no findings this pass can trust (no valid processing
                // order exists for it) -- report the circularity directly
                // rather than reading an order-dependent partial result.
                if let Some(&tj) = index_of.get(target.qualified_name.as_str()) {
                    if in_cycle.contains(&tj) {
                        let f = error(
                            "E518",
                            &cfg.file_path,
                            &format!(
                                "subConfigurations '{}' names local Configuration '{}', which is part of a circular local subConfigurations chain — its internal validity cannot be confirmed",
                                sc, tgt_id
                            ),
                        );
                        effective.push(f.clone());
                        own_findings.push(f);
                        continue;
                    }
                }

                // Ordinary structural validity: any error already raised
                // against the target's own file, either by the rest of this
                // pass (e.g. a plain E201 missing-required-field, which
                // check_feature_model_deep has no way to see) or by this same
                // function for a local target processed earlier in
                // dependency order (so a chain 3+ tiers deep propagates).
                let existing_errors: Vec<&Finding> = effective
                    .iter()
                    .filter(|f| f.severity == Severity::Error && f.file == target.file_path)
                    .collect();
                if !existing_errors.is_empty() {
                    let f = error(
                        "E518",
                        &cfg.file_path,
                        &format!(
                            "subConfigurations '{}' names local Configuration '{}', which is not internally valid: {} validation error(s) already found on it (e.g. {}: {})",
                            sc,
                            tgt_id,
                            existing_errors.len(),
                            existing_errors[0].code,
                            existing_errors[0].message
                        ),
                    );
                    effective.push(f.clone());
                    own_findings.push(f);
                    continue;
                }

                // SAT-semantics validity: deep feature-model analysis is not
                // one of the ordinary per-element passes, so it is run
                // directly here (on this same, already-in-hand `elements`
                // slice — no recursion).
                let rep = local_deep
                    .get_or_insert_with(|| crate::feature_model::check_feature_model_deep(elements));
                if rep.void {
                    let f = error(
                        "E518",
                        &cfg.file_path,
                        &format!(
                            "subConfigurations '{}' names local Configuration '{}', but this model's feature model is void (no valid configuration exists) — it cannot be consolidated",
                            sc, tgt_id
                        ),
                    );
                    effective.push(f.clone());
                    own_findings.push(f);
                } else if rep.invalid_configs.contains(&tgt_id) {
                    let f = error(
                        "E518",
                        &cfg.file_path,
                        &format!(
                            "subConfigurations '{}' names local Configuration '{}', which is not a valid model of the feature model (feature-check --deep: E225)",
                            sc, tgt_id
                        ),
                    );
                    effective.push(f.clone());
                    own_findings.push(f);
                }
                continue;
            }

            // 2. A loaded peer repo: the one a `repoImports:` mount path names
            // (translated to the peer-native qname, GH #146), else the first,
            // in declaration order, whose index knows the name.
            let mut resolved_in_repo = false;
            if let Some((repo, peer_name)) = sub_configuration_peer(config, sc) {
                let peer_name = peer_name.as_str();
                resolved_in_repo = true;
                'peer: {
                    // Genuinely load and parse the peer — the shallow qname/id
                    // index only proves existence (ADR-SYS-HPLE-001 Decision 1).
                    let peer_elements = peer_cache
                        .entry(repo.model_root.clone())
                        .or_insert_with(|| crate::walker::walk_model(&repo.model_root).ok());

                    let Some(peer_elements) = peer_elements else {
                        let f = error(
                            "E516",
                            &cfg.file_path,
                            &format!(
                                "subConfigurations '{}' names a Configuration in repo '{}', but the repo could not be loaded",
                                sc, repo.alias
                            ),
                        );
                        effective.push(f.clone());
                        own_findings.push(f);
                        break 'peer;
                    };

                    let peer_resolver = Resolver::new(peer_elements);
                    let Some(target) = peer_resolver.resolve_ref(peer_elements, peer_name) else {
                        // The shallow index (or the mount) said it exists but the
                        // real parse disagrees — degrade to dangling rather than
                        // accepting it.
                        let f = error(
                            "E516",
                            &cfg.file_path,
                            &format!(
                                "subConfigurations '{}' does not resolve to any element in repo '{}'",
                                sc, repo.alias
                            ),
                        );
                        effective.push(f.clone());
                        own_findings.push(f);
                        break 'peer;
                    };

                    if !matches!(target.frontmatter.element_type, Some(ElementType::Configuration)) {
                        let f = error(
                            "E517",
                            &cfg.file_path,
                            &format!(
                                "subConfigurations '{}' resolves to a {} in repo '{}', not a Configuration",
                                sc,
                                element_type_label(&target.frontmatter.element_type),
                                repo.alias
                            ),
                        );
                        effective.push(f.clone());
                        own_findings.push(f);
                        break 'peer;
                    }

                    let tgt_id = target
                        .frontmatter
                        .id
                        .clone()
                        .unwrap_or_else(|| target.qualified_name.clone());

                    // Bounded recursion guard (see HPLE_MAX_DEPTH doc comment).
                    // Read on *this* thread; the recursive step below seeds the
                    // *new* thread's own copy from `next_depth` (thread-locals do
                    // not propagate across a thread spawn).
                    let depth = HPLE_DEPTH.with(|d| d.get());
                    if depth >= HPLE_MAX_DEPTH {
                        let f = error(
                            "E518",
                            &cfg.file_path,
                            &format!(
                                "subConfigurations '{}' (Configuration '{}' in repo '{}') exceeds the maximum consolidation depth ({}) — check for a circular subConfigurations chain",
                                sc, tgt_id, repo.alias, HPLE_MAX_DEPTH
                            ),
                        );
                        effective.push(f.clone());
                        own_findings.push(f);
                        break 'peer;
                    }
                    // Loading a peer config installs that peer's `[linkTypes]` as the
                    // process-wide vocabulary (REQ-TRS-LINKTYPE-001); restore ours after.
                    let saved_link_types = crate::link_types::active();
                    let peer_config = ValidateConfig::with_model_root(repo.model_root.clone());
                    crate::link_types::install(&saved_link_types);
                    match run_peer_validation_on_dedicated_thread(peer_elements, &peer_config, depth + 1) {
                        Ok((peer_errors, peer_void, peer_invalid_configs)) => {
                            if !peer_errors.is_empty() {
                                let f = error(
                                    "E518",
                                    &cfg.file_path,
                                    &format!(
                                        "subConfigurations '{}' names Configuration '{}' in repo '{}', which is not internally valid: {} validation error(s) in that repo (e.g. {}: {})",
                                        sc,
                                        tgt_id,
                                        repo.alias,
                                        peer_errors.len(),
                                        peer_errors[0].code,
                                        peer_errors[0].message
                                    ),
                                );
                                effective.push(f.clone());
                                own_findings.push(f);
                            } else if peer_void {
                                let f = error(
                                    "E518",
                                    &cfg.file_path,
                                    &format!(
                                        "subConfigurations '{}' names Configuration '{}' in repo '{}', but that repo's feature model is void (no valid configuration exists)",
                                        sc, tgt_id, repo.alias
                                    ),
                                );
                                effective.push(f.clone());
                                own_findings.push(f);
                            } else if peer_invalid_configs.contains(&tgt_id) {
                                let f = error(
                                    "E518",
                                    &cfg.file_path,
                                    &format!(
                                        "subConfigurations '{}' names Configuration '{}' in repo '{}', which is not a valid model of that repo's feature model (feature-check --deep: E225)",
                                        sc, tgt_id, repo.alias
                                    ),
                                );
                                effective.push(f.clone());
                                own_findings.push(f);
                            }
                        }
                        Err(()) => {
                            // Fail closed: a validity gate that cannot confirm
                            // validity must not silently treat the target as valid.
                            let f = error(
                                "E518",
                                &cfg.file_path,
                                &format!(
                                    "subConfigurations '{}' names Configuration '{}' in repo '{}', but validating it failed unexpectedly — treating as not internally valid",
                                    sc, tgt_id, repo.alias
                                ),
                            );
                            effective.push(f.clone());
                            own_findings.push(f);
                        }
                    }
                }
            }

            if !resolved_in_repo {
                let f = error(
                    "E516",
                    &cfg.file_path,
                    &format!(
                        "subConfigurations '{}' does not resolve to any element, locally or in a loaded peer repo",
                        sc
                    ),
                );
                effective.push(f.clone());
                own_findings.push(f);
            }
        }
    }

    own_findings
}

/// Per-parameter metadata read off a `FeatureDef`'s `parameters:` entry, used
/// by [`parameter_binding_findings`] and its HPLE transitive-lookup helper
/// [`collect_reachable_feature_params`] below.
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

/// A `FeatureDef` parameter reached through `subConfigurations:`, plus the
/// extra per-parameter status `REQ-TRS-HPLE-003` needs to decide whether a
/// cross-tier `parameterBindings:` entry targeting it is genuinely open
/// (`PI-HPLE-BINDGUARD-001`) — layered on top of `REQ-TRS-HPLE-002`'s plain
/// resolution (`ParamMeta`).
struct TransitiveParamStatus {
    meta: ParamMeta,
    /// Whether the peer `Configuration` that actually owns this `FeatureDef`
    /// (the one whose own model declares it) selects it. `E519` fires when
    /// this is `false` — the cross-tier extension of `E203`.
    selected_by_owner: bool,
    /// A human-readable label for the *nearest-to-the-querying-Configuration*
    /// tier on the walked path — owner-inclusive — whose own
    /// `parameterBindings:` already supplies this exact `<feat>.<param>` key,
    /// if any. `E523` fires when this is `Some` — double-binding something a
    /// nearer tier already closed.
    already_bound_by: Option<String>,
}

/// A parameter resolved by [`parameter_binding_findings`]'s dotted-key
/// lookup — either genuinely local (`ParamMeta` from this model's own
/// `feature_params`) or reached transitively through `subConfigurations:`
/// (`TransitiveParamStatus`, carrying the extra cross-tier status
/// `REQ-TRS-HPLE-003` needs). Unifies the two so the shared per-parameter
/// checks (`E204`/`E205`/`E206`/`W027`) read one `ParamMeta` regardless of
/// which source it came from.
enum ResolvedParam<'a> {
    Local(&'a ParamMeta),
    Transitive(&'a TransitiveParamStatus),
}

impl ResolvedParam<'_> {
    fn meta(&self) -> &ParamMeta {
        match self {
            ResolvedParam::Local(m) => m,
            ResolvedParam::Transitive(s) => &s.meta,
        }
    }
}

/// Parse a `range:` string ("min..max" or inclusive "min..=max").
fn parse_param_range(s: &str) -> Option<(f64, f64)> {
    let (lo, hi) = s.split_once("..")?;
    let hi = hi.trim();
    let hi = hi.strip_prefix('=').unwrap_or(hi).trim();
    Some((lo.trim().parse().ok()?, hi.parse().ok()?))
}

/// Build the `<FeatureDef qname> -> <param name> -> ParamMeta` table for every
/// `FeatureDef` in `elements`, plus any `E230` (malformed `bindingTime:`)
/// findings raised along the way. Shared by [`parameter_binding_findings`]
/// for the local model and by [`collect_reachable_feature_params`] for each
/// peer model reached through `subConfigurations:` — a peer's own `E230`s are
/// its own model's problem (caught when that model is itself validated), so
/// callers scanning a peer discard the returned findings rather than folding
/// them into the consolidating model's report.
fn build_feature_params(elements: &[RawElement]) -> (HashMap<String, HashMap<String, ParamMeta>>, Vec<Finding>) {
    let mut findings: Vec<Finding> = Vec::new();
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
                let range = get("range").and_then(|v| v.as_str()).and_then(parse_param_range);
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
    (feature_params, findings)
}

/// Human-readable label for a `Configuration` reached by the walk below, used
/// in `E519`/`E523` messages — its `id` (or qname, if unset), suffixed with
/// the owning repo alias for a peer target.
fn config_label(cfg: &RawElement, repo_alias: Option<&str>) -> String {
    let name = cfg.frontmatter.id.clone().unwrap_or_else(|| cfg.qualified_name.clone());
    match repo_alias {
        Some(alias) => format!("{name} (in repo '{alias}')"),
        None => name,
    }
}

/// A tier's own `parameterBindings:` may already close a param that a deeper
/// tier's recursion just found reachable — checked *after* recursing into
/// `target`, so a nearer-to-the-original-querying-Configuration tier
/// (checked later, as recursion unwinds) overwrites a farther one, matching
/// `REQ-TRS-HPLE-003`'s "any one tier along the path" phrasing: whichever
/// binding is closest to the top wins the label, though only *whether* one
/// exists (`Some`/`None`) actually drives `E523`.
///
/// `tier_config` is the configuration of the model `target` lives in: a key
/// that tier wrote through one of *its own* `repoImports:` mounts is matched
/// by its peer-native form too (GH #146).
fn mark_already_bound_by(
    target: &RawElement,
    label: &str,
    tier_config: &ValidateConfig,
    out: &mut HashMap<String, HashMap<String, TransitiveParamStatus>>,
) {
    let Some(serde_yaml::Value::Mapping(m)) = target.frontmatter.effective_parameter_bindings() else {
        return;
    };
    let own_keys: HashSet<String> = m
        .keys()
        .filter_map(|k| k.as_str())
        .flat_map(|k| std::iter::once(k.to_string()).chain(unmount_binding_key(tier_config, k)))
        .collect();
    for (fname, pmap) in out.iter_mut() {
        for (pname, status) in pmap.iter_mut() {
            if own_keys.contains(&format!("{fname}.{pname}")) {
                status.already_bound_by = Some(label.to_string());
            }
        }
    }
}

/// Recursively gather `<FeatureDef qname> -> <param name> -> TransitiveParamStatus`
/// for every `FeatureDef` reachable from `cfg` through `subConfigurations:`, at
/// any depth — local targets and, transitively, each tier's own configured
/// peer repos (REQ-TRS-HPLE-002/003, `ADR-SYS-HPLE-001`). A local target's own
/// `FeatureDef`s live in the same `elements` slice as the caller (already
/// covered by the caller's own top-level [`build_feature_params`] call), so
/// this only needs to *recurse* through a local target's `subConfigurations:`
/// to reach whatever peer repos *it* consolidates — never re-walk `elements`
/// itself. A peer target's `FeatureDef`s are genuinely absent from `elements`
/// (§14's `LoadedRepo` only indexes peer qnames for existence-checking, per
/// `ADR-SYS-HPLE-001` Decision 1), so those are merged into `out` directly,
/// tagged with whether *that peer's own* `Configuration` selects the feature
/// (`selected_by_owner`) — the cross-tier stand-in for `E203`, since the
/// querying `Configuration` structurally cannot select a peer's own features.
///
/// Every hop on the way down — local or peer, whether or not it turns out to
/// be the `FeatureDef`'s actual owner — also gets a chance, via
/// [`mark_already_bound_by`] run right after its own recursive call returns,
/// to close out anything the deeper walk found still open: `REQ-TRS-HPLE-003`
/// permits *any single* tier along the path to supply a value, so a tier
/// nearer the top closing something a farther tier left open is exactly the
/// legitimate "deferral" this feature exists for — it is only a second
/// binding of the *same* parameter that's illegal (`E523`).
///
/// Resolution failures (dangling reference, wrong type, unreachable repo) are
/// silently skipped here — [`sub_configuration_findings`] already reports
/// those against `subConfigurations:` itself; this walk exists only to widen
/// what a *parameter* dotted key can resolve against, not to re-report
/// `subConfigurations:` errors a second time.
///
/// `visiting` guards against a `subConfigurations:` cycle (local or
/// cross-repo) sending this into an infinite walk; `depth` additionally caps
/// it at [`HPLE_MAX_DEPTH`], mirroring the peer-validity gate's own bound.
#[allow(clippy::too_many_arguments)]
fn collect_reachable_feature_params(
    elements: &[RawElement],
    resolver: &Resolver,
    cfg: &RawElement,
    config: &ValidateConfig,
    depth: u32,
    visiting: &mut HashSet<String>,
    out: &mut HashMap<String, HashMap<String, TransitiveParamStatus>>,
) {
    if depth > HPLE_MAX_DEPTH {
        return;
    }
    let Some(entries) = &cfg.frontmatter.sub_configurations else {
        return;
    };
    for raw_sc in entries {
        let sc = raw_sc.trim();
        if sc.is_empty() {
            continue;
        }

        // 1. Local resolution first (§14.4 order), mirroring `sub_configuration_findings`.
        if let Some(target) = resolver.resolve_ref(elements, sc) {
            if !matches!(target.frontmatter.element_type, Some(ElementType::Configuration)) {
                continue;
            }
            let key = format!("local:{}", target.qualified_name);
            if !visiting.insert(key.clone()) {
                continue; // already on the walk stack — local cycle, skip.
            }
            collect_reachable_feature_params(elements, resolver, target, config, depth + 1, visiting, out);
            mark_already_bound_by(target, &config_label(target, None), config, out);
            visiting.remove(&key);
            continue;
        }

        // 2. The peer repo `sc` names — through a `repoImports:` mount path
        // (GH #146) or, failing that, the first in declaration order whose
        // index knows it — mirroring `sub_configuration_findings`.
        if let Some((repo, peer_name)) = sub_configuration_peer(config, sc) {
            let Ok(peer_elements) = crate::walker::walk_model(&repo.model_root) else {
                continue;
            };
            let peer_resolver = Resolver::new(&peer_elements);
            let Some(target) = peer_resolver.resolve_ref(&peer_elements, &peer_name) else {
                continue;
            };
            if !matches!(target.frontmatter.element_type, Some(ElementType::Configuration)) {
                continue;
            }
            let key = format!("{}::{}", repo.model_root.display(), target.qualified_name);
            if !visiting.insert(key.clone()) {
                continue; // already on the walk stack — cross-repo cycle, skip.
            }
            let (peer_params, _peer_findings) = build_feature_params(&peer_elements);
            let peer_sel = crate::variability::canon_selection(
                &target.frontmatter.feature_selections(),
                &crate::variability::feature_id_to_qname(&peer_elements),
            );
            for (fname, pmap) in peer_params {
                let selected_by_owner = peer_sel.get(&fname).copied().unwrap_or(false);
                let entry = out.entry(fname).or_default();
                for (pname, meta) in pmap {
                    entry.entry(pname).or_insert(TransitiveParamStatus {
                        meta,
                        selected_by_owner,
                        already_bound_by: None,
                    });
                }
            }
            // Recurse using *this peer's own* configured `[repos]` — each
            // tier's peers are declared in that tier's own `.syscribe.toml`,
            // never inherited from the caller.
            // See the E518 site above: keep the local link-type vocabulary active.
            let saved_link_types = crate::link_types::active();
            let peer_config = ValidateConfig::with_model_root(repo.model_root.clone());
            crate::link_types::install(&saved_link_types);
            // The peer tier's own `repoImports:` mounts, for its own entries/keys.
            let peer_config = with_repo_mounts(&peer_elements, &peer_config);
            collect_reachable_feature_params(
                &peer_elements,
                &peer_resolver,
                target,
                &peer_config,
                depth + 1,
                visiting,
                out,
            );
            mark_already_bound_by(target, &config_label(target, Some(&repo.alias)), &peer_config, out);
            visiting.remove(&key);
        }
    }
}

/// FeatureDef parameter-binding validation (§9.7): E203–E206, E222, W027,
/// W017, plus the HPLE cross-tier extensions E519/E523. Shared by the main
/// `validate` pass and by `feature-check`, so a product line checked
/// holistically gets the same binding/range enforcement (GH #14). Dormant
/// unless at least one `FeatureDef` exists.
///
/// `config`/`resolver` extend the dotted `<FeatureDef>.<param>` lookup through
/// `subConfigurations:` (REQ-TRS-HPLE-002) — see [`collect_reachable_feature_params`].
/// The intrinsic per-parameter checks (`E204` fixed, `E205` range, `E206` enum,
/// `W027` runtime binding-time) apply identically whether the target parameter
/// is local or reached transitively. `E203` (feature not selected) stays
/// scoped to *this* `Configuration`'s own local `feature_params` — a
/// transitively-reached parameter's selection state is instead checked
/// against *its own owning tier's* selection via `E519`
/// (`PI-HPLE-BINDGUARD-001`, `REQ-TRS-HPLE-003`), the cross-tier extension of
/// the same reasoning. `E523` rejects a transitively-resolved binding that
/// some nearer tier on the path already supplies — REQ-TRS-HPLE-003's "only
/// one tier may close it" rule. The required-and-unbound `W017` sweep stays
/// scoped to local `feature_params` unchanged (whether a still-open
/// transitively-reachable parameter should escalate is `PI-HPLE-OPENPARAM-001`'s
/// completeness check, `REQ-TRS-HPLE-004` — a distinct, aggregate question
/// from any *individual* binding's legality here).
pub fn parameter_binding_findings(
    elements: &[RawElement],
    config: &ValidateConfig,
    resolver: &Resolver,
) -> Vec<Finding> {
    let mut findings: Vec<Finding> = Vec::new();
    let has_feature_def = elements
        .iter()
        .any(|e| matches!(e.frontmatter.element_type, Some(ElementType::FeatureDef)));
    if !has_feature_def {
        return findings;
    }
    let num = |v: &serde_yaml::Value| v.as_f64().or_else(|| v.as_i64().map(|i| i as f64));
    let config = &*with_repo_mounts(elements, config);

    let (feature_params, param_meta_findings) = build_feature_params(elements);
    findings.extend(param_meta_findings);

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
        // Populated lazily — only the first time a dotted key doesn't resolve
        // locally and this Configuration actually has `subConfigurations:` —
        // so a config with no hierarchy at all pays nothing extra.
        let mut transitive_params: Option<HashMap<String, HashMap<String, TransitiveParamStatus>>> = None;

        if let Some(serde_yaml::Value::Mapping(bindings)) = cfg.frontmatter.effective_parameter_bindings() {
            for (k, val) in bindings {
                let Some(path) = k.as_str() else { continue };
                let Some((feat, pname)) = path.rsplit_once('.') else {
                    findings.push(error("E222", file, &format!(
                        "parameterBindings key '{}' is not a '<FeatureDef>.<param>' path (the parameter member is separated by '.')", path)));
                    continue;
                };
                bound.insert(path.to_string());

                // Local lookup first; fall back to the subConfigurations
                // subtree (REQ-TRS-HPLE-002/003) only when this Configuration
                // actually declares one and the key isn't local.
                let local_meta = feature_params.get(feat).and_then(|p| p.get(pname));
                let is_local = feature_params.contains_key(feat);
                let transitive_status = if local_meta.is_some() {
                    None
                } else if !is_local && cfg.frontmatter.sub_configurations.is_some() {
                    let table = transitive_params.get_or_insert_with(|| {
                        let mut table = HashMap::new();
                        let mut visiting = HashSet::new();
                        collect_reachable_feature_params(
                            elements, resolver, cfg, config, 0, &mut visiting, &mut table,
                        );
                        table
                    });
                    // The table is keyed by peer-native FeatureDef qname; a key
                    // written through a `repoImports:` mount path is looked up
                    // by its translated form (GH #146).
                    let unmounted = config.unmount(feat).map(|(_, native)| native);
                    table
                        .get(feat)
                        .or_else(|| unmounted.as_ref().and_then(|n| table.get(n)))
                        .and_then(|p| p.get(pname))
                } else {
                    None
                };

                let resolved = local_meta
                    .map(ResolvedParam::Local)
                    .or_else(|| transitive_status.map(ResolvedParam::Transitive));
                let is_transitive = matches!(resolved, Some(ResolvedParam::Transitive(_)));

                let Some(resolved) = resolved else {
                    if is_local {
                        findings.push(error("E222", file, &format!(
                            "parameterBindings path '{}' references undeclared parameter '{}' on '{}'", path, pname, feat)));
                    } else {
                        findings.push(error("E222", file, &format!(
                            "parameterBindings path '{}' references unknown FeatureDef '{}'", path, feat)));
                    }
                    continue;
                };
                let meta = resolved.meta();

                // E203 (not selected) is scoped to local features only — a
                // transitively-resolved parameter is instead checked against
                // *its own owning tier's* selection via `E519` below (see
                // this function's doc comment).
                if !is_transitive && !is_selected(feat) {
                    findings.push(error("E203", file, &format!(
                        "parameterBindings binds '{}' but feature '{}' is not selected", path, feat)));
                }
                // E519/E523 (REQ-TRS-HPLE-003, PI-HPLE-BINDGUARD-001): the
                // cross-tier extension of E203, plus rejecting a double-bind
                // of something a nearer tier already closed.
                if let ResolvedParam::Transitive(status) = resolved {
                    if !status.selected_by_owner {
                        findings.push(error("E519", file, &format!(
                            "parameterBindings binds '{}' but feature '{}' is not selected by the Configuration that owns it — a cross-tier binding must target a feature the owning tier actually selects", path, feat)));
                    }
                    if let Some(who) = &status.already_bound_by {
                        findings.push(error("E523", file, &format!(
                            "parameterBindings binds '{}' but it is already bound by a nearer tier's Configuration '{}' — a parameter may be closed by only one tier along the path down to it", path, who)));
                    }
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

/// `W513` (REQ-TRS-HPLE-004, `PI-HPLE-OPENPARAM-001`): the transitive closure
/// of every `isRequired: true`, no-`default:` parameter — of every
/// `FeatureDef` actually selected anywhere in a `Configuration`'s
/// `subConfigurations:` subtree, at any depth — that remains unbound after
/// applying every `parameterBindings:` entry from that `Configuration` down
/// through every tier already resolved beneath it. Opt-in and `--deny`-
/// gateable, following the same posture as `W510`/`W511`/`W512`/`W023`/`W090`
/// — reported as a warning, **never** escalated to a hard error purely
/// because one tier's own isolated validation still finds it open (only the
/// repo actually positioned as the point of final assembly can correctly
/// decide "still open" means "genuinely incomplete" rather than "deliberately
/// deferred further up").
///
/// Reuses [`collect_reachable_feature_params`]'s walk directly — a parameter
/// only ever appears in `out` tagged `selected_by_owner` at the hop that
/// structurally owns its `FeatureDef` (a peer target's own model), and
/// `already_bound_by` already reflects every tier on the path, local or
/// peer, that supplies it via its own `parameterBindings:` — so "still open"
/// here is exactly "selected, required, no default, not runtime-bound, and
/// `already_bound_by` is still `None`, and not bound by this `Configuration`'s
/// own `parameterBindings:` either" (the walk itself never inspects `cfg`'s
/// own bindings — that check is added here). A purely local
/// `subConfigurations:` chain never contributes anything to `out` at all (see
/// that function's doc comment), so this stays silent for it too — a single,
/// shared feature model has no "some tier's job to eventually decide" concept
/// (an unbound required parameter there is already `W017`, unconditionally).
pub fn open_parameter_findings(
    elements: &[RawElement],
    config: &ValidateConfig,
    resolver: &Resolver,
) -> Vec<Finding> {
    let mut findings: Vec<Finding> = Vec::new();
    let config = &*with_repo_mounts(elements, config);
    for cfg in elements
        .iter()
        .filter(|e| matches!(e.frontmatter.element_type, Some(ElementType::Configuration)))
    {
        if cfg.frontmatter.sub_configurations.is_none() {
            continue;
        }
        let mut table: HashMap<String, HashMap<String, TransitiveParamStatus>> = HashMap::new();
        let mut visiting: HashSet<String> = HashSet::new();
        collect_reachable_feature_params(elements, resolver, cfg, config, 0, &mut visiting, &mut table);
        if table.is_empty() {
            continue; // purely local chain — see doc comment.
        }
        // Own keys in both their written and (for a mount path, GH #146)
        // peer-native forms — the table is keyed by peer-native qname.
        let own_bound: HashSet<String> = match cfg.frontmatter.effective_parameter_bindings() {
            Some(serde_yaml::Value::Mapping(m)) => m
                .keys()
                .filter_map(|k| k.as_str())
                .flat_map(|k| std::iter::once(k.to_string()).chain(unmount_binding_key(config, k)))
                .collect(),
            _ => HashSet::new(),
        };

        let mut fnames: Vec<&String> = table.keys().collect();
        fnames.sort();
        for fname in fnames {
            let pmap = &table[fname];
            let mut pnames: Vec<&String> = pmap.keys().collect();
            pnames.sort();
            for pname in pnames {
                let status = &pmap[pname];
                if !status.selected_by_owner
                    || status.meta.is_fixed
                    || !status.meta.is_required
                    || status.meta.has_default
                    || status.meta.binding_time == Some(2) // runtime — REQ-TRS-PARAM-004
                    || status.already_bound_by.is_some()
                {
                    continue;
                }
                let path = format!("{fname}.{pname}");
                if own_bound.contains(&path) {
                    continue;
                }
                findings.push(warning("W513", &cfg.file_path, &format!(
                    "required parameter '{}' of a feature selected somewhere in the subConfigurations subtree remains unbound after applying every parameterBindings: entry down through this Configuration's subtree", path)));
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
    link_prov: &crate::link_types::Provenance,
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
            for (pi, p) in parents.iter().enumerate() {
                // A `coverage = false` extending derivedFrom link does not make
                // its target a parent (REQ-TRS-LINKTYPE-006).
                if !link_prov.in_reverse_index(&cur.qualified_name, crate::link_types::BaseLink::DerivedFrom, pi) {
                    continue;
                }
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
/// `AuthoredFrom` is the legacy form (GH #131): an `allocatedFrom:` authored on
/// a non-`Allocation` *target* element. §12.9 makes `allocatedFrom` derived,
/// not authored, but older models (and the pre-#131 §12.1 table) put it on the
/// realising element, so it is still accepted — each resolved entry `S` yields
/// the edge `S → holder` — and feeds the same unified edge set, so the
/// `matrix --allocations` view, E314, W034 and the derived index all see it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllocForm {
    AllocatedTo,
    Element,
    AuthoredFrom,
}

impl AllocForm {
    /// How the form is named in the W503 redundancy message.
    fn label(self) -> &'static str {
        match self {
            AllocForm::AllocatedTo => "an allocatedTo",
            AllocForm::Element => "an Allocation element",
            AllocForm::AuthoredFrom => "an authored allocatedFrom on the target",
        }
    }
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
            // Legacy form — `allocatedFrom:` authored on a non-Allocation target
            // (GH #131; see `AllocForm::AuthoredFrom`): each source → this element.
            if let Some(ref froms) = elem.frontmatter.allocated_from {
                for from in froms {
                    if let Some(from_qn) = resolve(from) {
                        edges.push((
                            from_qn,
                            elem.qualified_name.clone(),
                            AllocForm::AuthoredFrom,
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
                        .get(serde_yaml::Value::String("allocatedFrom".into()))
                        .and_then(|v| v.as_str());
                    let to = feat
                        .get(serde_yaml::Value::String("allocatedTo".into()))
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
/// `(from_qname, to_qname)` from every authoring form (see [`AllocForm`]).
/// Consumed by `MG041`, `MG081`, `E314`, `W034`, `matrix --allocations`, and
/// the derived `allocatedFrom` index so the gates and the matrix can never
/// disagree.
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

/// Per-row integrity of one `FMEASheet` `entries:` row (GH #132, REQ-TRS-FMEA-004).
///
/// - `E923`: the row has no string `id:` (or is not a mapping), so
///   `walker::explode_fmea_entries` cannot key an `FMEAEntry` by it and drops
///   it — reported here, naming the 1-based row position and failure mode.
/// - `W928`: the row declares S, O and D **and** an explicit `rpn:` that
///   differs from `S × O × D`; the walker keeps the computed value, so the
///   authored one is silently overridden unless flagged. The factor/RPN
///   extraction mirrors the walker's exactly (`fmeaSeverity` falling back to
///   `severity`, values clamped to `u8`).
fn fmea_row_findings(file: &str, row_no: usize, row: &serde_yaml::Value) -> Vec<Finding> {
    let Some(map) = row.as_mapping() else {
        return vec![error(
            "E923",
            file,
            &format!("FMEA row {row_no} in `entries:` is not a mapping — it has no `id:` and is dropped from the analysis"),
        )];
    };
    let get = |k: &str| map.get(serde_yaml::Value::String(k.into()));
    let str_val = |k: &str| get(k).and_then(|v| v.as_str()).map(str::to_string);
    let u8_val = |k: &str| get(k).and_then(|v| v.as_u64()).map(|n| n.min(255) as u8);

    let Some(id) = str_val("id") else {
        let label = str_val("failureMode")
            .or_else(|| str_val("name"))
            .map(|l| format!(" ('{l}')"))
            .unwrap_or_default();
        return vec![error(
            "E923",
            file,
            &format!(
                "FMEA row {row_no}{label} in `entries:` has no `id:` — it cannot become an FMEAEntry \
                 and is dropped from validation and `fmea report`; give it an FM-* id"
            ),
        )];
    };

    let s = u8_val("fmeaSeverity").or_else(|| u8_val("severity"));
    let (o, d) = (u8_val("occurrence"), u8_val("detection"));
    let explicit = get("rpn").and_then(|v| v.as_u64());
    match (s, o, d, explicit) {
        (Some(s), Some(o), Some(d), Some(explicit)) => {
            let computed = s as u64 * o as u64 * d as u64;
            if explicit != computed {
                vec![warning(
                    "W928",
                    file,
                    &format!(
                        "FMEA row {row_no} ('{id}'): explicit rpn {explicit} differs from S×O×D = \
                         {s}×{o}×{d} = {computed}; the computed value {computed} is used — \
                         correct or remove `rpn:`"
                    ),
                )]
            } else {
                Vec::new()
            }
        }
        _ => Vec::new(),
    }
}

/// Configuration inheritance through `derivedFrom:` (spec §9.8/§9.11, GH #137):
/// `E234` dangling base, `E235` base is not a `Configuration`, `E236` cycle,
/// `E237` more than one base, `E215` base not `approved`/`released`. The
/// inheritance itself is materialized by the walker (`crate::config_inherit`).
pub fn configuration_inheritance_findings(elements: &[RawElement]) -> Vec<Finding> {
    use crate::config_inherit::{analyse, InheritanceProblem as P};
    let (_, problems) = analyse(elements);
    let id_of = |i: usize| {
        elements[i].frontmatter.id.clone().unwrap_or_else(|| elements[i].qualified_name.clone())
    };
    problems
        .into_iter()
        .map(|p| match p {
            P::Dangling { child, target } => error(
                "E234",
                &elements[child].file_path,
                &format!(
                    "Configuration '{}' derivedFrom '{}' does not resolve to any element in this model — a base Configuration must be local (consolidate a peer product line with subConfigurations:)",
                    id_of(child), target
                ),
            ),
            P::NotConfiguration { child, target, found } => error(
                "E235",
                &elements[child].file_path,
                &format!(
                    "Configuration '{}' derivedFrom '{}' resolves to a {} — a Configuration may only derive from another Configuration",
                    id_of(child), target, found
                ),
            ),
            P::Cycle { child, chain } => error(
                "E236",
                &elements[child].file_path,
                &format!(
                    "Configuration '{}' is on a derivedFrom inheritance cycle ({} -> {}) — it inherits nothing",
                    id_of(child), chain.join(" -> "), chain.first().cloned().unwrap_or_default()
                ),
            ),
            P::MultipleBases { child, count } => error(
                "E237",
                &elements[child].file_path,
                &format!(
                    "Configuration '{}' names {} derivedFrom bases — a Configuration derives from at most one base Configuration",
                    id_of(child), count
                ),
            ),
            P::UnreleasedBase { child, base, status } => error(
                "E215",
                &elements[child].file_path,
                &format!(
                    "Configuration '{}' derivedFrom base '{}' has status '{}' — a base Configuration must be approved or released",
                    id_of(child), base, status
                ),
            ),
        })
        .collect()
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

// ── W023 — implementedBy path existence (REQ-TRS-IFACE-002) ──────────────────

#[cfg(test)]
mod w023_implemented_by_tests {
    use super::*;
    use crate::config::ValidateConfig;
    use crate::element::{ParseIssue, RawFrontmatter};
    use std::fs;

    fn make_elem(qname: &str, yaml: &str, file_path: &str) -> RawElement {
        let fm: RawFrontmatter = serde_yaml::from_str(yaml).expect("yaml parse");
        RawElement {
            qualified_name: qname.to_string(),
            file_path: file_path.to_string(),
            frontmatter: fm,
            doc: String::new(),
            parse_issue: None::<ParseIssue>,
            derived: Default::default(),
            derive_findings: vec![],
            locale_docs: Default::default(),
        }
    }

    fn cfg_with_root(root: &std::path::Path) -> ValidateConfig {
        ValidateConfig {
            model_root: Some(root.to_path_buf()),
            repo_root: Some(root.to_path_buf()),
            ..ValidateConfig::default()
        }
    }

    fn tempdir() -> std::path::PathBuf {
        let p = std::env::temp_dir()
            .join(format!("syscribe-w023-{}-{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().subsec_nanos()));
        fs::create_dir_all(&p).unwrap();
        p
    }

    fn w023_count(findings: &[Finding]) -> usize {
        findings.iter().filter(|f| f.code == "W023").count()
    }

    // ── Part/PartDef (existing behaviour must be preserved) ───────────────────

    #[test]
    fn part_missing_path_fires_w023() {
        let dir = tempdir();
        let elem = make_elem(
            "Sys::Motor",
            "type: Part\nstatus: approved\nimplementedBy: src/motor.rs\n",
            "model/Sys/Motor.md",
        );
        let result = validate_with_config(&[elem], &cfg_with_root(&dir));
        assert_eq!(w023_count(&result.findings), 1);
    }

    #[test]
    fn part_existing_path_no_w023() {
        let dir = tempdir();
        let src = dir.join("src/motor.rs");
        fs::create_dir_all(src.parent().unwrap()).unwrap();
        fs::write(&src, "// motor").unwrap();
        let elem = make_elem(
            "Sys::Motor",
            "type: Part\nstatus: approved\nimplementedBy: src/motor.rs\n",
            "model/Sys/Motor.md",
        );
        let result = validate_with_config(&[elem], &cfg_with_root(&dir));
        assert!(w023_count(&result.findings) == 0, "unexpected W023: {:?}", result.findings);
    }

    #[test]
    fn part_draft_suppresses_w023() {
        let dir = tempdir();
        let elem = make_elem(
            "Sys::Motor",
            "type: Part\nstatus: draft\nimplementedBy: src/motor.rs\n",
            "model/Sys/Motor.md",
        );
        let result = validate_with_config(&[elem], &cfg_with_root(&dir));
        assert!(w023_count(&result.findings) == 0, "draft should suppress W023");
    }

    // ── InterfaceDef — REQ-TRS-IFACE-001 / REQ-TRS-IFACE-002 ─────────────────

    #[test]
    fn interfacedef_missing_path_fires_w023() {
        let dir = tempdir();
        let elem = make_elem(
            "Interfaces::PowerIface",
            "type: InterfaceDef\nstatus: approved\nimplementedBy: include/power_iface.h\n",
            "model/Interfaces/PowerIface.md",
        );
        let result = validate_with_config(&[elem], &cfg_with_root(&dir));
        assert_eq!(w023_count(&result.findings), 1, "expected W023 for missing path");
    }

    #[test]
    fn interfacedef_existing_path_no_w023() {
        let dir = tempdir();
        let hdr = dir.join("include/power_iface.h");
        fs::create_dir_all(hdr.parent().unwrap()).unwrap();
        fs::write(&hdr, "// power interface").unwrap();
        let elem = make_elem(
            "Interfaces::PowerIface",
            "type: InterfaceDef\nstatus: approved\nimplementedBy: include/power_iface.h\n",
            "model/Interfaces/PowerIface.md",
        );
        let result = validate_with_config(&[elem], &cfg_with_root(&dir));
        assert!(w023_count(&result.findings) == 0, "unexpected W023: {:?}", result.findings);
    }

    #[test]
    fn interfacedef_draft_suppresses_w023() {
        let dir = tempdir();
        let elem = make_elem(
            "Interfaces::PowerIface",
            "type: InterfaceDef\nstatus: draft\nimplementedBy: include/power_iface.h\n",
            "model/Interfaces/PowerIface.md",
        );
        let result = validate_with_config(&[elem], &cfg_with_root(&dir));
        assert!(w023_count(&result.findings) == 0, "draft should suppress W023");
    }

    #[test]
    fn interfacedef_remote_uri_no_w023() {
        let dir = tempdir();
        let elem = make_elem(
            "Interfaces::PowerIface",
            "type: InterfaceDef\nstatus: approved\nimplementedBy: https://example.com/power_iface.h\n",
            "model/Interfaces/PowerIface.md",
        );
        let result = validate_with_config(&[elem], &cfg_with_root(&dir));
        assert!(w023_count(&result.findings) == 0, "remote URI should not fire W023");
    }

    #[test]
    fn interfacedef_multiple_paths_reports_each_missing() {
        let dir = tempdir();
        let elem = make_elem(
            "Interfaces::PowerIface",
            "type: InterfaceDef\nstatus: approved\nimplementedBy:\n  - include/power_iface.h\n  - include/power_iface_ext.h\n",
            "model/Interfaces/PowerIface.md",
        );
        let result = validate_with_config(&[elem], &cfg_with_root(&dir));
        assert_eq!(w023_count(&result.findings), 2, "expected one W023 per missing path");
    }

    // ── Interface (instance) — same rules ────────────────────────────────────

    #[test]
    fn interface_instance_missing_path_fires_w023() {
        let dir = tempdir();
        let elem = make_elem(
            "Sys::PowerPort",
            "type: Interface\nstatus: approved\nimplementedBy: src/power_port.c\n",
            "model/Sys/PowerPort.md",
        );
        let result = validate_with_config(&[elem], &cfg_with_root(&dir));
        assert_eq!(w023_count(&result.findings), 1);
    }

    #[test]
    fn interface_instance_draft_suppresses_w023() {
        let dir = tempdir();
        let elem = make_elem(
            "Sys::PowerPort",
            "type: Interface\nstatus: draft\nimplementedBy: src/power_port.c\n",
            "model/Sys/PowerPort.md",
        );
        let result = validate_with_config(&[elem], &cfg_with_root(&dir));
        assert!(w023_count(&result.findings) == 0, "draft should suppress W023");
    }

    // ── Unrelated types are NOT affected ─────────────────────────────────────

    #[test]
    fn requirement_with_implemented_by_does_not_fire_w023() {
        let dir = tempdir();
        let elem = make_elem(
            "Reqs::SafetyReq",
            "type: Requirement\nid: REQ-SYS-001\nname: Safety\nstatus: approved\nimplementedBy: src/safety.rs\n",
            "model/Reqs/SafetyReq.md",
        );
        let result = validate_with_config(&[elem], &cfg_with_root(&dir));
        assert!(w023_count(&result.findings) == 0,
            "W023 must not fire for non-architecture element types");
    }
}

// ── PlanningItem (ADR-SYS-PLANITEM-001, REQ-TRS-PLANITEM-001) ────────────────

#[cfg(test)]
mod planning_item_tests {
    use super::*;
    use crate::config::ValidateConfig;
    use crate::element::{ParseIssue, RawFrontmatter};

    fn make_elem(qname: &str, yaml: &str, file_path: &str) -> RawElement {
        let fm: RawFrontmatter = serde_yaml::from_str(yaml).expect("yaml parse");
        RawElement {
            qualified_name: qname.to_string(),
            file_path: file_path.to_string(),
            frontmatter: fm,
            doc: String::new(),
            parse_issue: None::<ParseIssue>,
            derived: Default::default(),
            derive_findings: vec![],
            locale_docs: Default::default(),
        }
    }

    fn codes(findings: &[Finding]) -> Vec<&str> {
        findings.iter().map(|f| f.code).collect()
    }

    fn req(id: &str, qname: &str) -> RawElement {
        let mut e = make_elem(
            qname,
            &format!("type: Requirement\nid: {id}\nname: A requirement\nstatus: draft\n"),
            &format!("model/{}.md", qname.replace("::", "/")),
        );
        e.doc = "The system shall do the thing.".to_string();
        e
    }

    #[test]
    fn valid_planning_item_with_status_and_no_item_type_validates_cleanly() {
        let elements = vec![
            req("REQ-SCHED-001", "Requirements::SchedReq"),
            make_elem(
                "Planning::DoTheThing",
                "type: PlanningItem\nid: PI-SCHED-001\nname: Do the thing\nstatus: todo\nachieves: REQ-SCHED-001\n",
                "model/Planning/DoTheThing.md",
            ),
        ];
        let result = validate_with_config(&elements, &ValidateConfig::default());
        assert!(
            !codes(&result.findings).iter().any(|c| c.starts_with('E')),
            "unexpected errors: {:?}",
            result.findings
        );
    }

    #[test]
    fn valid_planning_item_with_valid_item_type_validates_cleanly() {
        let elements = vec![
            req("REQ-SCHED-002", "Requirements::SchedReq2"),
            make_elem(
                "Planning::FixTheBug",
                "type: PlanningItem\nid: PI-SCHED-002\nname: Fix the bug\nstatus: in_progress\nitemType: bug\nachieves: REQ-SCHED-002\n",
                "model/Planning/FixTheBug.md",
            ),
        ];
        let result = validate_with_config(&elements, &ValidateConfig::default());
        assert!(
            !codes(&result.findings).iter().any(|c| c.starts_with('E')),
            "unexpected errors: {:?}",
            result.findings
        );
    }

    #[test]
    fn invalid_id_shape_is_rejected() {
        let elem = make_elem(
            "Planning::BadId",
            "type: PlanningItem\nid: PI-bad-id\nname: Bad id\nstatus: todo\n",
            "model/Planning/BadId.md",
        );
        let result = validate_with_config(&[elem], &ValidateConfig::default());
        assert!(codes(&result.findings).contains(&"E706"), "expected E706: {:?}", result.findings);
    }

    #[test]
    fn missing_name_is_rejected() {
        let elem = make_elem(
            "Planning::NoName",
            "type: PlanningItem\nid: PI-SCHED-003\nstatus: todo\n",
            "model/Planning/NoName.md",
        );
        let result = validate_with_config(&[elem], &ValidateConfig::default());
        assert!(codes(&result.findings).contains(&"E707"), "expected E707: {:?}", result.findings);
    }

    #[test]
    fn missing_status_is_rejected() {
        let elem = make_elem(
            "Planning::NoStatus",
            "type: PlanningItem\nid: PI-SCHED-004\nname: No status\n",
            "model/Planning/NoStatus.md",
        );
        let result = validate_with_config(&[elem], &ValidateConfig::default());
        assert!(codes(&result.findings).contains(&"E707"), "expected E707: {:?}", result.findings);
    }

    #[test]
    fn out_of_vocabulary_status_is_rejected() {
        let elem = make_elem(
            "Planning::BadStatus",
            "type: PlanningItem\nid: PI-SCHED-005\nname: Bad status\nstatus: wontfix\n",
            "model/Planning/BadStatus.md",
        );
        let result = validate_with_config(&[elem], &ValidateConfig::default());
        assert!(codes(&result.findings).contains(&"E708"), "expected E708: {:?}", result.findings);
    }

    #[test]
    fn out_of_vocabulary_item_type_is_rejected() {
        let elem = make_elem(
            "Planning::BadItemType",
            "type: PlanningItem\nid: PI-SCHED-006\nname: Bad item type\nstatus: todo\nitemType: epic\n",
            "model/Planning/BadItemType.md",
        );
        let result = validate_with_config(&[elem], &ValidateConfig::default());
        assert!(codes(&result.findings).contains(&"E709"), "expected E709: {:?}", result.findings);
    }

    #[test]
    fn absent_item_type_validates_cleanly() {
        let elem = make_elem(
            "Planning::NoItemType",
            "type: PlanningItem\nid: PI-SCHED-007\nname: No item type\nstatus: done\n",
            "model/Planning/NoItemType.md",
        );
        let result = validate_with_config(&[elem], &ValidateConfig::default());
        assert!(
            !codes(&result.findings).contains(&"E709"),
            "absent itemType must not raise E709: {:?}",
            result.findings
        );
    }
}

// ── PlanningItem hierarchy (ADR-SYS-PLANITEM-001, REQ-TRS-PLANITEM-002) ──────

#[cfg(test)]
mod planning_item_hierarchy_tests {
    use super::*;
    use crate::config::ValidateConfig;
    use crate::element::{ParseIssue, RawFrontmatter};

    fn make_elem(qname: &str, yaml: &str, file_path: &str) -> RawElement {
        let fm: RawFrontmatter = serde_yaml::from_str(yaml).expect("yaml parse");
        RawElement {
            qualified_name: qname.to_string(),
            file_path: file_path.to_string(),
            frontmatter: fm,
            doc: String::new(),
            parse_issue: None::<ParseIssue>,
            derived: Default::default(),
            derive_findings: vec![],
            locale_docs: Default::default(),
        }
    }

    fn req(id: &str, qname: &str) -> RawElement {
        let mut e = make_elem(
            qname,
            &format!("type: Requirement\nid: {id}\nname: A requirement\nstatus: draft\n"),
            &format!("model/{}.md", qname.replace("::", "/")),
        );
        e.doc = "The system shall do the thing.".to_string();
        e
    }

    /// A `PlanningItem` with the given `parent:` (task #15). `achieves:` is left
    /// unset — callers that need a clean, error-free top-level item (no `parent`)
    /// use [`pi_ach`], which also supplies a resolvable `Requirement` companion
    /// element (REQ-TRS-PLANITEM-003's required-on-top-level rule, `E713`).
    fn pi(qname: &str, id: &str, name: &str, parent: Option<&str>) -> RawElement {
        let parent_line = parent.map(|p| format!("parent: {p}\n")).unwrap_or_default();
        make_elem(
            qname,
            &format!("type: PlanningItem\nid: {id}\nname: {name}\nstatus: todo\n{parent_line}"),
            &format!("model/{}.md", qname.replace("::", "/")),
        )
    }

    /// A top-level (no `parent`) `PlanningItem` plus a resolvable `Requirement`
    /// companion element for its `achieves:`. Returns `(planning_item, requirement)`
    /// — both must be included in the test's element slice.
    fn pi_ach(qname: &str, id: &str, name: &str, achieves_req_id: &str) -> (RawElement, RawElement) {
        let elem = make_elem(
            qname,
            &format!("type: PlanningItem\nid: {id}\nname: {name}\nstatus: todo\nachieves: {achieves_req_id}\n"),
            &format!("model/{}.md", qname.replace("::", "/")),
        );
        let req_qname = format!("{qname}AchievesReq");
        (elem, req(achieves_req_id, &req_qname))
    }

    fn codes(findings: &[Finding]) -> Vec<&str> {
        findings.iter().map(|f| f.code).collect()
    }

    #[test]
    fn three_level_chain_resolves_and_computes_children() {
        let (top, top_req) = pi_ach("Planning::Top", "PI-TOP-001", "Top", "REQ-TOP-001");
        let elements = vec![
            top,
            top_req,
            pi("Planning::Mid", "PI-MID-001", "Mid", Some("PI-TOP-001")),
            pi("Planning::Leaf", "PI-LEAF-001", "Leaf", Some("PI-MID-001")),
        ];
        let result = validate_with_config(&elements, &ValidateConfig::default());
        assert!(
            !codes(&result.findings).iter().any(|c| c.starts_with('E')),
            "unexpected errors: {:?}",
            result.findings
        );
        assert_eq!(
            result.planning_children.get("PI-TOP-001").map(|v| v.as_slice()),
            Some(["PI-MID-001".to_string()].as_slice())
        );
        assert_eq!(
            result.planning_children.get("PI-MID-001").map(|v| v.as_slice()),
            Some(["PI-LEAF-001".to_string()].as_slice())
        );
        // The deepest node has no children -> leaf.
        assert!(result.planning_children.get("PI-LEAF-001").is_none_or(|v| v.is_empty()));
    }

    #[test]
    fn parent_naming_non_planning_item_is_rejected() {
        let elements = vec![
            make_elem(
                "Requirements::SomeReq",
                "type: Requirement\nid: REQ-SYS-001\nname: Some requirement\nstatus: draft\n",
                "model/Requirements/SomeReq.md",
            ),
            pi("Planning::Child", "PI-CHILD-001", "Child", Some("REQ-SYS-001")),
        ];
        let result = validate_with_config(&elements, &ValidateConfig::default());
        assert!(codes(&result.findings).contains(&"E711"), "expected E711: {:?}", result.findings);
    }

    #[test]
    fn parent_naming_nothing_resolvable_is_rejected() {
        let elements = vec![pi("Planning::Child", "PI-CHILD-002", "Child", Some("PI-NOPE-999"))];
        let result = validate_with_config(&elements, &ValidateConfig::default());
        assert!(codes(&result.findings).contains(&"E710"), "expected E710: {:?}", result.findings);
    }

    #[test]
    fn two_node_cycle_is_detected_gracefully() {
        let elements = vec![
            pi("Planning::A", "PI-AA-001", "A", Some("PI-BB-001")),
            pi("Planning::B", "PI-BB-001", "B", Some("PI-AA-001")),
        ];
        let result = validate_with_config(&elements, &ValidateConfig::default());
        assert!(codes(&result.findings).contains(&"E712"), "expected E712: {:?}", result.findings);
    }

    #[test]
    fn three_node_cycle_is_detected_gracefully() {
        let elements = vec![
            pi("Planning::A", "PI-AA-002", "A", Some("PI-BB-002")),
            pi("Planning::B", "PI-BB-002", "B", Some("PI-CC-002")),
            pi("Planning::C", "PI-CC-002", "C", Some("PI-AA-002")),
        ];
        let result = validate_with_config(&elements, &ValidateConfig::default());
        assert!(codes(&result.findings).contains(&"E712"), "expected E712: {:?}", result.findings);
    }

    #[test]
    fn no_parent_is_top_level() {
        let (solo, solo_req) = pi_ach("Planning::Solo", "PI-SOLO-001", "Solo", "REQ-SOLO-001");
        let elements = vec![solo, solo_req];
        let result = validate_with_config(&elements, &ValidateConfig::default());
        assert!(
            !codes(&result.findings).iter().any(|c| c.starts_with('E')),
            "unexpected errors: {:?}",
            result.findings
        );
        // Top-level: the element itself carries no `parent:` (checked at the
        // frontmatter level — the map only tells us about children).
        assert!(elements[0].frontmatter.parent.is_none());
    }

    #[test]
    fn item_with_children_is_not_a_leaf() {
        let (parent, parent_req) = pi_ach("Planning::Parent", "PI-PARENT-001", "Parent", "REQ-PARENT-001");
        let elements = vec![
            parent,
            parent_req,
            pi("Planning::Child", "PI-CHILD-003", "Child", Some("PI-PARENT-001")),
        ];
        let result = validate_with_config(&elements, &ValidateConfig::default());
        let is_leaf = result.planning_children.get("PI-PARENT-001").is_none_or(|v| v.is_empty());
        assert!(!is_leaf, "a PlanningItem with a child must not be a leaf");
    }

    #[test]
    fn lone_item_with_no_parent_and_no_children_is_top_level_and_leaf() {
        let (lone, lone_req) = pi_ach("Planning::Lone", "PI-LONE-001", "Lone", "REQ-LONE-001");
        let elements = vec![lone, lone_req];
        let result = validate_with_config(&elements, &ValidateConfig::default());
        assert!(
            !codes(&result.findings).iter().any(|c| c.starts_with('E')),
            "a lone PlanningItem must validate cleanly: {:?}",
            result.findings
        );
        let is_top_level = elements[0].frontmatter.parent.is_none();
        let is_leaf = result.planning_children.get("PI-LONE-001").is_none_or(|v| v.is_empty());
        assert!(is_top_level && is_leaf, "a lone PlanningItem must be both top-level and a leaf");
    }
}

// ── PlanningItem achieves (ADR-SYS-PLANITEM-001, REQ-TRS-PLANITEM-003) ───────

#[cfg(test)]
mod planning_item_achieves_tests {
    use super::*;
    use crate::config::ValidateConfig;
    use crate::element::{ParseIssue, RawFrontmatter};

    fn make_elem(qname: &str, yaml: &str, file_path: &str) -> RawElement {
        let fm: RawFrontmatter = serde_yaml::from_str(yaml).expect("yaml parse");
        RawElement {
            qualified_name: qname.to_string(),
            file_path: file_path.to_string(),
            frontmatter: fm,
            doc: String::new(),
            parse_issue: None::<ParseIssue>,
            derived: Default::default(),
            derive_findings: vec![],
            locale_docs: Default::default(),
        }
    }

    fn req(id: &str, qname: &str, extra: &str) -> RawElement {
        let status_line = if extra.contains("status:") { "" } else { "status: draft\n" };
        let mut e = make_elem(
            qname,
            &format!("type: Requirement\nid: {id}\nname: A requirement\n{status_line}{extra}"),
            &format!("model/{}.md", qname.replace("::", "/")),
        );
        e.doc = "The system shall do the thing.".to_string();
        e
    }

    fn codes(findings: &[Finding]) -> Vec<&str> {
        findings.iter().map(|f| f.code).collect()
    }

    #[test]
    fn top_level_single_achieves_id_form_validates_cleanly() {
        let elements = vec![
            req("REQ-ACH-001", "Requirements::AchReq1", ""),
            make_elem(
                "Planning::Top",
                "type: PlanningItem\nid: PI-ACH-001\nname: Top\nstatus: todo\nachieves: REQ-ACH-001\n",
                "model/Planning/Top.md",
            ),
        ];
        let result = validate_with_config(&elements, &ValidateConfig::default());
        assert!(
            !codes(&result.findings).iter().any(|c| c.starts_with('E')),
            "unexpected errors: {:?}",
            result.findings
        );
    }

    #[test]
    fn top_level_multi_achieves_list_validates_cleanly() {
        let elements = vec![
            req("REQ-ACH-002", "Requirements::AchReq2", ""),
            req("REQ-ACH-003", "Requirements::AchReq3", ""),
            make_elem(
                "Planning::Top2",
                "type: PlanningItem\nid: PI-ACH-002\nname: Top2\nstatus: todo\nachieves: [REQ-ACH-002, REQ-ACH-003]\n",
                "model/Planning/Top2.md",
            ),
        ];
        let result = validate_with_config(&elements, &ValidateConfig::default());
        assert!(
            !codes(&result.findings).iter().any(|c| c.starts_with('E')),
            "unexpected errors: {:?}",
            result.findings
        );
    }

    #[test]
    fn top_level_empty_achieves_is_rejected() {
        let elem = make_elem(
            "Planning::EmptyAch",
            "type: PlanningItem\nid: PI-ACH-004\nname: Empty\nstatus: todo\nachieves: []\n",
            "model/Planning/EmptyAch.md",
        );
        let result = validate_with_config(&[elem], &ValidateConfig::default());
        assert!(codes(&result.findings).contains(&"E713"), "expected E713: {:?}", result.findings);
    }

    #[test]
    fn top_level_absent_achieves_is_rejected() {
        let elem = make_elem(
            "Planning::NoAch",
            "type: PlanningItem\nid: PI-ACH-005\nname: NoAch\nstatus: todo\n",
            "model/Planning/NoAch.md",
        );
        let result = validate_with_config(&[elem], &ValidateConfig::default());
        assert!(codes(&result.findings).contains(&"E713"), "expected E713: {:?}", result.findings);
    }

    #[test]
    fn non_top_level_with_no_achieves_validates_cleanly() {
        let elements = vec![
            req("REQ-ACH-006", "Requirements::AchReq6", ""),
            make_elem(
                "Planning::TopWithAch",
                "type: PlanningItem\nid: PI-ACH-006\nname: TopWithAch\nstatus: todo\nachieves: REQ-ACH-006\n",
                "model/Planning/TopWithAch.md",
            ),
            make_elem(
                "Planning::Child",
                "type: PlanningItem\nid: PI-ACH-007\nname: Child\nstatus: todo\nparent: PI-ACH-006\n",
                "model/Planning/Child.md",
            ),
        ];
        let result = validate_with_config(&elements, &ValidateConfig::default());
        assert!(
            !codes(&result.findings).iter().any(|c| c.starts_with('E')),
            "a non-top-level item with no achieves must validate cleanly: {:?}",
            result.findings
        );
    }

    #[test]
    fn dangling_achieves_target_is_rejected() {
        // Empirically confirmed (see commit history): before E714 existed, this
        // produced ZERO findings — no generic cross-reference infrastructure
        // catches a dangling `achieves:` target on its own (unlike verifies/
        // derivedFrom's E102/E103; matching satisfies:'s silence instead).
        let elem = make_elem(
            "Planning::Dangling",
            "type: PlanningItem\nid: PI-ACH-008\nname: Dangling\nstatus: todo\nachieves: REQ-NOPE-999\n",
            "model/Planning/Dangling.md",
        );
        let result = validate_with_config(&[elem], &ValidateConfig::default());
        assert!(codes(&result.findings).contains(&"E714"), "expected E714: {:?}", result.findings);
    }

    #[test]
    fn achieves_target_wrong_type_is_rejected() {
        let elements = vec![
            make_elem(
                "Planning::OtherItem",
                "type: PlanningItem\nid: PI-ACH-009\nname: OtherItem\nstatus: todo\nachieves: REQ-ACH-010\n",
                "model/Planning/OtherItem.md",
            ),
            req("REQ-ACH-010", "Requirements::AchReq10", ""),
            make_elem(
                "Planning::WrongTarget",
                "type: PlanningItem\nid: PI-ACH-011\nname: WrongTarget\nstatus: todo\nachieves: PI-ACH-009\n",
                "model/Planning/WrongTarget.md",
            ),
        ];
        let result = validate_with_config(&elements, &ValidateConfig::default());
        assert!(codes(&result.findings).contains(&"E715"), "expected E715: {:?}", result.findings);
    }

    #[test]
    fn achieves_does_not_suppress_w300_on_target_requirement() {
        // A leaf, approved Requirement named only via `achieves:` (never `satisfies:`)
        // must still raise W300 — achieves must not be treated as a satisfier.
        let elements = vec![
            req("REQ-ACH-020", "Requirements::LeafReq", "status: approved\n"),
            make_elem(
                "Planning::AchievesLeaf",
                "type: PlanningItem\nid: PI-ACH-020\nname: AchievesLeaf\nstatus: todo\nachieves: REQ-ACH-020\n",
                "model/Planning/AchievesLeaf.md",
            ),
        ];
        let result = validate_with_config(&elements, &ValidateConfig::default());
        assert!(codes(&result.findings).contains(&"W300"), "expected W300 still fires: {:?}", result.findings);
    }

    #[test]
    fn achieves_does_not_trigger_e312_on_parent_requirement() {
        // A parent Requirement (has derivedChildren) named only via `achieves:`
        // (never `satisfies:`) must NOT raise E312 -- that rule stays scoped to
        // `satisfies:` only.
        let elements = vec![
            make_elem(
                "Decisions::SomeAdr",
                "type: ADR\nid: ADR-ACH-001\nname: Some decision\nstatus: accepted\n",
                "model/Decisions/SomeAdr.md",
            ),
            req(
                "REQ-ACH-030",
                "Requirements::ParentReq",
                "breakdownAdr: ADR-ACH-001\n",
            ),
            req(
                "REQ-ACH-031",
                "Requirements::ChildReq",
                "derivedFrom: [REQ-ACH-030]\nbreakdownAdr: ADR-ACH-001\n",
            ),
            make_elem(
                "Planning::AchievesParent",
                "type: PlanningItem\nid: PI-ACH-030\nname: AchievesParent\nstatus: todo\nachieves: REQ-ACH-030\n",
                "model/Planning/AchievesParent.md",
            ),
        ];
        let result = validate_with_config(&elements, &ValidateConfig::default());
        assert!(
            !codes(&result.findings).contains(&"E312"),
            "achieves: must not trigger E312 on its target: {:?}",
            result.findings
        );
    }
}

// ── PlanningItem-scoped completion check (issue #114, REQ-TRS-PLANITEM-010) ──

#[cfg(test)]
mod planning_item_completion_w310_tests {
    use super::*;
    use crate::config::ValidateConfig;
    use crate::element::{ParseIssue, RawFrontmatter};

    fn make_elem(qname: &str, yaml: &str, file_path: &str) -> RawElement {
        let fm: RawFrontmatter = serde_yaml::from_str(yaml).expect("yaml parse");
        RawElement {
            qualified_name: qname.to_string(),
            file_path: file_path.to_string(),
            frontmatter: fm,
            doc: String::new(),
            parse_issue: None::<ParseIssue>,
            derived: Default::default(),
            derive_findings: vec![],
            locale_docs: Default::default(),
        }
    }

    fn req(id: &str, qname: &str, extra: &str) -> RawElement {
        let status_line = if extra.contains("status:") { "" } else { "status: approved\n" };
        let mut e = make_elem(
            qname,
            &format!("type: Requirement\nid: {id}\nname: A requirement\nreqDomain: software\n{status_line}{extra}"),
            &format!("model/{}.md", qname.replace("::", "/")),
        );
        e.doc = "The system shall do the thing.".to_string();
        e
    }

    fn tc(id: &str, qname: &str, verifies: &str, status: &str, test_level: &str) -> RawElement {
        make_elem(
            qname,
            &format!(
                "type: TestCase\nid: {id}\nname: A test\nstatus: {status}\ntestLevel: {test_level}\nverifies: [{verifies}]\n"
            ),
            &format!("model/{}.md", qname.replace("::", "/")),
        )
    }

    fn pi(id: &str, qname: &str, status: &str, achieves: &str) -> RawElement {
        make_elem(
            qname,
            &format!("type: PlanningItem\nid: {id}\nname: An item\nstatus: {status}\nachieves: [{achieves}]\n"),
            &format!("model/Planning/{}.md", qname.replace("::", "/")),
        )
    }

    fn codes(findings: &[Finding]) -> Vec<&str> {
        findings.iter().map(|f| f.code).collect()
    }

    #[test]
    fn done_item_achieving_a_leaf_requirement_with_no_active_testcase_raises_w310() {
        let elements = vec![
            req("REQ-W310-001", "Requirements::Leaf1", ""),
            tc("TC-W310-001", "Tests::T1", "REQ-W310-001", "draft", "L1"),
            pi("PI-W310-001", "Planning::Item1", "done", "REQ-W310-001"),
        ];
        let result = validate_with_config(&elements, &ValidateConfig::default());
        assert!(codes(&result.findings).contains(&"W310"), "expected W310: {:?}", result.findings);
    }

    #[test]
    fn done_item_achieving_a_leaf_requirement_with_an_active_testcase_raises_nothing() {
        let elements = vec![
            req("REQ-W310-002", "Requirements::Leaf2", ""),
            tc("TC-W310-002", "Tests::T2", "REQ-W310-002", "active", "L1"),
            pi("PI-W310-002", "Planning::Item2", "done", "REQ-W310-002"),
        ];
        let result = validate_with_config(&elements, &ValidateConfig::default());
        assert!(!codes(&result.findings).contains(&"W310"), "expected no W310: {:?}", result.findings);
    }

    #[test]
    fn done_item_achieving_a_parent_requirement_with_only_leaf_level_active_coverage_raises_w310() {
        // Mirrors W305's own bar: a parent Requirement needs an *integration-level*
        // (L3/L4/L5) active TestCase directly on itself -- an active L1 alone does
        // not satisfy it, even though W002's "any active TestCase" bar would.
        let elements = vec![
            req("REQ-W310-010", "Requirements::Parent3", ""),
            req(
                "REQ-W310-011",
                "Requirements::Child3",
                "derivedFrom: [REQ-W310-010]\nbreakdownAdr: ADR-W310-001\n",
            ),
            make_elem(
                "Decisions::AdrW310",
                "type: ADR\nid: ADR-W310-001\nname: Some decision\nstatus: accepted\n",
                "model/Decisions/AdrW310.md",
            ),
            tc("TC-W310-010", "Tests::T3", "REQ-W310-010", "active", "L1"),
            pi("PI-W310-003", "Planning::Item3", "done", "REQ-W310-010"),
        ];
        let result = validate_with_config(&elements, &ValidateConfig::default());
        assert!(codes(&result.findings).contains(&"W310"), "expected W310 on parent with only L1 coverage: {:?}", result.findings);
    }

    #[test]
    fn done_item_achieving_a_parent_requirement_with_active_integration_level_coverage_raises_nothing() {
        let elements = vec![
            req("REQ-W310-020", "Requirements::Parent4", ""),
            req(
                "REQ-W310-021",
                "Requirements::Child4",
                "derivedFrom: [REQ-W310-020]\nbreakdownAdr: ADR-W310-002\n",
            ),
            make_elem(
                "Decisions::AdrW310b",
                "type: ADR\nid: ADR-W310-002\nname: Some decision\nstatus: accepted\n",
                "model/Decisions/AdrW310b.md",
            ),
            tc("TC-W310-020", "Tests::T4", "REQ-W310-020", "active", "L3"),
            pi("PI-W310-004", "Planning::Item4", "done", "REQ-W310-020"),
        ];
        let result = validate_with_config(&elements, &ValidateConfig::default());
        assert!(!codes(&result.findings).contains(&"W310"), "expected no W310 on parent with L3 coverage: {:?}", result.findings);
    }

    #[test]
    fn non_done_items_are_never_checked() {
        for status in ["todo", "in_progress", "blocked"] {
            let elements = vec![
                req("REQ-W310-030", "Requirements::Leaf5", ""),
                tc("TC-W310-030", "Tests::T5", "REQ-W310-030", "draft", "L1"),
                pi("PI-W310-005", "Planning::Item5", status, "REQ-W310-030"),
            ];
            let result = validate_with_config(&elements, &ValidateConfig::default());
            assert!(
                !codes(&result.findings).contains(&"W310"),
                "status {status} must not raise W310: {:?}",
                result.findings
            );
        }
    }

    #[test]
    fn dangling_achieves_target_is_not_also_flagged_by_w310() {
        let elements = vec![pi("PI-W310-006", "Planning::Item6", "done", "REQ-W310-NONEXISTENT")];
        let result = validate_with_config(&elements, &ValidateConfig::default());
        assert!(codes(&result.findings).contains(&"E714"), "expected E714: {:?}", result.findings);
        assert!(!codes(&result.findings).contains(&"W310"), "E714 target must not also raise W310: {:?}", result.findings);
    }

    #[test]
    fn wrong_kind_achieves_target_is_not_also_flagged_by_w310() {
        let elements = vec![
            req("REQ-W310-050", "Requirements::Dummy7", ""),
            pi("PI-W310-007", "Planning::Item7", "todo", "REQ-W310-050"),
            pi("PI-W310-008", "Planning::Item8", "done", "PI-W310-007"),
        ];
        let result = validate_with_config(&elements, &ValidateConfig::default());
        assert!(codes(&result.findings).contains(&"E715"), "expected E715: {:?}", result.findings);
        assert!(!codes(&result.findings).contains(&"W310"), "E715 target must not also raise W310: {:?}", result.findings);
    }

    #[test]
    fn w310_message_names_both_the_planning_item_and_the_requirement() {
        let elements = vec![
            req("REQ-W310-040", "Requirements::Leaf6", ""),
            tc("TC-W310-040", "Tests::T6", "REQ-W310-040", "draft", "L1"),
            pi("PI-W310-009", "Planning::Item9", "done", "REQ-W310-040"),
        ];
        let result = validate_with_config(&elements, &ValidateConfig::default());
        let msg = result
            .findings
            .iter()
            .find(|f| f.code == "W310")
            .map(|f| f.message.clone())
            .expect("W310 present");
        assert!(msg.contains("PI-W310-009"), "message should name the PlanningItem: {msg}");
        assert!(msg.contains("REQ-W310-040"), "message should name the Requirement: {msg}");
    }
}

// ── PlanningItem claim-overlap check (issue #115, W311) ──────────────────────

#[cfg(test)]
mod planning_item_claim_overlap_w311_tests {
    use super::*;
    use crate::config::ValidateConfig;
    use crate::element::{ParseIssue, RawFrontmatter};

    fn make_elem(qname: &str, yaml: &str, file_path: &str) -> RawElement {
        let fm: RawFrontmatter = serde_yaml::from_str(yaml).expect("yaml parse");
        RawElement {
            qualified_name: qname.to_string(),
            file_path: file_path.to_string(),
            frontmatter: fm,
            doc: String::new(),
            parse_issue: None::<ParseIssue>,
            derived: Default::default(),
            derive_findings: vec![],
            locale_docs: Default::default(),
        }
    }

    fn req(id: &str, qname: &str) -> RawElement {
        make_elem(
            qname,
            &format!("type: Requirement\nid: {id}\nname: A requirement\nreqDomain: software\nstatus: approved\n"),
            &format!("model/{}.md", qname.replace("::", "/")),
        )
    }

    fn codes(findings: &[Finding]) -> Vec<&str> {
        findings.iter().map(|f| f.code).collect()
    }

    fn w311_messages(findings: &[Finding]) -> Vec<&str> {
        findings.iter().filter(|f| f.code == "W311").map(|f| f.message.as_str()).collect()
    }

    #[test]
    fn two_in_progress_items_sharing_an_achieves_requirement_raise_w311() {
        let elements = vec![
            req("REQ-W311-001", "Requirements::Shared1"),
            make_elem(
                "Planning::ItemA",
                "type: PlanningItem\nid: PI-W311-001\nname: A\nstatus: in_progress\nachieves: [REQ-W311-001]\n",
                "model/Planning/ItemA.md",
            ),
            make_elem(
                "Planning::ItemB",
                "type: PlanningItem\nid: PI-W311-002\nname: B\nstatus: in_progress\nachieves: [REQ-W311-001]\n",
                "model/Planning/ItemB.md",
            ),
        ];
        let result = validate_with_config(&elements, &ValidateConfig::default());
        assert!(codes(&result.findings).contains(&"W311"), "{:?}", result.findings);
        let msgs = w311_messages(&result.findings);
        assert!(msgs.iter().any(|m| m.contains("PI-W311-001") && m.contains("PI-W311-002") && m.contains("REQ-W311-001")), "{msgs:?}");
    }

    #[test]
    fn two_items_with_disjoint_achieves_raise_nothing() {
        let elements = vec![
            req("REQ-W311-010", "Requirements::Disjoint1"),
            req("REQ-W311-011", "Requirements::Disjoint2"),
            make_elem(
                "Planning::ItemC",
                "type: PlanningItem\nid: PI-W311-003\nname: C\nstatus: in_progress\nachieves: [REQ-W311-010]\n",
                "model/Planning/ItemC.md",
            ),
            make_elem(
                "Planning::ItemD",
                "type: PlanningItem\nid: PI-W311-004\nname: D\nstatus: in_progress\nachieves: [REQ-W311-011]\n",
                "model/Planning/ItemD.md",
            ),
        ];
        let result = validate_with_config(&elements, &ValidateConfig::default());
        assert!(!codes(&result.findings).contains(&"W311"), "{:?}", result.findings);
    }

    #[test]
    fn a_claimed_but_not_in_progress_item_still_counts_as_active() {
        let elements = vec![
            req("REQ-W311-020", "Requirements::Shared2"),
            make_elem(
                "Planning::ItemE",
                "type: PlanningItem\nid: PI-W311-005\nname: E\nstatus: todo\nachieves: [REQ-W311-020]\nclaimedBy: agent-1\nclaimedAt: \"2026-09-13T00:00:00Z\"\n",
                "model/Planning/ItemE.md",
            ),
            make_elem(
                "Planning::ItemF",
                "type: PlanningItem\nid: PI-W311-006\nname: F\nstatus: in_progress\nachieves: [REQ-W311-020]\n",
                "model/Planning/ItemF.md",
            ),
        ];
        let result = validate_with_config(&elements, &ValidateConfig::default());
        assert!(codes(&result.findings).contains(&"W311"), "a claimed todo item must still count as active: {:?}", result.findings);
    }

    #[test]
    fn two_todo_unclaimed_items_sharing_achieves_raise_nothing() {
        let elements = vec![
            req("REQ-W311-030", "Requirements::Shared3"),
            make_elem(
                "Planning::ItemG",
                "type: PlanningItem\nid: PI-W311-007\nname: G\nstatus: todo\nachieves: [REQ-W311-030]\n",
                "model/Planning/ItemG.md",
            ),
            make_elem(
                "Planning::ItemH",
                "type: PlanningItem\nid: PI-W311-008\nname: H\nstatus: todo\nachieves: [REQ-W311-030]\n",
                "model/Planning/ItemH.md",
            ),
        ];
        let result = validate_with_config(&elements, &ValidateConfig::default());
        assert!(!codes(&result.findings).contains(&"W311"), "{:?}", result.findings);
    }

    #[test]
    fn two_items_sharing_an_evidence_path_raise_w311() {
        let elements = vec![
            make_elem(
                "Planning::ItemI",
                "type: PlanningItem\nid: PI-W311-009\nname: I\nstatus: in_progress\nevidence:\n  - path: src/shared.rs\n",
                "model/Planning/ItemI.md",
            ),
            make_elem(
                "Planning::ItemJ",
                "type: PlanningItem\nid: PI-W311-010\nname: J\nstatus: in_progress\nevidence:\n  - path: src/shared.rs\n",
                "model/Planning/ItemJ.md",
            ),
        ];
        let result = validate_with_config(&elements, &ValidateConfig::default());
        let msgs = w311_messages(&result.findings);
        assert!(msgs.iter().any(|m| m.contains("evidence.path") && m.contains("src/shared.rs")), "{msgs:?}");
    }

    #[test]
    fn w311_fires_once_per_pair_not_once_per_side() {
        let elements = vec![
            req("REQ-W311-040", "Requirements::Shared4"),
            make_elem(
                "Planning::ItemK",
                "type: PlanningItem\nid: PI-W311-011\nname: K\nstatus: in_progress\nachieves: [REQ-W311-040]\n",
                "model/Planning/ItemK.md",
            ),
            make_elem(
                "Planning::ItemL",
                "type: PlanningItem\nid: PI-W311-012\nname: L\nstatus: in_progress\nachieves: [REQ-W311-040]\n",
                "model/Planning/ItemL.md",
            ),
        ];
        let result = validate_with_config(&elements, &ValidateConfig::default());
        let count = result.findings.iter().filter(|f| f.code == "W311").count();
        assert_eq!(count, 1, "expected exactly one W311 for the pair, got {:?}", result.findings);
    }
}

// ── PlanningItem evidence (ADR-SYS-PLANITEM-001, REQ-TRS-PLANITEM-005) ───────

#[cfg(test)]
mod planning_item_evidence_tests {
    use super::*;
    use crate::config::ValidateConfig;
    use crate::element::{ParseIssue, RawFrontmatter};
    use std::fs;

    fn make_elem(qname: &str, yaml: &str, file_path: &str) -> RawElement {
        let fm: RawFrontmatter = serde_yaml::from_str(yaml).expect("yaml parse");
        RawElement {
            qualified_name: qname.to_string(),
            file_path: file_path.to_string(),
            frontmatter: fm,
            doc: String::new(),
            parse_issue: None::<ParseIssue>,
            derived: Default::default(),
            derive_findings: vec![],
            locale_docs: Default::default(),
        }
    }

    fn req(id: &str, qname: &str) -> RawElement {
        let mut e = make_elem(
            qname,
            &format!("type: Requirement\nid: {id}\nname: A requirement\nstatus: draft\n"),
            &format!("model/{}.md", qname.replace("::", "/")),
        );
        e.doc = "The system shall do the thing.".to_string();
        e
    }

    /// A minimally-valid top-level PlanningItem (satisfies E713 via a
    /// companion, always-resolvable `achieves:` Requirement) carrying the
    /// given raw `evidence:` YAML block.
    fn pi_with_evidence(qname: &str, id: &str, achieves_req_id: &str, evidence_yaml: &str) -> RawElement {
        make_elem(
            qname,
            &format!(
                "type: PlanningItem\nid: {id}\nname: PI\nstatus: todo\nachieves: {achieves_req_id}\n{evidence_yaml}"
            ),
            &format!("model/{}.md", qname.replace("::", "/")),
        )
    }

    fn codes(findings: &[Finding]) -> Vec<&str> {
        findings.iter().map(|f| f.code).collect()
    }

    fn cfg_with_root(root: &std::path::Path) -> ValidateConfig {
        ValidateConfig {
            model_root: Some(root.to_path_buf()),
            repo_root: Some(root.to_path_buf()),
            ..ValidateConfig::default()
        }
    }

    fn tempdir() -> std::path::PathBuf {
        let p = std::env::temp_dir().join(format!(
            "syscribe-planitem-evidence-{}-{}",
            std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().subsec_nanos()
        ));
        fs::create_dir_all(&p).unwrap();
        p
    }

    #[test]
    fn ref_entries_of_different_kinds_validate_cleanly() {
        // Prove ref: is genuinely unrestricted by kind (ADR-SYS-PLANITEM-001
        // Decision 3): a Part, a TestCase, another PlanningItem, and an ADR all
        // resolve cleanly with zero type gating.
        let elements = vec![
            req("REQ-EV-001", "Requirements::EvReq1"),
            make_elem("Arch::SomePart", "type: Part\nname: SomePart\n", "model/Arch/SomePart.md"),
            {
                let mut e = make_elem(
                    "Verification::SomeTest",
                    "type: TestCase\nid: TC-EV-001\nname: A test\nstatus: draft\ntestLevel: L1\nverifies: [REQ-EV-001]\n",
                    "model/Verification/SomeTest.md",
                );
                e.doc = "```gherkin\nFeature: A test\nScenario: it works\n  Given a thing\n  When it happens\n  Then it works\n```\n".to_string();
                e
            },
            make_elem(
                "Decisions::SomeAdr",
                "type: ADR\nid: ADR-EV-001\nname: Some decision\nstatus: accepted\n",
                "model/Decisions/SomeAdr.md",
            ),
            make_elem(
                // status: in_progress (not done) -- this element's own leaf/
                // evidence state is incidental to this test, which only cares
                // that it's a valid ref: target of kind PlanningItem. `done`
                // would additionally trip REQ-TRS-PLANITEM-006's E719.
                "Planning::OtherItem",
                "type: PlanningItem\nid: PI-EV-002\nname: Other\nstatus: in_progress\nparent: PI-EV-001\n",
                "model/Planning/OtherItem.md",
            ),
            pi_with_evidence(
                "Planning::MainItem",
                "PI-EV-001",
                "REQ-EV-001",
                "evidence:\n  - ref: Arch::SomePart\n  - ref: TC-EV-001\n  - ref: ADR-EV-001\n  - ref: PI-EV-002\n",
            ),
        ];
        let result = validate_with_config(&elements, &ValidateConfig::default());
        assert!(
            !codes(&result.findings).iter().any(|c| c.starts_with('E')),
            "unexpected errors: {:?}",
            result.findings
        );
    }

    #[test]
    fn unresolved_ref_without_rationale_is_rejected() {
        let elements = vec![
            req("REQ-EV-010", "Requirements::EvReq10"),
            pi_with_evidence(
                "Planning::Item10",
                "PI-EV-010",
                "REQ-EV-010",
                "evidence:\n  - ref: PI-NOPE-999\n",
            ),
        ];
        let result = validate_with_config(&elements, &ValidateConfig::default());
        assert!(codes(&result.findings).contains(&"E716"), "expected E716: {:?}", result.findings);
    }

    #[test]
    fn unresolved_ref_with_rationale_is_waived() {
        let elements = vec![
            req("REQ-EV-011", "Requirements::EvReq11"),
            pi_with_evidence(
                "Planning::Item11",
                "PI-EV-011",
                "REQ-EV-011",
                "evidence:\n  - ref: PI-NOPE-999\n    rationale: tracked externally, not yet in the model\n",
            ),
        ];
        let result = validate_with_config(&elements, &ValidateConfig::default());
        assert!(
            !codes(&result.findings).contains(&"E716"),
            "a rationale-carrying entry must waive the check: {:?}",
            result.findings
        );
    }

    #[test]
    fn path_entry_pointing_at_a_real_file_validates_cleanly() {
        let dir = tempdir();
        fs::write(dir.join("proof.md"), "proof").unwrap();
        let elements = vec![
            req("REQ-EV-020", "Requirements::EvReq20"),
            pi_with_evidence(
                "Planning::Item20",
                "PI-EV-020",
                "REQ-EV-020",
                "evidence:\n  - path: proof.md\n",
            ),
        ];
        let result = validate_with_config(&elements, &cfg_with_root(&dir));
        assert!(
            !codes(&result.findings).contains(&"E717"),
            "unexpected E717 for an existing path: {:?}",
            result.findings
        );
    }

    #[test]
    fn path_entry_pointing_at_a_missing_file_without_rationale_is_rejected() {
        let dir = tempdir();
        let elements = vec![
            req("REQ-EV-021", "Requirements::EvReq21"),
            pi_with_evidence(
                "Planning::Item21",
                "PI-EV-021",
                "REQ-EV-021",
                "evidence:\n  - path: does-not-exist.md\n",
            ),
        ];
        let result = validate_with_config(&elements, &cfg_with_root(&dir));
        assert!(codes(&result.findings).contains(&"E717"), "expected E717: {:?}", result.findings);
    }

    #[test]
    fn path_entry_pointing_at_a_missing_file_with_rationale_is_waived() {
        let dir = tempdir();
        let elements = vec![
            req("REQ-EV-022", "Requirements::EvReq22"),
            pi_with_evidence(
                "Planning::Item22",
                "PI-EV-022",
                "REQ-EV-022",
                "evidence:\n  - path: does-not-exist.md\n    rationale: proof lives in an external system, path is a placeholder\n",
            ),
        ];
        let result = validate_with_config(&elements, &cfg_with_root(&dir));
        assert!(
            !codes(&result.findings).contains(&"E717"),
            "a rationale-carrying entry must waive the check: {:?}",
            result.findings
        );
    }

    #[test]
    fn remote_uri_path_entry_is_accepted_without_local_check() {
        let dir = tempdir();
        let elements = vec![
            req("REQ-EV-023", "Requirements::EvReq23"),
            pi_with_evidence(
                "Planning::Item23",
                "PI-EV-023",
                "REQ-EV-023",
                "evidence:\n  - path: https://example.com/proof.pdf\n",
            ),
        ];
        let result = validate_with_config(&elements, &cfg_with_root(&dir));
        assert!(
            !codes(&result.findings).contains(&"E717"),
            "a remote URI must not be checked locally: {:?}",
            result.findings
        );
    }

    #[test]
    fn waiver_is_per_entry_not_blanket() {
        // Two problem entries in one list: the first is rationale-waived, the
        // second is a genuinely unresolved ref with no rationale. Only the
        // second must be flagged -- proving the waiver doesn't leak across
        // entries.
        let dir = tempdir();
        let elements = vec![
            req("REQ-EV-030", "Requirements::EvReq30"),
            pi_with_evidence(
                "Planning::Item30",
                "PI-EV-030",
                "REQ-EV-030",
                "evidence:\n  - path: does-not-exist.md\n    rationale: placeholder, tracked externally\n  - ref: PI-NOPE-ALSO-999\n",
            ),
        ];
        let result = validate_with_config(&elements, &cfg_with_root(&dir));
        let cs = codes(&result.findings);
        assert!(!cs.contains(&"E717"), "the waived path entry must not be flagged: {:?}", result.findings);
        assert!(cs.contains(&"E716"), "the unwaived ref entry must still be flagged: {:?}", result.findings);
    }
}

// ── Argument.evidence regression (review of 96b0bba/d6d8dc8) ────────────────
//
// Broadening the shared `evidence` field to `Vec<serde_yaml::Value>` (for
// PlanningItem's ref:/path:/rationale: mappings) meant a non-scalar
// Argument.evidence: entry, which previously failed the whole file's YAML
// parse outright (E002, back when the field was Vec<String>), now parses
// cleanly but silently vanished from every consumer (validate, safety-case,
// suspect list) with zero diagnostic. E718 closes that gap at the source
// (validate) — the other consumers intentionally keep filtering silently,
// since validate is where malformed-frontmatter diagnostics belong.

#[cfg(test)]
mod argument_evidence_regression_tests {
    use super::*;
    use crate::config::ValidateConfig;
    use crate::element::{ParseIssue, RawFrontmatter};

    fn make_elem(qname: &str, yaml: &str, file_path: &str) -> RawElement {
        let fm: RawFrontmatter = serde_yaml::from_str(yaml).expect("yaml parse");
        RawElement {
            qualified_name: qname.to_string(),
            file_path: file_path.to_string(),
            frontmatter: fm,
            doc: String::new(),
            parse_issue: None::<ParseIssue>,
            derived: Default::default(),
            derive_findings: vec![],
            locale_docs: Default::default(),
        }
    }

    fn codes(findings: &[Finding]) -> Vec<&str> {
        findings.iter().map(|f| f.code).collect()
    }

    #[test]
    fn nonscalar_argument_evidence_entry_raises_e718() {
        // The exact adversarial fixture from the review: a mix of a real scalar
        // ref and a PlanningItem-shaped mapping entry in one Argument.evidence:.
        let elements = vec![
            {
                let mut e = make_elem(
                    "Requirements::SomeReq",
                    "type: Requirement\nid: REQ-ARGEV-001\nname: Some requirement\nstatus: draft\n",
                    "model/Requirements/SomeReq.md",
                );
                e.doc = "The system shall do the thing.".to_string();
                e
            },
            make_elem(
                "Safety::SomeArgument",
                "type: Argument\nid: ARG-EV-001\nname: Some argument\nstatus: draft\nevidence:\n  - REQ-ARGEV-001\n  - ref: NotAScalar\n    rationale: \"planning-item-shaped entry, must not silently vanish\"\n",
                "model/Safety/SomeArgument.md",
            ),
        ];
        let result = validate_with_config(&elements, &ValidateConfig::default());
        assert!(
            codes(&result.findings).contains(&"E718"),
            "expected E718 for the non-scalar evidence entry: {:?}",
            result.findings
        );
        // The real scalar entry must still resolve cleanly -- E718 doesn't
        // blanket-suppress the rest of the list's normal checks.
        assert!(
            !codes(&result.findings).contains(&"E855"),
            "the valid scalar entry must not spuriously raise E855: {:?}",
            result.findings
        );
    }

    #[test]
    fn all_scalar_argument_evidence_validates_cleanly() {
        // Regression guard the other way: a normal, all-scalar evidence: list
        // must not spuriously raise E718.
        let elements = vec![
            {
                let mut e = make_elem(
                    "Requirements::SomeReq2",
                    "type: Requirement\nid: REQ-ARGEV-002\nname: Some requirement\nstatus: draft\n",
                    "model/Requirements/SomeReq2.md",
                );
                e.doc = "The system shall do the thing.".to_string();
                e
            },
            make_elem(
                "Safety::SomeArgument2",
                "type: Argument\nid: ARG-EV-002\nname: Some argument\nstatus: draft\nevidence: REQ-ARGEV-002\n",
                "model/Safety/SomeArgument2.md",
            ),
        ];
        let result = validate_with_config(&elements, &ValidateConfig::default());
        assert!(
            !codes(&result.findings).contains(&"E718"),
            "an all-scalar evidence: list must not raise E718: {:?}",
            result.findings
        );
    }
}

// ── PlanningItem leaf-evidence rule (ADR-SYS-PLANITEM-001, REQ-TRS-PLANITEM-006) ──

#[cfg(test)]
mod planning_item_leaf_evidence_tests {
    use super::*;
    use crate::config::ValidateConfig;
    use crate::element::{ParseIssue, RawFrontmatter};

    fn make_elem(qname: &str, yaml: &str, file_path: &str) -> RawElement {
        let fm: RawFrontmatter = serde_yaml::from_str(yaml).expect("yaml parse");
        RawElement {
            qualified_name: qname.to_string(),
            file_path: file_path.to_string(),
            frontmatter: fm,
            doc: String::new(),
            parse_issue: None::<ParseIssue>,
            derived: Default::default(),
            derive_findings: vec![],
            locale_docs: Default::default(),
        }
    }

    fn req(id: &str, qname: &str) -> RawElement {
        let mut e = make_elem(
            qname,
            &format!("type: Requirement\nid: {id}\nname: A requirement\nstatus: draft\n"),
            &format!("model/{}.md", qname.replace("::", "/")),
        );
        e.doc = "The system shall do the thing.".to_string();
        e
    }

    /// A top-level (no `parent:`) leaf PlanningItem with the given `status:`
    /// and raw `evidence:` YAML block, plus a companion, always-resolvable
    /// `achieves:` Requirement (so E713 never interferes with these tests).
    fn leaf_pi(qname: &str, id: &str, achieves_req_id: &str, status: &str, evidence_yaml: &str) -> RawElement {
        make_elem(
            qname,
            &format!(
                "type: PlanningItem\nid: {id}\nname: Leaf\nstatus: {status}\nachieves: {achieves_req_id}\n{evidence_yaml}"
            ),
            &format!("model/{}.md", qname.replace("::", "/")),
        )
    }

    fn codes(findings: &[Finding]) -> Vec<&str> {
        findings.iter().map(|f| f.code).collect()
    }

    #[test]
    fn leaf_done_with_resolving_evidence_validates_cleanly() {
        let elements = vec![
            req("REQ-LE-001", "Requirements::LeReq1"),
            leaf_pi(
                "Planning::Leaf1",
                "PI-LE-001",
                "REQ-LE-001",
                "done",
                "evidence:\n  - ref: REQ-LE-001\n",
            ),
        ];
        let result = validate_with_config(&elements, &ValidateConfig::default());
        assert!(
            !codes(&result.findings).contains(&"E719"),
            "unexpected E719: {:?}",
            result.findings
        );
    }

    #[test]
    fn leaf_done_with_no_evidence_is_rejected() {
        let elements = vec![
            req("REQ-LE-002", "Requirements::LeReq2"),
            leaf_pi("Planning::Leaf2", "PI-LE-002", "REQ-LE-002", "done", ""),
        ];
        let result = validate_with_config(&elements, &ValidateConfig::default());
        assert!(codes(&result.findings).contains(&"E719"), "expected E719: {:?}", result.findings);
    }

    #[test]
    fn leaf_done_with_only_waived_evidence_is_still_rejected() {
        // Every entry carries rationale: -- none of them counts as proof, so
        // this must still fail even though the list is non-empty.
        let elements = vec![
            req("REQ-LE-003", "Requirements::LeReq3"),
            leaf_pi(
                "Planning::Leaf3",
                "PI-LE-003",
                "REQ-LE-003",
                "done",
                "evidence:\n  - ref: PI-NOPE-999\n    rationale: not tracked in model yet\n  - path: does-not-exist.md\n    rationale: external artifact placeholder\n",
            ),
        ];
        let result = validate_with_config(&elements, &ValidateConfig::default());
        assert!(
            codes(&result.findings).contains(&"E719"),
            "a waived-only evidence list must not satisfy the rule: {:?}",
            result.findings
        );
    }

    #[test]
    fn leaf_in_non_done_statuses_with_no_evidence_raises_nothing() {
        // (status, id-safe tag) -- REQ-*/PI-* ids require uppercase-alnum
        // segments, so the raw status string (e.g. "in_progress") can't be
        // used directly in an id.
        for (status, tag) in [("todo", "TODO"), ("in_progress", "INPROG"), ("blocked", "BLK")] {
            let elements = vec![
                req(&format!("REQ-LE-{tag}-001"), &format!("Requirements::LeReq{tag}")),
                leaf_pi(
                    &format!("Planning::Leaf{tag}"),
                    &format!("PI-LE-{tag}-001"),
                    &format!("REQ-LE-{tag}-001"),
                    status,
                    "",
                ),
            ];
            let result = validate_with_config(&elements, &ValidateConfig::default());
            assert!(
                !codes(&result.findings).contains(&"E719"),
                "status '{}' must not raise E719 regardless of missing evidence: {:?}",
                status,
                result.findings
            );
        }
    }

    #[test]
    fn non_leaf_done_with_no_evidence_raises_nothing() {
        // A parent PlanningItem (has a child) is not constrained by this rule
        // at all -- status: done and zero evidence of its own must not fire.
        let elements = vec![
            req("REQ-LE-010", "Requirements::LeReq10"),
            leaf_pi("Planning::Parent10", "PI-LE-010", "REQ-LE-010", "done", ""),
            make_elem(
                "Planning::Child10",
                "type: PlanningItem\nid: PI-LE-011\nname: Child\nstatus: todo\nparent: PI-LE-010\n",
                "model/Planning/Child10.md",
            ),
        ];
        let result = validate_with_config(&elements, &ValidateConfig::default());
        assert!(
            !codes(&result.findings).contains(&"E719"),
            "a non-leaf must never be constrained by this rule: {:?}",
            result.findings
        );
    }
}

// ── PlanningItem blockedBy (ADR-SYS-PLANITEM-001 addendum, REQ-TRS-PLANITEM-007) ──

#[cfg(test)]
mod planning_item_blocked_by_tests {
    use super::*;
    use crate::config::ValidateConfig;
    use crate::element::{ParseIssue, RawFrontmatter};

    fn make_elem(qname: &str, yaml: &str, file_path: &str) -> RawElement {
        let fm: RawFrontmatter = serde_yaml::from_str(yaml).expect("yaml parse");
        RawElement {
            qualified_name: qname.to_string(),
            file_path: file_path.to_string(),
            frontmatter: fm,
            doc: String::new(),
            parse_issue: None::<ParseIssue>,
            derived: Default::default(),
            derive_findings: vec![],
            locale_docs: Default::default(),
        }
    }

    fn req(id: &str, qname: &str) -> RawElement {
        let mut e = make_elem(
            qname,
            &format!("type: Requirement\nid: {id}\nname: A requirement\nstatus: draft\n"),
            &format!("model/{}.md", qname.replace("::", "/")),
        );
        e.doc = "The system shall do the thing.".to_string();
        e
    }

    /// A top-level (no `parent:`) PlanningItem with the given `status:` and raw
    /// `blockedBy:` YAML line, plus a companion, always-resolvable `achieves:`
    /// Requirement (so E713 never interferes with these tests).
    fn pi(qname: &str, id: &str, achieves_req_id: &str, status: &str, blocked_by_yaml: &str) -> RawElement {
        make_elem(
            qname,
            &format!(
                "type: PlanningItem\nid: {id}\nname: Item\nstatus: {status}\nachieves: {achieves_req_id}\n{blocked_by_yaml}"
            ),
            &format!("model/{}.md", qname.replace("::", "/")),
        )
    }

    fn codes(findings: &[Finding]) -> Vec<&str> {
        findings.iter().map(|f| f.code).collect()
    }

    #[test]
    fn resolving_blocked_by_on_another_planning_item_validates_cleanly() {
        let elements = vec![
            req("REQ-BB-001", "Requirements::BbReq1"),
            req("REQ-BB-002", "Requirements::BbReq2"),
            pi("Planning::Blocker", "PI-BB-001", "REQ-BB-001", "todo", ""),
            pi(
                "Planning::Blocked",
                "PI-BB-002",
                "REQ-BB-002",
                "blocked",
                "blockedBy: PI-BB-001\n",
            ),
        ];
        let result = validate_with_config(&elements, &ValidateConfig::default());
        assert!(
            !codes(&result.findings).iter().any(|c| c.starts_with('E')),
            "unexpected errors: {:?}",
            result.findings
        );
        assert!(!codes(&result.findings).contains(&"W308"), "{:?}", result.findings);
    }

    #[test]
    fn blocked_by_an_unresolved_target_is_e720() {
        let elements = vec![
            req("REQ-BB-003", "Requirements::BbReq3"),
            pi(
                "Planning::Blocked3",
                "PI-BB-003",
                "REQ-BB-003",
                "blocked",
                "blockedBy: PI-DOES-NOT-EXIST-999\n",
            ),
        ];
        let result = validate_with_config(&elements, &ValidateConfig::default());
        assert!(codes(&result.findings).contains(&"E720"), "expected E720: {:?}", result.findings);
    }

    #[test]
    fn blocked_by_a_non_planning_item_still_resolves_cleanly() {
        // Permissive, unrestricted-by-kind resolution (matches evidence.ref:) --
        // an undecided ADR (or any other element) is a legitimate blocker, not
        // just another PlanningItem. No E715-style "wrong kind" check exists
        // for blockedBy:.
        let elements = vec![
            req("REQ-BB-004", "Requirements::BbReq4"),
            make_elem(
                "Decisions::PendingAdr",
                "type: ADR\nid: ADR-BB-001\nname: Pending decision\nstatus: proposed\n",
                "model/Decisions/PendingAdr.md",
            ),
            pi(
                "Planning::Blocked4",
                "PI-BB-004",
                "REQ-BB-004",
                "blocked",
                "blockedBy: ADR-BB-001\n",
            ),
        ];
        let result = validate_with_config(&elements, &ValidateConfig::default());
        assert!(
            !codes(&result.findings).iter().any(|c| c.starts_with('E')),
            "unexpected errors: {:?}",
            result.findings
        );
    }

    #[test]
    fn two_node_blocked_by_cycle_is_e721() {
        let elements = vec![
            req("REQ-BB-005", "Requirements::BbReq5"),
            req("REQ-BB-006", "Requirements::BbReq6"),
            pi(
                "Planning::CycleA",
                "PI-BB-005",
                "REQ-BB-005",
                "blocked",
                "blockedBy: PI-BB-006\n",
            ),
            pi(
                "Planning::CycleB",
                "PI-BB-006",
                "REQ-BB-006",
                "blocked",
                "blockedBy: PI-BB-005\n",
            ),
        ];
        let result = validate_with_config(&elements, &ValidateConfig::default());
        assert!(codes(&result.findings).contains(&"E721"), "expected E721: {:?}", result.findings);
    }

    #[test]
    fn self_blocked_by_is_e721() {
        let elements = vec![
            req("REQ-BB-007", "Requirements::BbReq7"),
            pi(
                "Planning::SelfBlocked",
                "PI-BB-007",
                "REQ-BB-007",
                "blocked",
                "blockedBy: PI-BB-007\n",
            ),
        ];
        let result = validate_with_config(&elements, &ValidateConfig::default());
        assert!(codes(&result.findings).contains(&"E721"), "expected E721: {:?}", result.findings);
    }

    #[test]
    fn non_empty_blocked_by_while_not_blocked_is_w308() {
        let elements = vec![
            req("REQ-BB-008", "Requirements::BbReq8"),
            req("REQ-BB-009", "Requirements::BbReq9"),
            pi("Planning::Blocker8", "PI-BB-008", "REQ-BB-008", "todo", ""),
            pi(
                "Planning::Stale9",
                "PI-BB-009",
                "REQ-BB-009",
                "in_progress",
                "blockedBy: PI-BB-008\n",
            ),
        ];
        let result = validate_with_config(&elements, &ValidateConfig::default());
        assert!(codes(&result.findings).contains(&"W308"), "expected W308: {:?}", result.findings);
        // Never escalated to an error just because the status looks stale.
        assert!(
            !codes(&result.findings).iter().any(|c| c.starts_with('E')),
            "unexpected errors: {:?}",
            result.findings
        );
    }

    #[test]
    fn status_blocked_with_empty_blocked_by_raises_nothing() {
        let elements = vec![
            req("REQ-BB-010", "Requirements::BbReq10"),
            pi("Planning::Blocked10", "PI-BB-010", "REQ-BB-010", "blocked", ""),
        ];
        let result = validate_with_config(&elements, &ValidateConfig::default());
        assert!(
            !codes(&result.findings).iter().any(|c| c == &"E720" || c == &"W308"),
            "blocked with no blockedBy: is a legitimate transient state: {:?}",
            result.findings
        );
    }
}

// ── PlanningItem assignedTo (REQ-TRS-PLANITEM-008) ──────────────────────────

#[cfg(test)]
mod planning_item_assigned_to_tests {
    use super::*;
    use crate::config::ValidateConfig;
    use crate::element::{ParseIssue, RawFrontmatter};

    fn make_elem(qname: &str, yaml: &str, file_path: &str) -> RawElement {
        let fm: RawFrontmatter = serde_yaml::from_str(yaml).expect("yaml parse");
        RawElement {
            qualified_name: qname.to_string(),
            file_path: file_path.to_string(),
            frontmatter: fm,
            doc: String::new(),
            parse_issue: None::<ParseIssue>,
            derived: Default::default(),
            derive_findings: vec![],
            locale_docs: Default::default(),
        }
    }

    fn req(id: &str, qname: &str) -> RawElement {
        let mut e = make_elem(
            qname,
            &format!("type: Requirement\nid: {id}\nname: A requirement\nstatus: draft\n"),
            &format!("model/{}.md", qname.replace("::", "/")),
        );
        e.doc = "The system shall do the thing.".to_string();
        e
    }

    /// A top-level (no `parent:`) PlanningItem with the given `assignedTo:`
    /// (raw YAML value, unquoted), plus a companion, always-resolvable
    /// `achieves:` Requirement (so E713 never interferes with these tests).
    fn pi(qname: &str, id: &str, achieves_req_id: &str, assigned_to: &str) -> RawElement {
        make_elem(
            qname,
            &format!(
                "type: PlanningItem\nid: {id}\nname: Item\nstatus: todo\nachieves: {achieves_req_id}\nassignedTo: {assigned_to}\n"
            ),
            &format!("model/{}.md", qname.replace("::", "/")),
        )
    }

    fn users(ids: &[&str]) -> ValidateConfig {
        ValidateConfig {
            users: ids.iter().map(|s| (s.to_string(), format!("{s} display name"))).collect(),
            ..ValidateConfig::default()
        }
    }

    fn codes(findings: &[Finding]) -> Vec<&str> {
        findings.iter().map(|f| f.code).collect()
    }

    #[test]
    fn assigned_to_a_declared_user_validates_cleanly() {
        let elements = vec![
            req("REQ-AT-001", "Requirements::AtReq1"),
            pi("Planning::Assigned1", "PI-AT-001", "REQ-AT-001", "alice"),
        ];
        let result = validate_with_config(&elements, &users(&["alice", "bob"]));
        assert!(!codes(&result.findings).contains(&"E722"), "{:?}", result.findings);
    }

    #[test]
    fn assigned_to_an_undeclared_user_is_e722() {
        let elements = vec![
            req("REQ-AT-002", "Requirements::AtReq2"),
            pi("Planning::Assigned2", "PI-AT-002", "REQ-AT-002", "mallory"),
        ];
        let result = validate_with_config(&elements, &users(&["alice", "bob"]));
        assert!(codes(&result.findings).contains(&"E722"), "expected E722: {:?}", result.findings);
    }

    #[test]
    fn assigned_to_is_dormant_when_users_is_not_configured() {
        // No [users] table at all (default ValidateConfig has an empty roster)
        // -- assignedTo: is accepted unchecked (for roster membership;
        // format is still always checked -- "anyone-at-all" is valid format).
        let elements = vec![
            req("REQ-AT-003", "Requirements::AtReq3"),
            pi("Planning::Assigned3", "PI-AT-003", "REQ-AT-003", "anyone-at-all"),
        ];
        let result = validate_with_config(&elements, &ValidateConfig::default());
        assert!(
            !codes(&result.findings).iter().any(|c| c == &"E722" || c == &"E723"),
            "{:?}",
            result.findings
        );
    }

    #[test]
    fn assigned_to_a_malformed_username_is_e723_regardless_of_roster() {
        // Uppercase, spaces -- not a Unix-style username. Checked even with
        // no [users] table configured at all (format is an intrinsic
        // constraint, not a roster-membership one).
        let elements = vec![
            req("REQ-AT-005", "Requirements::AtReq5"),
            pi("Planning::Assigned5", "PI-AT-005", "REQ-AT-005", "\"Alice Nakamura\""),
        ];
        let result = validate_with_config(&elements, &ValidateConfig::default());
        assert!(codes(&result.findings).contains(&"E723"), "expected E723: {:?}", result.findings);
        // Not doubled up with E722 for the same underlying defect.
        assert!(!codes(&result.findings).contains(&"E722"), "{:?}", result.findings);
    }

    #[test]
    fn malformed_users_key_is_w309_and_excluded_from_the_roster() {
        let elements = vec![
            req("REQ-AT-006", "Requirements::AtReq6"),
            pi("Planning::Assigned6", "PI-AT-006", "REQ-AT-006", "alice"),
        ];
        let mut cfg = users(&["alice"]);
        cfg.users.insert("Not-A-Valid-Key".to_string(), "Someone".to_string());
        let result = validate_with_config(&elements, &cfg);
        assert!(codes(&result.findings).contains(&"W309"), "expected W309: {:?}", result.findings);
        // The well-formed "alice" entry still works normally alongside the
        // malformed one -- one bad entry doesn't take down the whole roster.
        assert!(!codes(&result.findings).contains(&"E722"), "{:?}", result.findings);
    }

    #[test]
    fn no_assigned_to_never_raises_e722_regardless_of_roster() {
        let elements = vec![
            req("REQ-AT-004", "Requirements::AtReq4"),
            make_elem(
                "Planning::Unassigned4",
                "type: PlanningItem\nid: PI-AT-004\nname: Item\nstatus: todo\nachieves: REQ-AT-004\n",
                "model/Planning/Unassigned4.md",
            ),
        ];
        let result = validate_with_config(&elements, &users(&["alice"]));
        assert!(!codes(&result.findings).contains(&"E722"), "{:?}", result.findings);
    }
}

// ── W600 typedBy: documentation fallback (REQ-TRS-VAL-017) ──────────────────

#[cfg(test)]
mod w600_typed_by_documentation_tests {
    use super::*;
    use crate::element::{ParseIssue, RawFrontmatter};

    fn make_elem(qname: &str, yaml: &str, file_path: &str, doc: &str) -> RawElement {
        let fm: RawFrontmatter = serde_yaml::from_str(yaml).expect("yaml parse");
        RawElement {
            qualified_name: qname.to_string(),
            file_path: file_path.to_string(),
            frontmatter: fm,
            doc: doc.to_string(),
            parse_issue: None::<ParseIssue>,
            derived: Default::default(),
            derive_findings: vec![],
            locale_docs: Default::default(),
        }
    }

    fn codes(findings: &[Finding]) -> Vec<&str> {
        findings.iter().map(|f| f.code).collect()
    }

    #[test]
    fn part_usage_typed_by_a_documented_partdef_raises_no_w600() {
        let elements = vec![
            make_elem(
                "DocumentedDef",
                "type: PartDef\nname: DocumentedDef\n",
                "model/DocumentedDef.md",
                "Real documentation here.",
            ),
            make_elem(
                "x",
                "type: Part\nname: x\ntypedBy: DocumentedDef\n",
                "model/x.md",
                "",
            ),
        ];
        let result = validate_with_config(&elements, &ValidateConfig::default());
        assert!(!codes(&result.findings).contains(&"W600"), "{:?}", result.findings);
    }

    #[test]
    fn a_partdef_itself_still_raises_w600_regardless_of_anything_else() {
        let elements = vec![make_elem(
            "UndocumentedDef",
            "type: PartDef\nname: UndocumentedDef\n",
            "model/UndocumentedDef.md",
            "",
        )];
        let result = validate_with_config(&elements, &ValidateConfig::default());
        assert!(codes(&result.findings).contains(&"W600"), "{:?}", result.findings);
    }

    #[test]
    fn part_usage_typed_by_an_equally_undocumented_partdef_still_raises_w600() {
        let elements = vec![
            make_elem(
                "UndocumentedDef2",
                "type: PartDef\nname: UndocumentedDef2\n",
                "model/UndocumentedDef2.md",
                "",
            ),
            make_elem(
                "y",
                "type: Part\nname: y\ntypedBy: UndocumentedDef2\n",
                "model/y.md",
                "",
            ),
        ];
        let result = validate_with_config(&elements, &ValidateConfig::default());
        let w600_count = codes(&result.findings).iter().filter(|c| **c == "W600").count();
        assert_eq!(w600_count, 2, "expected W600 for both the def and the usage: {:?}", result.findings);
    }

    #[test]
    fn part_usage_with_unresolvable_typed_by_still_raises_w600() {
        let elements = vec![make_elem(
            "z",
            "type: Part\nname: z\ntypedBy: NoSuchThing\n",
            "model/z.md",
            "",
        )];
        let result = validate_with_config(&elements, &ValidateConfig::default());
        assert!(codes(&result.findings).contains(&"W600"), "{:?}", result.findings);
    }

    // ── cross-package typedBy: (REQ-TRS-SYSMLV2-016) ────────────────────────

    #[test]
    fn a_package_relative_typed_by_reference_across_packages_raises_no_w600() {
        // "SysML2::Services::Documented" is the real qname; "x" (declared
        // inside SysML2::System) carries the literal, package-relative
        // typedBy: text a .sysml author actually wrote -- "Services::Documented",
        // not the full qname. Before REQ-TRS-SYSMLV2-016 this never resolved
        // via the plain resolve_ref, so W600 fired incorrectly.
        let elements = vec![
            make_elem(
                "SysML2::Services::Documented",
                "type: PartDef\nname: Documented\n",
                "model/SysML2/Services.md",
                "Real documentation here.",
            ),
            make_elem(
                "SysML2::System::x",
                "type: Part\nname: x\ntypedBy: Services::Documented\n",
                "model/SysML2/System.md",
                "",
            ),
        ];
        let result = validate_with_config(&elements, &ValidateConfig::default());
        assert!(!codes(&result.findings).contains(&"W600"), "{:?}", result.findings);
    }

    #[test]
    fn a_package_relative_typed_by_reference_to_an_undocumented_target_still_raises_w600() {
        let elements = vec![
            make_elem(
                "SysML2::Services::Undocumented",
                "type: PartDef\nname: Undocumented\n",
                "model/SysML2/Services.md",
                "",
            ),
            make_elem(
                "SysML2::System::y",
                "type: Part\nname: y\ntypedBy: Services::Undocumented\n",
                "model/SysML2/System.md",
                "",
            ),
        ];
        let result = validate_with_config(&elements, &ValidateConfig::default());
        let w600_count = codes(&result.findings).iter().filter(|c| **c == "W600").count();
        assert_eq!(w600_count, 2, "expected W600 for both the def and the usage: {:?}", result.findings);
    }
}

// ── W007 scoped supertype:/typedBy: usage tracking (REQ-TRS-SYSMLV2-017) ────

#[cfg(test)]
mod w007_scoped_usage_tracking_tests {
    use super::*;
    use crate::element::{ParseIssue, RawFrontmatter};

    fn make_elem(qname: &str, yaml: &str, file_path: &str) -> RawElement {
        let fm: RawFrontmatter = serde_yaml::from_str(yaml).expect("yaml parse");
        RawElement {
            qualified_name: qname.to_string(),
            file_path: file_path.to_string(),
            frontmatter: fm,
            doc: "some documentation".to_string(),
            parse_issue: None::<ParseIssue>,
            derived: Default::default(),
            derive_findings: vec![],
            locale_docs: Default::default(),
        }
    }

    /// True if a `W007` finding names `qname` as the unused element (matching
    /// its message, since `W007`'s file is always the reporting element's own
    /// file — the assertion has to key off the message, not the file, when a
    /// test's element list has more than one `*Def`).
    fn w007_fires_for(findings: &[Finding], qname: &str) -> bool {
        findings
            .iter()
            .any(|f| f.code == "W007" && f.message.contains(&format!("'{}'", qname)))
    }

    /// Issue #107's own minimal repro: `Documented`, declared in `Services`, is
    /// used only via a package-relative `typedBy:` written from `System` — the
    /// literal text a `.sysml` author produces (`ingest.rs` does no resolution
    /// of its own), not `Documented`'s real full qname. Before this fix, only
    /// an exact-match `resolve_ref` was tried, so this counted as unused.
    #[test]
    fn a_package_relative_typed_by_reference_across_packages_counts_as_usage() {
        let elements = vec![
            make_elem(
                "Services::Documented",
                "type: PartDef\nname: Documented\n",
                "model/Services.md",
            ),
            make_elem(
                "System::Top",
                "type: PartDef\nname: Top\n",
                "model/System.md",
            ),
            make_elem(
                "System::Top::x",
                "type: Part\nname: x\ntypedBy: Services::Documented\n",
                "model/System.md",
            ),
        ];
        let result = validate_with_config(&elements, &ValidateConfig::default());
        assert!(
            !w007_fires_for(&result.findings, "Services::Documented"),
            "{:?}",
            result.findings
        );
    }

    /// The same widening applies to `supertype:` (specialization), not only
    /// `typedBy:` — `W007` tracks both as "used as a supertype or type".
    #[test]
    fn a_package_relative_supertype_reference_across_packages_counts_as_usage() {
        let elements = vec![
            make_elem(
                "Services::Base",
                "type: PartDef\nname: Base\n",
                "model/Services.md",
            ),
            make_elem(
                "System::Top",
                "type: PartDef\nname: Top\n",
                "model/System.md",
            ),
            make_elem(
                "System::Derived",
                "type: PartDef\nname: Derived\nsupertype: Services::Base\n",
                "model/System.md",
            ),
        ];
        let result = validate_with_config(&elements, &ValidateConfig::default());
        assert!(
            !w007_fires_for(&result.findings, "Services::Base"),
            "{:?}",
            result.findings
        );
    }

    /// A genuinely unused *Def (nothing anywhere references it as
    /// supertype:/typedBy:) still fires W007 — the widening is additive, not a
    /// blanket suppression.
    #[test]
    fn a_genuinely_unused_def_still_raises_w007() {
        let elements = vec![make_elem(
            "Services::Orphan",
            "type: PartDef\nname: Orphan\n",
            "model/Services.md",
        )];
        let result = validate_with_config(&elements, &ValidateConfig::default());
        assert!(
            w007_fires_for(&result.findings, "Services::Orphan"),
            "{:?}",
            result.findings
        );
    }

    /// Same-package resolution (already working before this fix) is unaffected.
    #[test]
    fn a_same_package_typed_by_reference_still_counts_as_usage() {
        let elements = vec![
            make_elem(
                "Services::Documented",
                "type: PartDef\nname: Documented\n",
                "model/Services.md",
            ),
            make_elem(
                "Services::x",
                "type: Part\nname: x\ntypedBy: Documented\n",
                "model/Services.md",
            ),
        ];
        let result = validate_with_config(&elements, &ValidateConfig::default());
        assert!(
            !w007_fires_for(&result.findings, "Services::Documented"),
            "{:?}",
            result.findings
        );
    }

    /// A `typedBy:` nested inside a `features:` entry is scoped the same way as
    /// a top-level one (both go through `collect_typed_by_refs`).
    #[test]
    fn a_package_relative_feature_typed_by_reference_across_packages_counts_as_usage() {
        let elements = vec![
            make_elem(
                "Services::Documented",
                "type: PartDef\nname: Documented\n",
                "model/Services.md",
            ),
            make_elem(
                "System::Top",
                "type: PartDef\nname: Top\nfeatures:\n  - name: sub\n    typedBy: Services::Documented\n",
                "model/System.md",
            ),
        ];
        let result = validate_with_config(&elements, &ValidateConfig::default());
        assert!(
            !w007_fires_for(&result.findings, "Services::Documented"),
            "{:?}",
            result.findings
        );
    }
}

/// `satisfies:` — E313 (domain mismatch), E843/W808 (integrity-level inheritance),
/// and the W300 leaf-assignment coverage count are deliberately
/// type-agnostic on the *source* element: `Part`/`PartDef` is the common case,
/// but every other kind `Resolver::is_verify_target`/E104 already treats as
/// requirement/architecture-shaped for a SysMLv2- or plugin-synthesized
/// `verifies:` target — `StateDef`/`ActionDef` (a behavioral definition, the
/// same claim `refines:`'s `E316` already made explicit, `REQ-TRS-MG-010`),
/// `Connection`/`ConnectionDef`, `Interface`/`InterfaceDef`, `Item`/`ItemDef`,
/// `Attribute`/`AttributeDef`, `Port`/`PortDef`, `Allocation` — satisfies a
/// requirement equally legitimately. `Flow`/`FlowDef` is included too, on the
/// same "artifact that can fulfil a requirement" reasoning, even though it
/// isn't in E104's fixed kind list. These tests pin that behavior down so a
/// future refactor doesn't accidentally narrow it back to Part/PartDef only.
#[cfg(test)]
mod satisfies_shape_tests {
    use super::*;
    use crate::config::ValidateConfig;
    use crate::element::{ParseIssue, RawFrontmatter};

    fn make_elem(qname: &str, yaml: &str, file_path: &str) -> RawElement {
        let fm: RawFrontmatter = serde_yaml::from_str(yaml).expect("yaml parse");
        RawElement {
            qualified_name: qname.to_string(),
            file_path: file_path.to_string(),
            frontmatter: fm,
            doc: "Some documentation.".to_string(),
            parse_issue: None::<ParseIssue>,
            derived: Default::default(),
            derive_findings: vec![],
            locale_docs: Default::default(),
        }
    }

    fn codes(findings: &[Finding]) -> Vec<&str> {
        findings.iter().map(|f| f.code).collect()
    }

    fn req(id: &str, qname: &str, extra: &str) -> RawElement {
        let mut e = make_elem(
            qname,
            &format!("type: Requirement\nid: {id}\nname: A requirement\nstatus: approved\nreqDomain: software\n{extra}"),
            &format!("model/{}.md", qname.replace("::", "/")),
        );
        e.doc = "The system shall do the thing.".to_string();
        e
    }

    #[test]
    fn statedef_satisfies_domain_mismatch_raises_e313_same_as_partdef_would() {
        let elements = vec![
            req("REQ-SM-001", "Requirements::SmReq", ""),
            make_elem(
                "Behavior::FlightModeSM",
                "type: StateDef\nname: FlightModeSM\ndomain: hardware\nsatisfies:\n  - REQ-SM-001\n",
                "model/Behavior/FlightModeSM.md",
            ),
        ];
        let result = validate_with_config(&elements, &ValidateConfig::default());
        assert!(codes(&result.findings).contains(&"E313"), "expected E313: {:?}", result.findings);
    }

    #[test]
    fn actiondef_satisfies_matching_domain_raises_no_e313() {
        let elements = vec![
            req("REQ-ACT-001", "Requirements::ActReq", ""),
            make_elem(
                "Behavior::ArmMotors",
                "type: ActionDef\nname: ArmMotors\ndomain: software\nsatisfies:\n  - REQ-ACT-001\n",
                "model/Behavior/ArmMotors.md",
            ),
        ];
        let result = validate_with_config(&elements, &ValidateConfig::default());
        assert!(
            !codes(&result.findings).contains(&"E313"),
            "matching domain must not raise E313: {:?}",
            result.findings
        );
    }

    #[test]
    fn statedef_satisfies_missing_integrity_level_raises_e843_same_as_partdef_would() {
        let elements = vec![
            req("REQ-SM-002", "Requirements::SmReq2", "asilLevel: C\n"),
            make_elem(
                "Behavior::FlightModeSM2",
                "type: StateDef\nname: FlightModeSM2\nsatisfies:\n  - REQ-SM-002\n",
                "model/Behavior/FlightModeSM2.md",
            ),
        ];
        let result = validate_with_config(&elements, &ValidateConfig::default());
        assert!(codes(&result.findings).contains(&"E843"), "expected E843: {:?}", result.findings);
    }

    #[test]
    fn actiondef_satisfies_matching_integrity_level_raises_no_e843() {
        let elements = vec![
            req("REQ-ACT-002", "Requirements::ActReq2", "asilLevel: B\n"),
            make_elem(
                "Behavior::ArmMotors2",
                "type: ActionDef\nname: ArmMotors2\nasilLevel: B\nsatisfies:\n  - REQ-ACT-002\n",
                "model/Behavior/ArmMotors2.md",
            ),
        ];
        let result = validate_with_config(&elements, &ValidateConfig::default());
        assert!(
            !codes(&result.findings).contains(&"E843"),
            "matching integrity level must not raise E843: {:?}",
            result.findings
        );
    }

    #[test]
    fn statedef_satisfier_counts_toward_w300_leaf_assignment_coverage() {
        let elements = vec![
            req("REQ-SM-003", "Requirements::SmReq3", ""),
            make_elem(
                "Behavior::FlightModeSM3",
                "type: StateDef\nname: FlightModeSM3\ndomain: software\nsatisfies:\n  - REQ-SM-003\n",
                "model/Behavior/FlightModeSM3.md",
            ),
        ];
        let result = validate_with_config(&elements, &ValidateConfig::default());
        assert!(
            !codes(&result.findings).contains(&"W300"),
            "a StateDef satisfier must count as coverage, same as a PartDef would: {:?}",
            result.findings
        );
    }

    #[test]
    fn one_partdef_and_one_statedef_jointly_satisfying_a_leaf_requirement_is_legitimate() {
        let elements = vec![
            req("REQ-SM-004", "Requirements::SmReq4", ""),
            make_elem(
                "Arch::FlowController",
                "type: PartDef\nname: FlowController\ndomain: software\nsatisfies:\n  - REQ-SM-004\n",
                "model/Arch/FlowController.md",
            ),
            make_elem(
                "Behavior::FlightModeSM4",
                "type: StateDef\nname: FlightModeSM4\ndomain: software\nsatisfies:\n  - REQ-SM-004\n",
                "model/Behavior/FlightModeSM4.md",
            ),
        ];
        let result = validate_with_config(&elements, &ValidateConfig::default());
        assert!(
            !codes(&result.findings).contains(&"W301"),
            "W301 is retired (GH #121) — a structural + behavioural pair may jointly satisfy a leaf: {:?}",
            result.findings
        );
    }

    /// Every kind `is_verify_target`/E104 already treats as
    /// requirement/architecture-shaped (Connection/ConnectionDef,
    /// Interface/InterfaceDef, Item/ItemDef, Attribute/AttributeDef,
    /// Port/PortDef, Allocation), plus Flow/FlowDef on the same reasoning,
    /// is credited as a leaf requirement's satisfier — no W300 for any of them.
    #[test]
    fn every_endorsed_shape_counts_toward_w300_leaf_assignment_coverage() {
        let shapes: &[(&str, &str)] = &[
            ("REQ-SH-001", "Connection"),
            ("REQ-SH-002", "ConnectionDef"),
            ("REQ-SH-003", "Interface"),
            ("REQ-SH-004", "InterfaceDef"),
            ("REQ-SH-005", "Item"),
            ("REQ-SH-006", "ItemDef"),
            ("REQ-SH-007", "Attribute"),
            ("REQ-SH-008", "AttributeDef"),
            ("REQ-SH-009", "Port"),
            ("REQ-SH-010", "PortDef"),
            ("REQ-SH-011", "Allocation"),
            ("REQ-SH-012", "Flow"),
            ("REQ-SH-013", "FlowDef"),
        ];
        let mut elements = Vec::new();
        for (req_id, kind) in shapes {
            let req_qname = format!("Requirements::{}", req_id.replace('-', "_"));
            elements.push(req(req_id, &req_qname, ""));
            elements.push(make_elem(
                &format!("Arch::Satisfier{kind}"),
                &format!("type: {kind}\nname: Satisfier{kind}\ndomain: software\nsatisfies:\n  - {req_id}\n"),
                &format!("model/Arch/Satisfier{kind}.md"),
            ));
        }
        let result = validate_with_config(&elements, &ValidateConfig::default());
        assert!(
            !codes(&result.findings).contains(&"W300"),
            "every endorsed shape should count as a satisfier, no W300 expected: {:?}",
            result.findings
        );
    }

    #[test]
    fn connection_and_flow_satisfies_domain_mismatch_raises_e313() {
        let elements = vec![
            req("REQ-CF-001", "Requirements::CfReq1", ""),
            make_elem(
                "Arch::DataLink",
                "type: Connection\nname: DataLink\ndomain: hardware\nsatisfies:\n  - REQ-CF-001\n",
                "model/Arch/DataLink.md",
            ),
            req("REQ-CF-002", "Requirements::CfReq2", ""),
            make_elem(
                "Arch::TelemetryFlow",
                "type: Flow\nname: TelemetryFlow\ndomain: hardware\nsatisfies:\n  - REQ-CF-002\n",
                "model/Arch/TelemetryFlow.md",
            ),
        ];
        let result = validate_with_config(&elements, &ValidateConfig::default());
        let e313_count = codes(&result.findings).iter().filter(|c| **c == "E313").count();
        assert_eq!(e313_count, 2, "expected E313 for both the Connection and the Flow: {:?}", result.findings);
    }

    #[test]
    fn interface_and_allocation_satisfies_missing_integrity_level_raises_e843() {
        let elements = vec![
            req("REQ-IA-001", "Requirements::IaReq1", "asilLevel: B\n"),
            make_elem(
                "Arch::TelemetryIface",
                "type: Interface\nname: TelemetryIface\nsatisfies:\n  - REQ-IA-001\n",
                "model/Arch/TelemetryIface.md",
            ),
            req("REQ-IA-002", "Requirements::IaReq2", "silLevel: 2\n"),
            make_elem(
                "Arch::PowerAllocation",
                "type: Allocation\nname: PowerAllocation\nsatisfies:\n  - REQ-IA-002\n",
                "model/Arch/PowerAllocation.md",
            ),
        ];
        let result = validate_with_config(&elements, &ValidateConfig::default());
        let e843_count = codes(&result.findings).iter().filter(|c| **c == "E843").count();
        assert_eq!(e843_count, 2, "expected E843 for both the Interface and the Allocation: {:?}", result.findings);
    }

    #[test]
    fn a_partdef_and_an_item_jointly_satisfying_a_leaf_requirement_is_legitimate() {
        let elements = vec![
            req("REQ-MIX-001", "Requirements::MixReq", ""),
            make_elem(
                "Arch::FlowController",
                "type: PartDef\nname: FlowController\ndomain: software\nsatisfies:\n  - REQ-MIX-001\n",
                "model/Arch/FlowController.md",
            ),
            make_elem(
                "Arch::TelemetryPacket",
                "type: Item\nname: TelemetryPacket\ndomain: software\nsatisfies:\n  - REQ-MIX-001\n",
                "model/Arch/TelemetryPacket.md",
            ),
        ];
        let result = validate_with_config(&elements, &ValidateConfig::default());
        assert!(
            !codes(&result.findings).contains(&"W301"),
            "W301 is retired (GH #121) — a PartDef and an Item may jointly satisfy a leaf: {:?}",
            result.findings
        );
    }
}

// ── User-defined link types (ADR-SYS-LINKTYPE-001) ──────────────────────────

#[cfg(test)]
mod link_type_tests {
    use super::*;
    use crate::config::ValidateConfig;
    use crate::element::{ParseIssue, RawFrontmatter};
    use crate::link_types::LinkTypeRegistry;

    fn make_elem(qname: &str, yaml: &str) -> RawElement {
        let fm: RawFrontmatter = serde_yaml::from_str(yaml).expect("yaml parse");
        RawElement {
            qualified_name: qname.to_string(),
            file_path: format!("model/{}.md", qname.replace("::", "/")),
            frontmatter: fm,
            doc: "The system shall do the thing.".to_string(),
            parse_issue: None::<ParseIssue>,
            derived: Default::default(),
            derive_findings: vec![],
            locale_docs: Default::default(),
        }
    }

    fn req(id: &str, extra: &str) -> RawElement {
        make_elem(
            &format!("Requirements::{id}"),
            &format!("type: Requirement\nid: {id}\nname: A requirement\nstatus: approved\nreqDomain: software\n{extra}"),
        )
    }

    fn part(name: &str, extra: &str) -> RawElement {
        make_elem(&format!("Arch::{name}"), &format!("type: PartDef\nname: {name}\ndomain: software\n{extra}"))
    }

    fn cfg(toml_text: &str) -> ValidateConfig {
        ValidateConfig { link_types: LinkTypeRegistry::from_toml_str(toml_text), ..ValidateConfig::default() }
    }

    fn run(elements: &[RawElement], toml_text: &str) -> Vec<Finding> {
        validate_with_config(elements, &cfg(toml_text)).findings
    }

    /// Findings of `code` whose file mentions `file_part`.
    fn hits<'a>(findings: &'a [Finding], code: &str, file_part: &str) -> Vec<&'a Finding> {
        findings.iter().filter(|f| f.code == code && f.file.contains(file_part)).collect()
    }

    fn any_link_code(findings: &[Finding]) -> bool {
        findings.iter().any(|f| f.code.starts_with("E63") || f.code == "W630" || f.code == "W631")
    }

    #[test]
    fn a_model_without_link_types_or_links_is_unaffected() {
        let elements = vec![req("REQ-001", ""), part("Ctl", "satisfies: [REQ-001]\n")];
        let plain = validate(&elements).findings;
        let with_empty_cfg = run(&elements, "");
        let mut a: Vec<(&str, String)> = plain.iter().map(|f| (f.code, f.message.clone())).collect();
        let mut b: Vec<(&str, String)> = with_empty_cfg.iter().map(|f| (f.code, f.message.clone())).collect();
        a.sort();
        b.sort();
        assert_eq!(a, b);
        assert!(!any_link_code(&plain));
    }

    #[test]
    fn w630_is_reported_against_the_config_file_and_links_are_not_w047() {
        let elements = vec![req("REQ-001", ""), part("Ctl", "links:\n  informs: REQ-001\n")];
        let f = run(&elements, "[linkTypes.informs]\n[linkTypes.Bad-Name]\n");
        assert_eq!(hits(&f, "W630", ".syscribe.toml").len(), 1, "{f:?}");
        assert!(hits(&f, "W047", "Ctl").is_empty(), "links: must not be W047: {f:?}");
        assert!(!f.iter().any(|x| x.code.starts_with("E63")), "{f:?}");
    }

    #[test]
    fn e630_lists_declared_types_or_hints_how_to_declare_one() {
        let elements = vec![req("REQ-001", ""), part("Ctl", "links:\n  influences: REQ-001\n")];
        let f = run(&elements, "[linkTypes.informs]\n");
        let e = hits(&f, "E630", "Ctl");
        assert_eq!(e.len(), 1);
        assert!(e[0].message.contains("informs"), "{}", e[0].message);

        let f = run(&elements, "");
        let e = hits(&f, "E630", "Ctl");
        assert_eq!(e.len(), 1);
        assert!(e[0].message.contains("[linkTypes."), "{}", e[0].message);
    }

    #[test]
    fn e631_for_non_mapping_and_bad_values() {
        let elements = vec![
            req("REQ-001", ""),
            part("NotAMap", "links:\n  - REQ-001\n"),
            part("BadValue", "links:\n  informs:\n    x: 1\n"),
            part("BadList", "links:\n  informs: [REQ-001, {a: b}]\n"),
        ];
        let f = run(&elements, "[linkTypes.informs]\n");
        assert_eq!(hits(&f, "E631", "NotAMap").len(), 1);
        assert_eq!(hits(&f, "E631", "BadValue").len(), 1);
        assert_eq!(hits(&f, "E631", "BadList").len(), 1);
    }

    #[test]
    fn e632_for_a_dangling_target_only() {
        let elements = vec![
            req("REQ-001", ""),
            part("Good", "links:\n  informs: [REQ-001, Requirements::REQ-001]\n"),
            part("Dangling", "links:\n  informs: REQ-999\n"),
        ];
        let f = run(&elements, "[linkTypes.informs]\n");
        assert!(hits(&f, "E632", "Good").is_empty());
        assert_eq!(hits(&f, "E632", "Dangling").len(), 1);
    }

    #[test]
    fn e633_and_e634_enforce_source_and_target_types() {
        let elements = vec![
            req("REQ-001", ""),
            req("REQ-002", "links:\n  mitigates: REQ-001\n"),
            part("GoodCtl", "links:\n  mitigates: REQ-001\n"),
            part("BadTarget", "links:\n  mitigates: Arch::GoodCtl\n"),
        ];
        let f = run(&elements, "[linkTypes.mitigates]\nsourceTypes = [\"PartDef\"]\ntargetTypes = [\"Requirement\"]\n");
        assert!(hits(&f, "E633", "GoodCtl").is_empty() && hits(&f, "E634", "GoodCtl").is_empty());
        assert_eq!(hits(&f, "E633", "REQ-002").len(), 1);
        let e634 = hits(&f, "E634", "BadTarget");
        assert_eq!(e634.len(), 1);
        assert!(e634[0].message.contains("Arch::GoodCtl"));
    }

    #[test]
    fn e635_and_w631_bound_the_target_count_and_w631_is_draft_suppressed() {
        let toml_text = "[linkTypes.dependsOn]\nsourceTypes = [\"Requirement\"]\ncardinality = \"1..2\"\n";
        let elements = vec![
            req("REQ-001", "links:\n  dependsOn: REQ-002\n"),
            req("REQ-002", "links:\n  dependsOn: REQ-001\n"),
            req("REQ-010", "links:\n  dependsOn: [REQ-001, REQ-002, REQ-020]\n"),
            req("REQ-020", ""),
            make_elem(
                "Requirements::REQ-021",
                "type: Requirement\nid: REQ-021\nname: r\nstatus: draft\nreqDomain: software\n",
            ),
            part("NotInScope", ""),
        ];
        let f = run(&elements, toml_text);
        assert_eq!(hits(&f, "E635", "REQ-010").len(), 1);
        assert!(hits(&f, "E635", "REQ-001").is_empty());
        assert_eq!(hits(&f, "W631", "REQ-020").len(), 1);
        assert!(hits(&f, "W631", "REQ-021").is_empty(), "draft is suppressed");
        assert!(hits(&f, "W631", "NotInScope").is_empty(), "type outside sourceTypes");
        assert!(hits(&f, "W631", "REQ-001").is_empty());
    }

    #[test]
    fn e636_reports_each_cycle_once_including_self_links() {
        let toml_text = "[linkTypes.precedes]\nacyclic = true\n[linkTypes.relates]\n";
        let elements = vec![
            req("REQ-001", "links:\n  precedes: REQ-002\n"),
            req("REQ-002", "links:\n  precedes: REQ-003\n"),
            req("REQ-003", "links:\n  precedes: REQ-001\n"),
            req("REQ-004", "links:\n  precedes: REQ-004\n"),
            req("REQ-005", "links:\n  relates: REQ-006\n  precedes: REQ-006\n"),
            req("REQ-006", "links:\n  relates: REQ-005\n"),
        ];
        let f = run(&elements, toml_text);
        let e636: Vec<&Finding> = f.iter().filter(|x| x.code == "E636").collect();
        assert_eq!(e636.len(), 2, "{e636:?}");
        assert!(e636.iter().any(|x| ["REQ-001", "REQ-002", "REQ-003"].iter().all(|m| x.message.contains(m))));
        assert!(e636.iter().any(|x| x.message.contains("REQ-004")));
        assert!(!e636.iter().any(|x| x.message.contains("REQ-005")), "non-acyclic cycle is not checked");
    }

    #[test]
    fn extends_inherits_base_rules_minus_relaxed_codes() {
        let toml_text = "[linkTypes.partSat]\nextends = \"satisfies\"\nrelax = [\"E313\"]\n\
                         [linkTypes.strictSat]\nextends = \"satisfies\"\n";
        let hw = |name: &str, extra: &str| {
            make_elem(&format!("Arch::{name}"), &format!("type: PartDef\nname: {name}\ndomain: hardware\n{extra}"))
        };
        let elements = vec![
            req("REQ-001", ""),
            req("REQ-002", ""),
            req("REQ-003", ""),
            hw("HwA", "links:\n  partSat: REQ-001\n"),
            hw("HwB", "links:\n  strictSat: REQ-002\n"),
            hw("HwC", "satisfies: [REQ-003]\nlinks:\n  partSat: REQ-003\n"),
        ];
        let f = run(&elements, toml_text);
        assert!(hits(&f, "E313", "HwA").is_empty(), "relaxed: {f:?}");
        assert_eq!(hits(&f, "E313", "HwB").len(), 1, "inherited");
        assert_eq!(hits(&f, "E313", "HwC").len(), 1, "built-in never relaxed, relaxed twin silent");
        // Coverage: all three requirements are satisfied (no W300).
        for id in ["REQ-001", "REQ-002", "REQ-003"] {
            assert!(hits(&f, "W300", id).is_empty(), "{id}: {f:?}");
        }
    }

    #[test]
    fn coverage_false_keeps_checks_but_gives_no_credit() {
        let toml_text = "[linkTypes.weakSat]\nextends = \"satisfies\"\ncoverage = false\n";
        let elements = vec![
            req("REQ-001", ""),
            make_elem("Arch::Hw", "type: PartDef\nname: Hw\ndomain: hardware\nlinks:\n  weakSat: REQ-001\n"),
        ];
        let f = run(&elements, toml_text);
        assert_eq!(hits(&f, "W300", "REQ-001").len(), 1, "no coverage credit");
        assert_eq!(hits(&f, "E313", "Hw").len(), 1, "base check still applies");
    }

    #[test]
    fn e310_relaxation_needs_every_derived_from_like_link_to_relax_it() {
        let toml_text = "[linkTypes.inspiredBy]\nextends = \"derivedFrom\"\nrelax = [\"E310\"]\n\
                         [linkTypes.refinedFrom]\nextends = \"derivedFrom\"\n";
        let elements = vec![
            req("REQ-001", ""),
            req("REQ-011", "links:\n  inspiredBy: REQ-001\n"),
            req("REQ-012", "links:\n  refinedFrom: REQ-001\n"),
            req("REQ-013", "derivedFrom: [REQ-001]\nlinks:\n  inspiredBy: REQ-001\n"),
        ];
        let f = run(&elements, toml_text);
        assert!(hits(&f, "E310", "REQ-011").is_empty(), "{f:?}");
        assert_eq!(hits(&f, "E310", "REQ-012").len(), 1);
        assert_eq!(hits(&f, "E310", "REQ-013").len(), 1, "a built-in derivedFrom keeps E310 in force");
    }

    #[test]
    fn coverage_false_derived_from_does_not_make_the_target_a_parent() {
        let toml_text = "[linkTypes.weakDerived]\nextends = \"derivedFrom\"\nrelax = [\"E310\"]\ncoverage = false\n";
        let elements = vec![
            req("REQ-001", ""),
            req("REQ-002", "links:\n  weakDerived: REQ-001\n"),
            part("Ctl", "satisfies: [REQ-001]\n"),
        ];
        let result = validate_with_config(&elements, &cfg(toml_text));
        assert!(!result.derived_children.contains_key("REQ-001"));
        assert!(hits(&result.findings, "E312", "REQ-001").is_empty(), "not a parent ⇒ no E312");
    }

    #[test]
    fn verifies_extension_relaxing_e104_feeds_verified_by() {
        let toml_text = "[linkTypes.checks]\nextends = \"verifies\"\nrelax = [\"E104\"]\n";
        let elements = vec![
            part("Ctl", ""),
            make_elem(
                "Tests::TC-001",
                "type: TestCase\nid: TC-001\nname: t\nstatus: active\ntestLevel: L1\nverifies: []\nlinks:\n  checks: Arch::Ctl\n",
            ),
        ];
        let result = validate_with_config(&elements, &cfg(toml_text));
        assert!(hits(&result.findings, "E104", "TC-001").is_empty(), "{:?}", result.findings);
        assert_eq!(result.verified_by.get("Arch::Ctl"), Some(&vec!["TC-001".to_string()]));
    }

    /// A TestCase whose only link to `req` is an extending-`verifies` link of
    /// `link_type`.
    fn tc_linking(id: &str, link_type: &str, req: &str, extra: &str) -> RawElement {
        make_elem(
            &format!("Tests::{id}"),
            &format!(
                "type: TestCase\nid: {id}\nname: t\nstatus: active\ntestLevel: L3\nlinks:\n  {link_type}: {req}\n{extra}"
            ),
        )
    }

    const VERIFY_TYPES: &str = "[linkTypes.weakV]\nextends = \"verifies\"\ncoverage = false\n\
                                [linkTypes.strongV]\nextends = \"verifies\"\n\
                                [linkTypes.weakD]\nextends = \"derivedFrom\"\ncoverage = false\nrelax = [\"E310\"]\n";

    /// Per-configuration coverage fixture: feature `Wdt` selected in CONF-A,
    /// REQ-020 active only with `Wdt`, verified by one TestCase via `link_type`.
    fn w015_model(link_type: &str) -> Vec<RawElement> {
        vec![
            make_elem("Features::Wdt", "type: FeatureDef\nid: FEAT-WDT\nname: Wdt\ngroupKind: optional\n"),
            make_elem(
                "Configs::CONF-A",
                "type: Configuration\nid: CONF-FX-A-001\nname: a\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features::Wdt: true\n",
            ),
            req("REQ-020", "appliesWhen: Features::Wdt\n"),
            tc_linking("TC-020", link_type, "REQ-020", "appliesWhen: Features::Wdt\n"),
        ]
    }

    #[test]
    fn w015_is_not_cleared_by_a_coverage_false_verifies_extension() {
        let weak = run(&w015_model("weakV"), VERIFY_TYPES);
        assert!(weak.iter().any(|f| f.code == "W015" && f.message.contains("REQ-020")), "{weak:?}");
        let strong = run(&w015_model("strongV"), VERIFY_TYPES);
        assert!(!strong.iter().any(|f| f.code == "W015"), "{strong:?}");
    }

    /// An active TestPlan demonstrating REQ-001, whose one member reaches it only
    /// through `tc_link` (verifies REQ-00x) and, for REQ-002, `REQ-002`'s own
    /// `derived_link` to REQ-001.
    fn w614_model(tc_link: &str, target: &str, derived_link: &str) -> Vec<RawElement> {
        vec![
            req("REQ-001", ""),
            req("REQ-002", &format!("links:\n  {derived_link}: REQ-001\n")),
            tc_linking("TC-001", tc_link, target, ""),
            make_elem(
                "Tests::TP-FX-A-001",
                "type: TestPlan\nid: TP-FX-A-001\nname: p\nstatus: active\nscope: integration\ndemonstrates: [REQ-001]\ntestCases: [TC-001]\n",
            ),
        ]
    }

    #[test]
    fn w614_is_not_cleared_by_coverage_false_verifies_or_derived_from() {
        let w614 = |m: Vec<RawElement>| run(&m, VERIFY_TYPES).iter().any(|f| f.code == "W614");
        assert!(!w614(w614_model("strongV", "REQ-001", "weakD")), "credited verifier clears W614");
        assert!(w614(w614_model("weakV", "REQ-001", "weakD")), "coverage=false verifies must not clear W614");
        // Goal closure through a coverage=false derivedFrom does not credit REQ-001.
        assert!(w614(w614_model("strongV", "REQ-002", "weakD")), "coverage=false derivedFrom must not close the goal");
    }

    #[test]
    fn verified_by_and_derived_children_have_no_duplicate_from_a_twin_extension() {
        let toml_text = "[linkTypes.checks]\nextends = \"verifies\"\n[linkTypes.refinedFrom]\nextends = \"derivedFrom\"\n";
        let elements = vec![
            req("REQ-001", ""),
            req("REQ-002", "derivedFrom: [REQ-001]\nbreakdownAdr: ADR-001\nlinks:\n  refinedFrom: REQ-001\n"),
            make_elem("Dec::ADR-001", "type: ADR\nid: ADR-001\nname: a\nstatus: accepted\n"),
            tc_linking("TC-001", "checks", "REQ-002", "verifies: [REQ-002]\n"),
        ];
        let result = validate_with_config(&elements, &cfg(toml_text));
        assert_eq!(result.verified_by.get("REQ-002"), Some(&vec!["TC-001".to_string()]));
        assert_eq!(result.derived_children.get("REQ-001"), Some(&vec!["REQ-002".to_string()]));
    }

    #[test]
    fn a_twin_derived_from_extension_is_one_decomposition_channel_not_two() {
        // REQ-002 derives from the ASIL D REQ-001 by both `derivedFrom:` and an
        // extending twin; as a single channel it is W860, never a false E865.
        let toml_text = "[linkTypes.refinedFrom]\nextends = \"derivedFrom\"\n[linkTypes.strongSat]\nextends = \"satisfies\"\n";
        let elements = vec![
            req("REQ-001", "asilLevel: D\n"),
            req(
                "REQ-002",
                "asilLevel: B\nderivedFrom: [REQ-001]\nbreakdownAdr: ADR-001\nsatisfies: [Arch::X]\nlinks:\n  refinedFrom: REQ-001\n  strongSat: Arch::X\n",
            ),
            make_elem("Dec::ADR-001", "type: ADR\nid: ADR-001\nname: a\nstatus: accepted\n"),
            make_elem("Arch::X", "type: PartDef\nname: X\ndomain: software\nasilLevel: B\n"),
        ];
        let f = run(&elements, toml_text);
        assert!(!f.iter().any(|x| x.code == "E865"), "{f:?}");
        assert!(f.iter().any(|x| x.code == "W860"), "{f:?}");
    }

    #[test]
    fn a_target_listed_twice_counts_once_toward_cardinality() {
        let toml_text = "[linkTypes.dependsOn]\nsourceTypes = [\"Requirement\"]\ncardinality = \"2..2\"\n";
        let elements = vec![
            req("REQ-001", ""),
            req("REQ-002", ""),
            req("REQ-010", "links:\n  dependsOn: [REQ-001, REQ-001]\n"),
            req("REQ-011", "links:\n  dependsOn: [REQ-001, REQ-002, REQ-002]\n"),
        ];
        let f = run(&elements, toml_text);
        assert_eq!(hits(&f, "W631", "REQ-010").len(), 1, "[A, A] is one target, under the lower bound: {f:?}");
        assert!(hits(&f, "E635", "REQ-011").is_empty(), "[A, B, B] is two targets, within bounds: {f:?}");
        assert!(hits(&f, "W631", "REQ-011").is_empty(), "{f:?}");
    }

    #[test]
    fn base_rule_findings_on_contributed_entries_name_the_link_type() {
        let toml_text = "[linkTypes.strictSat]\nextends = \"satisfies\"\n[linkTypes.refinedFrom]\nextends = \"derivedFrom\"\n";
        let hw = |name: &str, extra: &str| {
            make_elem(&format!("Arch::{name}"), &format!("type: PartDef\nname: {name}\ndomain: hardware\n{extra}"))
        };
        let elements = vec![
            req("REQ-001", ""),
            req("REQ-002", ""),
            req("REQ-012", "links:\n  refinedFrom: REQ-001\n"),
            hw("HwB", "links:\n  strictSat: REQ-001\n"),
            hw("HwC", "satisfies: [REQ-002]\n"),
        ];
        let f = run(&elements, toml_text);
        let e313b = hits(&f, "E313", "HwB");
        assert!(e313b[0].message.ends_with("(via links.strictSat)"), "{}", e313b[0].message);
        let e313c = hits(&f, "E313", "HwC");
        assert!(!e313c[0].message.contains("via links"), "authored message unchanged: {}", e313c[0].message);
        let e310 = hits(&f, "E310", "REQ-012");
        assert!(e310[0].message.ends_with("(via links.refinedFrom)"), "{}", e310[0].message);
    }

    #[test]
    fn e632_gets_the_root_name_hint() {
        let mut root = make_elem("", "type: Package\nname: Demo\n");
        root.file_path = "model/_index.md".to_string();
        let elements = vec![root, req("REQ-001", ""), part("Ctl", "links:\n  informs: Demo::Requirements::REQ-001\n")];
        let f = run(&elements, "[linkTypes.informs]\n");
        let e632 = hits(&f, "E632", "Ctl");
        assert_eq!(e632.len(), 1, "{f:?}");
        assert!(e632[0].message.contains("did you mean 'Requirements::REQ-001'"), "{}", e632[0].message);
    }
}

#[cfg(test)]
mod fmea_row_tests {
    //! GH #132 / REQ-TRS-FMEA-004 — E923 (row without id) and W928 (explicit
    //! rpn disagreeing with S×O×D).
    use super::fmea_row_findings;

    fn row(yaml: &str) -> serde_yaml::Value {
        serde_yaml::from_str(yaml).expect("yaml")
    }

    #[test]
    fn row_without_id_is_e923_naming_position_and_failure_mode() {
        let f = fmea_row_findings("s.md", 2, &row("failureMode: Stuck valve\nfmeaSeverity: 3"));
        assert_eq!(f.len(), 1, "{f:?}");
        assert_eq!(f[0].code, "E923");
        assert!(f[0].message.contains("row 2") && f[0].message.contains("Stuck valve"), "{}", f[0].message);
    }

    #[test]
    fn non_mapping_row_is_e923() {
        let f = fmea_row_findings("s.md", 1, &row("just a string"));
        assert_eq!(f.iter().map(|x| x.code).collect::<Vec<_>>(), vec!["E923"]);
    }

    #[test]
    fn disagreeing_explicit_rpn_is_w928_with_both_values() {
        let f = fmea_row_findings(
            "s.md",
            3,
            &row("id: FM-X-001\nfmeaSeverity: 5\noccurrence: 4\ndetection: 3\nrpn: 100"),
        );
        assert_eq!(f.len(), 1, "{f:?}");
        assert_eq!(f[0].code, "W928");
        assert!(f[0].message.contains("100") && f[0].message.contains("= 60"), "{}", f[0].message);
    }

    #[test]
    fn severity_alias_counts_as_s() {
        let f = fmea_row_findings("s.md", 1, &row("id: FM-X-001\nseverity: 2\noccurrence: 2\ndetection: 2\nrpn: 9"));
        assert_eq!(f.iter().map(|x| x.code).collect::<Vec<_>>(), vec!["W928"]);
    }

    #[test]
    fn consistent_or_partial_rpn_is_silent() {
        assert!(fmea_row_findings("s.md", 1, &row("id: FM-X-001\nfmeaSeverity: 2\noccurrence: 3\ndetection: 4\nrpn: 24")).is_empty());
        assert!(fmea_row_findings("s.md", 1, &row("id: FM-X-001\nfmeaSeverity: 5\noccurrence: 4\nrpn: 80")).is_empty());
        assert!(fmea_row_findings("s.md", 1, &row("id: FM-X-001\nfmeaSeverity: 5\noccurrence: 4\ndetection: 3")).is_empty());
    }
}

// ── E927 — FaultTreeEvent `ref:` (REQ-TRS-FTA-002, issue #148) ───────────────

#[cfg(test)]
mod e927_fault_tree_event_ref_tests {
    use super::*;
    use crate::element::{ParseIssue, RawFrontmatter};

    fn make_elem(qname: &str, yaml: &str) -> RawElement {
        let fm: RawFrontmatter = serde_yaml::from_str(yaml).expect("yaml parse");
        RawElement {
            qualified_name: qname.to_string(),
            file_path: format!("model/{}.md", qname.replace("::", "/")),
            frontmatter: fm,
            doc: String::new(),
            parse_issue: None::<ParseIssue>,
            derived: Default::default(),
            derive_findings: vec![],
            locale_docs: Default::default(),
        }
    }

    fn count(findings: &[Finding], file: &str, code: &str) -> usize {
        findings.iter().filter(|f| f.code == code && f.file == file).count()
    }

    fn fte(r: &str) -> RawElement {
        make_elem(
            "FT::FTE-FTX-001",
            &format!("type: FaultTreeEvent\nid: FTE-FTX-001\nname: e\neventKind: basic\nref: {r}\n"),
        )
    }

    #[test]
    fn resolving_ref_by_qname_and_id_is_clean() {
        let part = make_elem("Arch::Valve", "type: PartDef\nname: Valve\n");
        let req = make_elem(
            "Arch::REQ-FTX-001",
            "type: Requirement\nid: REQ-FTX-001\nname: r\nstatus: draft\n",
        );
        for r in ["Arch::Valve", "REQ-FTX-001"] {
            let ev = fte(r);
            let file = ev.file_path.clone();
            let res = validate(&[part.clone(), req.clone(), ev]);
            assert_eq!(count(&res.findings, &file, "E927"), 0, "ref {r}");
            assert_eq!(count(&res.findings, &file, "W047"), 0, "ref {r}");
        }
    }

    #[test]
    fn dangling_ref_is_e927() {
        let ev = fte("Arch::Nope");
        let file = ev.file_path.clone();
        let res = validate(&[ev]);
        assert_eq!(count(&res.findings, &file, "E927"), 1);
        assert_eq!(count(&res.findings, &file, "W047"), 0);
    }

    #[test]
    fn ref_on_other_types_stays_unrecognized() {
        let gate = make_elem(
            "FT::FTG-FTX-001",
            "type: FaultTreeGate\nid: FTG-FTX-001\nname: g\ngateType: OR\nref: Arch::Nope\n",
        );
        let file = gate.file_path.clone();
        let res = validate(&[gate]);
        assert_eq!(count(&res.findings, &file, "W047"), 1);
        assert_eq!(count(&res.findings, &file, "E927"), 0);
    }

    #[test]
    fn resolved_ref_is_a_graph_edge() {
        use crate::graph::{build_graph, EdgeKind};
        let part = make_elem("Arch::Valve", "type: PartDef\nname: Valve\n");
        let (g, idx) = build_graph(&[part, fte("Arch::Valve")]);
        let (src, dst) = (idx["FT::FTE-FTX-001"], idx["Arch::Valve"]);
        assert!(g
            .edges_connecting(src, dst)
            .any(|e| *e.weight() == EdgeKind::FaultTreeEventRef));
        assert_eq!(EdgeKind::FaultTreeEventRef.name(), "faultTreeEventRef");
    }
}
