//! The Diagram intermediate representation (`REQ-TRS-VIS-001`, `ADR-SYS-VIS-001`).
//!
//! A [`DiagramGraph`] is a plain, serialisable value: a flat list of
//! [`Node`]s (nesting by `parent`) and [`Edge`]s with closed kind vocabularies,
//! plus per-kind [`LayoutHints`]. It knows nothing about sprotty, PlantUML or
//! SVG — every renderer and exporter is a pure function of this value, and
//! nothing downstream reads `shapes:`/`edges:`/`layout:` frontmatter directly.
//!
//! Pins ([`Node::pin`], [`Edge::waypoints`]) are optional geometry. A node
//! without a pin is laid out by the browser's ELK engine; a node with one is
//! fixed. The IR carries both and leaves the choice to the consumer.

use serde::{Deserialize, Serialize};

use crate::element::ElementType;

/// The diagram kinds that have an IR. `Mermaid`/`PlantUML`-kind diagrams are
/// hand-authored bodies and have none.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DiagramKind {
    Bdd,
    Ibd,
    StateMachine,
    Sequence,
    Requirement,
    Allocation,
    UseCase,
    /// `diagramKind: Action` — an action-flow view (`REQ-TRS-VIS-019`).
    Action,
    /// `diagramKind: FeatureModel` — a feature diagram (`REQ-TRS-FMED-001`).
    FeatureModel,
    /// `diagramKind: FaultTree` — a fault tree with gate symbols and the
    /// quantitative overlay of `fta::analyze_fault_tree` (GH #223).
    FaultTree,
    /// `diagramKind: AttackTree` — an attack tree coloured by rolled-up
    /// attack feasibility (GH #223).
    AttackTree,
    /// `diagramKind: SafetyCase` (alias `GSN`) — a Goal Structuring Notation
    /// argument (GH #223).
    SafetyCase,
    /// `diagramKind: Custom`, the legacy `SVG` default, or no `diagramKind` at
    /// all: a manifest with no kind-specific conventions.
    Custom,
}

impl DiagramKind {
    /// Map a `diagramKind:` frontmatter value to a kind. `None` for the
    /// hand-authored kinds (`Mermaid`, `PlantUML`) and for anything unknown.
    pub fn parse(s: Option<&str>) -> Option<DiagramKind> {
        match s {
            None | Some("SVG") | Some("Custom") => Some(DiagramKind::Custom),
            Some("BDD") => Some(DiagramKind::Bdd),
            Some("IBD") => Some(DiagramKind::Ibd),
            Some("StateMachine") => Some(DiagramKind::StateMachine),
            Some("Sequence") => Some(DiagramKind::Sequence),
            Some("Requirement") => Some(DiagramKind::Requirement),
            Some("Allocation") => Some(DiagramKind::Allocation),
            Some("UseCase") => Some(DiagramKind::UseCase),
            Some("Action") => Some(DiagramKind::Action),
            Some("FeatureModel") => Some(DiagramKind::FeatureModel),
            Some("FaultTree") => Some(DiagramKind::FaultTree),
            Some("AttackTree") => Some(DiagramKind::AttackTree),
            Some("SafetyCase") | Some("GSN") => Some(DiagramKind::SafetyCase),
            _ => None,
        }
    }

    /// The `diagramKind:` spelling.
    pub fn as_str(&self) -> &'static str {
        match self {
            DiagramKind::Bdd => "BDD",
            DiagramKind::Ibd => "IBD",
            DiagramKind::StateMachine => "StateMachine",
            DiagramKind::Sequence => "Sequence",
            DiagramKind::Requirement => "Requirement",
            DiagramKind::Allocation => "Allocation",
            DiagramKind::UseCase => "UseCase",
            DiagramKind::Action => "Action",
            DiagramKind::FeatureModel => "FeatureModel",
            DiagramKind::FaultTree => "FaultTree",
            DiagramKind::AttackTree => "AttackTree",
            DiagramKind::SafetyCase => "SafetyCase",
            DiagramKind::Custom => "Custom",
        }
    }
}

/// Node roles — the union of every `kind:` value spec §8.16.8 defines for
/// shapes, across all diagram kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NodeKind {
    /// IBD: the subject's outer box.
    Boundary,
    /// A definition or usage box (BDD/IBD/Requirement/Allocation `block`, and
    /// every element-type-named kind such as `PartDef` or `Part`).
    Block,
    Port,
    Compartment,
    Label,
    Note,
    // Sequence
    Lifeline,
    Actor,
    Activation,
    Fragment,
    // StateMachine
    State,
    Initial,
    Final,
    Choice,
    History,
    // Requirement
    Requirement,
    TestCase,
    // Allocation
    Swimlane,
    // UseCase
    SystemBoundary,
    UseCase,
    // Action (REQ-TRS-VIS-019)
    Action,
    Fork,
    Join,
    Decision,
    Merge,
    // FeatureModel (REQ-TRS-FMED-001)
    Feature,
    // FaultTree / AttackTree (GH #223): gate symbols by function, events by
    // `eventKind`, and the leaf step of an attack tree.
    GateAnd,
    GateOr,
    GateXor,
    GateNot,
    GateInhibit,
    EventBasic,
    EventUndeveloped,
    EventHouse,
    Step,
    // SafetyCase / GSN (GH #223)
    Goal,
    /// A goal declared (or found) undeveloped: the goal box with GSN's diamond.
    UndevelopedGoal,
    Strategy,
    Solution,
    Context,
    Justification,
    Assumption,
}

impl NodeKind {
    /// The `kind:` spelling (spec §8.16.8).
    pub fn as_str(&self) -> &'static str {
        match self {
            NodeKind::Boundary => "boundary",
            NodeKind::Block => "block",
            NodeKind::Port => "port",
            NodeKind::Compartment => "compartment",
            NodeKind::Label => "label",
            NodeKind::Note => "note",
            NodeKind::Lifeline => "lifeline",
            NodeKind::Actor => "actor",
            NodeKind::Activation => "activation",
            NodeKind::Fragment => "fragment",
            NodeKind::State => "state",
            NodeKind::Initial => "initial",
            NodeKind::Final => "final",
            NodeKind::Choice => "choice",
            NodeKind::History => "history",
            NodeKind::Requirement => "requirement",
            NodeKind::TestCase => "testcase",
            NodeKind::Swimlane => "swimlane",
            NodeKind::SystemBoundary => "system-boundary",
            NodeKind::UseCase => "usecase",
            NodeKind::Action => "action",
            NodeKind::Fork => "fork",
            NodeKind::Join => "join",
            NodeKind::Decision => "decision",
            NodeKind::Merge => "merge",
            NodeKind::Feature => "feature",
            NodeKind::GateAnd => "gate-and",
            NodeKind::GateOr => "gate-or",
            NodeKind::GateXor => "gate-xor",
            NodeKind::GateNot => "gate-not",
            NodeKind::GateInhibit => "gate-inhibit",
            NodeKind::EventBasic => "event-basic",
            NodeKind::EventUndeveloped => "event-undeveloped",
            NodeKind::EventHouse => "event-house",
            NodeKind::Step => "step",
            NodeKind::Goal => "goal",
            NodeKind::UndevelopedGoal => "undeveloped-goal",
            NodeKind::Strategy => "strategy",
            NodeKind::Solution => "solution",
            NodeKind::Context => "context",
            NodeKind::Justification => "justification",
            NodeKind::Assumption => "assumption",
        }
    }

    /// Parse a role name, case-insensitively, with `-`/`_` treated alike
    /// (`system-boundary`, `SystemBoundary` and `system_boundary` all parse).
    pub fn parse_role(s: &str) -> Option<NodeKind> {
        let norm: String = s.chars().filter(|c| *c != '-' && *c != '_').collect::<String>().to_ascii_lowercase();
        Some(match norm.as_str() {
            "boundary" => NodeKind::Boundary,
            "block" => NodeKind::Block,
            "port" => NodeKind::Port,
            "compartment" => NodeKind::Compartment,
            "label" => NodeKind::Label,
            "note" => NodeKind::Note,
            "lifeline" => NodeKind::Lifeline,
            "actor" => NodeKind::Actor,
            "activation" => NodeKind::Activation,
            "fragment" => NodeKind::Fragment,
            "state" => NodeKind::State,
            "initial" => NodeKind::Initial,
            "final" => NodeKind::Final,
            "choice" => NodeKind::Choice,
            "history" => NodeKind::History,
            "requirement" => NodeKind::Requirement,
            "testcase" => NodeKind::TestCase,
            "swimlane" => NodeKind::Swimlane,
            "systemboundary" => NodeKind::SystemBoundary,
            "usecase" => NodeKind::UseCase,
            "action" => NodeKind::Action,
            "fork" | "forknode" => NodeKind::Fork,
            "join" | "joinnode" => NodeKind::Join,
            "decision" | "decisionnode" => NodeKind::Decision,
            "merge" | "mergenode" => NodeKind::Merge,
            "feature" => NodeKind::Feature,
            "gateand" => NodeKind::GateAnd,
            "gateor" => NodeKind::GateOr,
            "gatexor" => NodeKind::GateXor,
            "gatenot" => NodeKind::GateNot,
            "gateinhibit" => NodeKind::GateInhibit,
            "eventbasic" => NodeKind::EventBasic,
            "eventundeveloped" => NodeKind::EventUndeveloped,
            "eventhouse" => NodeKind::EventHouse,
            "step" => NodeKind::Step,
            "goal" => NodeKind::Goal,
            "undevelopedgoal" => NodeKind::UndevelopedGoal,
            "strategy" => NodeKind::Strategy,
            "solution" => NodeKind::Solution,
            "context" => NodeKind::Context,
            "justification" => NodeKind::Justification,
            "assumption" => NodeKind::Assumption,
            _ => return None,
        })
    }

    /// Whether nodes of this kind are containers whose children are laid out
    /// inside them (as opposed to decorations attached to a parent).
    pub fn is_container(&self) -> bool {
        matches!(
            self,
            NodeKind::Boundary
                | NodeKind::Block
                | NodeKind::Swimlane
                | NodeKind::SystemBoundary
                | NodeKind::Fragment
                | NodeKind::State
                | NodeKind::Action
        )
    }
}

/// Edge roles — the union of every `kind:` value spec §8.16.8 defines for
/// edges, across all diagram kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EdgeKind {
    // IBD
    Connection,
    Flow,
    Binding,
    Succession,
    // BDD / UseCase
    Inheritance,
    Association,
    Composition,
    Aggregation,
    Dependency,
    // Sequence
    Message,
    Return,
    Create,
    Destroy,
    // StateMachine
    Transition,
    // Requirement
    Containment,
    Derive,
    Satisfy,
    Verify,
    Refine,
    Trace,
    Copy,
    // Allocation
    Allocation,
    // UseCase
    Include,
    Extend,
    // FeatureModel (REQ-TRS-FMED-001)
    /// Parent to child in the feature tree.
    FeatureChild,
    /// A cross-tree `requires:` constraint.
    Requires,
    /// A cross-tree `excludes:` constraint.
    Excludes,
    // FaultTree / AttackTree (GH #223)
    /// A gate to one of its `inputs:`.
    GateInput,
    /// A gate input on the easiest attack path (drawn heavier).
    CriticalPath,
    // SafetyCase / GSN (GH #223)
    /// GSN SupportedBy: a goal or strategy to what supports it.
    SupportedBy,
    /// GSN InContextOf: a goal or strategy to its context, justification or assumption.
    InContextOf,
}

impl EdgeKind {
    /// The `kind:` spelling (spec §8.16.8).
    pub fn as_str(&self) -> &'static str {
        match self {
            EdgeKind::Connection => "connection",
            EdgeKind::Flow => "flow",
            EdgeKind::Binding => "binding",
            EdgeKind::Succession => "succession",
            EdgeKind::Inheritance => "inheritance",
            EdgeKind::Association => "association",
            EdgeKind::Composition => "composition",
            EdgeKind::Aggregation => "aggregation",
            EdgeKind::Dependency => "dependency",
            EdgeKind::Message => "message",
            EdgeKind::Return => "return",
            EdgeKind::Create => "create",
            EdgeKind::Destroy => "destroy",
            EdgeKind::Transition => "transition",
            EdgeKind::Containment => "containment",
            EdgeKind::Derive => "derive",
            EdgeKind::Satisfy => "satisfy",
            EdgeKind::Verify => "verify",
            EdgeKind::Refine => "refine",
            EdgeKind::Trace => "trace",
            EdgeKind::Copy => "copy",
            EdgeKind::Allocation => "allocation",
            EdgeKind::Include => "include",
            EdgeKind::Extend => "extend",
            EdgeKind::FeatureChild => "child",
            EdgeKind::Requires => "requires",
            EdgeKind::Excludes => "excludes",
            EdgeKind::GateInput => "input",
            EdgeKind::CriticalPath => "criticalPath",
            EdgeKind::SupportedBy => "supportedBy",
            EdgeKind::InContextOf => "inContextOf",
        }
    }

    /// Parse an edge `kind:`, case-insensitively, accepting the link-field
    /// spellings authors reach for (`derivedFrom`, `verifies`, `satisfies`,
    /// `allocatedTo`, `generalization`, `usage`) as aliases of the §8.16.8
    /// names.
    pub fn parse(s: &str) -> Option<EdgeKind> {
        let norm: String = s.chars().filter(|c| *c != '-' && *c != '_').collect::<String>().to_ascii_lowercase();
        Some(match norm.as_str() {
            "connection" | "connect" => EdgeKind::Connection,
            "flow" => EdgeKind::Flow,
            "binding" | "bind" => EdgeKind::Binding,
            "succession" => EdgeKind::Succession,
            "inheritance" | "generalization" | "specialization" | "specializes" | "subclassification" => EdgeKind::Inheritance,
            "association" => EdgeKind::Association,
            "composition" => EdgeKind::Composition,
            "aggregation" => EdgeKind::Aggregation,
            "dependency" | "usage" => EdgeKind::Dependency,
            "message" => EdgeKind::Message,
            "return" => EdgeKind::Return,
            "create" => EdgeKind::Create,
            "destroy" => EdgeKind::Destroy,
            "transition" => EdgeKind::Transition,
            "containment" | "contains" => EdgeKind::Containment,
            "derive" | "derivedfrom" | "derivereqt" | "derives" => EdgeKind::Derive,
            "satisfy" | "satisfies" | "satisfiedby" => EdgeKind::Satisfy,
            "verify" | "verifies" | "verifiedby" => EdgeKind::Verify,
            "refine" | "refines" => EdgeKind::Refine,
            "trace" | "traces" => EdgeKind::Trace,
            "copy" => EdgeKind::Copy,
            "allocation" | "allocate" | "allocatedto" | "allocatedfrom" => EdgeKind::Allocation,
            "include" | "includes" => EdgeKind::Include,
            "extend" | "extends" => EdgeKind::Extend,
            "child" | "featurechild" => EdgeKind::FeatureChild,
            "requires" => EdgeKind::Requires,
            "excludes" => EdgeKind::Excludes,
            "input" | "gateinput" => EdgeKind::GateInput,
            "criticalpath" => EdgeKind::CriticalPath,
            "supportedby" => EdgeKind::SupportedBy,
            "incontextof" => EdgeKind::InContextOf,
            _ => return None,
        })
    }

    /// Whether the edge is drawn directed (has an arrowhead at the target).
    pub fn is_directed(&self) -> bool {
        !matches!(self, EdgeKind::Connection | EdgeKind::Binding | EdgeKind::Association | EdgeKind::Containment | EdgeKind::FeatureChild | EdgeKind::Excludes | EdgeKind::GateInput | EdgeKind::CriticalPath)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PortDirection {
    In,
    Out,
    Inout,
}

impl PortDirection {
    pub fn parse(s: &str) -> Option<PortDirection> {
        match s.trim().to_ascii_lowercase().as_str() {
            "in" => Some(PortDirection::In),
            "out" => Some(PortDirection::Out),
            "inout" | "in/out" => Some(PortDirection::Inout),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            PortDirection::In => "in",
            PortDirection::Out => "out",
            PortDirection::Inout => "inout",
        }
    }
}

/// Which side of its parent a port sits on, when fixed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Side {
    North,
    East,
    South,
    West,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

/// A pinned position and (optionally) size.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub w: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub h: Option<f64>,
}

/// One node of the diagram.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Node {
    /// Diagram-local id: the manifest key, or the deterministic id of a
    /// derived node. Pins are keyed by it; the sprotty model uses it as the
    /// element id.
    pub id: String,
    /// The model reference as the author wrote it (or the derived element's
    /// qualified name). Kept verbatim even when unresolved: it is what
    /// `sysml:ref` and diagram sync need.
    pub element_ref: String,
    /// Whether `element_ref` resolves to a model element (exactly, not via an
    /// ancestor). Unresolved nodes are drawn, dashed, never dropped; the
    /// `W402` finding is the validator's job.
    pub resolved: bool,
    /// The resolved element's type name (`PartDef`, `Requirement`, …), or the
    /// type an element-type-named manifest `kind:` implied.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub element_type: Option<String>,
    pub kind: NodeKind,
    /// Display name: an explicit `label:`, else the element's `name`, else the
    /// last segment of `element_ref`.
    pub label: String,
    /// SysMLv2 stereotype text without guillemets (`part def`, `port`, …).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stereotype: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub direction: Option<PortDirection>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub side: Option<Side>,
    /// Compartment content, one entry per line (attributes, ports, …).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub lines: Vec<String>,
    #[serde(default)]
    pub is_abstract: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pin: Option<Rect>,
    /// Applied-stereotype banners (`REQ-TRS-VIS-012`): the name of every
    /// `MetadataDef` the referenced element applies via its `metadata:` list,
    /// drawn as `«Name»` beneath the kind stereotype.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub banners: Vec<String>,
    /// Feature notation state of a `FeatureModel` node (`REQ-TRS-FMED-001`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub feature: Option<FeatureMark>,
    /// The generic analysis overlay (GH #223): status text, a value, a tone
    /// and badges drawn on the node by every writer. Absent on a node that
    /// carries none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mark: Option<NodeMark>,
}

/// The tone of a [`NodeMark`]: how the node is coloured (`style::node_style`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Tone {
    Ok,
    Warn,
    Bad,
    /// Muted: out of the picture's story (an unreachable event, a context node).
    #[default]
    Neutral,
}

impl Tone {
    pub fn as_str(&self) -> &'static str {
        match self {
            Tone::Ok => "ok",
            Tone::Warn => "warn",
            Tone::Bad => "bad",
            Tone::Neutral => "neutral",
        }
    }
}

/// A generic node overlay (GH #223): what an analysis says about the element
/// the node stands for. Every field is optional text the writers draw as extra
/// label lines under the name (status, then value, then badges) and the tone
/// colours the node.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeMark {
    /// A short status word (`single point`, `UNDEVELOPED`, `supported`, …).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// A value line (`P = 2.0e-9`, `feasibility high`, …).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(default)]
    pub tone: Tone,
    /// Short flags drawn on one line (`W035`, `CCF`, …).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub badges: Vec<String>,
    /// Draw the outline heavier: the node is on the path or set the picture is about.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub emphasis: bool,
}

/// What a feature diagram draws on a feature beyond its name: whether it is a
/// mandatory member of its parent, and how its own children are grouped.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeatureMark {
    pub mandatory: bool,
    /// `optional`, `alternative` or `or`: the group kind of the feature's children.
    pub group: String,
    /// Number of direct children, so a collapsed node can show how many it hides.
    pub child_count: usize,
    /// Stable `FEAT-*` id, when the feature has one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The `requires:` entries this feature declares, as qualified names (the editor lists them to remove one).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub requires: Vec<String>,
    /// The `excludes:` entries this feature declares, as qualified names.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub excludes: Vec<String>,
    /// The parameter declarations as written, for the editor.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parameters: Vec<serde_json::Value>,
}

/// One edge of the diagram.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Edge {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub element_ref: Option<String>,
    /// Source node id.
    pub source: String,
    /// Target node id.
    pub target: String,
    pub kind: EdgeKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Pinned routing, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub waypoints: Option<Vec<Point>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LayoutAlgorithm {
    /// Sugiyama-style layered layout (ELK `layered`).
    Layered,
    /// Positions are given; only edges are routed (ELK `fixed`).
    Fixed,
    /// A forest laid out as trees, parents centred over their children (ELK
    /// `mrtree`): a feature diagram.
    Tree,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LayoutDirection {
    Down,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PortConstraints {
    Free,
    FixedSide,
}

/// Engine-independent layout hints, derived from the diagram kind. The
/// sprotty writer turns them into ELK option ids; the text writers ignore them.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LayoutHints {
    pub algorithm: LayoutAlgorithm,
    pub direction: LayoutDirection,
    /// Lay children out inside their parents (compound graph) rather than
    /// treating every node as top-level.
    pub hierarchical: bool,
    pub port_constraints: PortConstraints,
    /// Edges of these kinds point "up" the layering (a supertype above its
    /// subtypes): the layout engine reverses them for layering purposes.
    pub reversed_kinds: Vec<EdgeKind>,
    /// Edges of these kinds take no part in layout: they are drawn after the
    /// nodes are placed (a feature diagram's cross-tree constraints, which
    /// would otherwise pull the tree out of shape).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub overlay_kinds: Vec<EdgeKind>,
}

impl LayoutHints {
    pub fn for_kind(kind: DiagramKind) -> LayoutHints {
        match kind {
            DiagramKind::Bdd => LayoutHints {
                algorithm: LayoutAlgorithm::Layered,
                direction: LayoutDirection::Down,
                hierarchical: false,
                port_constraints: PortConstraints::Free,
                reversed_kinds: vec![EdgeKind::Inheritance],
                overlay_kinds: vec![],
            },
            DiagramKind::Ibd => LayoutHints {
                algorithm: LayoutAlgorithm::Layered,
                direction: LayoutDirection::Right,
                hierarchical: true,
                port_constraints: PortConstraints::FixedSide,
                reversed_kinds: vec![],
                overlay_kinds: vec![],
            },
            DiagramKind::StateMachine => LayoutHints {
                algorithm: LayoutAlgorithm::Layered,
                direction: LayoutDirection::Down,
                hierarchical: true,
                port_constraints: PortConstraints::Free,
                reversed_kinds: vec![],
                overlay_kinds: vec![],
            },
            DiagramKind::Sequence => LayoutHints {
                algorithm: LayoutAlgorithm::Fixed,
                direction: LayoutDirection::Down,
                hierarchical: false,
                port_constraints: PortConstraints::Free,
                reversed_kinds: vec![],
                overlay_kinds: vec![],
            },
            DiagramKind::Requirement => LayoutHints {
                algorithm: LayoutAlgorithm::Layered,
                direction: LayoutDirection::Down,
                hierarchical: false,
                port_constraints: PortConstraints::Free,
                reversed_kinds: vec![EdgeKind::Derive, EdgeKind::Satisfy, EdgeKind::Verify, EdgeKind::Refine],
                overlay_kinds: vec![],
            },
            DiagramKind::Allocation => LayoutHints {
                algorithm: LayoutAlgorithm::Layered,
                direction: LayoutDirection::Right,
                hierarchical: true,
                port_constraints: PortConstraints::Free,
                reversed_kinds: vec![],
                overlay_kinds: vec![],
            },
            DiagramKind::UseCase => LayoutHints {
                algorithm: LayoutAlgorithm::Layered,
                direction: LayoutDirection::Right,
                hierarchical: true,
                port_constraints: PortConstraints::Free,
                reversed_kinds: vec![EdgeKind::Inheritance],
                overlay_kinds: vec![],
            },
            DiagramKind::Action => LayoutHints {
                algorithm: LayoutAlgorithm::Layered,
                direction: LayoutDirection::Down,
                hierarchical: true,
                port_constraints: PortConstraints::Free,
                reversed_kinds: vec![],
                overlay_kinds: vec![],
            },
            DiagramKind::FeatureModel => LayoutHints {
                algorithm: LayoutAlgorithm::Tree,
                direction: LayoutDirection::Down,
                hierarchical: false,
                port_constraints: PortConstraints::Free,
                reversed_kinds: vec![],
                overlay_kinds: vec![EdgeKind::Requires, EdgeKind::Excludes],
            },
            // Trees and arguments read top-down; the layered engine (not mrtree)
            // because a shared event or sub-goal makes the structure a DAG.
            DiagramKind::FaultTree | DiagramKind::AttackTree | DiagramKind::SafetyCase => LayoutHints {
                algorithm: LayoutAlgorithm::Layered,
                direction: LayoutDirection::Down,
                hierarchical: false,
                port_constraints: PortConstraints::Free,
                reversed_kinds: vec![],
                overlay_kinds: vec![],
            },
            DiagramKind::Custom => LayoutHints {
                algorithm: LayoutAlgorithm::Layered,
                direction: LayoutDirection::Down,
                hierarchical: true,
                port_constraints: PortConstraints::Free,
                reversed_kinds: vec![],
                overlay_kinds: vec![],
            },
        }
    }
}

/// A whole diagram.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagramGraph {
    pub kind: DiagramKind,
    /// The `Diagram` element's qualified name.
    pub qualified_name: String,
    /// The `Diagram` element's display name.
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub layout_hints: LayoutHints,
    /// Whether the content was generated from the model (`subject:` and no
    /// `shapes:`, `REQ-TRS-VIS-003`) rather than listed in a manifest. A
    /// derived graph is regenerated on every build, so its node set is not
    /// something an editor can add to or delete from; only pins persist.
    #[serde(default)]
    pub derived: bool,
}

impl DiagramGraph {
    pub fn empty(kind: DiagramKind, qualified_name: &str, name: &str, subject: Option<&str>) -> DiagramGraph {
        DiagramGraph {
            kind,
            qualified_name: qualified_name.to_string(),
            name: name.to_string(),
            subject: subject.map(str::to_string),
            nodes: Vec::new(),
            edges: Vec::new(),
            layout_hints: LayoutHints::for_kind(kind),
            derived: false,
        }
    }

    pub fn node(&self, id: &str) -> Option<&Node> {
        self.nodes.iter().find(|n| n.id == id)
    }

    /// Direct children of `id`, in declaration order.
    pub fn children_of<'a>(&'a self, id: &'a str) -> impl Iterator<Item = &'a Node> + 'a {
        self.nodes.iter().filter(move |n| n.parent.as_deref() == Some(id))
    }

    /// Nodes with no parent, in declaration order.
    pub fn roots(&self) -> impl Iterator<Item = &Node> {
        self.nodes.iter().filter(|n| n.parent.is_none())
    }

    /// Whether every node carries a pin — the precondition for drawing the
    /// graph without a layout engine (`REQ-TRS-VIS-010`).
    pub fn is_fully_pinned(&self) -> bool {
        !self.nodes.is_empty() && self.nodes.iter().all(|n| n.pin.is_some())
    }

    /// Ids of pinned nodes, in declaration order.
    pub fn pinned_ids(&self) -> Vec<&str> {
        self.nodes.iter().filter(|n| n.pin.is_some()).map(|n| n.id.as_str()).collect()
    }
}

/// The SysMLv2 stereotype text for an element type: `PartDef` → `part def`,
/// `RequirementDef` → `requirement def`, `TestCase` → `test case`,
/// `EventOccurrenceDef` → `event occurrence def`. PascalCase split on
/// upper-case boundaries, lower-cased, joined with spaces.
pub fn stereotype_for_type(t: &ElementType) -> String {
    stereotype_for_type_name(t.name())
}

/// [`stereotype_for_type`] on a type name string.
pub fn stereotype_for_type_name(name: &str) -> String {
    let mut out = String::with_capacity(name.len() + 4);
    let chars: Vec<char> = name.chars().collect();
    for (i, c) in chars.iter().enumerate() {
        if c.is_ascii_uppercase() {
            let prev_lower = i > 0 && chars[i - 1].is_ascii_lowercase();
            let next_lower = i + 1 < chars.len() && chars[i + 1].is_ascii_lowercase();
            // Split before an upper-case letter that follows a lower-case one
            // (`PartDef`), or that starts a new word after an acronym run
            // (`ADRRecord` → `adr record`).
            if i > 0 && (prev_lower || (next_lower && chars[i - 1].is_ascii_uppercase())) {
                out.push(' ');
            }
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(*c);
        }
    }
    out
}

/// The deterministic shape id of a derived node (`REQ-TRS-VIS-003`): `s-`
/// plus the qualified name lower-cased with `::` and every run of
/// non-alphanumerics replaced by `-`.
pub fn derived_shape_id(qualified_name: &str) -> String {
    let mut out = String::from("s-");
    let mut pending_dash = false;
    for c in qualified_name.chars() {
        if c.is_ascii_alphanumeric() {
            if pending_dash && out.len() > 2 {
                out.push('-');
            }
            pending_dash = false;
            out.push(c.to_ascii_lowercase());
        } else {
            pending_dash = true;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagram_kind_parses_frontmatter_spellings_and_rejects_hand_authored_kinds() {
        assert_eq!(DiagramKind::parse(Some("BDD")), Some(DiagramKind::Bdd));
        assert_eq!(DiagramKind::parse(Some("IBD")), Some(DiagramKind::Ibd));
        assert_eq!(DiagramKind::parse(None), Some(DiagramKind::Custom));
        assert_eq!(DiagramKind::parse(Some("SVG")), Some(DiagramKind::Custom));
        assert_eq!(DiagramKind::parse(Some("Mermaid")), None);
        assert_eq!(DiagramKind::parse(Some("PlantUML")), None);
        assert_eq!(DiagramKind::parse(Some("bdd")), None, "kinds are case-sensitive like the validator");
    }

    #[test]
    fn node_kind_role_parsing_is_case_and_separator_insensitive() {
        assert_eq!(NodeKind::parse_role("block"), Some(NodeKind::Block));
        assert_eq!(NodeKind::parse_role("Port"), Some(NodeKind::Port));
        assert_eq!(NodeKind::parse_role("system-boundary"), Some(NodeKind::SystemBoundary));
        assert_eq!(NodeKind::parse_role("SystemBoundary"), Some(NodeKind::SystemBoundary));
        assert_eq!(NodeKind::parse_role("testcase"), Some(NodeKind::TestCase));
        assert_eq!(NodeKind::parse_role("gizmo"), None);
    }

    #[test]
    fn edge_kind_accepts_link_field_aliases() {
        assert_eq!(EdgeKind::parse("derivedFrom"), Some(EdgeKind::Derive));
        assert_eq!(EdgeKind::parse("verifies"), Some(EdgeKind::Verify));
        assert_eq!(EdgeKind::parse("allocatedTo"), Some(EdgeKind::Allocation));
        assert_eq!(EdgeKind::parse("generalization"), Some(EdgeKind::Inheritance));
        assert_eq!(EdgeKind::parse("usage"), Some(EdgeKind::Dependency));
        assert_eq!(EdgeKind::parse("FLOW"), Some(EdgeKind::Flow));
        assert_eq!(EdgeKind::parse("teleport"), None);
    }

    #[test]
    fn stereotypes_split_pascal_case() {
        assert_eq!(stereotype_for_type(&ElementType::PartDef), "part def");
        assert_eq!(stereotype_for_type(&ElementType::Part), "part");
        assert_eq!(stereotype_for_type(&ElementType::RequirementDef), "requirement def");
        assert_eq!(stereotype_for_type(&ElementType::TestCase), "test case");
        assert_eq!(stereotype_for_type(&ElementType::EventOccurrenceDef), "event occurrence def");
        assert_eq!(stereotype_for_type(&ElementType::ADR), "adr");
    }

    #[test]
    fn derived_shape_ids_are_deterministic_and_slug_like() {
        assert_eq!(derived_shape_id("UAV::Power::PowerSystem::pdu"), "s-uav-power-powersystem-pdu");
        assert_eq!(derived_shape_id("A"), "s-a");
        assert_eq!(derived_shape_id("Weird  Name::x_y"), "s-weird-name-x-y");
    }

    #[test]
    fn layout_hints_follow_the_design_per_kind() {
        let bdd = LayoutHints::for_kind(DiagramKind::Bdd);
        assert_eq!(bdd.direction, LayoutDirection::Down);
        assert!(!bdd.hierarchical);
        assert_eq!(bdd.reversed_kinds, vec![EdgeKind::Inheritance]);
        let ibd = LayoutHints::for_kind(DiagramKind::Ibd);
        assert_eq!(ibd.direction, LayoutDirection::Right);
        assert!(ibd.hierarchical);
        assert_eq!(ibd.port_constraints, PortConstraints::FixedSide);
    }

    #[test]
    fn graph_helpers_walk_nesting_and_pins() {
        let mut g = DiagramGraph::empty(DiagramKind::Ibd, "D::X", "X", Some("S"));
        let mk = |id: &str, parent: Option<&str>, pin: Option<Rect>| Node {
            id: id.into(),
            element_ref: id.into(),
            resolved: true,
            element_type: None,
            kind: NodeKind::Block,
            label: id.into(),
            stereotype: None,
            parent: parent.map(str::to_string),
            direction: None,
            side: None,
            lines: vec![],
            is_abstract: false,
            pin,
            banners: vec![],
            feature: None,
            mark: None,
        };
        g.nodes.push(mk("a", None, Some(Rect { x: 0.0, y: 0.0, w: None, h: None })));
        g.nodes.push(mk("b", Some("a"), None));
        assert_eq!(g.roots().count(), 1);
        assert_eq!(g.children_of("a").map(|n| n.id.as_str()).collect::<Vec<_>>(), vec!["b"]);
        assert!(!g.is_fully_pinned());
        assert_eq!(g.pinned_ids(), vec!["a"]);
        assert!(DiagramGraph::empty(DiagramKind::Bdd, "D", "D", None).is_fully_pinned() == false);
    }

    #[test]
    fn ir_round_trips_through_json() {
        let mut g = DiagramGraph::empty(DiagramKind::Bdd, "D::X", "X", None);
        g.edges.push(Edge {
            id: "e1".into(),
            element_ref: None,
            source: "a".into(),
            target: "b".into(),
            kind: EdgeKind::Inheritance,
            label: None,
            waypoints: Some(vec![Point { x: 1.0, y: 2.0 }]),
        });
        let json = serde_json::to_string(&g).unwrap();
        let back: DiagramGraph = serde_json::from_str(&json).unwrap();
        assert_eq!(back, g);
        assert!(json.contains("\"kind\":\"inheritance\""));
        assert!(json.contains("\"derived\":false"));
        // An older JSON without the flag still deserialises (manifest default).
        let legacy = json.replace(",\"derived\":false", "");
        assert!(!legacy.contains("derived"));
        let back: DiagramGraph = serde_json::from_str(&legacy).unwrap();
        assert!(!back.derived);
    }
}
