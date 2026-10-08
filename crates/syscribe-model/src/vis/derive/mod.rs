//! Derived diagram content (`REQ-TRS-VIS-003`): a `Diagram` that declares a
//! `subject:` and no `shapes:` has its IR generated from the model by the
//! generator for its kind. BDD ([`bdd`]) and IBD ([`ibd`]) ship first
//! (`REQ-TRS-VIS-004`/`-005`); the other kinds are `REQ-TRS-VIS-015` and
//! yield an empty graph until then.
//!
//! Every generator is a pure function of `(subject, elements, resolver,
//! filters)`. Shape ids are deterministic ([`super::ir::derived_shape_id`]),
//! so `layout:` pins survive regeneration. Problems are reported as issues
//! for the validator — `W417` (an `include:`/`exclude:` entry naming no member
//! of the subject) and `W418` (a subject whose type is not valid for the
//! diagram kind; the graph is then empty) — never as a panic or a silent
//! empty picture.

pub mod action;
pub mod allocation;
pub mod bdd;
pub mod feature;
pub mod ibd;
pub mod requirement;
pub mod sequence;
pub mod state;

use crate::element::{ElementType, RawElement};
use crate::resolver::Resolver;

use super::ir::{stereotype_for_type, DiagramGraph, DiagramKind, Node, NodeKind, PortDirection};
use super::manifest::{apply_layout, Issue};

pub(crate) fn w417(message: String) -> Issue {
    Issue { code: "W417", message }
}

pub(crate) fn w418(message: String) -> Issue {
    Issue { code: "W418", message }
}

/// The `include:`/`exclude:` member filters of a derived diagram.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Filters {
    pub include: Vec<String>,
    pub exclude: Vec<String>,
}

impl Filters {
    pub fn of(elem: &RawElement) -> Filters {
        Filters {
            include: elem.frontmatter.include.clone().unwrap_or_default(),
            exclude: elem.frontmatter.exclude.clone().unwrap_or_default(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.include.is_empty() && self.exclude.is_empty()
    }

    /// Whether a member with qualified name `qname` (and short name `short`)
    /// is kept: in `include:` when that list is non-empty, and not in
    /// `exclude:`. An entry matches either the full qualified name or the
    /// member's name relative to the subject.
    pub fn keeps(&self, qname: &str, short: &str) -> bool {
        let hit = |list: &[String]| list.iter().any(|e| e == qname || e == short);
        (self.include.is_empty() || hit(&self.include)) && !hit(&self.exclude)
    }

    /// `W417` for every `include:`/`exclude:` entry that matched no candidate
    /// member (`candidates` are `(qname, short)` pairs).
    pub fn unmatched_issues(&self, candidates: &[(String, String)], issues: &mut Vec<Issue>) {
        for (field, list) in [("include", &self.include), ("exclude", &self.exclude)] {
            for entry in list {
                if !candidates.iter().any(|(q, s)| q == entry || s == entry) {
                    issues.push(w417(format!("`{field}` entry '{entry}' names no member of the subject")));
                }
            }
        }
    }
}

/// Derive the IR of `elem` (a `Diagram` with a `subject:` and no `shapes:`).
pub fn derive(elem: &RawElement, kind: DiagramKind, elements: &[RawElement], resolver: &Resolver) -> (DiagramGraph, Vec<Issue>) {
    let fm = &elem.frontmatter;
    let name = fm
        .name
        .clone()
        .unwrap_or_else(|| elem.qualified_name.rsplit("::").next().unwrap_or(&elem.qualified_name).to_string());
    let mut graph = DiagramGraph::empty(kind, &elem.qualified_name, &name, fm.subject.as_deref());
    graph.derived = true;
    let mut issues = Vec::new();
    let filters = Filters::of(elem);

    let subject = fm.subject.as_deref().and_then(|s| resolver.resolve_ref(elements, s));
    // An unresolved subject is `W401` (validator); nothing to derive from.
    if let Some(subject) = subject {
        match kind {
            DiagramKind::Bdd => bdd::generate(&mut graph, subject, elements, resolver, &filters, &mut issues),
            DiagramKind::Ibd => ibd::generate(&mut graph, subject, elements, resolver, &filters, &mut issues),
            // REQ-TRS-VIS-018..022 (the follow-on kinds of REQ-TRS-VIS-015).
            DiagramKind::StateMachine => state::generate(&mut graph, subject, elements, resolver, &filters, &mut issues),
            DiagramKind::Action => action::generate(&mut graph, subject, elements, resolver, &filters, &mut issues),
            DiagramKind::Requirement => requirement::generate(&mut graph, subject, elements, resolver, &filters, &mut issues),
            DiagramKind::Sequence => sequence::generate(&mut graph, subject, elements, resolver, &filters, &mut issues),
            DiagramKind::Allocation => allocation::generate(&mut graph, subject, elements, resolver, &filters, &mut issues),
            DiagramKind::FeatureModel => feature::generate(&mut graph, subject, elements, resolver, &filters, &mut issues),
            // UseCase and Custom have no generator.
            _ => {}
        }
    }

    apply_layout(&mut graph, fm.layout.as_ref(), &mut issues);
    (graph, issues)
}

// ── helpers shared by the generators ───────────────────────────────────────

/// The strings of a `string | [string]` frontmatter value (`typedBy:`,
/// `supertype:`).
pub(crate) fn yaml_strings(v: Option<&serde_yaml::Value>) -> Vec<String> {
    match v {
        Some(serde_yaml::Value::String(s)) => vec![s.clone()],
        Some(serde_yaml::Value::Sequence(seq)) => seq.iter().filter_map(|x| x.as_str().map(str::to_string)).collect(),
        _ => Vec::new(),
    }
}

pub(crate) fn map_str<'a>(m: &'a serde_yaml::Mapping, key: &str) -> Option<&'a str> {
    m.get(serde_yaml::Value::String(key.into())).and_then(|v| v.as_str())
}

pub(crate) fn short_name(qname: &str) -> &str {
    qname.rsplit("::").next().unwrap_or(qname)
}

pub(crate) fn display_name(e: &RawElement) -> String {
    e.frontmatter.name.clone().unwrap_or_else(|| short_name(&e.qualified_name).to_string())
}

/// One inline `features:` entry, classified.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Feature {
    pub name: String,
    pub role: FeatureRole,
    /// First `typedBy:` value as written.
    pub typed_by: Option<String>,
    pub direction: Option<PortDirection>,
    pub multiplicity: Option<String>,
    pub unit: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FeatureRole {
    Attribute,
    Port,
    Part,
    Other,
}

/// Classify every inline feature of `e`. A feature is a part usage when its
/// `type:` says so or its `typedBy:` resolves to a `PartDef`/`ItemDef`; a port
/// when `type: Port` or `typedBy:` resolves to a `PortDef`; an attribute
/// otherwise (the §3.6.1 default).
pub(crate) fn features_of(e: &RawElement, elements: &[RawElement], resolver: &Resolver) -> Vec<Feature> {
    let Some(list) = e.frontmatter.features.as_ref() else { return Vec::new() };
    let mut out = Vec::new();
    for v in list {
        let serde_yaml::Value::Mapping(m) = v else { continue };
        let Some(name) = map_str(m, "name") else { continue };
        let typed_by = yaml_strings(m.get(serde_yaml::Value::String("typedBy".into()))).into_iter().next();
        let declared = map_str(m, "type");
        let target_type = typed_by
            .as_deref()
            .and_then(|t| resolver.resolve_ref(elements, t))
            .and_then(|t| t.frontmatter.element_type.clone());
        let role = match declared {
            Some("Port") => FeatureRole::Port,
            Some("Part") | Some("Item") => FeatureRole::Part,
            Some("Attribute") => FeatureRole::Attribute,
            Some(_) => FeatureRole::Other,
            None => match target_type {
                Some(ElementType::PortDef) => FeatureRole::Port,
                Some(ElementType::PartDef) | Some(ElementType::ItemDef) => FeatureRole::Part,
                _ => FeatureRole::Attribute,
            },
        };
        out.push(Feature {
            name: name.to_string(),
            role,
            typed_by,
            direction: map_str(m, "direction").and_then(PortDirection::parse),
            multiplicity: map_str(m, "multiplicity").map(str::to_string),
            unit: map_str(m, "unit").map(str::to_string),
        });
    }
    out
}

/// A `Block`-kind node for a resolved definition/usage element, with its
/// applied-stereotype banners (`REQ-TRS-VIS-012`).
pub(crate) fn block_node(
    id: String,
    e: &RawElement,
    kind: NodeKind,
    parent: Option<String>,
    label: String,
    elements: &[RawElement],
    resolver: &Resolver,
) -> Node {
    let et = e.frontmatter.element_type.as_ref();
    Node {
        id,
        element_ref: e.qualified_name.clone(),
        resolved: true,
        element_type: et.map(|t| t.name().to_string()),
        kind,
        label,
        stereotype: et.map(stereotype_for_type),
        parent,
        direction: None,
        side: None,
        lines: Vec::new(),
        is_abstract: e.frontmatter.is_abstract.unwrap_or(false),
        pin: None,
        banners: super::banners_of(e, elements, resolver),
        feature: None,
    }
}

/// A `Port` node for an inline port feature `owner_qname::name`.
pub(crate) fn port_node(id: String, owner_qname: &str, f: &Feature, parent: &str) -> Node {
    Node {
        id,
        element_ref: format!("{owner_qname}::{}", f.name),
        resolved: true,
        element_type: Some("Port".to_string()),
        kind: NodeKind::Port,
        label: f.name.clone(),
        stereotype: Some("port".to_string()),
        parent: Some(parent.to_string()),
        direction: f.direction,
        side: None,
        lines: Vec::new(),
        is_abstract: false,
        pin: None,
        banners: Vec::new(),
        feature: None,
    }
}

/// The compartment line for an attribute/port feature: `name : Type [unit]`
/// or `port name : Type`.
pub(crate) fn feature_line(f: &Feature) -> String {
    let ty = f.typed_by.as_deref().map(short_name).unwrap_or("");
    let mut s = String::new();
    if f.role == FeatureRole::Port {
        s.push_str("port ");
    }
    s.push_str(&f.name);
    if !ty.is_empty() {
        s.push_str(" : ");
        s.push_str(ty);
    }
    if let Some(m) = f.multiplicity.as_deref() {
        if m != "1" {
            s.push_str(&format!(" [{m}]"));
        }
    }
    if let Some(u) = f.unit.as_deref() {
        s.push_str(&format!(" [{}]", short_name(u)));
    }
    if let Some(d) = f.direction {
        if f.role == FeatureRole::Port {
            s.push_str(&format!(" ({})", d.as_str()));
        }
    }
    s
}

#[cfg(test)]
pub(crate) mod testkit {
    use super::*;
    use crate::element::RawFrontmatter;

    pub fn raw(qname: &str, t: ElementType, extra: impl FnOnce(&mut RawFrontmatter)) -> RawElement {
        let mut fm = RawFrontmatter { element_type: Some(t), ..Default::default() };
        fm.name = Some(short_name(qname).to_string());
        extra(&mut fm);
        RawElement {
            qualified_name: qname.to_string(),
            file_path: format!("{}.md", qname.replace("::", "/")),
            frontmatter: fm,
            doc: String::new(),
            parse_issue: None,
            derived: Default::default(),
            derive_findings: Vec::new(),
            locale_docs: Default::default(),
            about_notes: Vec::new(),
        }
    }

    pub fn yaml(s: &str) -> serde_yaml::Value {
        serde_yaml::from_str(s).unwrap()
    }

    pub fn yaml_list(s: &str) -> Vec<serde_yaml::Value> {
        serde_yaml::from_str(s).unwrap()
    }

    /// The fixture model every derive test shares: a `Sys` package with a
    /// definition hierarchy, ports, and a composed `PowerSystem`.
    pub fn model() -> Vec<RawElement> {
        vec![
            raw("Sys", ElementType::Package, |_| {}),
            raw("Sys::Base", ElementType::PartDef, |fm| fm.is_abstract = Some(true)),
            raw("Sys::Engine", ElementType::PartDef, |fm| {
                fm.supertype = Some(yaml("Sys::Base"));
                fm.features = Some(yaml_list(
                    "- {name: mass, typedBy: ScalarValues::Real, unit: SI::kg}\n- {name: powerOut, type: Port, typedBy: Sys::PowerPort, direction: out}\n",
                ));
            }),
            raw("Sys::Motor", ElementType::PartDef, |fm| {
                fm.supertype = Some(yaml("Sys::Base"));
                fm.features = Some(yaml_list("- {name: powerIn, type: Port, typedBy: Sys::PowerPort, direction: in}\n"));
            }),
            raw("Sys::PowerPort", ElementType::PortDef, |_| {}),
            raw("Sys::PowerLink", ElementType::ConnectionDef, |fm| {
                fm.ends = Some(yaml_list("- {name: source, typedBy: Sys::Engine}\n- {name: target, typedBy: Sys::Motor}\n"));
            }),
            raw("Sys::PowerSystem", ElementType::PartDef, |fm| {
                fm.features = Some(yaml_list(
                    "- {name: engine, typedBy: Sys::Engine}\n- {name: motor, typedBy: Sys::Motor, multiplicity: \"2\"}\n- {name: mainOut, type: Port, typedBy: Sys::PowerPort, direction: out}\n",
                ));
                fm.connections = Some(yaml_list("- {typedBy: Sys::PowerLink, from: engine.powerOut, to: motor.powerIn}\n"));
                fm.binding_connections = Some(yaml_list("- {left: motor.powerIn, right: mainOut}\n"));
            }),
            // A child Part element of PowerSystem (file-per-usage form).
            raw("Sys::PowerSystem::aux", ElementType::Part, |fm| {
                fm.typed_by = Some(yaml("Sys::Motor"));
            }),
            // Not a structural kind: never on a BDD.
            raw("Sys::Startup", ElementType::ActionDef, |_| {}),
            // A stereotype and a definition applying it (REQ-TRS-VIS-012 banners).
            raw("Sys::Safety", ElementType::MetadataDef, |_| {}),
            raw("Sys::Sensor", ElementType::PartDef, |fm| {
                fm.metadata = Some(yaml_list("- Sys::Safety\n- {type: ModelingMetadata::Rationale, text: why}\n"));
            }),
            // A state machine (REQ-TRS-VIS-018): nested transitions in both
            // accept spellings, a top-level one in the deprecated aliases,
            // entry/do actions in string and map form, initial and final.
            raw("Sys::Modes", ElementType::StateDef, |fm| {
                fm.sub_states = Some(yaml_list(
                    "- name: off\n  isInitial: true\n  transitions:\n    - target: on\n      accept: {payload: Cmds::StartCommand}\n      guard: \"fuel > 0\"\n      effect: {name: doStart, typedBy: Sys::Startup}\n\
                     - name: on\n  entryAction: Sys::Startup\n  doAction: {name: runLoop, typedBy: Sys::Loop}\n  transitions:\n    - target: off\n      accept: Cmds::StopCommand\n      effect: Sys::Shutdown\n    - target: fault\n      guard: \"temp > max\"\n\
                     - name: fault\n  isFinal: true\n",
                ));
                fm.transitions = Some(yaml_list("- {from: fault, to: off, trigger: Cmds::ResetCommand}\n"));
            }),
            // A machine whose `active` substate is typed by `Modes` (a container, one level deep).
            raw("Sys::Mission", ElementType::StateDef, |fm| {
                fm.sub_states = Some(yaml_list(
                    "- name: idle\n  isInitial: true\n  transitions:\n    - target: active\n      accept: Cmds::Go\n- name: active\n  typedBy: Sys::Modes\n  transitions:\n    - target: idle\n      guard: done\n",
                ));
            }),
            // A state usage subject reads its definition.
            raw("Sys::modes", ElementType::State, |fm| fm.typed_by = Some(yaml("Sys::Modes"))),
            // An action flow (REQ-TRS-VIS-019): perform/send/accept steps, an
            // if/else, a loop, fork/join control nodes and successions.
            raw("Sys::Flight", ElementType::ActionDef, |fm| {
                fm.sub_actions = Some(yaml_list(
                    "- {name: takeoff, kind: PerformAction, typedBy: Sys::Startup}\n\
                     - name: check\n  kind: IfAction\n  condition: \"wind > 12\"\n  then:\n    - {name: abort, kind: SendAction, payload: Cmds::Abort, via: ctrlOut}\n  else:\n    - {name: proceed, kind: PerformAction, typedBy: Sys::Nav}\n\
                     - name: cruise\n  kind: LoopAction\n  loopKind: for\n  variable: wp\n  sequence: waypoints\n  body:\n    - {name: await, kind: AcceptAction, payload: Cmds::Fix, trigger: {kind: change, condition: \"near(wp)\"}}\n    - {name: advance, kind: Action}\n\
                     - {name: land, kind: PerformAction, typedBy: Sys::Shutdown}\n",
                ));
                fm.control_nodes = Some(yaml_list("- {name: start, kind: ForkNode}\n- {name: end, kind: JoinNode}\n"));
                fm.succession_connections = Some(yaml_list(
                    "- {after: start, before: takeoff}\n- {after: takeoff, before: check}\n- {after: check, before: cruise, guard: ok}\n- {after: cruise, before: land}\n- {after: land, before: end}\n",
                ));
                fm.flow_connections = Some(yaml_list("- {from: takeoff.alt, to: cruise.alt}\n"));
            }),
            // An action usage subject reads its definition.
            raw("Sys::flight", ElementType::Action, |fm| fm.typed_by = Some(yaml("Sys::Flight"))),
            // A requirements package (REQ-TRS-VIS-020): a parent requirement, a
            // RequirementDef owning a derived child, a satisfying PartDef and a
            // verifying TestCase. Kept out of `Sys` so the BDD tests above see
            // exactly the members they list.
            raw("Reqs", ElementType::Package, |_| {}),
            raw("Reqs::Parent", ElementType::Requirement, |fm| {
                fm.name = Some("Parent requirement".into());
                fm.id = Some("REQ-TK-001".into());
                fm.status = Some("approved".into());
            }),
            raw("Reqs::Safety", ElementType::RequirementDef, |fm| fm.is_abstract = Some(true)),
            raw("Reqs::Safety::Child", ElementType::Requirement, |fm| {
                fm.id = Some("REQ-TK-002".into());
                fm.derived_from = Some(vec!["REQ-TK-001".into()]);
            }),
            raw("Reqs::Controller", ElementType::PartDef, |fm| fm.satisfies = Some(vec!["REQ-TK-002".into()])),
            raw("Reqs::ControllerTest", ElementType::TestCase, |fm| {
                fm.id = Some("TC-TK-001".into());
                fm.verifies = Some(vec!["REQ-TK-002".into()]);
            }),
            // An allocations package (REQ-TRS-VIS-022) with every pair form: a
            // part's own `allocatedTo:`, an AllocationDef's `allocations:` (both
            // key spellings), and an Allocation element's top-level pair plus an
            // inline `features:` entry whose target does not resolve.
            raw("Alloc", ElementType::Package, |_| {}),
            raw("Alloc::CtrlSw", ElementType::PartDef, |fm| fm.allocated_to = Some(vec!["Sys::Motor".into()])),
            raw("Alloc::FnDef", ElementType::AllocationDef, |fm| {
                fm.allocations = Some(yaml_list(
                    "- {name: navToCtrl, allocatedFrom: Sys::Startup, allocatedTo: Alloc::CtrlSw}\n- {name: navToMotor, from: Sys::Startup, to: Sys::Motor}\n",
                ));
            }),
            raw("Alloc::FnToHw", ElementType::Allocation, |fm| {
                fm.allocated_from = Some(vec!["Sys::Startup".into()]);
                fm.allocated_to = Some(vec!["Sys::Engine".into()]);
                fm.features = Some(yaml_list("- {name: ctrlToGhost, type: Allocation, allocatedFrom: Reqs::Controller, allocatedTo: Ghost::Hw}\n"));
            }),
            // A behaviour package for the Sequence generator (REQ-TRS-VIS-021),
            // outside `Sys` so the BDD member list above is untouched: a
            // controller that performs `Startup` and owns the `statusIn` port,
            // a motor, an operator actor, and the action with a send `to` a
            // part, an accept `via` a port, an if with a send in each branch
            // (one to an unresolvable chain), a loop with a body send, and
            // successions that reorder the declaration.
            raw("Flow", ElementType::Package, |_| {}),
            raw("Flow::Controller", ElementType::PartDef, |fm| {
                fm.features = Some(yaml_list("- {name: statusIn, type: Port, typedBy: Sys::PowerPort, direction: in}\n"));
                fm.performs = Some(yaml_list("- {name: run, typedBy: Flow::Startup}\n"));
            }),
            raw("Flow::Motor", ElementType::PartDef, |_| {}),
            raw("Flow::Operator", ElementType::PartDef, |_| {}),
            raw("Flow::Startup", ElementType::ActionDef, |fm| {
                fm.actors = Some(vec!["Flow::Operator".into()]);
                fm.sub_actions = Some(yaml_list(
                    "- {name: spinUp, kind: PerformAction, typedBy: Sys::Startup}\n\
                     - {name: sendPower, kind: SendAction, payload: Flow::PowerCmd, to: Flow::Motor}\n\
                     - {name: awaitReady, kind: AcceptAction, payload: Flow::Ready, via: statusIn}\n\
                     - {name: checkTemp, kind: IfAction, condition: \"temp > 90\", then: [{name: coolDown, kind: SendAction, payload: Flow::Cool, to: fan.ctrl}], else: [{name: proceed, kind: SendAction, payload: Flow::Go, to: Flow::Motor}]}\n\
                     - {name: pollLoop, kind: LoopAction, loopKind: while, condition: \"not ready\", body: [{name: poll, kind: SendAction, payload: Flow::Poll, via: statusIn}]}\n\
                     - {name: finish, kind: PerformAction, typedBy: Sys::Startup}\n",
                ));
                fm.succession_connections = Some(yaml_list(
                    "- {after: spinUp, before: awaitReady}\n- {after: awaitReady, before: sendPower}\n- {after: sendPower, before: pollLoop}\n- {after: pollLoop, before: checkTemp}\n- {after: checkTemp, before: finish}\n",
                ));
            }),
        ]
    }

    pub fn diagram(kind: &str, subject: &str, extra: impl FnOnce(&mut RawFrontmatter)) -> RawElement {
        raw("Diagrams::D", ElementType::Diagram, |fm| {
            fm.diagram_kind = Some(kind.into());
            fm.subject = Some(subject.into());
            extra(fm);
        })
    }

    pub fn derive_it(d: &RawElement) -> (DiagramGraph, Vec<Issue>) {
        let mut elements = model();
        elements.push(d.clone());
        let resolver = Resolver::new(&elements);
        derive(d, DiagramKind::parse(d.frontmatter.diagram_kind.as_deref()).unwrap(), &elements, &resolver)
    }
}

#[cfg(test)]
mod tests {
    use super::testkit::*;
    use super::*;

    #[test]
    fn filters_match_qualified_or_short_names() {
        let f = Filters { include: vec!["Sys::Engine".into(), "motor".into()], exclude: vec!["aux".into()] };
        assert!(f.keeps("Sys::Engine", "Engine"));
        assert!(f.keeps("Sys::PowerSystem::motor", "motor"));
        assert!(!f.keeps("Sys::Motor", "Motor"));
        assert!(!f.keeps("Sys::PowerSystem::aux", "aux"));
        let mut issues = Vec::new();
        f.unmatched_issues(&[("Sys::Engine".into(), "Engine".into())], &mut issues);
        let msgs: Vec<&str> = issues.iter().map(|i| i.message.as_str()).collect();
        assert_eq!(issues.iter().filter(|i| i.code == "W417").count(), 2, "{msgs:?}");
        assert!(msgs.iter().any(|m| m.contains("`include` entry 'motor'")));
        assert!(msgs.iter().any(|m| m.contains("`exclude` entry 'aux'")));
    }

    #[test]
    fn features_are_classified_by_type_or_target() {
        let elements = model();
        let resolver = Resolver::new(&elements);
        let ps = elements.iter().find(|e| e.qualified_name == "Sys::PowerSystem").unwrap();
        let fs = features_of(ps, &elements, &resolver);
        assert_eq!(fs.iter().map(|f| f.role).collect::<Vec<_>>(), vec![FeatureRole::Part, FeatureRole::Part, FeatureRole::Port]);
        assert_eq!(fs[2].direction, Some(PortDirection::Out));
        let engine = elements.iter().find(|e| e.qualified_name == "Sys::Engine").unwrap();
        let fs = features_of(engine, &elements, &resolver);
        assert_eq!(fs[0].role, FeatureRole::Attribute);
        assert_eq!(feature_line(&fs[0]), "mass : Real [kg]");
        assert_eq!(feature_line(&fs[1]), "port powerOut : PowerPort (out)");
    }

    #[test]
    fn unresolved_subject_yields_empty_graph_without_issues() {
        let d = diagram("BDD", "Nope::Missing", |_| {});
        let (g, issues) = derive_it(&d);
        assert!(g.nodes.is_empty());
        assert!(issues.is_empty(), "W401 is the validator's finding, not a derive issue: {issues:?}");
    }

    #[test]
    fn kinds_without_a_generator_yield_an_empty_graph() {
        let d = diagram("UseCase", "Sys::Engine", |_| {});
        let (g, issues) = derive_it(&d);
        assert!(g.nodes.is_empty() && issues.is_empty());
        assert!(g.derived, "a derived graph is flagged even when its generator yields nothing");
    }

    #[test]
    fn block_nodes_carry_applied_stereotype_banners() {
        let d = diagram("BDD", "Sys", |_| {});
        let (g, _) = derive_it(&d);
        assert_eq!(g.node("s-sys-sensor").unwrap().banners, vec!["Safety", "Rationale"]);
        assert!(g.node("s-sys-engine").unwrap().banners.is_empty());
    }

    #[test]
    fn layout_pins_apply_to_derived_ids() {
        let d = diagram("BDD", "Sys", |fm| {
            fm.layout = Some(yaml("s-sys-engine: {x: 5, y: 6}\ns-gone: {x: 0, y: 0}\n"));
        });
        let (g, issues) = derive_it(&d);
        assert_eq!(g.node("s-sys-engine").unwrap().pin.map(|p| (p.x, p.y)), Some((5.0, 6.0)));
        assert_eq!(issues.iter().filter(|i| i.code == "W416").count(), 1);
    }
}
