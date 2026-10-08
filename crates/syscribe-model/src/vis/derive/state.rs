//! The StateMachine generator (`REQ-TRS-VIS-018`).
//!
//! Subject: a `StateDef`, or a `State`/`ExhibitState` usage (which reads the
//! `StateDef` it is typed by). Produces one `State` node per `subStates:`
//! entry (spec §8.8.2) with `entry / …`, `do / …` and `exit / …` compartment
//! lines, an `Initial` pseudostate per region with a transition to every
//! `isInitial: true` state, a `Final` node every `isFinal: true` state
//! transitions to, and one `Transition` edge per transition in either
//! placement — nested under a substate or top-level with `source:`, the
//! deprecated `from`/`to`/`trigger` aliases read like the canonical keys,
//! exactly as the validator's `W07x` extractor does — labelled
//! `<accept> [<guard>] / <effect>`. A substate typed by a `StateDef` with its
//! own `subStates:` becomes a container holding that machine's states, one
//! level deep. An edge is emitted only when both endpoint states are nodes of
//! the diagram; a transition with a missing endpoint is left to `W929`.

use std::collections::HashMap;

use crate::element::{ElementType, RawElement};
use crate::resolver::Resolver;

use super::super::ir::{derived_shape_id, DiagramGraph, Edge, EdgeKind, Node, NodeKind};
use super::super::manifest::Issue;
use super::{map_str, short_name, w418, yaml_strings, Filters};

/// One transition, normalised from either placement and either spelling —
/// the generator's mirror of the validator's `StateEdge`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Transition {
    pub name: Option<String>,
    pub source: Option<String>,
    pub target: Option<String>,
    /// Display text of the accepter: the payload's last segment, or
    /// `after <expr>` / `when <expr>` / `at <expr>` for a time/change trigger.
    pub accept: Option<String>,
    pub guard: Option<String>,
    /// The effect's `name`, else the last segment of its `typedBy`.
    pub effect: Option<String>,
}

impl Transition {
    /// `<accept> [<guard>] / <effect>` with the absent parts omitted; `None`
    /// when the transition carries none of the three.
    pub fn label(&self) -> Option<String> {
        let mut s = String::new();
        if let Some(a) = &self.accept {
            s.push_str(a);
        }
        if let Some(g) = &self.guard {
            if !s.is_empty() {
                s.push(' ');
            }
            s.push_str(&format!("[{g}]"));
        }
        if let Some(e) = &self.effect {
            if !s.is_empty() {
                s.push(' ');
            }
            s.push_str(&format!("/ {e}"));
        }
        (!s.is_empty()).then_some(s)
    }
}

fn non_empty(s: Option<&str>) -> Option<String> {
    s.map(str::trim).filter(|s| !s.is_empty()).map(str::to_string)
}

/// The display text of an `accept:` value: a string is a payload; a map
/// carries `payload`, or a time/change trigger under `after`/`when`/`at`.
fn accept_text(v: &serde_yaml::Value) -> Option<String> {
    match v {
        serde_yaml::Value::String(s) => non_empty(Some(s)).map(|s| short_name(&s).to_string()),
        serde_yaml::Value::Mapping(m) => {
            if let Some(p) = non_empty(map_str(m, "payload")) {
                return Some(short_name(&p).to_string());
            }
            for key in ["after", "when", "at"] {
                if let Some(x) = non_empty(map_str(m, key)) {
                    return Some(format!("{key} {x}"));
                }
            }
            None
        }
        _ => None,
    }
}

/// The display text of an action value (`entryAction:`, `effect:`, …): a
/// string is a qualified name (last segment shown); a map shows its `name`,
/// else the last segment of its `typedBy`.
pub(crate) fn action_text(v: &serde_yaml::Value) -> Option<String> {
    match v {
        serde_yaml::Value::String(s) => non_empty(Some(s)).map(|s| short_name(&s).to_string()),
        serde_yaml::Value::Mapping(m) => non_empty(map_str(m, "name"))
            .or_else(|| non_empty(map_str(m, "typedBy")).map(|t| short_name(&t).to_string())),
        _ => None,
    }
}

fn parse_transition(t: &serde_yaml::Value, implicit_source: Option<&str>) -> Option<Transition> {
    let m = t.as_mapping()?;
    let source = non_empty(map_str(m, "source").or_else(|| map_str(m, "from"))).or_else(|| implicit_source.map(str::to_string));
    let target = non_empty(map_str(m, "target").or_else(|| map_str(m, "to")));
    let accept = m
        .get(serde_yaml::Value::String("accept".into()))
        .and_then(accept_text)
        .or_else(|| non_empty(map_str(m, "trigger")).map(|s| short_name(&s).to_string()));
    Some(Transition {
        name: non_empty(map_str(m, "name")),
        source,
        target,
        accept,
        guard: non_empty(map_str(m, "guard")),
        effect: m.get(serde_yaml::Value::String("effect".into())).and_then(action_text),
    })
}

/// Every transition of one region: each substate's nested `transitions:`
/// (the substate is the implicit source), then the region's top-level list
/// (explicit `source:`), in authoring order. Mirrors the validator's
/// `transitions_from`.
pub(crate) fn transitions_of(sub_states: Option<&[serde_yaml::Value]>, top: Option<&[serde_yaml::Value]>) -> Vec<Transition> {
    let mut out = Vec::new();
    for s in sub_states.unwrap_or(&[]) {
        let Some(sm) = s.as_mapping() else { continue };
        let name = map_str(sm, "name");
        if let Some(serde_yaml::Value::Sequence(ts)) = sm.get(serde_yaml::Value::String("transitions".into())) {
            out.extend(ts.iter().filter_map(|t| parse_transition(t, name)));
        }
    }
    out.extend(top.unwrap_or(&[]).iter().filter_map(|t| parse_transition(t, None)));
    out
}

/// A region to resolve transitions in once every node exists.
struct Region {
    /// Qualified-name prefix of the region's states (`<subject>` or
    /// `<subject>::<container>`).
    path: String,
    transitions: Vec<Transition>,
}

fn state_node(id: String, qname: &str, label: String, parent: Option<String>) -> Node {
    Node {
        id,
        element_ref: qname.to_string(),
        resolved: true,
        element_type: Some("State".to_string()),
        kind: NodeKind::State,
        label,
        stereotype: None,
        parent,
        direction: None,
        side: None,
        lines: Vec::new(),
        is_abstract: false,
        pin: None,
        banners: Vec::new(),
        feature: None,
    }
}

fn pseudo_node(id: String, qname: &str, kind: NodeKind, parent: Option<String>) -> Node {
    Node {
        id,
        element_ref: qname.to_string(),
        resolved: true,
        element_type: None,
        kind,
        label: String::new(),
        stereotype: None,
        parent,
        direction: None,
        side: None,
        lines: Vec::new(),
        is_abstract: false,
        pin: None,
        banners: Vec::new(),
        feature: None,
    }
}

fn bool_of(m: &serde_yaml::Mapping, key: &str) -> bool {
    m.get(serde_yaml::Value::String(key.into())).and_then(|v| v.as_bool()).unwrap_or(false)
}

/// Emit the states of one region (`sub_states`) under `parent`, with their
/// compartments and pseudostates; nested machines one level deep. Returns
/// the region's transitions for the edge pass.
#[allow(clippy::too_many_arguments)]
fn emit_region(
    graph: &mut DiagramGraph,
    regions: &mut Vec<Region>,
    owner_id: &str,
    path: &str,
    owner_qname: &str,
    parent: Option<&str>,
    sub_states: &[serde_yaml::Value],
    top: Option<&[serde_yaml::Value]>,
    keep: &dyn Fn(&str) -> bool,
    nested: bool,
    elements: &[RawElement],
    resolver: &Resolver,
) {
    let states: Vec<&serde_yaml::Mapping> = sub_states
        .iter()
        .filter_map(|v| v.as_mapping())
        .filter(|m| map_str(m, "name").map(|n| keep(n)).unwrap_or(false))
        .collect();
    let any_initial = states.iter().any(|m| bool_of(m, "isInitial"));
    let any_final = states.iter().any(|m| bool_of(m, "isFinal"));
    let initial_id = format!("{owner_id}-initial");
    let final_id = format!("{owner_id}-final");
    if any_initial {
        graph.nodes.push(pseudo_node(initial_id.clone(), owner_qname, NodeKind::Initial, parent.map(str::to_string)));
    }
    let mut pseudo_edges: Vec<Edge> = Vec::new();
    for m in &states {
        let Some(name) = map_str(m, "name") else { continue };
        let qn = format!("{path}::{name}");
        let id = derived_shape_id(&qn);
        graph.nodes.push(state_node(id.clone(), &qn, name.to_string(), parent.map(str::to_string)));
        let lines: Vec<String> = [("entry", "entryAction"), ("do", "doAction"), ("exit", "exitAction")]
            .iter()
            .filter_map(|(word, key)| m.get(serde_yaml::Value::String((*key).into())).and_then(action_text).map(|a| format!("{word} / {a}")))
            .collect();
        if !lines.is_empty() {
            graph.nodes.push(Node {
                id: format!("{id}-compartment"),
                element_ref: qn.clone(),
                resolved: true,
                element_type: None,
                kind: NodeKind::Compartment,
                label: String::new(),
                stereotype: None,
                parent: Some(id.clone()),
                direction: None,
                side: None,
                lines,
                is_abstract: false,
                pin: None,
                banners: Vec::new(),
                feature: None,
            });
        }
        if bool_of(m, "isInitial") {
            pseudo_edges.push(Edge {
                id: format!("e-transition-{initial_id}-{id}"),
                element_ref: Some(qn.clone()),
                source: initial_id.clone(),
                target: id.clone(),
                kind: EdgeKind::Transition,
                label: None,
                waypoints: None,
            });
        }
        if bool_of(m, "isFinal") {
            pseudo_edges.push(Edge {
                id: format!("e-transition-{id}-{final_id}"),
                element_ref: Some(qn.clone()),
                source: id.clone(),
                target: final_id.clone(),
                kind: EdgeKind::Transition,
                label: None,
                waypoints: None,
            });
        }
        // A substate typed by a machine of its own becomes a container, one level deep.
        if !nested {
            let inner = map_str(m, "typedBy")
                .and_then(|t| resolver.resolve_ref(elements, t))
                .filter(|d| matches!(d.frontmatter.element_type, Some(ElementType::StateDef)))
                .filter(|d| d.frontmatter.sub_states.as_ref().map(|s| !s.is_empty()).unwrap_or(false));
            if let Some(def) = inner {
                emit_region(
                    graph,
                    regions,
                    &id,
                    &qn,
                    &qn,
                    Some(&id),
                    def.frontmatter.sub_states.as_deref().unwrap_or(&[]),
                    def.frontmatter.transitions.as_deref(),
                    &|_| true,
                    true,
                    elements,
                    resolver,
                );
            }
        }
    }
    if any_final {
        graph.nodes.push(pseudo_node(final_id, owner_qname, NodeKind::Final, parent.map(str::to_string)));
    }
    graph.edges.extend(pseudo_edges);
    regions.push(Region { path: path.to_string(), transitions: transitions_of(Some(sub_states), top) });
}

pub fn generate(
    graph: &mut DiagramGraph,
    subject: &RawElement,
    elements: &[RawElement],
    resolver: &Resolver,
    filters: &Filters,
    issues: &mut Vec<Issue>,
) {
    let Some(st) = subject.frontmatter.element_type.as_ref() else { return };
    let def: &RawElement = match st {
        ElementType::StateDef => subject,
        ElementType::State | ElementType::ExhibitState => {
            let typed = yaml_strings(subject.frontmatter.typed_by.as_ref()).into_iter().next();
            match typed.as_deref().and_then(|t| resolver.resolve_ref(elements, t)) {
                Some(d) if matches!(d.frontmatter.element_type, Some(ElementType::StateDef)) => d,
                _ => {
                    issues.push(w418(format!(
                        "`subject` '{}' is a {} whose `typedBy` does not resolve to a StateDef",
                        subject.qualified_name,
                        st.name()
                    )));
                    return;
                }
            }
        }
        _ => {
            issues.push(w418(format!(
                "`subject` '{}' is a {} — a StateMachine subject must be a StateDef, State or ExhibitState",
                subject.qualified_name,
                st.name()
            )));
            return;
        }
    };
    let sq = subject.qualified_name.as_str();
    let subject_id = derived_shape_id(sq);
    let sub_states: &[serde_yaml::Value] = def.frontmatter.sub_states.as_deref().unwrap_or(&[]);

    let candidates: Vec<(String, String)> = sub_states
        .iter()
        .filter_map(|v| v.as_mapping().and_then(|m| map_str(m, "name")))
        .map(|n| (format!("{sq}::{n}"), n.to_string()))
        .collect();
    filters.unmatched_issues(&candidates, issues);
    let keep = |name: &str| filters.keeps(&format!("{sq}::{name}"), name);

    let mut regions: Vec<Region> = Vec::new();
    emit_region(
        graph,
        &mut regions,
        &subject_id,
        sq,
        sq,
        None,
        sub_states,
        def.frontmatter.transitions.as_deref(),
        &keep,
        false,
        elements,
        resolver,
    );

    // ── transitions: resolved by name within the region, else relative to the
    // subject, else as an absolute qualified name ─────────────────────────
    let resolve = |region: &str, name: &str| -> Option<String> {
        let tries = [format!("{region}::{name}"), format!("{sq}::{name}"), name.to_string()];
        tries
            .iter()
            .map(|q| derived_shape_id(q))
            .find(|id| graph.node(id).map(|n| n.kind == NodeKind::State).unwrap_or(false))
    };
    let mut counters: HashMap<String, usize> = HashMap::new();
    let mut edges: Vec<Edge> = Vec::new();
    for region in &regions {
        for t in &region.transitions {
            let (Some(src_name), Some(tgt_name)) = (t.source.as_deref(), t.target.as_deref()) else { continue };
            let (Some(src), Some(tgt)) = (resolve(&region.path, src_name), resolve(&region.path, tgt_name)) else { continue };
            let base = format!("e-transition-{src}-{tgt}");
            let n = counters.entry(base.clone()).or_insert(0);
            *n += 1;
            let id = if *n == 1 { base } else { format!("{base}-{n}") };
            let element_ref = match &t.name {
                Some(name) => format!("{}::{name}", region.path),
                None => format!("{}::{src_name}", region.path),
            };
            edges.push(Edge { id, element_ref: Some(element_ref), source: src, target: tgt, kind: EdgeKind::Transition, label: t.label(), waypoints: None });
        }
    }
    graph.edges.extend(edges);
}

#[cfg(test)]
mod tests {
    use super::super::testkit::*;
    use super::*;

    fn edges_of(g: &DiagramGraph) -> Vec<(String, String, Option<String>)> {
        g.edges.iter().map(|e| (e.source.clone(), e.target.clone(), e.label.clone())).collect()
    }

    #[test]
    fn statedef_subject_yields_states_pseudostates_and_labelled_transitions() {
        let d = diagram("StateMachine", "Sys::Modes", |_| {});
        let (g, issues) = derive_it(&d);
        assert!(issues.is_empty(), "{issues:?}");
        let states: Vec<&str> = g.nodes.iter().filter(|n| n.kind == NodeKind::State).map(|n| n.id.as_str()).collect();
        assert_eq!(states, vec!["s-sys-modes-off", "s-sys-modes-on", "s-sys-modes-fault"]);
        let on = g.node("s-sys-modes-on").unwrap();
        assert_eq!(on.label, "on");
        assert_eq!(on.element_ref, "Sys::Modes::on");
        assert_eq!(on.element_type.as_deref(), Some("State"));
        assert!(on.parent.is_none());
        // entry/do/exit lines in a compartment: string form shows the last segment, map form its name.
        let comp = g.node("s-sys-modes-on-compartment").unwrap();
        assert_eq!(comp.kind, NodeKind::Compartment);
        assert_eq!(comp.lines, vec!["entry / Startup", "do / runLoop"]);
        assert!(g.node("s-sys-modes-off-compartment").is_none());
        // Pseudostates.
        assert_eq!(g.node("s-sys-modes-initial").unwrap().kind, NodeKind::Initial);
        assert_eq!(g.node("s-sys-modes-final").unwrap().kind, NodeKind::Final);
        let edges = edges_of(&g);
        assert!(edges.contains(&("s-sys-modes-initial".into(), "s-sys-modes-off".into(), None)));
        assert!(edges.contains(&("s-sys-modes-fault".into(), "s-sys-modes-final".into(), None)));
        // Nested transition with accept map + guard + effect map; string accept; top-level with
        // the deprecated aliases; guard only.
        assert!(edges.contains(&("s-sys-modes-off".into(), "s-sys-modes-on".into(), Some("StartCommand [fuel > 0] / doStart".into()))));
        assert!(edges.contains(&("s-sys-modes-on".into(), "s-sys-modes-off".into(), Some("StopCommand / Shutdown".into()))));
        assert!(edges.contains(&("s-sys-modes-on".into(), "s-sys-modes-fault".into(), Some("[temp > max]".into()))));
        assert!(edges.contains(&("s-sys-modes-fault".into(), "s-sys-modes-off".into(), Some("ResetCommand".into()))), "{edges:?}");
        assert_eq!(g.edges.iter().filter(|e| e.kind == EdgeKind::Transition).count(), g.edges.len());
        assert_eq!(g.edges.len(), 6, "{edges:?}");
        assert!(g.edges.iter().any(|e| e.id == "e-transition-s-sys-modes-off-s-sys-modes-on"));
        assert_eq!(g.layout_hints.hierarchical, true);
    }

    #[test]
    fn a_substate_typed_by_a_machine_becomes_a_container_one_level_deep() {
        let d = diagram("StateMachine", "Sys::Mission", |_| {});
        let (g, issues) = derive_it(&d);
        assert!(issues.is_empty(), "{issues:?}");
        let active = g.node("s-sys-mission-active").unwrap();
        assert!(active.parent.is_none());
        let inner: Vec<&str> = g.children_of("s-sys-mission-active").filter(|n| n.kind == NodeKind::State).map(|n| n.id.as_str()).collect();
        assert_eq!(inner, vec!["s-sys-mission-active-off", "s-sys-mission-active-on", "s-sys-mission-active-fault"]);
        // The inner region has its own pseudostates, inside the container.
        assert_eq!(g.node("s-sys-mission-active-initial").unwrap().parent.as_deref(), Some("s-sys-mission-active"));
        assert_eq!(g.node("s-sys-mission-active-final").unwrap().parent.as_deref(), Some("s-sys-mission-active"));
        let edges = edges_of(&g);
        assert!(edges.contains(&("s-sys-mission-active-initial".into(), "s-sys-mission-active-off".into(), None)));
        assert!(edges.contains(&("s-sys-mission-active-off".into(), "s-sys-mission-active-on".into(), Some("StartCommand [fuel > 0] / doStart".into()))));
        // The outer transition between the top-level states.
        assert!(edges.contains(&("s-sys-mission-idle".into(), "s-sys-mission-active".into(), Some("Go".into()))));
        assert!(edges.contains(&("s-sys-mission-active".into(), "s-sys-mission-idle".into(), Some("[done]".into()))));
        // `Modes` itself is not expanded a second level: no `s-sys-mission-active-on-…` children.
        assert!(g.children_of("s-sys-mission-active-on").all(|n| n.kind == NodeKind::Compartment));
    }

    #[test]
    fn a_state_usage_subject_reads_its_definition() {
        let d = diagram("StateMachine", "Sys::modes", |_| {});
        let (g, issues) = derive_it(&d);
        assert!(issues.is_empty(), "{issues:?}");
        assert_eq!(g.nodes.iter().filter(|n| n.kind == NodeKind::State).count(), 3);
        assert!(g.node("s-sys-modes-on").is_some(), "ids are keyed by the subject usage's name");
    }

    #[test]
    fn a_transition_to_a_missing_state_produces_no_edge() {
        let mut elements = model();
        let modes = elements.iter_mut().find(|e| e.qualified_name == "Sys::Modes").unwrap();
        modes.frontmatter.transitions =
            Some(yaml_list("- {from: fault, to: off, trigger: Cmds::ResetCommand}\n- {source: on, target: ghost}\n- {target: off}\n"));
        let d = diagram("StateMachine", "Sys::Modes", |_| {});
        elements.push(d.clone());
        let resolver = Resolver::new(&elements);
        let (g, issues) = super::super::derive(&d, crate::vis::ir::DiagramKind::StateMachine, &elements, &resolver);
        assert!(issues.is_empty());
        assert_eq!(g.edges.len(), 6, "the dangling and the source-less transitions draw nothing (W929 is the validator's)");
    }

    #[test]
    fn include_and_exclude_filter_states_and_flag_unknown_entries() {
        let d = diagram("StateMachine", "Sys::Modes", |fm| fm.include = Some(vec!["off".into(), "Sys::Modes::on".into(), "limbo".into()]));
        let (g, issues) = derive_it(&d);
        let states: Vec<&str> = g.nodes.iter().filter(|n| n.kind == NodeKind::State).map(|n| n.id.as_str()).collect();
        assert_eq!(states, vec!["s-sys-modes-off", "s-sys-modes-on"]);
        assert!(g.node("s-sys-modes-final").is_none(), "no final state survives the include");
        assert_eq!(g.edges.len(), 3, "initial→off, off→on, on→off: {:?}", edges_of(&g));
        assert_eq!(issues, vec![super::super::w417("`include` entry 'limbo' names no member of the subject".into())]);

        let d = diagram("StateMachine", "Sys::Modes", |fm| fm.exclude = Some(vec!["fault".into()]));
        let (g, issues) = derive_it(&d);
        assert!(issues.is_empty());
        assert!(g.node("s-sys-modes-fault").is_none());
        assert!(g.edges.iter().all(|e| !e.source.contains("fault") && !e.target.contains("fault")));
    }

    #[test]
    fn wrong_subject_type_is_w418_and_empty() {
        let d = diagram("StateMachine", "Sys::Engine", |_| {});
        let (g, issues) = derive_it(&d);
        assert!(g.nodes.is_empty());
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].code, "W418");
        assert!(issues[0].message.contains("PartDef"));
    }

    #[test]
    fn transition_labels_follow_the_accept_guard_effect_format() {
        let t = |a: Option<&str>, g: Option<&str>, e: Option<&str>| Transition {
            name: None,
            source: None,
            target: None,
            accept: a.map(str::to_string),
            guard: g.map(str::to_string),
            effect: e.map(str::to_string),
        };
        assert_eq!(t(Some("Cmd"), Some("x > 1"), Some("Go")).label().as_deref(), Some("Cmd [x > 1] / Go"));
        assert_eq!(t(Some("Cmd"), None, None).label().as_deref(), Some("Cmd"));
        assert_eq!(t(None, Some("x > 1"), None).label().as_deref(), Some("[x > 1]"));
        assert_eq!(t(None, None, Some("Go")).label().as_deref(), Some("/ Go"));
        assert_eq!(t(None, None, None).label(), None);
        // Time and change triggers.
        let ts = transitions_of(None, Some(&yaml_list("- {source: a, target: b, accept: {after: 5 s}}\n- {source: a, target: b, accept: {when: ready}}\n- {source: a, target: b, trigger: Cmds::Stop}\n")));
        assert_eq!(ts.iter().map(|t| t.accept.as_deref()).collect::<Vec<_>>(), vec![Some("after 5 s"), Some("when ready"), Some("Stop")]);
    }
}
