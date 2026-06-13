use serde::{Deserialize, Serialize};

/// Serde helper: accept either a plain YAML string or a sequence of strings.
/// Allows `allocatedFrom: SC-001` and `allocatedFrom: [SC-001, SC-002]` both to
/// deserialize into `Option<Vec<String>>`.
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
/// resolvable `def` reference are skipped.
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
                        .filter(|(k, _)| k != "type" && k != "apply" && k != "def")
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
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
    pub supertype: Option<serde_yaml::Value>,
    pub typed_by: Option<serde_yaml::Value>,
    pub subsets: Option<Vec<String>>,
    pub redefines: Option<serde_yaml::Value>,
    pub conjugates: Option<String>,
    pub multiplicity: Option<String>,
    pub is_abstract: Option<bool>,
    pub is_variation: Option<bool>,
    pub is_reference: Option<bool>,
    pub is_derived: Option<bool>,
    pub is_constant: Option<bool>,
    pub is_readonly: Option<bool>,
    pub is_portion: Option<bool>,
    pub is_ordered: Option<bool>,
    pub is_nonunique: Option<bool>,
    pub is_end: Option<bool>,
    pub is_individual: Option<bool>,
    pub direction: Option<String>,
    pub value: Option<serde_yaml::Value>,
    pub value_kind: Option<String>,
    pub expression: Option<String>,
    #[serde(default, deserialize_with = "features_de::deserialize")]
    pub features: Option<Vec<serde_yaml::Value>>,
    pub metadata: Option<Vec<serde_yaml::Value>>,
    pub connections: Option<Vec<serde_yaml::Value>>,
    pub flow_connections: Option<Vec<serde_yaml::Value>>,
    pub binding_connections: Option<Vec<serde_yaml::Value>>,
    pub succession_connections: Option<Vec<serde_yaml::Value>>,
    pub sub_states: Option<Vec<serde_yaml::Value>>,
    pub transitions: Option<Vec<serde_yaml::Value>>,
    pub exhibits_states: Option<Vec<String>>,
    pub performs: Option<Vec<serde_yaml::Value>>,
    pub operations: Option<Vec<serde_yaml::Value>>,
    pub actors: Option<Vec<String>>,
    pub steps: Option<Vec<String>>,
    pub text: Option<String>,
    pub verifies: Option<Vec<String>>,
    pub objective: Option<String>,
    #[serde(default, deserialize_with = "string_or_vec::deserialize")]
    pub allocated_from: Option<Vec<String>>,
    #[serde(default, deserialize_with = "string_or_vec::deserialize")]
    pub allocated_to: Option<Vec<String>>,
    pub expose: Option<Vec<serde_yaml::Value>>,
    pub viewpoint: Option<String>,
    pub stakeholders: Option<Vec<String>>,
    pub concerns: Option<Vec<String>>,
    pub methods: Option<Vec<String>>,
    pub diagram_kind: Option<String>,
    pub svg_mode: Option<String>,
    pub svg_file: Option<String>,
    pub subject: Option<String>,
    pub shapes: Option<serde_yaml::Value>,
    pub edges: Option<serde_yaml::Value>,
    pub layout: Option<serde_yaml::Value>,
    pub imports: Option<Vec<serde_yaml::Value>>,
    pub depends_on: Option<Vec<String>>,
    // derivedFrom is a list for both native Requirements and SysML RequirementDef
    pub derived_from: Option<Vec<String>>,
    /// REQ-TRS-MG-001 — MagicGrid `«refine»`: a `UseCaseDef`/`UseCase` gives concrete
    /// behavioural meaning to a requirement. Optional list of cross-references
    /// (qname or stable `REQ-*` id), resolved like `verifies:`/`derivedFrom:`.
    pub refines: Option<Vec<String>>,
    pub assume: Option<Vec<serde_yaml::Value>>,
    pub requires: Option<Vec<serde_yaml::Value>>,
    pub about: Option<serde_yaml::Value>,
    pub locale: Option<String>,
    pub verdict_type: Option<String>,
    pub extends: Option<Vec<serde_yaml::Value>>,
    pub extension_points: Option<Vec<serde_yaml::Value>>,
    pub clients: Option<Vec<String>>,
    pub suppliers: Option<Vec<String>>,
    // Native Requirement fields (§8.11.6)
    pub id: Option<String>,
    /// **Deprecated / removed as a label** (REQ-TRS-NAME-002). Every element now labels
    /// via `name`; `title` is no longer a recognized label field. It is still parsed
    /// here only so the validator can detect a stray `title:` and reject it via `E025`.
    pub title: Option<String>,
    pub status: Option<String>,
    pub sil_level: Option<u8>,
    pub asil_level: Option<String>,
    /// ASIL/SIL decomposition argument type (§22.3): `independent` | `redundant` | `diverse`.
    /// Informational; surfaced in the safety-case report.
    pub decomposition_kind: Option<String>,
    pub dal_level: Option<String>,
    pub wcet: Option<String>,
    pub tags: Option<Vec<String>>,
    pub verification_method: Option<String>,
    pub requirement_kind: Option<String>,
    // Native TestCase fields (§8.12.5)
    pub test_level: Option<String>,
    pub coverage_target: Option<String>,
    pub source_file: Option<String>,
    pub test_functions: Option<Vec<serde_yaml::Value>>,

    // Native TestPlan fields (GH #38; REQ-TRS-PLAN-001..004)
    /// `scope:` — free-form, recommended vocab unit|smoke|integration|hil|
    /// certification|security|regression (else W610).
    pub scope: Option<String>,
    /// `configurations:` — scalar or list of `Configuration` references. Absent
    /// → config-agnostic (applies to every Configuration). Each must resolve to a
    /// `Configuration` (else E606).
    #[serde(default, deserialize_with = "string_or_vec::deserialize")]
    pub configurations: Option<Vec<String>>,
    /// `demonstrates:` — scalar or list of Requirement/SafetyGoal/
    /// CybersecurityGoal/Argument the plan is offered as evidence for (else E603).
    #[serde(default, deserialize_with = "string_or_vec::deserialize")]
    pub demonstrates: Option<Vec<String>>,
    /// `testCases:` — scalar or list of explicit `TestCase` members (else E601).
    #[serde(rename = "testCases", default, deserialize_with = "string_or_vec::deserialize")]
    pub test_cases: Option<Vec<String>>,
    /// `selection:` — additive membership query (REQ-TRS-PLAN-003).
    pub selection: Option<TestPlanSelection>,
    /// §12.8 — implementation trace: architecture element → source artifact(s).
    #[serde(default, deserialize_with = "string_or_vec::deserialize")]
    pub implemented_by: Option<Vec<String>>,

    /// §3 — external reference(s): this element represents an artifact managed in
    /// another tool (a DNG requirement, a SysML-tool element, …). Opaque strings
    /// (URI or tool-qualified token); string or list. Never a model cross-ref target.
    #[serde(default, deserialize_with = "string_or_vec::deserialize")]
    pub ext_ref: Option<Vec<String>>,

    // §3.1 — identity override
    pub qualified_name: Option<String>,

    // §3.2 — classification flags
    pub is_variant: Option<bool>,
    pub is_composite: Option<bool>,
    pub portion_kind: Option<String>,

    // §8.3.2 — Port usage
    pub is_conjugated: Option<bool>,

    // §8.4.x — connection/binding elements
    pub ends: Option<Vec<serde_yaml::Value>>,

    // §8.5.2 — EnumerationDef
    pub values: Option<Vec<serde_yaml::Value>>,

    // §8.6.1 — FlowDef
    pub item_type: Option<String>,

    // §8.7.1 + §8.9.1 — Action/Calculation body
    pub body: Option<String>,
    pub body_language: Option<String>,
    /// `CalculationDef` (§22.2): qualified name of a `ConstraintDef` bounding the budget result.
    pub evaluate: Option<String>,
    // Native ReviewRecord fields (§19, GH #71). `recordedAt` is the thin pointer to the
    // external review (e.g. a GitHub PR/review URL); the model keeps the baselined anchor.
    pub review_type: Option<String>,
    pub review_date: Option<String>,
    #[serde(default, deserialize_with = "string_or_vec::deserialize")]
    pub reviewed_by: Option<Vec<String>>,
    #[serde(default, deserialize_with = "string_or_vec::deserialize")]
    pub reviews: Option<Vec<String>>,
    pub items: Option<Vec<serde_yaml::Value>>,
    pub recorded_at: Option<String>,
    // Native TradeStudy fields (§15, GH #63). `objective` (Requirement) is shared above.
    pub criteria: Option<Vec<serde_yaml::Value>>,
    pub alternatives: Option<Vec<serde_yaml::Value>>,
    pub scores: Option<Vec<serde_yaml::Value>>,
    pub decision: Option<String>,
    // Native IEC 62443 Zone/Conduit fields (§13, GH #61).
    #[serde(rename = "targetSL")]
    pub target_sl: Option<u8>,
    #[serde(rename = "achievedSL")]
    pub achieved_sl: Option<u8>,
    #[serde(default, deserialize_with = "string_or_vec::deserialize")]
    pub members: Option<Vec<String>>,
    pub rationale: Option<String>,
    pub from_zone: Option<String>,
    pub to_zone: Option<String>,
    #[serde(default, deserialize_with = "string_or_vec::deserialize")]
    pub protocols: Option<Vec<String>>,
    pub in_zone: Option<String>,
    /// §14.3 — `repoImports:` on a Package `_index.md`: a list of mappings
    /// `{repo, qname, as}` mounting a sub-tree from a peer repo declared in
    /// `[repos]`. Untyped here; the validator reads the `repo`/`qname`/`as` keys.
    pub repo_imports: Option<Vec<serde_yaml::Value>>,
    pub sub_actions: Option<Vec<serde_yaml::Value>>,
    pub control_nodes: Option<Vec<serde_yaml::Value>>,
    pub return_type: Option<String>,

    // §8.8.1 — StateDef entry/do/exit
    pub entry_action: Option<serde_yaml::Value>,
    pub do_action: Option<serde_yaml::Value>,
    pub exit_action: Option<serde_yaml::Value>,
    pub is_parallel: Option<bool>,

    // §8.10.2 — Constraint usage
    pub is_asserted: Option<bool>,
    pub is_negated: Option<bool>,

    // §8.11.1 — RequirementDef
    pub framed_concerns: Option<Vec<String>>,

    // §8.11.4 — satisfaction/verification
    pub satisfies: Option<Vec<String>>,

    // §8.12.1 — Case elements
    pub objectives: Option<Vec<serde_yaml::Value>>,
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

    // §8.15.1 — MetadataDef
    pub annotates: Option<Vec<String>>,
    pub is_semantic: Option<bool>,

    // §3.7 — package
    pub filter_condition: Option<String>,
    pub aliases: Option<Vec<serde_yaml::Value>>,

    // §3.12 — representation
    pub rep: Option<String>,

    // §3.3 — InterfaceDef constraints
    pub constraints: Option<Vec<serde_yaml::Value>>,

    // §8.2.4 — OccurrenceDef
    pub time_slices: Option<Vec<serde_yaml::Value>>,
    pub snapshots: Option<Vec<serde_yaml::Value>>,

    // §9.4 — variant reference
    pub variant_of: Option<String>,

    // §9.6 — FeatureDef
    pub group_kind: Option<String>,
    pub cardinality: Option<String>,
    pub parent_feature: Option<String>,
    pub excludes: Option<Vec<String>>,
    pub contributes_to: Option<String>,
    /// Membership flag (REQ-TRS-FM-004): when `true`, the feature is mandatory
    /// (forced on with its parent, or root-selected when top-level) independently
    /// of `groupKind`. Legacy `groupKind: mandatory` remains a shorthand.
    pub mandatory: Option<bool>,

    // §9.7 — FeatureDef parameters (also used by ActionDef/CalculationDef as a
    // generic typed-parameter list; only FeatureDef parameters are validated).
    pub parameters: Option<Vec<serde_yaml::Value>>,

    // §9.8 — Configuration
    pub feature_model: Option<String>,
    pub parameter_bindings: Option<serde_yaml::Value>,
    pub baseline_ref: Option<String>,

    // §9.10 — PLE conditioning (any element)
    pub applies_when: Option<serde_yaml::Value>,

    // §8.11.6 — native Requirement traceability (§12)
    pub req_domain: Option<String>,
    pub breakdown_adr: Option<String>,

    /// REQ-TRS-SAFE-006 (ISO 26262-9 §7) — freedom-from-interference / partitioning
    /// rationale (YAML: `ffiRationale`). A non-empty string on a shared allocation
    /// target or on a source excuses a mixed-criticality sharing (clears W034).
    pub ffi_rationale: Option<String>,

    // §3.14 — domain classification
    pub domain: Option<String>,
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

    // §T4-TARA — TARASheet section tables (ISO/SAE 21434)
    // Each is a list of row-mappings exploded by the walker into Tier-2 elements.
    pub damage_table: Option<Vec<serde_yaml::Value>>,   // → DamageScenario rows  (YAML: damageTable)
    pub threat_table: Option<Vec<serde_yaml::Value>>,   // → ThreatScenario rows   (YAML: threatTable)
    pub goal_table: Option<Vec<serde_yaml::Value>>,     // → CybersecurityGoal rows (YAML: goalTable)
    pub control_table: Option<Vec<serde_yaml::Value>>,  // → SecurityControl rows  (YAML: controlTable)

    // §T4 — FaultTree (IEC 61025 / ISO 26262-9)
    pub top_event: Option<String>,              // SafetyGoal ref (YAML: topEvent)
    pub mission_time: Option<String>,           // e.g. "1e9 h" (YAML: missionTime)
    pub gate_type: Option<String>,              // FaultTreeGate: AND|OR|XOR|NOT|inhibit (YAML: gateType)
    pub inputs: Option<Vec<String>>,            // FaultTreeGate input refs (YAML: inputs)
    pub event_kind: Option<String>,             // FaultTreeEvent: basic|undeveloped|house (YAML: eventKind)
    pub failure_rate: Option<f64>,              // FaultTreeEvent failure rate /h (YAML: failureRate)
    pub probability: Option<f64>,               // cut-set or top-event probability (YAML: probability)
    pub fmea_ref: Option<String>,               // FaultTreeEvent → reconciling FMEAEntry (YAML: fmeaRef)

    // §T4 — AttackTree (ISO/SAE 21434 §15.7 attack path analysis)
    pub threat_ref: Option<String>,             // AttackTree → ThreatScenario ref (YAML: threatRef)
    // §T4 — FMEDA diagnostic coverage (ISO 26262-5 §8-9), documented for FaultTreeEvent.
    pub diagnostic_coverage: Option<f64>,         // DC, 0.0–1.0 (YAML: diagnosticCoverage)
    pub latent_diagnostic_coverage: Option<f64>,  // DCl, 0.0–1.0 (YAML: latentDiagnosticCoverage)

    // §T4 — FMEASheet / FMEAEntry (IEC 60812 / SAE J1739)
    pub entries: Option<Vec<serde_yaml::Value>>, // FMEASheet sub-entries (YAML: entries)
    pub failure_mode: Option<String>,            // FMEAEntry: what fails (YAML: failureMode)
    pub effect: Option<String>,                  // FMEAEntry: consequence (YAML: effect)
    pub cause: Option<String>,                   // FMEAEntry: root cause (YAML: cause)
    pub fmea_severity: Option<u8>,               // FMEAEntry severity 1–10 (YAML: fmeaSeverity)
    pub occurrence: Option<u8>,                  // FMEAEntry occurrence 1–10 (YAML: occurrence)
    pub detection: Option<u8>,                   // FMEAEntry detection 1–10 (YAML: detection)
    pub rpn: Option<u32>,                        // FMEAEntry Risk Priority Number (YAML: rpn)
    pub recommended_action: Option<String>,      // FMEAEntry mitigation (YAML: recommendedAction)
    pub fta_ref: Option<String>,                 // FMEAEntry → reconciling FaultTreeEvent (YAML: ftaRef)
    #[serde(skip)]
    pub unknown_fmea_keys: Vec<String>,          // keys not in recognised set; validator emits E922

    // §T2 — HazardousEvent (ISO 26262 §7 HARA)
    pub severity: Option<String>,               // S0-S3
    pub exposure: Option<String>,               // E0-E4
    pub controllability: Option<String>,        // C0-C3
    pub operational_situation: Option<String>,  // free-text operating scenario
    // IEC 61508 §3 risk graph parameters (alternative to ISO 26262 S/E/C)
    pub consequence: Option<String>,            // Ca | Cb | Cc | Cd
    pub freq_exposure: Option<String>,          // Fa | Fb  (YAML: freqExposure)
    pub avoidance: Option<String>,              // Pa | Pb
    pub demand_rate: Option<String>,            // W1 | W2 | W3  (YAML: demandRate)

    // §T2 — SafetyGoal (ISO 26262 §7 / IEC 61508 / ISO 13849)
    pub safe_state: Option<String>,             // description of the safe state
    pub ftti: Option<String>,                   // Fault Tolerant Time Interval (e.g. "20ms")
    pub hazardous_events: Option<Vec<String>>,  // HazardousEvent id/qname refs
    pub pl_level: Option<String>,               // ISO 13849-1 Performance Level: a|b|c|d|e (YAML: plLevel)

    // §T2 — DamageScenario (ISO/SAE 21434 §15)
    pub damage_severity: Option<String>,        // severe|major|moderate|negligible
    pub impact_categories: Option<Vec<String>>, // safety|financial|operational|privacy
    // DamageScenario.assets: references to Asset elements (REQ-TRS-TYPE-017, YAML: assets)
    #[serde(default, deserialize_with = "string_or_vec::deserialize")]
    pub assets: Option<Vec<String>>,

    /// §T4 safety↔security co-engineering (ISO 26262 ⇄ ISO/SAE 21434) — cross-link
    /// from a `DamageScenario`/`ThreatScenario` to the `HazardousEvent`/`SafetyGoal`
    /// it endangers. String or list. Resolved via `Resolver::resolve_ref`; target
    /// must be a `HazardousEvent` or `SafetyGoal` (else E844). (YAML: hazardRef)
    #[serde(default, deserialize_with = "string_or_vec::deserialize")]
    pub hazard_ref: Option<Vec<String>>,

    // §T2 — ThreatScenario (ISO/SAE 21434 §15)
    pub attack_feasibility: Option<String>,     // high|medium|low|very_low
    pub attack_vector: Option<String>,          // network|adjacent|local|physical
    pub damage_scenarios: Option<Vec<String>>,  // DamageScenario id/qname refs
    /// §T2 risk treatment decision (ISO/SAE 21434 §9 / §15.9): avoid|reduce|share|retain.
    /// Invalid value → E845. (YAML: riskTreatment)
    pub risk_treatment: Option<String>,
    /// §T2 free-text residual-risk note after treatment (no validation). (YAML: residualRisk)
    pub residual_risk: Option<String>,

    // §T2 — CybersecurityGoal (ISO/SAE 21434 §15)
    pub cal_level: Option<String>,              // CAL1-CAL4
    pub security_property: Option<String>,      // confidentiality|integrity|availability|authenticity
    pub threat_scenarios: Option<Vec<String>>,  // ThreatScenario id/qname refs

    // §T2 — SecurityControl (ISO/SAE 21434)
    pub control_type: Option<String>,           // prevention|detection|response|recovery
    pub implements_goals: Option<Vec<String>>,  // CybersecurityGoal id/qname refs

    // §T2 — VulnerabilityReport
    pub cvss_score: Option<f64>,                // 0.0-10.0
    pub cve_id: Option<String>,                 // CVE-YYYY-NNNNN
    pub affected_elements: Option<Vec<String>>, // qualified names of affected model elements
    pub mitigated_by: Option<Vec<String>>,      // SecurityControl id/qname refs

    // §T2 — upstream goal links for native Requirement
    // YAML: derivedFromCybersecurityGoal; alias: derivedFromSecurityGoal (legacy)
    #[serde(alias = "derivedFromSecurityGoal")]
    pub derived_from_cybersecurity_goal: Option<String>,
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
    /// `Argument.evidence` (YAML: evidence) — refs to supporting Requirement /
    /// TestCase / sub-Argument / AssumptionOfUse (the GSN children). String or list;
    /// each ref resolves via the Resolver (else E855).
    #[serde(default, deserialize_with = "string_or_vec::deserialize")]
    pub evidence: Option<Vec<String>>,
    /// `AssumptionOfUse.appliesTo` (YAML: appliesTo) — the SafetyGoal / Argument /
    /// Requirement this SRAC constrains. String or list; each ref resolves via the
    /// Resolver (else E858).
    #[serde(default, deserialize_with = "string_or_vec::deserialize")]
    pub applies_to: Option<Vec<String>>,

    // §T2 — TestCase security test method (REQ-TRS-SEC-008; ISO/SAE 21434 §13.3)
    // Valid: fuzz|penetration_test|security_regression|vulnerability_scan|threat_modeling
    // Invalid → W809. (YAML: securityTestMethod)
    pub security_test_method: Option<String>,

    // §T2 — Asset (REQ-TRS-TYPE-017; ISO/SAE 21434 §15.3 asset identification)
    // cybersecurityProperties: list of confidentiality|integrity|availability|authenticity (YAML: cybersecurityProperties)
    pub cybersecurity_properties: Option<Vec<String>>,
    pub asset_owner: Option<String>,          // qname/id of owning architecture element (YAML: assetOwner)
    pub related_safety_goal: Option<String>,  // SG-* ref for co-engineering (YAML: relatedSafetyGoal)

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

    // Catch-all for unknown fields
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_yaml::Value>,
}

impl RawFrontmatter {
    /// Feature selections declared on a `Configuration` (§9.8): the `features:`
    /// map of `FeatureDef qualified name -> bool`. Returns an empty map for
    /// elements that are not configurations or that declare no selections.
    ///
    /// The `features:` key is stored as a one-element vector wrapping the YAML
    /// mapping (see `features_de`); this unwraps it.
    pub fn feature_selections(&self) -> std::collections::BTreeMap<String, bool> {
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
    /// Findings produced by the derive pass (E501, E502). Gathered by the validator.
    #[serde(skip)]
    pub derive_findings: Vec<(String, String, String)>, // (code, file, message)
}
