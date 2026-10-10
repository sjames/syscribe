use serde::{Deserialize, Serialize};

/// Serde helper: accept either a plain YAML string or a sequence of strings.
/// Allows `allocatedFrom: SC-001` and `allocatedFrom: [SC-001, SC-002]` both to
/// deserialize into `Option<Vec<String>>`.
/// Serialises a one-element list as a bare string, so a field authored as a single
/// reference keeps the JSON shape and content hash it had when it was a `String`
/// (GH #232: `supersedes` became a list without changing existing elements' hashes).
mod one_or_many {
    use serde::Serializer;
    pub fn serialize<S: Serializer>(v: &Option<Vec<String>>, s: S) -> Result<S::Ok, S::Error> {
        match v {
            None => s.serialize_none(),
            Some(list) if list.len() == 1 => s.serialize_str(&list[0]),
            Some(list) => serde::Serialize::serialize(list, s),
        }
    }
}

mod string_or_vec {
    use serde::{Deserialize, Deserializer};
    pub fn deserialize<'de, D>(d: D) -> Result<Option<Vec<String>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v: Option<serde_yaml::Value> = Option::deserialize(d)?;
        match v {
            None | Some(serde_yaml::Value::Null) => Ok(None),
            Some(serde_yaml::Value::String(s)) => Ok(Some(vec![s])),
            Some(serde_yaml::Value::Sequence(seq)) => {
                let mut out = Vec::with_capacity(seq.len());
                for item in seq {
                    match item {
                        serde_yaml::Value::String(s) => out.push(s),
                        other => return Err(serde::de::Error::custom(
                            format!("expected string in sequence, got {:?}", other)
                        )),
                    }
                }
                Ok(Some(out))
            }
            other => Err(serde::de::Error::custom(
                format!("expected string or list for allocatedFrom/allocatedTo, got {:?}", other)
            )),
        }
    }
}

/// Serde helpers for the two cybersecurity single-ref fields that historically
/// took one string only: accept a string or a list, and on a type mismatch name
/// the offending field instead of the generic serde message (which surfaced as
/// a misleading `E002` "not valid YAML").
mod named_string_or_vec {
    use serde::{Deserialize, Deserializer};
    fn de<'de, D: Deserializer<'de>>(d: D, field: &str) -> Result<Option<Vec<String>>, D::Error> {
        let v: Option<serde_yaml::Value> = Option::deserialize(d)?;
        let bad = |what: &str| serde::de::Error::custom(format!(
            "`{field}` must be a string or a list of strings, got {what}"));
        match v {
            None | Some(serde_yaml::Value::Null) => Ok(None),
            Some(serde_yaml::Value::String(s)) => Ok(Some(vec![s])),
            Some(serde_yaml::Value::Sequence(seq)) => {
                let mut out = Vec::with_capacity(seq.len());
                for item in seq {
                    match item {
                        serde_yaml::Value::String(s) => out.push(s),
                        _ => return Err(bad("a non-string list entry")),
                    }
                }
                Ok(Some(out))
            }
            Some(serde_yaml::Value::Mapping(_)) => Err(bad("a mapping")),
            Some(_) => Err(bad("a non-string scalar")),
        }
    }
    pub fn security_property<'de, D: Deserializer<'de>>(d: D) -> Result<Option<Vec<String>>, D::Error> {
        de(d, "securityProperty")
    }
    pub fn derived_from_cybersecurity_goal<'de, D: Deserializer<'de>>(d: D) -> Result<Option<Vec<String>>, D::Error> {
        de(d, "derivedFromCybersecurityGoal")
    }
}

/// Serde helper: accept either a single YAML value or a sequence of values,
/// normalizing to `Vec<serde_yaml::Value>`. Used by fields whose entries may be
/// heterogeneous by design — e.g. `evidence:`, shared between GSN `Argument`
/// (a flat list of scalar element refs, §8.18) and `PlanningItem` (a list of
/// duck-typed `ref:`/`path:`/`rationale:` mappings, REQ-TRS-PLANITEM-005).
/// A bare scalar (`evidence: TC-001`) still normalizes to a one-element list,
/// preserving `Argument.evidence`'s existing scalar-or-list acceptance.
mod value_or_vec {
    use serde::{Deserialize, Deserializer};
    pub fn deserialize<'de, D>(d: D) -> Result<Option<Vec<serde_yaml::Value>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v: Option<serde_yaml::Value> = Option::deserialize(d)?;
        match v {
            None | Some(serde_yaml::Value::Null) => Ok(None),
            Some(serde_yaml::Value::Sequence(seq)) => Ok(Some(seq)),
            Some(other) => Ok(Some(vec![other])),
        }
    }
}

/// Serde helper for the `features:` key, which is overloaded:
///   * a **sequence** of inline feature declarations (§3.6), or
///   * a **map** of `FeatureDef qname: bool` selections on a `Configuration` (§9.8).
///
/// Both shapes are stored as `Option<Vec<serde_yaml::Value>>`; a map is wrapped
/// as a single-element vector holding the mapping, so existing call sites that
/// iterate inline declarations are unaffected. Read selections back via
/// [`RawFrontmatter::feature_selections`].
mod features_de {
    use serde::{Deserialize, Deserializer};
    pub fn deserialize<'de, D>(d: D) -> Result<Option<Vec<serde_yaml::Value>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v: Option<serde_yaml::Value> = Option::deserialize(d)?;
        match v {
            None | Some(serde_yaml::Value::Null) => Ok(None),
            Some(serde_yaml::Value::Sequence(seq)) => Ok(Some(seq)),
            Some(m @ serde_yaml::Value::Mapping(_)) => Ok(Some(vec![m])),
            other => Err(serde::de::Error::custom(format!(
                "expected a sequence or mapping for `features`, got {:?}",
                other
            ))),
        }
    }
}

/// All recognized SysML element types.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ElementType {
    // Definitions (§2.1)
    PartDef,
    ItemDef,
    AttributeDef,
    PortDef,
    ConnectionDef,
    InterfaceDef,
    ActionDef,
    ConstraintDef,
    RequirementDef,
    CalculationDef,
    StateDef,
    FlowDef,
    UseCaseDef,
    ViewpointDef,
    ViewDef,
    MetadataDef,
    EnumerationDef,
    OccurrenceDef,
    EventOccurrenceDef,
    VerificationCaseDef,
    AnalysisCaseDef,
    AllocationDef,
    ConcernDef,
    CaseDef,
    IndividualDef,
    SuccessionDef,
    RenderingDef,
    // Usages (§2.2)
    Part,
    Item,
    Attribute,
    Port,
    Connection,
    Interface,
    Action,
    Constraint,
    Requirement,
    Calculation,
    State,
    Flow,
    UseCase,
    View,
    Metadata,
    Allocation,
    ExhibitState,
    Concern,
    Case,
    AnalysisCase,
    VerificationCase,
    Occurrence,
    EventOccurrence,
    Individual,
    Succession,
    BindingConnector,
    Enumeration,
    Rendering,
    FeatureDef,    // PLE type (§9.6)
    Configuration, // PLE type (§9.8)
    // Native elements (not SysML usages — own schema)
    TestCase,
    TestPlan,         // TP-* — configuration-bound test campaign (GH #38)
    ADR,              // Architecture Decision Record (§8.17)
    Baseline,         // BL-* — frozen release snapshot (ADR-SYS-BASELINE-001)
    PlanningItem,     // PI-* — native planning/tracking work item (ADR-SYS-PLANITEM-001)
    ReviewRecord,     // RR-* — formal review event + traceability (§19, GH #71)
    TradeStudy,       // TRD-* — weighted-criteria evaluation of alternatives (§15, GH #63)
    Zone,             // ZN-* — IEC 62443 security zone (§13, GH #61)
    Conduit,          // CD-* — IEC 62443 conduit between zones (§13, GH #61)
    // Confirmation measure (ISO 26262-2 §6 / ISO/SAE 21434 §7) — CM-*
    ConfirmationMeasure,
    // Safety analysis (ISO 26262 HARA)
    HazardousEvent,
    SafetyGoal,
    // Security analysis (ISO/SAE 21434 TARA)
    DamageScenario,
    ThreatScenario,
    CybersecurityGoal,
    SecurityControl,
    VulnerabilityReport,
    // Asset identification (ISO/SAE 21434 §15.3) — ASSET-* id-identified
    Asset,
    // Fault Tree Analysis (IEC 61025 / ISO 26262-9)
    FaultTree,
    FaultTreeGate,
    FaultTreeEvent,
    // Attack path analysis (ISO/SAE 21434 §15.7)
    AttackTree,
    AttackTreeGate,
    AttackStep,
    // FMEA (IEC 60812 / SAE J1739)
    FMEASheet,
    FMEAEntry,
    // GSN safety-argument layer (issue #20)
    Argument,         // ARG-* — a GSN node (claim/strategy/solution)
    AssumptionOfUse,  // AOU-* — safety-related application condition (SRAC)
    // TARA container (ISO/SAE 21434) — exploded by walker into Tier-2 types
    TARASheet,
    // Single-file feature model (REQ-TRS-FM-005) — a `featureTree:` sheet
    // exploded by the walker into ordinary `FeatureDef` elements, one per
    // tree node, mirroring the multi-file (directory-per-feature) form.
    FeatureModel,
    // Namespace
    Package,
    LibraryPackage,
    Namespace,
    // Relationship
    Dependency,
    Diagram,
    // Fallback
    #[serde(other)]
    Unknown,
}

impl ElementType {
    /// Every concrete element type, in declaration order (the `Unknown`
    /// fallback excluded). The single enumeration consumers iterate — e.g. the
    /// `template` command's known-types list and its every-type test (GH #135).
    /// [`ElementType::name`]'s exhaustive match forces a new variant to be
    /// named there; add it here too (the `all_*` tests pin the two together).
    pub const ALL: &'static [ElementType] = &[
        ElementType::PartDef,
        ElementType::ItemDef,
        ElementType::AttributeDef,
        ElementType::PortDef,
        ElementType::ConnectionDef,
        ElementType::InterfaceDef,
        ElementType::ActionDef,
        ElementType::ConstraintDef,
        ElementType::RequirementDef,
        ElementType::CalculationDef,
        ElementType::StateDef,
        ElementType::FlowDef,
        ElementType::UseCaseDef,
        ElementType::ViewpointDef,
        ElementType::ViewDef,
        ElementType::MetadataDef,
        ElementType::EnumerationDef,
        ElementType::OccurrenceDef,
        ElementType::EventOccurrenceDef,
        ElementType::VerificationCaseDef,
        ElementType::AnalysisCaseDef,
        ElementType::AllocationDef,
        ElementType::ConcernDef,
        ElementType::CaseDef,
        ElementType::IndividualDef,
        ElementType::SuccessionDef,
        ElementType::RenderingDef,
        ElementType::Part,
        ElementType::Item,
        ElementType::Attribute,
        ElementType::Port,
        ElementType::Connection,
        ElementType::Interface,
        ElementType::Action,
        ElementType::Constraint,
        ElementType::Requirement,
        ElementType::Calculation,
        ElementType::State,
        ElementType::Flow,
        ElementType::UseCase,
        ElementType::View,
        ElementType::Metadata,
        ElementType::Allocation,
        ElementType::ExhibitState,
        ElementType::Concern,
        ElementType::Case,
        ElementType::AnalysisCase,
        ElementType::VerificationCase,
        ElementType::Occurrence,
        ElementType::EventOccurrence,
        ElementType::Individual,
        ElementType::Succession,
        ElementType::BindingConnector,
        ElementType::Enumeration,
        ElementType::Rendering,
        ElementType::FeatureDef,
        ElementType::Configuration,
        ElementType::TestCase,
        ElementType::TestPlan,
        ElementType::ADR,
        ElementType::Baseline,
        ElementType::PlanningItem,
        ElementType::ReviewRecord,
        ElementType::TradeStudy,
        ElementType::Zone,
        ElementType::Conduit,
        ElementType::ConfirmationMeasure,
        ElementType::HazardousEvent,
        ElementType::SafetyGoal,
        ElementType::DamageScenario,
        ElementType::ThreatScenario,
        ElementType::CybersecurityGoal,
        ElementType::SecurityControl,
        ElementType::VulnerabilityReport,
        ElementType::Asset,
        ElementType::FaultTree,
        ElementType::FaultTreeGate,
        ElementType::FaultTreeEvent,
        ElementType::AttackTree,
        ElementType::AttackTreeGate,
        ElementType::AttackStep,
        ElementType::FMEASheet,
        ElementType::FMEAEntry,
        ElementType::Argument,
        ElementType::AssumptionOfUse,
        ElementType::TARASheet,
        ElementType::FeatureModel,
        ElementType::Package,
        ElementType::LibraryPackage,
        ElementType::Namespace,
        ElementType::Dependency,
        ElementType::Diagram,
    ];

    /// The type's canonical name — exactly the `type:` value an author writes.
    /// Exhaustive on purpose: adding a variant fails to compile until it is
    /// named here (and, by convention, listed in [`ElementType::ALL`]).
    pub fn name(&self) -> &'static str {
        match self {
            ElementType::PartDef => "PartDef",
            ElementType::ItemDef => "ItemDef",
            ElementType::AttributeDef => "AttributeDef",
            ElementType::PortDef => "PortDef",
            ElementType::ConnectionDef => "ConnectionDef",
            ElementType::InterfaceDef => "InterfaceDef",
            ElementType::ActionDef => "ActionDef",
            ElementType::ConstraintDef => "ConstraintDef",
            ElementType::RequirementDef => "RequirementDef",
            ElementType::CalculationDef => "CalculationDef",
            ElementType::StateDef => "StateDef",
            ElementType::FlowDef => "FlowDef",
            ElementType::UseCaseDef => "UseCaseDef",
            ElementType::ViewpointDef => "ViewpointDef",
            ElementType::ViewDef => "ViewDef",
            ElementType::MetadataDef => "MetadataDef",
            ElementType::EnumerationDef => "EnumerationDef",
            ElementType::OccurrenceDef => "OccurrenceDef",
            ElementType::EventOccurrenceDef => "EventOccurrenceDef",
            ElementType::VerificationCaseDef => "VerificationCaseDef",
            ElementType::AnalysisCaseDef => "AnalysisCaseDef",
            ElementType::AllocationDef => "AllocationDef",
            ElementType::ConcernDef => "ConcernDef",
            ElementType::CaseDef => "CaseDef",
            ElementType::IndividualDef => "IndividualDef",
            ElementType::SuccessionDef => "SuccessionDef",
            ElementType::RenderingDef => "RenderingDef",
            ElementType::Part => "Part",
            ElementType::Item => "Item",
            ElementType::Attribute => "Attribute",
            ElementType::Port => "Port",
            ElementType::Connection => "Connection",
            ElementType::Interface => "Interface",
            ElementType::Action => "Action",
            ElementType::Constraint => "Constraint",
            ElementType::Requirement => "Requirement",
            ElementType::Calculation => "Calculation",
            ElementType::State => "State",
            ElementType::Flow => "Flow",
            ElementType::UseCase => "UseCase",
            ElementType::View => "View",
            ElementType::Metadata => "Metadata",
            ElementType::Allocation => "Allocation",
            ElementType::ExhibitState => "ExhibitState",
            ElementType::Concern => "Concern",
            ElementType::Case => "Case",
            ElementType::AnalysisCase => "AnalysisCase",
            ElementType::VerificationCase => "VerificationCase",
            ElementType::Occurrence => "Occurrence",
            ElementType::EventOccurrence => "EventOccurrence",
            ElementType::Individual => "Individual",
            ElementType::Succession => "Succession",
            ElementType::BindingConnector => "BindingConnector",
            ElementType::Enumeration => "Enumeration",
            ElementType::Rendering => "Rendering",
            ElementType::FeatureDef => "FeatureDef",
            ElementType::Configuration => "Configuration",
            ElementType::TestCase => "TestCase",
            ElementType::TestPlan => "TestPlan",
            ElementType::ADR => "ADR",
            ElementType::Baseline => "Baseline",
            ElementType::PlanningItem => "PlanningItem",
            ElementType::ReviewRecord => "ReviewRecord",
            ElementType::TradeStudy => "TradeStudy",
            ElementType::Zone => "Zone",
            ElementType::Conduit => "Conduit",
            ElementType::ConfirmationMeasure => "ConfirmationMeasure",
            ElementType::HazardousEvent => "HazardousEvent",
            ElementType::SafetyGoal => "SafetyGoal",
            ElementType::DamageScenario => "DamageScenario",
            ElementType::ThreatScenario => "ThreatScenario",
            ElementType::CybersecurityGoal => "CybersecurityGoal",
            ElementType::SecurityControl => "SecurityControl",
            ElementType::VulnerabilityReport => "VulnerabilityReport",
            ElementType::Asset => "Asset",
            ElementType::FaultTree => "FaultTree",
            ElementType::FaultTreeGate => "FaultTreeGate",
            ElementType::FaultTreeEvent => "FaultTreeEvent",
            ElementType::AttackTree => "AttackTree",
            ElementType::AttackTreeGate => "AttackTreeGate",
            ElementType::AttackStep => "AttackStep",
            ElementType::FMEASheet => "FMEASheet",
            ElementType::FMEAEntry => "FMEAEntry",
            ElementType::Argument => "Argument",
            ElementType::AssumptionOfUse => "AssumptionOfUse",
            ElementType::TARASheet => "TARASheet",
            ElementType::FeatureModel => "FeatureModel",
            ElementType::Package => "Package",
            ElementType::LibraryPackage => "LibraryPackage",
            ElementType::Namespace => "Namespace",
            ElementType::Dependency => "Dependency",
            ElementType::Diagram => "Diagram",
            ElementType::Unknown => "Unknown",
        }
    }
}

impl ElementType {
    /// Whether this type is **id-identified** — its identity is a stable `id`
    /// (shortName such as `REQ-*`, `HE-*`). Every other type is **name-identified**:
    /// its identity is its `name`/path.
    ///
    /// Under the unified identity model (REQ-TRS-NAME-002) the human-readable label is
    /// **`name`** on every type regardless of identity class; the `title` field is
    /// removed as a label (declaring it raises `E025`). This predicate therefore only
    /// distinguishes *identity* (id vs name), not the label field.
    ///
    /// `FeatureDef` is deliberately **not** here: it is name-identified yet may also
    /// carry an optional `FEAT-*` `id` (REQ-TRS-ID-006). The `id` axis (shortName) and
    /// the label axis (always `name`) are independent.
    pub fn is_id_identified(&self) -> bool {
        matches!(
            self,
            ElementType::Requirement
                | ElementType::TestCase
                | ElementType::TestPlan
                | ElementType::Configuration
                | ElementType::ADR
                | ElementType::Baseline
                | ElementType::PlanningItem
                | ElementType::ReviewRecord
                | ElementType::TradeStudy
                | ElementType::Zone
                | ElementType::Conduit
                | ElementType::ConfirmationMeasure
                | ElementType::HazardousEvent
                | ElementType::SafetyGoal
                | ElementType::DamageScenario
                | ElementType::ThreatScenario
                | ElementType::CybersecurityGoal
                | ElementType::SecurityControl
                | ElementType::VulnerabilityReport
                | ElementType::TARASheet
                | ElementType::FaultTree
                | ElementType::FaultTreeGate
                | ElementType::FaultTreeEvent
                | ElementType::FMEASheet
                | ElementType::FMEAEntry
                | ElementType::AttackTree
                | ElementType::AttackTreeGate
                | ElementType::AttackStep
                | ElementType::Argument
                | ElementType::AssumptionOfUse
                | ElementType::Asset
        )
    }
}

/// One parsed `metadata:` application — a stereotype (MetadataDef) applied to an element,
/// plus its tagged-value keys (REQ-TRS-META-001).
#[derive(Debug, Clone)]
pub struct MetaApply {
    /// The referenced `MetadataDef` (qualified name or id).
    pub def: String,
    /// Tagged values supplied by the application (key, value).
    pub values: Vec<(String, serde_yaml::Value)>,
}

/// Parse an element's `metadata:` list into stereotype applications. Each entry is either a
/// bare string reference, or a map carrying tagged values whose `apply` (alias `def`) key
/// names the MetadataDef and whose other keys are the tagged values. Entries without a
/// resolvable `def` reference are skipped. The reserved keys `name` (the application's own
/// declared name, `@n : T` in SysML v2) and `about` (the written target of an application a
/// SysML v2 submodel could not attach, REQ-TRS-SYSMLV2-087) are never tagged values.
pub fn metadata_applications(metadata: &Option<Vec<serde_yaml::Value>>) -> Vec<MetaApply> {
    let mut out = Vec::new();
    let Some(list) = metadata else { return out };
    for entry in list {
        match entry {
            serde_yaml::Value::String(s) => {
                out.push(MetaApply { def: s.clone(), values: Vec::new() });
            }
            serde_yaml::Value::Mapping(m) => {
                // The def reference key is `type` (SysMLv2 metadata application, spec §8.15.2);
                // `apply`/`def` are accepted aliases.
                let def = ["type", "apply", "def"]
                    .iter()
                    .find_map(|k| m.get(serde_yaml::Value::from(*k)).and_then(|v| v.as_str()));
                if let Some(def) = def {
                    let values = m
                        .iter()
                        .filter_map(|(k, v)| k.as_str().map(|k| (k.to_string(), v.clone())))
                        .filter(|(k, _)| !matches!(k.as_str(), "type" | "apply" | "def" | "name" | "about"))
                        .collect();
                    out.push(MetaApply { def: def.to_string(), values });
                }
            }
            _ => {}
        }
    }
    out
}

/// A TestPlan's additive `selection:` membership query (REQ-TRS-PLAN-003).
/// An absent sub-field is *no constraint*; a block with no sub-fields at all
/// matches *nothing* (not everything). Draft TestCases are never swept here.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TestPlanSelection {
    /// Subset of L1–L5 (else E602).
    pub test_levels: Option<Vec<String>>,
    /// Subset of system|hardware|software, derived transitively from a candidate
    /// TestCase's `verifies:` targets' `reqDomain:` (else E605).
    pub domains: Option<Vec<String>>,
    /// Matched against TestCase `tags`.
    pub tags: Option<Vec<String>>,
}

impl TestPlanSelection {
    /// True when the block carries no sub-field constraints at all (matches nothing).
    pub fn is_empty(&self) -> bool {
        self.test_levels.is_none() && self.domains.is_none() && self.tags.is_none()
    }
}

/// `frozenScope` on a `Baseline` (REQ-TRS-BL-003): the selector defining which
/// elements a baseline freezes. All fields optional; absent `package` ⇒ whole model,
/// and the filters compose as a logical AND. `Baseline` elements are always excluded
/// from the resolved set.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FrozenScope {
    /// A `Configuration` (id/qname) or ad-hoc feature set; when set, scope resolution
    /// first projects the model to that variant (REQ-TRS-BL-011).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub config: Option<String>,
    /// Seed references whose transitive trace closure is the scope (REQ-TRS-BL-012).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub closure_from: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub package: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub types: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

/// The generated `seal` block on a `Baseline` (REQ-TRS-BL-002/004): the frozen
/// aggregate hash, the in-scope element count, and the manifest path.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BaselineSeal {
    pub aggregate_hash: String,
    pub element_count: usize,
    pub manifest: String,
}

/// Parsed frontmatter from a `.md` model file.
/// All fields except `element_type` are optional — absent means "use default".
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RawFrontmatter {
    #[serde(rename = "type")]
    pub element_type: Option<ElementType>,
    pub name: Option<String>,
    pub short_name: Option<String>,
    pub visibility: Option<String>,
    /// §3.16 — display / reading order. A generic, optional ordinal used as the
    /// primary sort key when this element is presented alongside its peers in an
    /// ordered output (requirements report, coverage matrix, web-UI tree). Elements
    /// sort by ascending `displayOrder`; an element with no `displayOrder` sorts
    /// **after** every element that declares one, ties then breaking on the stable
    /// identifier. A decimal is accepted so a new element can be inserted between two
    /// neighbours (e.g. `15` between `10` and `20`) without renumbering. (REQ-TRS-ORDER-001)
    pub display_order: Option<f64>,
    pub supertype: Option<serde_yaml::Value>,
    pub typed_by: Option<serde_yaml::Value>,
    pub subsets: Option<Vec<String>>,
    pub redefines: Option<serde_yaml::Value>,
    pub multiplicity: Option<String>,
    pub is_abstract: Option<bool>,
    pub direction: Option<String>,
    /// `unit:` on an `Attribute` — qualified/simple name of the unit of a quantity-valued `value:`
    /// (the spec's inline-feature `unit` shorthand, §3.6.1, on a standalone element;
    /// `REQ-TRS-SYSMLV2-055`).
    pub unit: Option<String>,
    #[serde(default, deserialize_with = "features_de::deserialize")]
    pub features: Option<Vec<serde_yaml::Value>>,
    pub connections: Option<Vec<serde_yaml::Value>>,
    pub verifies: Option<Vec<String>>,
    #[serde(default, deserialize_with = "string_or_vec::deserialize")]
    pub allocated_to: Option<Vec<String>>,
    pub diagram_kind: Option<String>,
    pub subject: Option<String>,
    // derivedFrom is a list for both native Requirements and SysML RequirementDef
    #[serde(default, deserialize_with = "string_or_vec::deserialize")]
    pub derived_from: Option<Vec<String>>,
    pub requires: Option<Vec<serde_yaml::Value>>,
    // Native Requirement fields (§8.11.6)
    pub id: Option<String>,
    pub status: Option<String>,
    /// §8.11.6 — Requirement classification in the stakeholder/system decomposition:
    /// `stakeholder` | `system` | `derived`. Recognised, first-class field (a plain
    /// unrecognised `reqClass:` would otherwise be silently dropped and warned via
    /// W047). (REQ-TRS-SCHEMA-002)
    pub req_class: Option<String>,
    pub tags: Option<Vec<String>>,
    pub verification_method: Option<String>,
    // Native TestCase fields (§8.12.5)
    pub test_level: Option<String>,
    pub source_file: Option<String>,
    pub test_functions: Option<Vec<serde_yaml::Value>>,

    // Native TestPlan fields (GH #38; REQ-TRS-PLAN-001..004)
    /// `scope:` — free-form, recommended vocab unit|smoke|integration|hil|
    /// certification|security|regression (else W610).
    pub scope: Option<String>,
    /// `testCases:` — scalar or list of explicit `TestCase` members (else E601).
    #[serde(rename = "testCases", default, deserialize_with = "string_or_vec::deserialize")]
    pub test_cases: Option<Vec<String>>,
    /// §12.8 — implementation trace: architecture element → source artifact(s).
    #[serde(default, deserialize_with = "string_or_vec::deserialize")]
    pub implemented_by: Option<Vec<String>>,

    /// REQ-TRS-LINKTYPE-002 — user-defined links (ADR-SYS-LINKTYPE-001): a map from
    /// a link-type name declared in `[linkTypes.<name>]` of `.syscribe.toml` to a
    /// reference or list of references (id or qname, resolved like `satisfies:`).
    /// Kept as the raw YAML value — never coerced — so a malformed shape is
    /// reported by the validator (`E631`) instead of failing the whole parse, and
    /// the field round-trips byte-for-byte through every write path. Being a
    /// recognised field it never lands in `extra` (no `W047`). Read it through
    /// `crate::link_types::parse_links`/`declared_links`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub links: Option<serde_yaml::Value>,

    // §PlanningItem (ADR-SYS-PLANITEM-001) — native planning/tracking hierarchy.
    /// `parent:` — at most one other `PlanningItem` (REQ-TRS-PLANITEM-002). A
    /// single scalar reference, deliberately unlike `Requirement.derivedFrom`'s
    /// list: `PlanningItem` is a strict single-parent tree, not a DAG. A
    /// `PlanningItem` with no `parent:` is top-level. Resolved like `derivedFrom`
    /// (id-or-qname); the reverse `children` index and cycle detection are
    /// computed the same way `derivedChildren`/`E017` are for `Requirement`.
    pub parent: Option<String>,
    /// `achieves:` — one or more native `Requirement`s (or a `SafetyGoal`, `CybersecurityGoal`,
    /// `ADR`, `Argument`, `TestPlan` or `Baseline`, GH #240) this `PlanningItem` exists
    /// to achieve (REQ-TRS-PLANITEM-003). Unlike `parent:`, legitimately a list
    /// (scalar or list accepted, like `derivedFrom`). Required (non-empty) on a
    /// top-level item (no `parent:`); optional otherwise. A distinct field from
    /// `satisfies:` — deliberately does not participate in that field's
    /// `W300`/`E312` architecture-satisfaction machinery.
    #[serde(default, deserialize_with = "string_or_vec::deserialize")]
    pub achieves: Option<Vec<String>>,
    /// `blockedBy:` — one or more elements this `PlanningItem` is waiting on before
    /// it can proceed (REQ-TRS-PLANITEM-007). Most commonly another `PlanningItem`,
    /// but resolved permissively, unrestricted by kind — exactly like
    /// `evidence.ref:` (REQ-TRS-PLANITEM-005) — since an undecided `ADR` or any
    /// other unmet model dependency is an equally legitimate blocker. A dangling
    /// entry is an error; a `blockedBy:` chain (through other `PlanningItem`s) that
    /// cycles back to itself is an error, same posture as `parent:`
    /// (REQ-TRS-PLANITEM-002). Deliberately not required to be non-empty when
    /// `status: blocked` (unlike `evidence:` on a `done` leaf) — see the ADR
    /// addendum.
    #[serde(default, deserialize_with = "string_or_vec::deserialize")]
    pub blocked_by: Option<Vec<String>>,
    /// `assignedTo:` — the single user this `PlanningItem` is assigned to
    /// (REQ-TRS-PLANITEM-008), a plain id string, not a cross-reference (users
    /// are not model elements). A single scalar, deliberately unlike
    /// `achieves:`/`blockedBy:` — mirrors `parent:`'s "one at a time" shape
    /// rather than GitHub's own multi-assignee model; see the ADR addendum for
    /// the rejected multi-assignee alternative. Checked against the declared
    /// roster (`[users] ids` in `.syscribe.toml`, `ValidateConfig::users`) only
    /// when that roster is non-empty — dormant, like every other config-gated
    /// check, when `[users]` is not configured at all.
    pub assigned_to: Option<String>,
    /// `claimedBy:` — an opaque agent/session id that has claimed this
    /// `PlanningItem` for active work (issue #115), paired with `claimedAt:`
    /// (an opaque ISO-8601-ish timestamp string, never itself format-validated
    /// — same posture as `wcet:`/`extRef:`, free-text metadata rather than a
    /// cross-reference or a parsed datetime). Advisory, not a filesystem lock:
    /// the value is a coordination signal ("is anyone already on this?") for
    /// an orchestrating process running multiple agents against one model,
    /// written/cleared only by `syscribe claim`/`syscribe release` — never
    /// hand-authored, though nothing stops it structurally.
    pub claimed_by: Option<String>,
    pub claimed_at: Option<String>,

    /// §3 — external reference(s): this element represents an artifact managed in
    /// another tool (a DNG requirement, a SysML-tool element, …). Opaque strings
    /// (URI or tool-qualified token); string or list. Never a model cross-ref target.
    #[serde(default, deserialize_with = "string_or_vec::deserialize")]
    pub ext_ref: Option<Vec<String>>,

    // §8.6.1 — FlowDef
    pub item_type: Option<String>,

    // §8.11.4 — satisfaction/verification
    pub satisfies: Option<Vec<String>>,

    // §9.6 — FeatureDef
    pub group_kind: Option<String>,
    pub excludes: Option<Vec<String>>,

    // §9.7 — FeatureDef parameters (also used by ActionDef/CalculationDef as a
    // generic typed-parameter list; only FeatureDef parameters are validated).
    pub parameters: Option<Vec<serde_yaml::Value>>,

    /// `derive:` — a mapping of fieldName → formula evaluated by
    /// `derive::derive_pass` (REQ-TRS-DERIVE-001). A typed, recognised field on
    /// every element type (GH #141: it used to be read out of the `extra`
    /// catch-all, so declaring it falsely raised `W047`). Kept as a raw YAML
    /// value so a malformed block (not a mapping, non-string formula) is
    /// reported by the derive pass as `E505` instead of failing the whole
    /// file's deserialization.
    pub derive: Option<serde_yaml::Value>,

    // §9.8 — Configuration
    pub feature_model: Option<String>,

    // §9.10 — PLE conditioning (any element)
    pub applies_when: Option<serde_yaml::Value>,

    // §8.11.6 — native Requirement traceability (§12)
    pub req_domain: Option<String>,
    pub breakdown_adr: Option<String>,

    // §3.14 — domain classification
    pub domain: Option<String>,
    /// `evidence` — a shared YAML key with two independent shapes, kept as one
    /// `Vec<serde_yaml::Value>` field (via [`value_or_vec`]) since a flat struct
    /// can only bind one Rust field per YAML key:
    ///   - `Argument.evidence` (§8.18) — refs to supporting Requirement /
    ///     TestCase / sub-Argument / AssumptionOfUse (the GSN children); each
    ///     entry a scalar string, resolved via the Resolver (else E855).
    ///   - `PlanningItem.evidence` (REQ-TRS-PLANITEM-005) — a list of duck-typed
    ///     `ref:`/`path:`/`rationale:` mappings (see the `PlanningItem` section
    ///     below for the full shape). Recognised by which key an entry carries,
    ///     not a `type:` tag — the same idiom the `Allocation` `features:`-list
    ///     convention already establishes (an entry with both
    ///     `allocatedFrom`+`allocatedTo` is an edge regardless of any per-entry
    ///     `type:`).
    /// Scalar or list accepted for either shape (`value_or_vec`), matching
    /// `Argument.evidence`'s pre-existing acceptance.
    #[serde(default, deserialize_with = "value_or_vec::deserialize")]
    pub evidence: Option<Vec<serde_yaml::Value>>,

    /// §custom-fields (GH #39) — user-defined, freeform metadata attachable to any
    /// element. A flat map of `string -> scalar | list-of-scalars`. Distinct from the
    /// `extra` catch-all below: `custom_fields` is the *intentional, addressable* home
    /// for custom data, whereas `extra` swallows genuinely unknown top-level keys.
    /// `BTreeMap` gives a stable (sorted) serialization order so writes do not produce
    /// noisy round-trip diffs. Shape-checked by the validator (`W041`): each value must
    /// be a scalar or a list of scalars. (YAML: custom_fields — explicit snake_case,
    /// overriding the struct-level camelCase rename.)
    #[serde(
        rename = "custom_fields",
        default,
        skip_serializing_if = "std::collections::BTreeMap::is_empty"
    )]
    pub custom_fields: std::collections::BTreeMap<String, serde_yaml::Value>,

    /// Effective selection/bindings of a `Configuration` that inherits from a
    /// base through `derivedFrom:` (§9.8, GH #137) — materialized by the walker
    /// (`crate::config_inherit`). Never (de)serialized: the authored file is the
    /// source of truth; this is a computed view read through
    /// [`RawFrontmatter::feature_selections`] and
    /// [`RawFrontmatter::effective_parameter_bindings`].
    #[serde(skip)]
    pub inherited: Option<Box<InheritedConfiguration>>,
    /// `applies_when` was copied from the owning TARA/FMEA sheet, not authored on this
    /// row (GH #229). The sheet already carries the authored declaration, so per-element
    /// validity and nesting checks skip the copy rather than repeat the finding.
    #[serde(skip)]
    pub applies_when_inherited: bool,
    /// Rarely-set fields, boxed so an element that uses none of them pays one
    /// pointer instead of ~6 KB of empty `Option`s (`REQ-TRS-MCP-MEM-000`). Read and
    /// written as if they were fields of this struct, through `Deref`/`DerefMut`.
    #[serde(flatten)]
    pub cold: Boxed<ColdFrontmatter>,


    // Catch-all for unknown fields
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_yaml::Value>,
}

/// The rarely-set frontmatter fields of [`RawFrontmatter`], in three tiers so an
/// element pays only for the tier it uses: tier 1 (`ColdFrontmatter`) holds the
/// fields some models use, and reaches tier 2 (`ColdFrontmatter2`) and tier 3
/// (`ColdFrontmatter3`, the rarest: safety, security, and the like) through
/// `Deref` chains, so every field stays readable and writable as `fm.field`.
/// Each tier is allocated only when one of its fields is set (`REQ-TRS-MCP-MEM-000`).
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ColdFrontmatter {
    pub is_variation: Option<bool>,
    pub expression: Option<String>,
    pub metadata: Option<Vec<serde_yaml::Value>>,
    pub binding_connections: Option<Vec<serde_yaml::Value>>,
    pub succession_connections: Option<Vec<serde_yaml::Value>>,
    pub sub_states: Option<Vec<serde_yaml::Value>>,
    pub transitions: Option<Vec<serde_yaml::Value>>,
    pub exhibits_states: Option<Vec<String>>,
    pub operations: Option<Vec<serde_yaml::Value>>,
    pub actors: Option<Vec<String>>,
    pub steps: Option<Vec<String>>,
    #[serde(default, deserialize_with = "string_or_vec::deserialize")]
    pub allocated_from: Option<Vec<String>>,
    pub expose: Option<Vec<serde_yaml::Value>>,
    pub viewpoint: Option<String>,
    pub svg_mode: Option<String>,
    pub svg_file: Option<String>,
    pub puml_mode: Option<String>,
    pub puml_file: Option<String>,
    pub shapes: Option<serde_yaml::Value>,
    pub edges: Option<serde_yaml::Value>,
    pub layout: Option<serde_yaml::Value>,
    pub imports: Option<Vec<serde_yaml::Value>>,
    /// REQ-TRS-MG-001 — MagicGrid `«refine»`: a `UseCaseDef`/`UseCase` gives concrete
    /// behavioural meaning to a requirement. Optional list of cross-references
    /// (qname or stable `REQ-*` id), resolved like `verifies:`/`derivedFrom:`.
    pub refines: Option<Vec<String>>,
    pub about: Option<serde_yaml::Value>,
    pub locale: Option<String>,
    pub sil_level: Option<u8>,
    pub asil_level: Option<String>,
    /// ASIL/SIL decomposition argument type (§22.3): `independent` | `redundant` | `diverse`.
    /// Informational; surfaced in the safety-case report.
    pub decomposition_kind: Option<String>,
    /// Original (pre-decomposition) ASIL of a decomposed requirement (ISO 26262-9 §5):
    /// `decomposedFrom: D`, or implied by the `B(D)` notation in `asilLevel:`. Letter A–D.
    pub decomposed_from: Option<String>,
    pub wcet: Option<String>,
    /// `configurations:` — scalar or list of `Configuration` references. Absent
    /// → config-agnostic (applies to every Configuration). Each must resolve to a
    /// `Configuration` (else E606).
    #[serde(default, deserialize_with = "string_or_vec::deserialize")]
    pub configurations: Option<Vec<String>>,
    /// `demonstrates:` — scalar or list of Requirement/SafetyGoal/
    /// CybersecurityGoal/Argument the plan is offered as evidence for (else E603).
    #[serde(default, deserialize_with = "string_or_vec::deserialize")]
    pub demonstrates: Option<Vec<String>>,

    /// REQ-TRS-SUS-LINKS-001 — suspect-link baselines. A map from a trace-link
    /// **target identifier** (exactly as authored on the link — a stable id or a
    /// qualified name) to the algorithm-prefixed content hash (`blake3:<hex>`) of
    /// that target's normative projection (REQ-TRS-SUS-LINKS-002), captured at the
    /// moment the link was last reviewed. One map on the source (which holds the
    /// link, per §12.1) covers every link kind. `BTreeMap` → deterministic, sorted
    /// serialization so re-baselining produces minimal diffs.
    #[serde(rename = "traceBaselines", default, skip_serializing_if = "Option::is_none")]
    pub trace_baselines: Option<std::collections::BTreeMap<String, String>>,

    // §3.1 — identity override
    pub qualified_name: Option<String>,

    // §3.2 — classification flags
    pub is_variant: Option<bool>,

    // §8.4.x — connection/binding elements
    pub ends: Option<Vec<serde_yaml::Value>>,

    // §8.7.1 + §8.9.1 — Action/Calculation body
    pub body: Option<String>,
    pub body_language: Option<String>,
    /// `CalculationDef` (§22.2): qualified name of a `ConstraintDef` bounding the budget result.
    pub evaluate: Option<String>,
    // Native ReviewRecord fields (§19, GH #71). `recordedAt` is the thin pointer to the
    // external review (e.g. a GitHub PR/review URL); the model keeps the baselined anchor.
    pub review_type: Option<String>,
    #[serde(default, deserialize_with = "string_or_vec::deserialize")]
    pub reviews: Option<Vec<String>>,
    pub items: Option<Vec<serde_yaml::Value>>,
    // Native TradeStudy fields (§15, GH #63). `objective` (Requirement) is shared above.
    pub criteria: Option<Vec<serde_yaml::Value>>,
    pub alternatives: Option<Vec<serde_yaml::Value>>,
    pub scores: Option<Vec<serde_yaml::Value>>,
    // Native IEC 62443 Zone/Conduit fields (§13, GH #61).
    #[serde(rename = "targetSL")]
    pub target_sl: Option<u8>,
    #[serde(rename = "achievedSL")]
    pub achieved_sl: Option<u8>,
    #[serde(default, deserialize_with = "string_or_vec::deserialize")]
    pub members: Option<Vec<String>>,
    pub from_zone: Option<String>,
    pub to_zone: Option<String>,
    /// §14.3 — `repoImports:` on a Package `_index.md`: a list of mappings
    /// `{repo, qname, as}` mounting a sub-tree from a peer repo declared in
    /// `[repos]`. Untyped here; the validator reads the `repo`/`qname`/`as` keys.
    pub repo_imports: Option<Vec<serde_yaml::Value>>,
    /// `sysmlSubmodel: true` on a Package `_index.md` (`ADR-SYS-SYSMLV2-001`,
    /// `REQ-TRS-SYSMLV2-001`): every `.sysml`/`.kerml` file anywhere in that
    /// directory's subtree is parsed as native SysML v2/KerML textual notation
    /// instead of Markdown+YAML frontmatter. Handled by `crate::sysmlv2`.
    pub sysml_submodel: Option<bool>,
    pub sub_actions: Option<Vec<serde_yaml::Value>>,
    pub is_parallel: Option<bool>,

    // §8.12.1 — Case elements
    pub objectives: Option<Vec<serde_yaml::Value>>,

    // §8.15.1 — MetadataDef
    pub annotates: Option<Vec<String>>,
    /// Membership flag (REQ-TRS-FM-004): when `true`, the feature is mandatory
    /// (forced on with its parent, or root-selected when top-level) independently
    /// of `groupKind`. Legacy `groupKind: mandatory` remains a shorthand.
    pub mandatory: Option<bool>,

    /// `featureTree:` (REQ-TRS-FM-005) — on a `type: FeatureModel` sheet: the
    /// whole feature model as one **flat** list. Each entry is shaped like a
    /// `FeatureDef`'s own frontmatter (`id`, `mandatory`, `groupKind`,
    /// `cardinality`, `requires`, `excludes`, `parameters`, `buildExports`, an
    /// optional `doc:` body), but its `name:` is a **dot-separated relative
    /// path** from the sheet — e.g. `Platform.CortexM` — rather than a single
    /// basic name. This is a mini-DSL scoped to `featureTree:` entries only; it
    /// does not change `name:`'s meaning anywhere else in the format.
    ///
    /// The walker's explode pass (`walker::explode_feature_model_trees`) turns
    /// each entry into a synthetic `FeatureDef` `RawElement`: the dotted path is
    /// split on `.`, each segment becomes one `::`-joined qname component under
    /// the sheet's own qname (so `Platform.CortexM` under sheet `Features`
    /// yields `Features::Platform::CortexM` — exactly the qname a
    /// directory-per-feature layout would produce for the same tree shape), and
    /// the synthesized element's own `name:` is rewritten to just the last path
    /// segment (`CortexM`) — the same leaf label a per-file `FeatureDef` would
    /// carry. An ancestor segment need not have its own entry (mirrors today's
    /// multi-file behavior: a qname prefix that is not itself a `FeatureDef`
    /// simply implies no parent). Every downstream consumer (validator,
    /// `feature-check`, `matrix`, the web UI) sees the same kind of `FeatureDef`
    /// element either way. Purely additive/opt-in: unrelated to the existing
    /// per-attribute `features:` field.
    #[serde(rename = "featureTree")]
    pub feature_tree: Option<Vec<serde_yaml::Value>>,

    /// `crossTreeConstraints:` (REQ-TRS-FM-005) — on a `type: FeatureModel`
    /// sheet: a flat list of `{ feature, requires, excludes }` entries, kept
    /// separate from the `featureTree:` structural list so the model's
    /// requires/excludes edges can be reviewed as one section instead of
    /// scattered across entries (inline `requires:`/`excludes:` on a
    /// `featureTree:` entry still works too — this section is additive).
    /// `feature`/`requires`/`excludes` values resolve the same way: containing
    /// `::` → already an absolute qname; starting with `FEAT` → a stable id;
    /// otherwise → a dot-separated path relative to this sheet, resolved
    /// exactly like a `featureTree:` entry's `name:`. The walker's explode pass
    /// merges each resolved `requires`/`excludes` into the matching synthesized
    /// `FeatureDef`'s own field. A `feature:` that doesn't resolve to a
    /// `FeatureDef` synthesized from this same sheet is `E233` — there is
    /// nothing local to attach the constraint to.
    #[serde(rename = "crossTreeConstraints")]
    pub cross_tree_constraints: Option<Vec<serde_yaml::Value>>,

    /// `parameterConstraints:` (§9.7) — cross-feature numeric constraints,
    /// declared on a `Package`/`LibraryPackage`/`Namespace` `_index.md` or
    /// (REQ-TRS-FM-005) directly on a `type: FeatureModel` sheet. Evaluated by
    /// `feature-check` (`E213`/`E221`/`W014`/`W025`). Promoted to a typed field
    /// (previously read out of the `extra` catch-all) so declaring it no
    /// longer falsely raises `W047` on the very element type that hosts it.
    #[serde(rename = "parameterConstraints")]
    pub parameter_constraints: Option<Vec<serde_yaml::Value>>,
    pub parameter_bindings: Option<serde_yaml::Value>,

    /// `subConfigurations:` (REQ-TRS-HPLE-001, ADR-SYS-HPLE-001) — on a
    /// `Configuration`: one or more other `Configuration` elements (qname or
    /// stable `CONF-*` id) this `Configuration` consolidates — a hierarchical
    /// product-line composition. Each entry resolves like any other
    /// cross-reference: the local model first, then each loaded peer repo in
    /// declaration order (§14.4). Scalar or list, following the
    /// `derivedFrom`/`achieves` convention. Naturally empty/absent at a leaf
    /// tier with no lower-tier product lines to consolidate. Resolution and
    /// the peer-validity gate are a validator pass, not a parse-time concern.
    #[serde(default, deserialize_with = "string_or_vec::deserialize")]
    pub sub_configurations: Option<Vec<String>>,
    pub is_deployment_package: Option<bool>,

    /// REQ-TRS-SAFE-007 (ISO 26262-8 §5 DIA / ISO/SAE 21434 §7 CIA) — the
    /// accountable party/organisation for a work product (the DIA/CIA split,
    /// e.g. "OEM" / "Supplier-X"). Drives the opt-in W038 check. (YAML: responsibility)
    pub responsibility: Option<String>,

    /// REQ-TRS-SAFE-007 (ISO 26262-2 §6) — ConfirmationMeasure kind:
    /// confirmation_review | functional_safety_audit | functional_safety_assessment |
    /// cybersecurity_assessment. Invalid → E849. (YAML: measureType)
    pub measure_type: Option<String>,
    /// REQ-TRS-SAFE-007 — ConfirmationMeasure independence level: I1 | I2 | I3.
    /// Invalid → E850. (YAML: independenceLevel)
    pub independence_level: Option<String>,
    /// REQ-TRS-SAFE-007 — the work product(s) a ConfirmationMeasure confirms.
    /// String or list; each resolves via the Resolver (else E851). (YAML: confirms)
    #[serde(default, deserialize_with = "string_or_vec::deserialize")]
    pub confirms: Option<Vec<String>>,

    // §T4 — FaultTree (IEC 61025 / ISO 26262-9)
    pub top_event: Option<String>,              // SafetyGoal ref (YAML: topEvent)
    pub gate_type: Option<String>,              // FaultTreeGate: AND|OR|XOR|NOT|inhibit (YAML: gateType)
    pub inputs: Option<Vec<String>>,            // FaultTreeGate input refs (YAML: inputs)
    pub event_kind: Option<String>,             // FaultTreeEvent: basic|undeveloped|house (YAML: eventKind)
    pub failure_rate: Option<f64>,              // FaultTreeEvent failure rate /h (YAML: failureRate)
    /// REQ-TRS-FTA-002 (issue #148) — FaultTreeEvent → the model element whose
    /// failure the event represents (qualified name or stable id; typically a
    /// `Part`/`PartDef`). Dangling → E927. Only meaningful on `FaultTreeEvent`;
    /// on any other type it is still reported as an unrecognized field (W047).
    /// (YAML: ref)
    #[serde(rename = "ref")]
    pub event_ref: Option<String>,

    // §T4 — AttackTree (ISO/SAE 21434 §15.7 attack path analysis)
    pub threat_ref: Option<String>,             // AttackTree → ThreatScenario ref (YAML: threatRef)
    // §T4 — FMEDA diagnostic coverage (ISO 26262-5 §8-9), documented for FaultTreeEvent.
    pub diagnostic_coverage: Option<f64>,         // DC, 0.0–1.0 (YAML: diagnosticCoverage)

    // §T4 — FMEASheet / FMEAEntry (IEC 60812 / SAE J1739)
    pub entries: Option<Vec<serde_yaml::Value>>, // FMEASheet sub-entries (YAML: entries)
    pub failure_mode: Option<String>,            // FMEAEntry: what fails (YAML: failureMode)
    pub effect: Option<String>,                  // FMEAEntry: consequence (YAML: effect)
    pub cause: Option<String>,                   // FMEAEntry: root cause (YAML: cause)
    pub fmea_severity: Option<u8>,               // FMEAEntry severity 1–10 (YAML: fmeaSeverity)
    pub occurrence: Option<u8>,                  // FMEAEntry occurrence 1–10 (YAML: occurrence)
    pub detection: Option<u8>,                   // FMEAEntry detection 1–10 (YAML: detection)
    pub rpn: Option<u32>,                        // FMEAEntry Risk Priority Number (YAML: rpn)

    // §T2 — HazardousEvent (ISO 26262 §7 HARA)
    pub severity: Option<String>,               // S0-S3
    pub exposure: Option<String>,               // E0-E4
    pub controllability: Option<String>,        // C0-C3
    pub operational_situation: Option<String>,  // free-text operating scenario

    // §T2 — SafetyGoal (ISO 26262 §7 / IEC 61508 / ISO 13849)
    pub safe_state: Option<String>,             // description of the safe state
    pub hazardous_events: Option<Vec<String>>,  // HazardousEvent id/qname refs

    // §T2 — DamageScenario (ISO/SAE 21434 §15)
    pub damage_severity: Option<String>,        // severe|major|moderate|negligible
    pub impact_categories: Option<Vec<String>>, // safety|financial|operational|privacy

    /// §T4 safety↔security co-engineering (ISO 26262 ⇄ ISO/SAE 21434) — cross-link
    /// from a `DamageScenario`/`ThreatScenario` to the `HazardousEvent`/`SafetyGoal`
    /// it endangers. String or list. Resolved via `Resolver::resolve_ref`; target
    /// must be a `HazardousEvent` or `SafetyGoal` (else E844). (YAML: hazardRef)
    #[serde(default, deserialize_with = "string_or_vec::deserialize")]
    pub hazard_ref: Option<Vec<String>>,

    // §T2 — ThreatScenario (ISO/SAE 21434 §15)
    pub attack_feasibility: Option<String>,     // high|medium|low|very_low
    pub attack_vector: Option<String>,          // network|adjacent|local|physical
    // GH #222 — ISO/SAE 21434 attack-potential factors (ThreatScenario / AttackStep)
    // and per-category impact ratings (DamageScenario). All optional; see `risk.rs`.
    pub elapsed_time: Option<serde_yaml::Value>,
    pub expertise: Option<serde_yaml::Value>,
    pub knowledge: Option<serde_yaml::Value>,
    pub window_of_opportunity: Option<serde_yaml::Value>,
    pub equipment: Option<serde_yaml::Value>,
    pub safety_impact: Option<String>,
    pub financial_impact: Option<String>,
    pub operational_impact: Option<String>,
    pub privacy_impact: Option<String>,
    pub damage_scenarios: Option<Vec<String>>,  // DamageScenario id/qname refs

    // §T2 — CybersecurityGoal (ISO/SAE 21434 §15)
    pub cal_level: Option<String>,              // CAL1-CAL4
    #[serde(default, deserialize_with = "named_string_or_vec::security_property")]
    pub security_property: Option<Vec<String>>, // confidentiality|integrity|availability|authenticity (string or list)
    pub derived_from_safety_goal: Option<String>,   // SG-* that generated this requirement (YAML: derivedFromSafetyGoal)

    // §8.18 — GSN safety-argument layer (issue #20)
    /// `Argument.argumentType` (YAML: argumentType) ∈ {claim, strategy, solution};
    /// absent is treated as `claim`. Invalid → E854.
    pub argument_type: Option<String>,
    /// `Argument.supports` (YAML: supports) — the SafetyGoal or parent Argument this
    /// node argues for (the GSN supported goal). String or list; each ref resolves
    /// via the Resolver (else E855).
    #[serde(default, deserialize_with = "string_or_vec::deserialize")]
    pub supports: Option<Vec<String>>,
    /// `AssumptionOfUse.appliesTo` (YAML: appliesTo) — the SafetyGoal / Argument /
    /// Requirement this SRAC constrains. String or list; each ref resolves via the
    /// Resolver (else E858).
    #[serde(default, deserialize_with = "string_or_vec::deserialize")]
    pub applies_to: Option<Vec<String>>,

    // §T2 — TestCase security test method (REQ-TRS-SEC-008; ISO/SAE 21434 §13.3)
    // Valid: fuzz|penetration_test|security_regression|vulnerability_scan|threat_modeling
    // Invalid → W809. (YAML: securityTestMethod)
    pub security_test_method: Option<String>,
    #[serde(flatten)]
    pub tier2: Boxed<ColdFrontmatter2>,
}

/// Second tier of the rarely-set fields; see [`ColdFrontmatter`].
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ColdFrontmatter2 {
    pub conjugates: Option<String>,
    pub flow_connections: Option<Vec<serde_yaml::Value>>,
    pub performs: Option<Vec<serde_yaml::Value>>,
    pub objective: Option<String>,
    pub stakeholders: Option<Vec<String>>,
    pub concerns: Option<Vec<String>>,
    pub methods: Option<Vec<String>>,
    pub depends_on: Option<Vec<String>>,
    pub extends: Option<Vec<serde_yaml::Value>>,
    pub extension_points: Option<Vec<serde_yaml::Value>>,
    pub clients: Option<Vec<String>>,
    pub suppliers: Option<Vec<String>>,
    /// **Deprecated / removed as a label** (REQ-TRS-NAME-002). Every element now labels
    /// via `name`; `title` is no longer a recognized label field. It is still parsed
    /// here only so the validator can detect a stray `title:` and reject it via `E025`.
    pub title: Option<String>,
    pub dal_level: Option<String>,
    pub requirement_kind: Option<String>,
    pub coverage_target: Option<String>,
    /// `selection:` — additive membership query (REQ-TRS-PLAN-003).
    pub selection: Option<TestPlanSelection>,

    // §Baseline (ADR-SYS-BASELINE-001) — release-baseline fields on a `type: Baseline`.
    /// The baseline date (REQ-TRS-BL-001).
    pub date: Option<String>,

    // §8.5.2 — EnumerationDef
    pub values: Option<Vec<serde_yaml::Value>>,
    pub review_date: Option<String>,
    #[serde(default, deserialize_with = "string_or_vec::deserialize")]
    pub reviewed_by: Option<Vec<String>>,
    pub recorded_at: Option<String>,
    pub decision: Option<String>,
    pub rationale: Option<String>,
    #[serde(default, deserialize_with = "string_or_vec::deserialize")]
    pub protocols: Option<Vec<String>>,
    pub in_zone: Option<String>,
    /// `foreignFormat: <alias>` on a Package `_index.md` (`ADR-SYS-PLUGIN-002`):
    /// hands the package's entire directory subtree to the stdio-subprocess
    /// plugin named by `[plugins.<alias>]` in `.syscribe.toml`. Handled by
    /// `crate::plugins`.
    pub foreign_format: Option<String>,
    /// `annotationFormat: <label>` on a Package `_index.md` (`ADR-SYS-ANNOTATE-001`):
    /// hands the package's entire directory subtree to the in-process
    /// comment-marker scanner. `label` is a human-readable tag only (no
    /// `.syscribe.toml` indirection, unlike `foreignFormat:`) — the scan
    /// parameters (`marker`/`include`/`exclude`) live inline on this same
    /// `_index.md`. Handled by `crate::annotations`.
    pub annotation_format: Option<String>,
    /// `marker:` — a regex matched against each line of every scanned file;
    /// the first match on a line starts a marker block. Required when
    /// `annotationFormat:` is set (`E560` otherwise).
    pub marker: Option<String>,
    /// `include:` — glob patterns (relative to this package's directory,
    /// `**`/`*`/`?` supported) selecting which files are scanned for markers.
    /// Required, non-empty, when `annotationFormat:` is set (`E560` otherwise).
    /// On a derived `Diagram` (`REQ-TRS-VIS-003`) the same key lists the
    /// members of the subject to show.
    pub include: Option<Vec<String>>,
    /// `exclude:` — glob patterns excluded from `include:`'s matches; on a
    /// derived `Diagram`, members of the subject to drop.
    pub exclude: Option<Vec<String>>,
    /// On a derived BDD `Diagram`: composition levels followed beyond the subject's blocks (default 1).
    pub depth: Option<usize>,
    pub control_nodes: Option<Vec<serde_yaml::Value>>,

    // §8.10.2 — Constraint usage
    pub is_asserted: Option<bool>,
    pub is_semantic: Option<bool>,
    pub aliases: Option<Vec<serde_yaml::Value>>,

    // §3.12 — representation
    pub rep: Option<String>,

    // §3.3 — InterfaceDef constraints
    pub constraints: Option<Vec<serde_yaml::Value>>,
    pub parent_feature: Option<String>,

    /// `buildOverrides:` — on a `Configuration`: a flat mapping of `varName -> scalar`
    /// that wins over any `buildExports` or parameter `buildVar` contribution.
    /// Last-writer-wins semantics; resolves E050 conflicts. Consistent pattern with
    /// `parameter_bindings` (also `Option<serde_yaml::Value>`).
    #[serde(rename = "buildOverrides", default, skip_serializing_if = "Option::is_none")]
    pub build_overrides: Option<serde_yaml::Value>,

    /// REQ-TRS-ADR-001 (GH #159) — §8.17.1 `ADR` `deciders:`: the decision-makers,
    /// each a stakeholder `PartDef` qualified name or a free-text name. Opaque
    /// display metadata, never a cross-reference (a free-text name is legitimate),
    /// so it is not resolved. A scalar is accepted as a one-entry list. Only a
    /// schema field on an `ADR`; on any other type it is still reported as an
    /// unrecognized field (W047).
    #[serde(default, deserialize_with = "string_or_vec::deserialize")]
    pub deciders: Option<Vec<String>>,

    /// REQ-TRS-SAFE-006 (ISO 26262-9 §7) — freedom-from-interference / partitioning
    /// rationale (YAML: `ffiRationale`). A non-empty string on a shared allocation
    /// target or on a source excuses a mixed-criticality sharing (clears W034).
    pub ffi_rationale: Option<String>,
    pub fmea_ref: Option<String>,               // FaultTreeEvent → reconciling FMEAEntry (YAML: fmeaRef)
    // IEC 61508 §3 risk graph parameters (alternative to ISO 26262 S/E/C)
    pub consequence: Option<String>,            // Ca | Cb | Cc | Cd
    pub freq_exposure: Option<String>,          // Fa | Fb  (YAML: freqExposure)
    pub avoidance: Option<String>,              // Pa | Pb
    pub demand_rate: Option<String>,            // W1 | W2 | W3  (YAML: demandRate)
    pub ftti: Option<String>,                   // Fault Tolerant Time Interval (e.g. "20ms")
    pub pl_level: Option<String>,               // ISO 13849-1 Performance Level: a|b|c|d|e (YAML: plLevel)
    // DamageScenario.assets: references to Asset elements (REQ-TRS-TYPE-017, YAML: assets)
    #[serde(default, deserialize_with = "string_or_vec::deserialize")]
    pub assets: Option<Vec<String>>,
    /// §T2 risk treatment decision (ISO/SAE 21434 §9 / §15.9): avoid|reduce|share|retain.
    /// Invalid value → E845. (YAML: riskTreatment)
    pub risk_treatment: Option<String>,
    /// §T2 free-text residual-risk note after treatment (no validation). (YAML: residualRisk)
    pub residual_risk: Option<String>,
    pub threat_scenarios: Option<Vec<String>>,  // ThreatScenario id/qname refs

    // §T2 — SecurityControl (ISO/SAE 21434)
    pub control_type: Option<String>,           // prevention|detection|response|recovery
    pub implements_goals: Option<Vec<String>>,  // CybersecurityGoal id/qname refs

    // §T2 — VulnerabilityReport
    pub cvss_score: Option<f64>,                // 0.0-10.0
    pub affected_elements: Option<Vec<String>>, // qualified names of affected model elements
    pub mitigated_by: Option<Vec<String>>,      // SecurityControl id/qname refs

    // §T2 — upstream goal links for native Requirement
    // YAML: derivedFromCybersecurityGoal; alias: derivedFromSecurityGoal (legacy)
    #[serde(alias = "derivedFromSecurityGoal", default, deserialize_with = "named_string_or_vec::derived_from_cybersecurity_goal")]
    pub derived_from_cybersecurity_goal: Option<Vec<String>>,

    // §T2 — Asset (REQ-TRS-TYPE-017; ISO/SAE 21434 §15.3 asset identification)
    // cybersecurityProperties: list of confidentiality|integrity|availability|authenticity (YAML: cybersecurityProperties)
    pub cybersecurity_properties: Option<Vec<String>>,
    #[serde(flatten)]
    pub tier3: Boxed<ColdFrontmatter3>,
}

/// Third tier of the rarely-set fields; see [`ColdFrontmatter`].
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ColdFrontmatter3 {
    pub is_reference: Option<bool>,
    pub is_derived: Option<bool>,
    pub is_constant: Option<bool>,
    pub is_readonly: Option<bool>,
    pub is_portion: Option<bool>,
    pub is_ordered: Option<bool>,
    pub is_nonunique: Option<bool>,
    pub is_end: Option<bool>,
    pub is_individual: Option<bool>,
    pub value: Option<serde_yaml::Value>,
    pub value_kind: Option<String>,
    pub text: Option<String>,
    pub assume: Option<Vec<serde_yaml::Value>>,
    pub verdict_type: Option<String>,
    /// The accountable identity that approved the baseline (REQ-TRS-BL-001).
    pub approver: Option<String>,
    /// The intended source-control tag name (distinct from the `id`; REQ-TRS-BL-001).
    pub git_tag: Option<String>,
    /// The commit the baseline was sealed at, captured by `create` (REQ-TRS-BL-004).
    pub git_commit: Option<String>,
    /// The scope selector (REQ-TRS-BL-003). Named `frozenScope` to avoid colliding with
    /// the free-form TestPlan `scope` field.
    pub frozen_scope: Option<FrozenScope>,
    /// The generated content seal (REQ-TRS-BL-002).
    pub seal: Option<BaselineSeal>,
    /// What this element replaces: the `Baseline` a baseline replaces (REQ-TRS-BL-005), or the
    /// `ADR`s an ADR supersedes (GH #232). One reference or a list. Resolver-checked, not a
    /// suspect-tracked trace link.
    #[serde(default, serialize_with = "one_or_many::serialize")]
    pub supersedes: Option<Vec<String>>,
    pub is_composite: Option<bool>,
    pub portion_kind: Option<String>,

    // §8.3.2 — Port usage
    pub is_conjugated: Option<bool>,
    pub return_type: Option<String>,

    // §8.8.1 — StateDef entry/do/exit
    pub entry_action: Option<serde_yaml::Value>,
    pub do_action: Option<serde_yaml::Value>,
    pub exit_action: Option<serde_yaml::Value>,
    pub is_negated: Option<bool>,

    // §8.11.1 — RequirementDef
    pub framed_concerns: Option<Vec<String>>,
    #[serde(rename = "result")]
    pub result_type: Option<String>,

    // §8.12.3 — VerificationCaseDef
    pub verdict_expression: Option<String>,

    // §8.12.4 — UseCaseDef
    pub includes: Option<Vec<String>>,

    // §8.13 — Allocation convenience
    pub allocations: Option<Vec<serde_yaml::Value>>,

    // §8.14.1 — ViewpointDef
    pub satisfied_by: Option<Vec<String>>,

    // §8.14.2 — ViewDef
    pub rendering: Option<String>,

    // §3.7 — package
    pub filter_condition: Option<String>,

    // §8.2.4 — OccurrenceDef
    pub time_slices: Option<Vec<serde_yaml::Value>>,
    pub snapshots: Option<Vec<serde_yaml::Value>>,

    // §9.4 — variant reference
    pub variant_of: Option<String>,
    pub cardinality: Option<String>,
    pub contributes_to: Option<String>,

    // §9.9 — Build-system integration (build-config command)
    /// `buildExports:` — on a `FeatureDef`: a list of `{var, whenSelected, whenDeselected}`
    /// entries. Each entry declares a build variable emitted based on whether the feature
    /// is selected or deselected in a `Configuration`. `whenSelected` defaults to 1;
    /// `whenDeselected` absent means the variable is omitted when deselected.
    #[serde(rename = "buildExports", default, skip_serializing_if = "Option::is_none")]
    pub build_exports: Option<Vec<serde_yaml::Value>>,
    pub baseline_ref: Option<String>,

    // §T4-TARA — TARASheet section tables (ISO/SAE 21434)
    // Each is a list of row-mappings exploded by the walker into Tier-2 elements.
    pub damage_table: Option<Vec<serde_yaml::Value>>,   // → DamageScenario rows  (YAML: damageTable)
    pub threat_table: Option<Vec<serde_yaml::Value>>,   // → ThreatScenario rows   (YAML: threatTable)
    pub goal_table: Option<Vec<serde_yaml::Value>>,     // → CybersecurityGoal rows (YAML: goalTable)
    pub control_table: Option<Vec<serde_yaml::Value>>,  // → SecurityControl rows  (YAML: controlTable)
    pub mission_time: Option<String>,           // e.g. "1e9 h" (YAML: missionTime)
    pub probability: Option<f64>,               // cut-set or top-event probability (YAML: probability)
    pub latent_diagnostic_coverage: Option<f64>,  // DCl, 0.0–1.0 (YAML: latentDiagnosticCoverage)
    pub ccf_group: Option<String>,               // FaultTreeEvent common-cause group name (YAML: ccfGroup, GH #211)
    pub ccf_beta: Option<f64>,                   // FaultTreeEvent beta factor 0.0–1.0 (YAML: ccfBeta, GH #211)
    pub recommended_action: Option<String>,      // FMEAEntry mitigation (YAML: recommendedAction)
    pub fta_ref: Option<String>,                 // FMEAEntry → reconciling FaultTreeEvent (YAML: ftaRef)
    #[serde(skip)]
    pub unknown_fmea_keys: Vec<String>,          // keys not in recognised set; validator emits E922
    pub cve_id: Option<String>,                 // CVE-YYYY-NNNNN
    pub asset_owner: Option<String>,          // qname/id of owning architecture element (YAML: assetOwner)
    pub related_safety_goal: Option<String>,  // SG-* ref for co-engineering (YAML: relatedSafetyGoal)
    /// TARASheet `assetTable:` rows → Asset elements (YAML: assetTable)
    pub asset_table: Option<Vec<serde_yaml::Value>>,
    /// VulnerabilityReport CVSS vector string (YAML: cvssVector)
    pub cvss_vector: Option<String>,
    /// VulnerabilityReport declared severity bucket none|low|medium|high|critical (YAML: cvssSeverity)
    pub cvss_severity: Option<String>,
    /// VulnerabilityReport fixed-in version (YAML: fixedIn)
    pub fixed_in: Option<String>,
}

/// Flat deserialisation target for the rarely-set fields: serde's nested `flatten`
/// would leave their keys visible to the `extra` catch-all (spurious `W047`), so
/// the file is read into this one flat struct and split into tiers.
#[derive(Default, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ColdWire {
    pub is_variation: Option<bool>,
    pub expression: Option<String>,
    pub metadata: Option<Vec<serde_yaml::Value>>,
    pub binding_connections: Option<Vec<serde_yaml::Value>>,
    pub succession_connections: Option<Vec<serde_yaml::Value>>,
    pub sub_states: Option<Vec<serde_yaml::Value>>,
    pub transitions: Option<Vec<serde_yaml::Value>>,
    pub exhibits_states: Option<Vec<String>>,
    pub operations: Option<Vec<serde_yaml::Value>>,
    pub actors: Option<Vec<String>>,
    pub steps: Option<Vec<String>>,
    #[serde(default, deserialize_with = "string_or_vec::deserialize")]
    pub allocated_from: Option<Vec<String>>,
    pub expose: Option<Vec<serde_yaml::Value>>,
    pub viewpoint: Option<String>,
    pub svg_mode: Option<String>,
    pub svg_file: Option<String>,
    pub puml_mode: Option<String>,
    pub puml_file: Option<String>,
    pub shapes: Option<serde_yaml::Value>,
    pub edges: Option<serde_yaml::Value>,
    pub layout: Option<serde_yaml::Value>,
    pub imports: Option<Vec<serde_yaml::Value>>,
    /// REQ-TRS-MG-001 — MagicGrid `«refine»`: a `UseCaseDef`/`UseCase` gives concrete
    /// behavioural meaning to a requirement. Optional list of cross-references
    /// (qname or stable `REQ-*` id), resolved like `verifies:`/`derivedFrom:`.
    pub refines: Option<Vec<String>>,
    pub about: Option<serde_yaml::Value>,
    pub locale: Option<String>,
    pub sil_level: Option<u8>,
    pub asil_level: Option<String>,
    /// ASIL/SIL decomposition argument type (§22.3): `independent` | `redundant` | `diverse`.
    /// Informational; surfaced in the safety-case report.
    pub decomposition_kind: Option<String>,
    /// Original (pre-decomposition) ASIL of a decomposed requirement (ISO 26262-9 §5):
    /// `decomposedFrom: D`, or implied by the `B(D)` notation in `asilLevel:`. Letter A–D.
    pub decomposed_from: Option<String>,
    pub wcet: Option<String>,
    /// `configurations:` — scalar or list of `Configuration` references. Absent
    /// → config-agnostic (applies to every Configuration). Each must resolve to a
    /// `Configuration` (else E606).
    #[serde(default, deserialize_with = "string_or_vec::deserialize")]
    pub configurations: Option<Vec<String>>,
    /// `demonstrates:` — scalar or list of Requirement/SafetyGoal/
    /// CybersecurityGoal/Argument the plan is offered as evidence for (else E603).
    #[serde(default, deserialize_with = "string_or_vec::deserialize")]
    pub demonstrates: Option<Vec<String>>,

    /// REQ-TRS-SUS-LINKS-001 — suspect-link baselines. A map from a trace-link
    /// **target identifier** (exactly as authored on the link — a stable id or a
    /// qualified name) to the algorithm-prefixed content hash (`blake3:<hex>`) of
    /// that target's normative projection (REQ-TRS-SUS-LINKS-002), captured at the
    /// moment the link was last reviewed. One map on the source (which holds the
    /// link, per §12.1) covers every link kind. `BTreeMap` → deterministic, sorted
    /// serialization so re-baselining produces minimal diffs.
    #[serde(rename = "traceBaselines", default, skip_serializing_if = "Option::is_none")]
    pub trace_baselines: Option<std::collections::BTreeMap<String, String>>,

    // §3.1 — identity override
    pub qualified_name: Option<String>,

    // §3.2 — classification flags
    pub is_variant: Option<bool>,

    // §8.4.x — connection/binding elements
    pub ends: Option<Vec<serde_yaml::Value>>,

    // §8.7.1 + §8.9.1 — Action/Calculation body
    pub body: Option<String>,
    pub body_language: Option<String>,
    /// `CalculationDef` (§22.2): qualified name of a `ConstraintDef` bounding the budget result.
    pub evaluate: Option<String>,
    // Native ReviewRecord fields (§19, GH #71). `recordedAt` is the thin pointer to the
    // external review (e.g. a GitHub PR/review URL); the model keeps the baselined anchor.
    pub review_type: Option<String>,
    #[serde(default, deserialize_with = "string_or_vec::deserialize")]
    pub reviews: Option<Vec<String>>,
    pub items: Option<Vec<serde_yaml::Value>>,
    // Native TradeStudy fields (§15, GH #63). `objective` (Requirement) is shared above.
    pub criteria: Option<Vec<serde_yaml::Value>>,
    pub alternatives: Option<Vec<serde_yaml::Value>>,
    pub scores: Option<Vec<serde_yaml::Value>>,
    // Native IEC 62443 Zone/Conduit fields (§13, GH #61).
    #[serde(rename = "targetSL")]
    pub target_sl: Option<u8>,
    #[serde(rename = "achievedSL")]
    pub achieved_sl: Option<u8>,
    #[serde(default, deserialize_with = "string_or_vec::deserialize")]
    pub members: Option<Vec<String>>,
    pub from_zone: Option<String>,
    pub to_zone: Option<String>,
    /// §14.3 — `repoImports:` on a Package `_index.md`: a list of mappings
    /// `{repo, qname, as}` mounting a sub-tree from a peer repo declared in
    /// `[repos]`. Untyped here; the validator reads the `repo`/`qname`/`as` keys.
    pub repo_imports: Option<Vec<serde_yaml::Value>>,
    /// `sysmlSubmodel: true` on a Package `_index.md` (`ADR-SYS-SYSMLV2-001`,
    /// `REQ-TRS-SYSMLV2-001`): every `.sysml`/`.kerml` file anywhere in that
    /// directory's subtree is parsed as native SysML v2/KerML textual notation
    /// instead of Markdown+YAML frontmatter. Handled by `crate::sysmlv2`.
    pub sysml_submodel: Option<bool>,
    pub sub_actions: Option<Vec<serde_yaml::Value>>,
    pub is_parallel: Option<bool>,

    // §8.12.1 — Case elements
    pub objectives: Option<Vec<serde_yaml::Value>>,

    // §8.15.1 — MetadataDef
    pub annotates: Option<Vec<String>>,
    /// Membership flag (REQ-TRS-FM-004): when `true`, the feature is mandatory
    /// (forced on with its parent, or root-selected when top-level) independently
    /// of `groupKind`. Legacy `groupKind: mandatory` remains a shorthand.
    pub mandatory: Option<bool>,

    /// `featureTree:` (REQ-TRS-FM-005) — on a `type: FeatureModel` sheet: the
    /// whole feature model as one **flat** list. Each entry is shaped like a
    /// `FeatureDef`'s own frontmatter (`id`, `mandatory`, `groupKind`,
    /// `cardinality`, `requires`, `excludes`, `parameters`, `buildExports`, an
    /// optional `doc:` body), but its `name:` is a **dot-separated relative
    /// path** from the sheet — e.g. `Platform.CortexM` — rather than a single
    /// basic name. This is a mini-DSL scoped to `featureTree:` entries only; it
    /// does not change `name:`'s meaning anywhere else in the format.
    ///
    /// The walker's explode pass (`walker::explode_feature_model_trees`) turns
    /// each entry into a synthetic `FeatureDef` `RawElement`: the dotted path is
    /// split on `.`, each segment becomes one `::`-joined qname component under
    /// the sheet's own qname (so `Platform.CortexM` under sheet `Features`
    /// yields `Features::Platform::CortexM` — exactly the qname a
    /// directory-per-feature layout would produce for the same tree shape), and
    /// the synthesized element's own `name:` is rewritten to just the last path
    /// segment (`CortexM`) — the same leaf label a per-file `FeatureDef` would
    /// carry. An ancestor segment need not have its own entry (mirrors today's
    /// multi-file behavior: a qname prefix that is not itself a `FeatureDef`
    /// simply implies no parent). Every downstream consumer (validator,
    /// `feature-check`, `matrix`, the web UI) sees the same kind of `FeatureDef`
    /// element either way. Purely additive/opt-in: unrelated to the existing
    /// per-attribute `features:` field.
    #[serde(rename = "featureTree")]
    pub feature_tree: Option<Vec<serde_yaml::Value>>,

    /// `crossTreeConstraints:` (REQ-TRS-FM-005) — on a `type: FeatureModel`
    /// sheet: a flat list of `{ feature, requires, excludes }` entries, kept
    /// separate from the `featureTree:` structural list so the model's
    /// requires/excludes edges can be reviewed as one section instead of
    /// scattered across entries (inline `requires:`/`excludes:` on a
    /// `featureTree:` entry still works too — this section is additive).
    /// `feature`/`requires`/`excludes` values resolve the same way: containing
    /// `::` → already an absolute qname; starting with `FEAT` → a stable id;
    /// otherwise → a dot-separated path relative to this sheet, resolved
    /// exactly like a `featureTree:` entry's `name:`. The walker's explode pass
    /// merges each resolved `requires`/`excludes` into the matching synthesized
    /// `FeatureDef`'s own field. A `feature:` that doesn't resolve to a
    /// `FeatureDef` synthesized from this same sheet is `E233` — there is
    /// nothing local to attach the constraint to.
    #[serde(rename = "crossTreeConstraints")]
    pub cross_tree_constraints: Option<Vec<serde_yaml::Value>>,

    /// `parameterConstraints:` (§9.7) — cross-feature numeric constraints,
    /// declared on a `Package`/`LibraryPackage`/`Namespace` `_index.md` or
    /// (REQ-TRS-FM-005) directly on a `type: FeatureModel` sheet. Evaluated by
    /// `feature-check` (`E213`/`E221`/`W014`/`W025`). Promoted to a typed field
    /// (previously read out of the `extra` catch-all) so declaring it no
    /// longer falsely raises `W047` on the very element type that hosts it.
    #[serde(rename = "parameterConstraints")]
    pub parameter_constraints: Option<Vec<serde_yaml::Value>>,
    pub parameter_bindings: Option<serde_yaml::Value>,

    /// `subConfigurations:` (REQ-TRS-HPLE-001, ADR-SYS-HPLE-001) — on a
    /// `Configuration`: one or more other `Configuration` elements (qname or
    /// stable `CONF-*` id) this `Configuration` consolidates — a hierarchical
    /// product-line composition. Each entry resolves like any other
    /// cross-reference: the local model first, then each loaded peer repo in
    /// declaration order (§14.4). Scalar or list, following the
    /// `derivedFrom`/`achieves` convention. Naturally empty/absent at a leaf
    /// tier with no lower-tier product lines to consolidate. Resolution and
    /// the peer-validity gate are a validator pass, not a parse-time concern.
    #[serde(default, deserialize_with = "string_or_vec::deserialize")]
    pub sub_configurations: Option<Vec<String>>,
    pub is_deployment_package: Option<bool>,

    /// REQ-TRS-SAFE-007 (ISO 26262-8 §5 DIA / ISO/SAE 21434 §7 CIA) — the
    /// accountable party/organisation for a work product (the DIA/CIA split,
    /// e.g. "OEM" / "Supplier-X"). Drives the opt-in W038 check. (YAML: responsibility)
    pub responsibility: Option<String>,

    /// REQ-TRS-SAFE-007 (ISO 26262-2 §6) — ConfirmationMeasure kind:
    /// confirmation_review | functional_safety_audit | functional_safety_assessment |
    /// cybersecurity_assessment. Invalid → E849. (YAML: measureType)
    pub measure_type: Option<String>,
    /// REQ-TRS-SAFE-007 — ConfirmationMeasure independence level: I1 | I2 | I3.
    /// Invalid → E850. (YAML: independenceLevel)
    pub independence_level: Option<String>,
    /// REQ-TRS-SAFE-007 — the work product(s) a ConfirmationMeasure confirms.
    /// String or list; each resolves via the Resolver (else E851). (YAML: confirms)
    #[serde(default, deserialize_with = "string_or_vec::deserialize")]
    pub confirms: Option<Vec<String>>,

    // §T4 — FaultTree (IEC 61025 / ISO 26262-9)
    pub top_event: Option<String>,              // SafetyGoal ref (YAML: topEvent)
    pub gate_type: Option<String>,              // FaultTreeGate: AND|OR|XOR|NOT|inhibit (YAML: gateType)
    pub inputs: Option<Vec<String>>,            // FaultTreeGate input refs (YAML: inputs)
    pub event_kind: Option<String>,             // FaultTreeEvent: basic|undeveloped|house (YAML: eventKind)
    pub failure_rate: Option<f64>,              // FaultTreeEvent failure rate /h (YAML: failureRate)
    /// REQ-TRS-FTA-002 (issue #148) — FaultTreeEvent → the model element whose
    /// failure the event represents (qualified name or stable id; typically a
    /// `Part`/`PartDef`). Dangling → E927. Only meaningful on `FaultTreeEvent`;
    /// on any other type it is still reported as an unrecognized field (W047).
    /// (YAML: ref)
    #[serde(rename = "ref")]
    pub event_ref: Option<String>,

    // §T4 — AttackTree (ISO/SAE 21434 §15.7 attack path analysis)
    pub threat_ref: Option<String>,             // AttackTree → ThreatScenario ref (YAML: threatRef)
    // §T4 — FMEDA diagnostic coverage (ISO 26262-5 §8-9), documented for FaultTreeEvent.
    pub diagnostic_coverage: Option<f64>,         // DC, 0.0–1.0 (YAML: diagnosticCoverage)

    // §T4 — FMEASheet / FMEAEntry (IEC 60812 / SAE J1739)
    pub entries: Option<Vec<serde_yaml::Value>>, // FMEASheet sub-entries (YAML: entries)
    pub failure_mode: Option<String>,            // FMEAEntry: what fails (YAML: failureMode)
    pub effect: Option<String>,                  // FMEAEntry: consequence (YAML: effect)
    pub cause: Option<String>,                   // FMEAEntry: root cause (YAML: cause)
    pub fmea_severity: Option<u8>,               // FMEAEntry severity 1–10 (YAML: fmeaSeverity)
    pub occurrence: Option<u8>,                  // FMEAEntry occurrence 1–10 (YAML: occurrence)
    pub detection: Option<u8>,                   // FMEAEntry detection 1–10 (YAML: detection)
    pub rpn: Option<u32>,                        // FMEAEntry Risk Priority Number (YAML: rpn)

    // §T2 — HazardousEvent (ISO 26262 §7 HARA)
    pub severity: Option<String>,               // S0-S3
    pub exposure: Option<String>,               // E0-E4
    pub controllability: Option<String>,        // C0-C3
    pub operational_situation: Option<String>,  // free-text operating scenario

    // §T2 — SafetyGoal (ISO 26262 §7 / IEC 61508 / ISO 13849)
    pub safe_state: Option<String>,             // description of the safe state
    pub hazardous_events: Option<Vec<String>>,  // HazardousEvent id/qname refs

    // §T2 — DamageScenario (ISO/SAE 21434 §15)
    pub damage_severity: Option<String>,        // severe|major|moderate|negligible
    pub impact_categories: Option<Vec<String>>, // safety|financial|operational|privacy

    /// §T4 safety↔security co-engineering (ISO 26262 ⇄ ISO/SAE 21434) — cross-link
    /// from a `DamageScenario`/`ThreatScenario` to the `HazardousEvent`/`SafetyGoal`
    /// it endangers. String or list. Resolved via `Resolver::resolve_ref`; target
    /// must be a `HazardousEvent` or `SafetyGoal` (else E844). (YAML: hazardRef)
    #[serde(default, deserialize_with = "string_or_vec::deserialize")]
    pub hazard_ref: Option<Vec<String>>,

    // §T2 — ThreatScenario (ISO/SAE 21434 §15)
    pub attack_feasibility: Option<String>,     // high|medium|low|very_low
    pub attack_vector: Option<String>,          // network|adjacent|local|physical
    // GH #222 — ISO/SAE 21434 attack-potential factors (ThreatScenario / AttackStep)
    // and per-category impact ratings (DamageScenario). All optional; see `risk.rs`.
    pub elapsed_time: Option<serde_yaml::Value>,
    pub expertise: Option<serde_yaml::Value>,
    pub knowledge: Option<serde_yaml::Value>,
    pub window_of_opportunity: Option<serde_yaml::Value>,
    pub equipment: Option<serde_yaml::Value>,
    pub safety_impact: Option<String>,
    pub financial_impact: Option<String>,
    pub operational_impact: Option<String>,
    pub privacy_impact: Option<String>,
    pub damage_scenarios: Option<Vec<String>>,  // DamageScenario id/qname refs

    // §T2 — CybersecurityGoal (ISO/SAE 21434 §15)
    pub cal_level: Option<String>,              // CAL1-CAL4
    #[serde(default, deserialize_with = "named_string_or_vec::security_property")]
    pub security_property: Option<Vec<String>>, // confidentiality|integrity|availability|authenticity (string or list)
    pub derived_from_safety_goal: Option<String>,   // SG-* that generated this requirement (YAML: derivedFromSafetyGoal)

    // §8.18 — GSN safety-argument layer (issue #20)
    /// `Argument.argumentType` (YAML: argumentType) ∈ {claim, strategy, solution};
    /// absent is treated as `claim`. Invalid → E854.
    pub argument_type: Option<String>,
    /// `Argument.supports` (YAML: supports) — the SafetyGoal or parent Argument this
    /// node argues for (the GSN supported goal). String or list; each ref resolves
    /// via the Resolver (else E855).
    #[serde(default, deserialize_with = "string_or_vec::deserialize")]
    pub supports: Option<Vec<String>>,
    /// `AssumptionOfUse.appliesTo` (YAML: appliesTo) — the SafetyGoal / Argument /
    /// Requirement this SRAC constrains. String or list; each ref resolves via the
    /// Resolver (else E858).
    #[serde(default, deserialize_with = "string_or_vec::deserialize")]
    pub applies_to: Option<Vec<String>>,

    // §T2 — TestCase security test method (REQ-TRS-SEC-008; ISO/SAE 21434 §13.3)
    // Valid: fuzz|penetration_test|security_regression|vulnerability_scan|threat_modeling
    // Invalid → W809. (YAML: securityTestMethod)
    pub security_test_method: Option<String>,
    pub conjugates: Option<String>,
    pub flow_connections: Option<Vec<serde_yaml::Value>>,
    pub performs: Option<Vec<serde_yaml::Value>>,
    pub objective: Option<String>,
    pub stakeholders: Option<Vec<String>>,
    pub concerns: Option<Vec<String>>,
    pub methods: Option<Vec<String>>,
    pub depends_on: Option<Vec<String>>,
    pub extends: Option<Vec<serde_yaml::Value>>,
    pub extension_points: Option<Vec<serde_yaml::Value>>,
    pub clients: Option<Vec<String>>,
    pub suppliers: Option<Vec<String>>,
    /// **Deprecated / removed as a label** (REQ-TRS-NAME-002). Every element now labels
    /// via `name`; `title` is no longer a recognized label field. It is still parsed
    /// here only so the validator can detect a stray `title:` and reject it via `E025`.
    pub title: Option<String>,
    pub dal_level: Option<String>,
    pub requirement_kind: Option<String>,
    pub coverage_target: Option<String>,
    /// `selection:` — additive membership query (REQ-TRS-PLAN-003).
    pub selection: Option<TestPlanSelection>,

    // §Baseline (ADR-SYS-BASELINE-001) — release-baseline fields on a `type: Baseline`.
    /// The baseline date (REQ-TRS-BL-001).
    pub date: Option<String>,

    // §8.5.2 — EnumerationDef
    pub values: Option<Vec<serde_yaml::Value>>,
    pub review_date: Option<String>,
    #[serde(default, deserialize_with = "string_or_vec::deserialize")]
    pub reviewed_by: Option<Vec<String>>,
    pub recorded_at: Option<String>,
    pub decision: Option<String>,
    pub rationale: Option<String>,
    #[serde(default, deserialize_with = "string_or_vec::deserialize")]
    pub protocols: Option<Vec<String>>,
    pub in_zone: Option<String>,
    /// `foreignFormat: <alias>` on a Package `_index.md` (`ADR-SYS-PLUGIN-002`):
    /// hands the package's entire directory subtree to the stdio-subprocess
    /// plugin named by `[plugins.<alias>]` in `.syscribe.toml`. Handled by
    /// `crate::plugins`.
    pub foreign_format: Option<String>,
    /// `annotationFormat: <label>` on a Package `_index.md` (`ADR-SYS-ANNOTATE-001`):
    /// hands the package's entire directory subtree to the in-process
    /// comment-marker scanner. `label` is a human-readable tag only (no
    /// `.syscribe.toml` indirection, unlike `foreignFormat:`) — the scan
    /// parameters (`marker`/`include`/`exclude`) live inline on this same
    /// `_index.md`. Handled by `crate::annotations`.
    pub annotation_format: Option<String>,
    /// `marker:` — a regex matched against each line of every scanned file;
    /// the first match on a line starts a marker block. Required when
    /// `annotationFormat:` is set (`E560` otherwise).
    pub marker: Option<String>,
    /// `include:` — glob patterns (relative to this package's directory,
    /// `**`/`*`/`?` supported) selecting which files are scanned for markers.
    /// Required, non-empty, when `annotationFormat:` is set (`E560` otherwise).
    /// On a derived `Diagram` (`REQ-TRS-VIS-003`) the same key lists the
    /// members of the subject to show.
    pub include: Option<Vec<String>>,
    /// `exclude:` — glob patterns excluded from `include:`'s matches; on a
    /// derived `Diagram`, members of the subject to drop.
    pub exclude: Option<Vec<String>>,
    /// On a derived BDD `Diagram`: composition levels followed beyond the subject's blocks (default 1).
    pub depth: Option<usize>,
    pub control_nodes: Option<Vec<serde_yaml::Value>>,

    // §8.10.2 — Constraint usage
    pub is_asserted: Option<bool>,
    pub is_semantic: Option<bool>,
    pub aliases: Option<Vec<serde_yaml::Value>>,

    // §3.12 — representation
    pub rep: Option<String>,

    // §3.3 — InterfaceDef constraints
    pub constraints: Option<Vec<serde_yaml::Value>>,
    pub parent_feature: Option<String>,

    /// `buildOverrides:` — on a `Configuration`: a flat mapping of `varName -> scalar`
    /// that wins over any `buildExports` or parameter `buildVar` contribution.
    /// Last-writer-wins semantics; resolves E050 conflicts. Consistent pattern with
    /// `parameter_bindings` (also `Option<serde_yaml::Value>`).
    #[serde(rename = "buildOverrides", default, skip_serializing_if = "Option::is_none")]
    pub build_overrides: Option<serde_yaml::Value>,

    /// REQ-TRS-ADR-001 (GH #159) — §8.17.1 `ADR` `deciders:`: the decision-makers,
    /// each a stakeholder `PartDef` qualified name or a free-text name. Opaque
    /// display metadata, never a cross-reference (a free-text name is legitimate),
    /// so it is not resolved. A scalar is accepted as a one-entry list. Only a
    /// schema field on an `ADR`; on any other type it is still reported as an
    /// unrecognized field (W047).
    #[serde(default, deserialize_with = "string_or_vec::deserialize")]
    pub deciders: Option<Vec<String>>,

    /// REQ-TRS-SAFE-006 (ISO 26262-9 §7) — freedom-from-interference / partitioning
    /// rationale (YAML: `ffiRationale`). A non-empty string on a shared allocation
    /// target or on a source excuses a mixed-criticality sharing (clears W034).
    pub ffi_rationale: Option<String>,
    pub fmea_ref: Option<String>,               // FaultTreeEvent → reconciling FMEAEntry (YAML: fmeaRef)
    // IEC 61508 §3 risk graph parameters (alternative to ISO 26262 S/E/C)
    pub consequence: Option<String>,            // Ca | Cb | Cc | Cd
    pub freq_exposure: Option<String>,          // Fa | Fb  (YAML: freqExposure)
    pub avoidance: Option<String>,              // Pa | Pb
    pub demand_rate: Option<String>,            // W1 | W2 | W3  (YAML: demandRate)
    pub ftti: Option<String>,                   // Fault Tolerant Time Interval (e.g. "20ms")
    pub pl_level: Option<String>,               // ISO 13849-1 Performance Level: a|b|c|d|e (YAML: plLevel)
    // DamageScenario.assets: references to Asset elements (REQ-TRS-TYPE-017, YAML: assets)
    #[serde(default, deserialize_with = "string_or_vec::deserialize")]
    pub assets: Option<Vec<String>>,
    /// §T2 risk treatment decision (ISO/SAE 21434 §9 / §15.9): avoid|reduce|share|retain.
    /// Invalid value → E845. (YAML: riskTreatment)
    pub risk_treatment: Option<String>,
    /// §T2 free-text residual-risk note after treatment (no validation). (YAML: residualRisk)
    pub residual_risk: Option<String>,
    pub threat_scenarios: Option<Vec<String>>,  // ThreatScenario id/qname refs

    // §T2 — SecurityControl (ISO/SAE 21434)
    pub control_type: Option<String>,           // prevention|detection|response|recovery
    pub implements_goals: Option<Vec<String>>,  // CybersecurityGoal id/qname refs

    // §T2 — VulnerabilityReport
    pub cvss_score: Option<f64>,                // 0.0-10.0
    pub affected_elements: Option<Vec<String>>, // qualified names of affected model elements
    pub mitigated_by: Option<Vec<String>>,      // SecurityControl id/qname refs

    // §T2 — upstream goal links for native Requirement
    // YAML: derivedFromCybersecurityGoal; alias: derivedFromSecurityGoal (legacy)
    #[serde(alias = "derivedFromSecurityGoal", default, deserialize_with = "named_string_or_vec::derived_from_cybersecurity_goal")]
    pub derived_from_cybersecurity_goal: Option<Vec<String>>,

    // §T2 — Asset (REQ-TRS-TYPE-017; ISO/SAE 21434 §15.3 asset identification)
    // cybersecurityProperties: list of confidentiality|integrity|availability|authenticity (YAML: cybersecurityProperties)
    pub cybersecurity_properties: Option<Vec<String>>,
    pub is_reference: Option<bool>,
    pub is_derived: Option<bool>,
    pub is_constant: Option<bool>,
    pub is_readonly: Option<bool>,
    pub is_portion: Option<bool>,
    pub is_ordered: Option<bool>,
    pub is_nonunique: Option<bool>,
    pub is_end: Option<bool>,
    pub is_individual: Option<bool>,
    pub value: Option<serde_yaml::Value>,
    pub value_kind: Option<String>,
    pub text: Option<String>,
    pub assume: Option<Vec<serde_yaml::Value>>,
    pub verdict_type: Option<String>,
    /// The accountable identity that approved the baseline (REQ-TRS-BL-001).
    pub approver: Option<String>,
    /// The intended source-control tag name (distinct from the `id`; REQ-TRS-BL-001).
    pub git_tag: Option<String>,
    /// The commit the baseline was sealed at, captured by `create` (REQ-TRS-BL-004).
    pub git_commit: Option<String>,
    /// The scope selector (REQ-TRS-BL-003). Named `frozenScope` to avoid colliding with
    /// the free-form TestPlan `scope` field.
    pub frozen_scope: Option<FrozenScope>,
    /// The generated content seal (REQ-TRS-BL-002).
    pub seal: Option<BaselineSeal>,
    /// What this element replaces: the `Baseline` a baseline replaces (REQ-TRS-BL-005), or the
    /// `ADR`s an ADR supersedes (GH #232). One reference or a list. Resolver-checked, not a
    /// suspect-tracked trace link.
    #[serde(default, deserialize_with = "string_or_vec::deserialize", serialize_with = "one_or_many::serialize")]
    pub supersedes: Option<Vec<String>>,
    pub is_composite: Option<bool>,
    pub portion_kind: Option<String>,

    // §8.3.2 — Port usage
    pub is_conjugated: Option<bool>,
    pub return_type: Option<String>,

    // §8.8.1 — StateDef entry/do/exit
    pub entry_action: Option<serde_yaml::Value>,
    pub do_action: Option<serde_yaml::Value>,
    pub exit_action: Option<serde_yaml::Value>,
    pub is_negated: Option<bool>,

    // §8.11.1 — RequirementDef
    pub framed_concerns: Option<Vec<String>>,
    #[serde(rename = "result")]
    pub result_type: Option<String>,

    // §8.12.3 — VerificationCaseDef
    pub verdict_expression: Option<String>,

    // §8.12.4 — UseCaseDef
    pub includes: Option<Vec<String>>,

    // §8.13 — Allocation convenience
    pub allocations: Option<Vec<serde_yaml::Value>>,

    // §8.14.1 — ViewpointDef
    pub satisfied_by: Option<Vec<String>>,

    // §8.14.2 — ViewDef
    pub rendering: Option<String>,

    // §3.7 — package
    pub filter_condition: Option<String>,

    // §8.2.4 — OccurrenceDef
    pub time_slices: Option<Vec<serde_yaml::Value>>,
    pub snapshots: Option<Vec<serde_yaml::Value>>,

    // §9.4 — variant reference
    pub variant_of: Option<String>,
    pub cardinality: Option<String>,
    pub contributes_to: Option<String>,

    // §9.9 — Build-system integration (build-config command)
    /// `buildExports:` — on a `FeatureDef`: a list of `{var, whenSelected, whenDeselected}`
    /// entries. Each entry declares a build variable emitted based on whether the feature
    /// is selected or deselected in a `Configuration`. `whenSelected` defaults to 1;
    /// `whenDeselected` absent means the variable is omitted when deselected.
    #[serde(rename = "buildExports", default, skip_serializing_if = "Option::is_none")]
    pub build_exports: Option<Vec<serde_yaml::Value>>,
    pub baseline_ref: Option<String>,

    // §T4-TARA — TARASheet section tables (ISO/SAE 21434)
    // Each is a list of row-mappings exploded by the walker into Tier-2 elements.
    pub damage_table: Option<Vec<serde_yaml::Value>>,   // → DamageScenario rows  (YAML: damageTable)
    pub threat_table: Option<Vec<serde_yaml::Value>>,   // → ThreatScenario rows   (YAML: threatTable)
    pub goal_table: Option<Vec<serde_yaml::Value>>,     // → CybersecurityGoal rows (YAML: goalTable)
    pub control_table: Option<Vec<serde_yaml::Value>>,  // → SecurityControl rows  (YAML: controlTable)
    pub mission_time: Option<String>,           // e.g. "1e9 h" (YAML: missionTime)
    pub probability: Option<f64>,               // cut-set or top-event probability (YAML: probability)
    pub latent_diagnostic_coverage: Option<f64>,  // DCl, 0.0–1.0 (YAML: latentDiagnosticCoverage)
    pub ccf_group: Option<String>,               // FaultTreeEvent common-cause group name (YAML: ccfGroup, GH #211)
    pub ccf_beta: Option<f64>,                   // FaultTreeEvent beta factor 0.0–1.0 (YAML: ccfBeta, GH #211)
    pub recommended_action: Option<String>,      // FMEAEntry mitigation (YAML: recommendedAction)
    pub fta_ref: Option<String>,                 // FMEAEntry → reconciling FaultTreeEvent (YAML: ftaRef)
    #[serde(skip)]
    pub unknown_fmea_keys: Vec<String>,          // keys not in recognised set; validator emits E922
    pub cve_id: Option<String>,                 // CVE-YYYY-NNNNN
    pub asset_owner: Option<String>,          // qname/id of owning architecture element (YAML: assetOwner)
    pub related_safety_goal: Option<String>,  // SG-* ref for co-engineering (YAML: relatedSafetyGoal)
    /// TARASheet `assetTable:` rows → Asset elements (YAML: assetTable)
    pub asset_table: Option<Vec<serde_yaml::Value>>,
    /// VulnerabilityReport CVSS vector string (YAML: cvssVector)
    pub cvss_vector: Option<String>,
    /// VulnerabilityReport declared severity bucket none|low|medium|high|critical (YAML: cvssSeverity)
    pub cvss_severity: Option<String>,
    /// VulnerabilityReport fixed-in version (YAML: fixedIn)
    pub fixed_in: Option<String>,
}

impl ColdWire {
    fn into_tiers(self) -> ColdFrontmatter {
        let w = self;
        let (asil_eff, asil_orig) = crate::asil::split_notation(w.asil_level);
        ColdFrontmatter {
            is_variation: w.is_variation,
            expression: w.expression,
            metadata: w.metadata,
            binding_connections: w.binding_connections,
            succession_connections: w.succession_connections,
            sub_states: w.sub_states,
            transitions: w.transitions,
            exhibits_states: w.exhibits_states,
            operations: w.operations,
            actors: w.actors,
            steps: w.steps,
            allocated_from: w.allocated_from,
            expose: w.expose,
            viewpoint: w.viewpoint,
            svg_mode: w.svg_mode,
            svg_file: w.svg_file,
            puml_mode: w.puml_mode,
            puml_file: w.puml_file,
            shapes: w.shapes,
            edges: w.edges,
            layout: w.layout,
            imports: w.imports,
            refines: w.refines,
            about: w.about,
            locale: w.locale,
            sil_level: w.sil_level,
            asil_level: asil_eff,
            decomposition_kind: w.decomposition_kind,
            decomposed_from: w.decomposed_from.or(asil_orig),
            wcet: w.wcet,
            configurations: w.configurations,
            demonstrates: w.demonstrates,
            trace_baselines: w.trace_baselines,
            qualified_name: w.qualified_name,
            is_variant: w.is_variant,
            ends: w.ends,
            body: w.body,
            body_language: w.body_language,
            evaluate: w.evaluate,
            review_type: w.review_type,
            reviews: w.reviews,
            items: w.items,
            criteria: w.criteria,
            alternatives: w.alternatives,
            scores: w.scores,
            target_sl: w.target_sl,
            achieved_sl: w.achieved_sl,
            members: w.members,
            from_zone: w.from_zone,
            to_zone: w.to_zone,
            repo_imports: w.repo_imports,
            sysml_submodel: w.sysml_submodel,
            sub_actions: w.sub_actions,
            is_parallel: w.is_parallel,
            objectives: w.objectives,
            annotates: w.annotates,
            mandatory: w.mandatory,
            feature_tree: w.feature_tree,
            cross_tree_constraints: w.cross_tree_constraints,
            parameter_constraints: w.parameter_constraints,
            parameter_bindings: w.parameter_bindings,
            sub_configurations: w.sub_configurations,
            is_deployment_package: w.is_deployment_package,
            responsibility: w.responsibility,
            measure_type: w.measure_type,
            independence_level: w.independence_level,
            confirms: w.confirms,
            top_event: w.top_event,
            gate_type: w.gate_type,
            inputs: w.inputs,
            event_kind: w.event_kind,
            failure_rate: w.failure_rate,
            event_ref: w.event_ref,
            threat_ref: w.threat_ref,
            diagnostic_coverage: w.diagnostic_coverage,
            entries: w.entries,
            failure_mode: w.failure_mode,
            effect: w.effect,
            cause: w.cause,
            fmea_severity: w.fmea_severity,
            occurrence: w.occurrence,
            detection: w.detection,
            rpn: w.rpn,
            severity: w.severity,
            exposure: w.exposure,
            controllability: w.controllability,
            operational_situation: w.operational_situation,
            safe_state: w.safe_state,
            hazardous_events: w.hazardous_events,
            damage_severity: w.damage_severity,
            impact_categories: w.impact_categories,
            hazard_ref: w.hazard_ref,
            attack_feasibility: w.attack_feasibility,
            attack_vector: w.attack_vector,
            elapsed_time: w.elapsed_time,
            expertise: w.expertise,
            knowledge: w.knowledge,
            window_of_opportunity: w.window_of_opportunity,
            equipment: w.equipment,
            safety_impact: w.safety_impact,
            financial_impact: w.financial_impact,
            operational_impact: w.operational_impact,
            privacy_impact: w.privacy_impact,
            damage_scenarios: w.damage_scenarios,
            cal_level: w.cal_level,
            security_property: w.security_property,
            derived_from_safety_goal: w.derived_from_safety_goal,
            argument_type: w.argument_type,
            supports: w.supports,
            applies_to: w.applies_to,
            security_test_method: w.security_test_method,
            tier2: Boxed::of(ColdFrontmatter2 {
                conjugates: w.conjugates,
                flow_connections: w.flow_connections,
                performs: w.performs,
                objective: w.objective,
                stakeholders: w.stakeholders,
                concerns: w.concerns,
                methods: w.methods,
                depends_on: w.depends_on,
                extends: w.extends,
                extension_points: w.extension_points,
                clients: w.clients,
                suppliers: w.suppliers,
                title: w.title,
                dal_level: w.dal_level,
                requirement_kind: w.requirement_kind,
                coverage_target: w.coverage_target,
                selection: w.selection,
                date: w.date,
                values: w.values,
                review_date: w.review_date,
                reviewed_by: w.reviewed_by,
                recorded_at: w.recorded_at,
                decision: w.decision,
                rationale: w.rationale,
                protocols: w.protocols,
                in_zone: w.in_zone,
                foreign_format: w.foreign_format,
                annotation_format: w.annotation_format,
                marker: w.marker,
                include: w.include,
                exclude: w.exclude,
                depth: w.depth,
                control_nodes: w.control_nodes,
                is_asserted: w.is_asserted,
                is_semantic: w.is_semantic,
                aliases: w.aliases,
                rep: w.rep,
                constraints: w.constraints,
                parent_feature: w.parent_feature,
                build_overrides: w.build_overrides,
                deciders: w.deciders,
                ffi_rationale: w.ffi_rationale,
                fmea_ref: w.fmea_ref,
                consequence: w.consequence,
                freq_exposure: w.freq_exposure,
                avoidance: w.avoidance,
                demand_rate: w.demand_rate,
                ftti: w.ftti,
                pl_level: w.pl_level,
                assets: w.assets,
                risk_treatment: w.risk_treatment,
                residual_risk: w.residual_risk,
                threat_scenarios: w.threat_scenarios,
                control_type: w.control_type,
                implements_goals: w.implements_goals,
                cvss_score: w.cvss_score,
                affected_elements: w.affected_elements,
                mitigated_by: w.mitigated_by,
                derived_from_cybersecurity_goal: w.derived_from_cybersecurity_goal,
                cybersecurity_properties: w.cybersecurity_properties,
                tier3: Boxed::of(ColdFrontmatter3 {
                    is_reference: w.is_reference,
                    is_derived: w.is_derived,
                    is_constant: w.is_constant,
                    is_readonly: w.is_readonly,
                    is_portion: w.is_portion,
                    is_ordered: w.is_ordered,
                    is_nonunique: w.is_nonunique,
                    is_end: w.is_end,
                    is_individual: w.is_individual,
                    value: w.value,
                    value_kind: w.value_kind,
                    text: w.text,
                    assume: w.assume,
                    verdict_type: w.verdict_type,
                    approver: w.approver,
                    git_tag: w.git_tag,
                    git_commit: w.git_commit,
                    frozen_scope: w.frozen_scope,
                    seal: w.seal,
                    supersedes: w.supersedes,
                    is_composite: w.is_composite,
                    portion_kind: w.portion_kind,
                    is_conjugated: w.is_conjugated,
                    return_type: w.return_type,
                    entry_action: w.entry_action,
                    do_action: w.do_action,
                    exit_action: w.exit_action,
                    is_negated: w.is_negated,
                    framed_concerns: w.framed_concerns,
                    result_type: w.result_type,
                    verdict_expression: w.verdict_expression,
                    includes: w.includes,
                    allocations: w.allocations,
                    satisfied_by: w.satisfied_by,
                    rendering: w.rendering,
                    filter_condition: w.filter_condition,
                    time_slices: w.time_slices,
                    snapshots: w.snapshots,
                    variant_of: w.variant_of,
                    cardinality: w.cardinality,
                    contributes_to: w.contributes_to,
                    build_exports: w.build_exports,
                    baseline_ref: w.baseline_ref,
                    damage_table: w.damage_table,
                    threat_table: w.threat_table,
                    goal_table: w.goal_table,
                    control_table: w.control_table,
                    mission_time: w.mission_time,
                    probability: w.probability,
                    latent_diagnostic_coverage: w.latent_diagnostic_coverage,
                    ccf_group: w.ccf_group,
                    ccf_beta: w.ccf_beta,
                    recommended_action: w.recommended_action,
                    fta_ref: w.fta_ref,
                    unknown_fmea_keys: w.unknown_fmea_keys,
                    cve_id: w.cve_id,
                    asset_owner: w.asset_owner,
                    related_safety_goal: w.related_safety_goal,
                    asset_table: w.asset_table,
                    cvss_vector: w.cvss_vector,
                    cvss_severity: w.cvss_severity,
                    fixed_in: w.fixed_in,
                }),
            }),
        }
    }
}

impl<'de> Deserialize<'de> for ColdFrontmatter {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Ok(ColdWire::deserialize(d)?.into_tiers())
    }
}

impl<T: Default + PartialEq> Boxed<T> {
    fn of(v: T) -> Self {
        Boxed(if v == T::default() { None } else { Some(Box::new(v)) })
    }
}

/// `Option<Box<T>>` that is `None` whenever every field of `T` is unset, so a
/// flattened tier costs one pointer when unused.
#[derive(Debug, Clone, PartialEq)]
pub struct Boxed<T>(Option<Box<T>>);

impl<T> Default for Boxed<T> {
    fn default() -> Self {
        Boxed(None)
    }
}

impl<'de, T: Deserialize<'de> + Default + PartialEq> Deserialize<'de> for Boxed<T> {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let c = T::deserialize(d)?;
        Ok(Boxed(if c == T::default() { None } else { Some(Box::new(c)) }))
    }
}

impl<T: Serialize> Serialize for Boxed<T> {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match &self.0 {
            Some(c) => c.serialize(s),
            None => {
                use serde::ser::SerializeMap;
                s.serialize_map(Some(0))?.end()
            }
        }
    }
}

/// `Deref`/`DerefMut` from a holder to the next tier it boxes: reads of an unset
/// tier see a shared empty one, a write allocates it.
macro_rules! tier_deref {
    ($holder:ty, $field:ident, $target:ty) => {
        impl std::ops::Deref for $holder {
            type Target = $target;
            fn deref(&self) -> &$target {
                static EMPTY: std::sync::OnceLock<$target> = std::sync::OnceLock::new();
                self.$field.0.as_deref().unwrap_or_else(|| EMPTY.get_or_init(Default::default))
            }
        }
        impl std::ops::DerefMut for $holder {
            fn deref_mut(&mut self) -> &mut $target {
                self.$field.0.get_or_insert_with(Default::default)
            }
        }
    };
}
tier_deref!(RawFrontmatter, cold, ColdFrontmatter);
tier_deref!(ColdFrontmatter, tier2, ColdFrontmatter2);
tier_deref!(ColdFrontmatter2, tier3, ColdFrontmatter3);

impl RawFrontmatter {
    /// A requirement of a kind no architecture element can satisfy (`process`,
    /// `regulatory`, `deliverable`, GH #250): exempt from `W300`/`W302` and from the
    /// unsatisfied lists of `audit`, `stats` and the traceability diagram.
    pub fn is_non_allocatable_requirement(&self) -> bool {
        matches!(self.requirement_kind.as_deref(), Some("process" | "regulatory" | "deliverable"))
    }
}

impl RawFrontmatter {
    /// Whether any rarely-set tier is allocated (memory diagnostics and tests).
    pub fn has_cold_block(&self) -> bool {
        self.cold.0.is_some()
    }

    /// Drop every tier a mutable access left allocated but empty, so an element
    /// that uses none of those fields stays one pointer wide.
    pub fn shrink(&mut self) {
        fn trim<T: Default + PartialEq>(b: &mut Boxed<T>) {
            if b.0.as_deref().is_some_and(|c| *c == T::default()) {
                b.0 = None;
            }
        }
        if let Some(c1) = self.cold.0.as_deref_mut() {
            if let Some(c2) = c1.tier2.0.as_deref_mut() {
                trim(&mut c2.tier3);
            }
            trim(&mut c1.tier2);
        }
        trim(&mut self.cold);
    }
}


/// The effective (inherited + own) selection of a `Configuration` with a
/// `derivedFrom:` base (§9.8). See `crate::config_inherit`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct InheritedConfiguration {
    /// Effective feature selection, keyed by FeatureDef qualified name.
    pub features: std::collections::BTreeMap<String, bool>,
    /// Effective `parameterBindings:` mapping (own entries + inherited ones).
    pub parameter_bindings: Option<serde_yaml::Value>,
    /// Abstract features whose value in `features` was derived from the concrete selection
    /// (an abstract feature is not a choice, `E238`) rather than authored or inherited.
    pub derived: Vec<String>,
}

impl RawFrontmatter {
    /// The **effective** feature selection of a `Configuration` (§9.8): its
    /// `features:` map of `FeatureDef qualified name -> bool`, including the
    /// selections it inherits through `derivedFrom:` (own entries override
    /// inherited ones). Returns an empty map for elements that are not
    /// configurations or that declare (and inherit) no selections. Use
    /// [`RawFrontmatter::declared_feature_selections`] for the authored map only.
    pub fn feature_selections(&self) -> std::collections::BTreeMap<String, bool> {
        if let Some(inh) = &self.inherited {
            return inh.features.clone();
        }
        self.declared_feature_selections()
    }

    /// The **effective** `parameterBindings:` of a `Configuration` — own entries
    /// plus those inherited through `derivedFrom:` (§9.8). Equals the authored
    /// field when the configuration inherits nothing.
    pub fn effective_parameter_bindings(&self) -> Option<&serde_yaml::Value> {
        match &self.inherited {
            Some(inh) => inh.parameter_bindings.as_ref(),
            None => self.parameter_bindings.as_ref(),
        }
    }

    /// The `features:` selection map exactly as authored in this file (no
    /// inheritance).
    ///
    /// The `features:` key is stored as a one-element vector wrapping the YAML
    /// mapping (see `features_de`); this unwraps it.
    pub fn declared_feature_selections(&self) -> std::collections::BTreeMap<String, bool> {
        let mut out = std::collections::BTreeMap::new();
        if let Some(list) = &self.features {
            if let Some(serde_yaml::Value::Mapping(m)) = list.first() {
                for (k, v) in m {
                    if let (Some(k), Some(b)) = (k.as_str(), v.as_bool()) {
                        out.insert(k.to_string(), b);
                    }
                }
            }
        }
        out
    }

    /// §3.16 (REQ-TRS-ORDER-001) — sort key for display / reading order: the
    /// `displayOrder` value, or `+∞` when unset so unordered elements sink below
    /// every element that declares an order. Callers combine this with the stable
    /// identifier as a tie-break, e.g.
    /// `a.display_order_key().total_cmp(&b.display_order_key()).then_with(|| id_a.cmp(id_b))`.
    pub fn display_order_key(&self) -> f64 {
        self.display_order.unwrap_or(f64::INFINITY)
    }

    /// REQ-TRS-MG-* — read a MagicGrid overlay value (`mg_*`) from `custom_fields`
    /// as a string, coercing YAML scalars (string/number/bool) sensibly. Returns
    /// `None` if the key is absent or the value is not a representable scalar.
    pub fn mg_str(&self, key: &str) -> Option<String> {
        match self.custom_fields.get(key)? {
            serde_yaml::Value::String(s) => Some(s.clone()),
            serde_yaml::Value::Bool(b) => Some(b.to_string()),
            serde_yaml::Value::Number(n) => Some(n.to_string()),
            _ => None,
        }
    }

    /// REQ-TRS-MG-* — read a MagicGrid overlay value (`mg_*`) from `custom_fields`
    /// as a bool, coercing a YAML bool, or the strings `"true"`/`"false"`.
    pub fn mg_bool(&self, key: &str) -> Option<bool> {
        match self.custom_fields.get(key)? {
            serde_yaml::Value::Bool(b) => Some(*b),
            serde_yaml::Value::String(s) => match s.trim().to_ascii_lowercase().as_str() {
                "true" => Some(true),
                "false" => Some(false),
                _ => None,
            },
            _ => None,
        }
    }

    /// REQ-TRS-MG-* — read a MagicGrid overlay value (`mg_*`) from `custom_fields`
    /// as an `f64`, coercing a YAML number or a numeric string.
    pub fn mg_f64(&self, key: &str) -> Option<f64> {
        match self.custom_fields.get(key)? {
            serde_yaml::Value::Number(n) => n.as_f64(),
            serde_yaml::Value::String(s) => s.trim().parse::<f64>().ok(),
            _ => None,
        }
    }
}

/// A parse-time error recorded on a `RawElement` when frontmatter could not be
/// read.  Carried on the element so the validator can emit the right code (E001
/// or E002) rather than the generic W008 "no type field" warning.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseIssue {
    /// File does not begin with `---` (E001)
    NoFrontmatter,
    /// YAML between delimiters is not valid YAML 1.2 (E002)
    YamlError(String),
}

/// A parsed model element: qualified name + frontmatter + doc body.
#[derive(Debug, Clone, Serialize)]
pub struct RawElement {
    pub qualified_name: String,
    pub file_path: String,
    pub frontmatter: RawFrontmatter,
    pub doc: String,
    /// Set when the file had no `---` opener (E001) or unparseable YAML (E002).
    #[serde(skip)]
    pub parse_issue: Option<ParseIssue>,
    /// Computed fields from `derive:` blocks (REQ-TRS-DERIVE-001).
    /// Populated by `derive::derive_pass` after walking; visible to validator and query.
    #[serde(skip_serializing_if = "std::collections::HashMap::is_empty", default)]
    pub derived: std::collections::HashMap<String, serde_yaml::Value>,
    /// Findings gathered by the validator, contributed by more than one
    /// walker post-processing pass sharing this one vector: the derive pass
    /// (E504-E506; `crate::derive`) and native SysMLv2 submodel ingestion
    /// (W540; `crate::sysmlv2`), which runs earlier in `walker::walk_model`.
    /// Despite the field's name, it is not exclusively "derive pass" output.
    #[serde(skip)]
    pub derive_findings: Vec<(String, String, String)>, // (code, file, message)
    /// §3.10 locale documentation variants (REQ-TRS-PARSE-010, GH #160):
    /// `locale → doc body` contributed by variant files (`locale:` +
    /// `qualifiedName:` naming this element). Filled by
    /// `walker::attach_locale_variants`; the variant files themselves never
    /// become elements. The element's own body stays in `doc`.
    #[serde(skip_serializing_if = "std::collections::BTreeMap::is_empty", default)]
    pub locale_docs: std::collections::BTreeMap<String, String>,
    /// §3.10 `about:` comments (REQ-TRS-PARSE-011, GH #164): cross-element
    /// comments whose `about:` list names this element, in walk order. Filled
    /// by `walker::attach_about_comments`; the comment files themselves never
    /// become elements.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub about_notes: Vec<AboutNote>,
}

/// One §3.10 `about:` comment attached to an element (REQ-TRS-PARSE-011).
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct AboutNote {
    /// The comment's `name:` (its file stem when absent).
    pub name: String,
    /// Path of the comment file.
    pub file: String,
    /// The comment's `locale:`, when set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
    /// The comment's Markdown body.
    pub body: String,
}

#[cfg(test)]
mod element_type_all_tests {
    use super::ElementType;
    use std::collections::HashSet;

    #[test]
    fn all_names_round_trip_through_serde() {
        for t in ElementType::ALL {
            let parsed: ElementType = serde_yaml::from_str(t.name()).expect("parses");
            assert_eq!(&parsed, t, "{} does not round-trip", t.name());
        }
    }

    #[test]
    fn all_is_unique_and_excludes_unknown() {
        let names: HashSet<&str> = ElementType::ALL.iter().map(|t| t.name()).collect();
        assert_eq!(names.len(), ElementType::ALL.len(), "duplicate entry in ElementType::ALL");
        assert!(!ElementType::ALL.contains(&ElementType::Unknown));
    }

    #[test]
    fn all_lists_every_declared_variant() {
        // Pin ALL to the enum declaration itself so a new variant cannot be
        // forgotten here (GH #135).
        let src = include_str!("element.rs");
        let start = src.find("pub enum ElementType {").expect("enum");
        let body = &src[start..];
        let body = &body[..body.find("\n}\n").expect("enum end")];
        let declared: Vec<&str> = body
            .lines()
            .filter_map(|l| {
                let l = l.trim_start();
                let ident: String = l.chars().take_while(|c| c.is_ascii_alphanumeric()).collect();
                (!ident.is_empty()
                    && ident.chars().next().is_some_and(|c| c.is_ascii_uppercase())
                    && l[ident.len()..].starts_with(','))
                .then_some(&l[..ident.len()])
            })
            .filter(|n| *n != "Unknown")
            .collect();
        let listed: Vec<&str> = ElementType::ALL.iter().map(|t| t.name()).collect();
        assert_eq!(declared, listed, "ElementType::ALL is out of sync with the enum");
    }
}
