use serde::{Deserialize, Serialize};

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
    ADR,           // Architecture Decision Record (§8.17)
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
    pub allocated_from: Option<String>,
    pub allocated_to: Option<String>,
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
    pub title: Option<String>,
    pub status: Option<String>,
    pub sil_level: Option<u8>,
    pub asil_level: Option<String>,
    pub wcet: Option<String>,
    pub tags: Option<Vec<String>>,
    // Native TestCase fields (§8.12.5)
    pub test_level: Option<String>,
    pub source_file: Option<String>,
    pub test_functions: Option<Vec<serde_yaml::Value>>,

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

    // §9.8 — Configuration
    pub feature_model: Option<String>,
    pub parameter_bindings: Option<serde_yaml::Value>,
    pub baseline_ref: Option<String>,

    // §9.10 — PLE conditioning (any element)
    pub applies_when: Option<serde_yaml::Value>,

    // §8.11.6 — native Requirement traceability (§12)
    pub req_domain: Option<String>,
    pub breakdown_adr: Option<String>,

    // §3.14 — domain classification
    pub domain: Option<String>,
    pub is_deployment_package: Option<bool>,

    // Catch-all for unknown fields
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_yaml::Value>,
}

/// A parsed model element: qualified name + frontmatter + doc body.
#[derive(Debug, Clone, Serialize)]
pub struct RawElement {
    pub qualified_name: String,
    pub file_path: String,
    pub frontmatter: RawFrontmatter,
    pub doc: String,
}
