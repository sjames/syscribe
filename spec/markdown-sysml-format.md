# Markdown-SysML Format Specification

**Version:** 1.0  
**Status:** Draft  
**Reference:** OMG SysML v2.0 (formal/2026-03-02)

---

> **Disclaimer:** Syscribe is a modeling and documentation tool provided **"as-is" without warranty of any kind**. The authors and contributors accept no responsibility or liability for the use of this tool in safety-critical, life-critical, or mission-critical applications.
>
> Output from Syscribe — including validation results, generated templates, traceability reports, and analysis artifacts — **must be independently reviewed and verified by qualified engineers** before use in any certification, regulatory submission, or safety case. Compliance with standards such as ISO 26262, IEC 61508, ISO 13849-1, ISO/SAE 21434, IEC 61025, DO-178C, or any other functional safety or cybersecurity standard remains the **sole responsibility of the user and their organisation**.
>
> This tool does not replace a certified safety process, a qualified safety engineer, or a formal assessment body.

---

## Table of Contents

1. [Overview and Design Principles](#1-overview-and-design-principles)
2. [Element Type Inventory](#2-element-type-inventory)
3. [Common Frontmatter Schema](#3-common-frontmatter-schema)
4. [Directory and Namespace Conventions](#4-directory-and-namespace-conventions)
5. [Cross-Reference Syntax](#5-cross-reference-syntax)
6. [Multiplicity and Cardinality Syntax](#6-multiplicity-and-cardinality-syntax)
7. [Inline vs File-per-Element Rules](#7-inline-vs-file-per-element-rules)
8. [Element-Specific Schemas](#8-element-specific-schemas)
   - 8.1 [Package and Namespace Elements](#81-package-and-namespace-elements)
   - 8.2 [Structural Elements: Part, Item, Occurrence, Individual](#82-structural-elements-part-item-occurrence-individual)
   - 8.3 [Port and Interface Elements](#83-port-and-interface-elements)
   - 8.4 [Connection, Binding, and Succession Elements](#84-connection-binding-and-succession-elements)
   - 8.5 [Attribute and Enumeration Elements](#85-attribute-and-enumeration-elements)
   - 8.6 [Flow Elements](#86-flow-elements)
   - 8.7 [Action and Behavior Elements](#87-action-and-behavior-elements)
   - 8.8 [State Machine Elements](#88-state-machine-elements)
   - 8.9 [Calculation Elements](#89-calculation-elements)
   - 8.10 [Constraint Elements](#810-constraint-elements)
   - 8.11 [Requirement Elements](#811-requirement-elements)
     - 8.11.6 [Native `Requirement` Type](#8116-native-requirement-type)
   - 8.12 [Case Elements (Analysis, Verification, Use Case)](#812-case-elements-analysis-verification-use-case)
     - 8.12.5 [Native `TestCase` Type](#8125-native-testcase-type)
   - 8.13 [Allocation Elements](#813-allocation-elements)
   - 8.14 [View, Viewpoint, and Rendering Elements](#814-view-viewpoint-and-rendering-elements)
   - 8.15 [Metadata Elements](#815-metadata-elements)
   - 8.17 [Architecture Decision Records (ADR)](#817-architecture-decision-records-adr)
   - 8.18 [Safety and Security Analysis Elements](#818-safety-and-security-analysis-elements)
9. [Variability and Product Line Engineering](#9-variability-and-product-line-engineering)
   - 9.1–9.4 [Structural Variation (isVariation / isVariant)](#91-variation-definitions)
   - 9.5 [Product Line Engineering Overview](#95-product-line-engineering-overview)
   - 9.6 [`FeatureDef` — Feature Model Element](#96-featuredef--feature-model-element)
   - 9.7 [Feature Parametrization](#97-feature-parametrization)
   - 9.8 [`Configuration` — Feature Selection](#98-configuration--feature-selection)
   - 9.9 [Two-Level Feature Models](#99-two-level-feature-models)
   - 9.10 [`appliesWhen:` — Conditional Model Elements](#910-applieswhen--conditional-model-elements)
   - 9.11 [PLE Validation Rules](#911-ple-validation-rules)
   - 9.12 [PLE Graph Edges](#912-ple-graph-edges)
10. [Worked Examples](#10-worked-examples)
11. [Parser and Tool Contract](#11-parser-and-tool-contract)
    - 11.10 [Native-Element ID-Based Cross-Reference Resolution](#1110-native-element-id-based-cross-reference-resolution)
    - 11.11 [Computed Reverse Indices and Coverage](#1111-computed-reverse-indices-and-coverage)
    - 11.12 [Validation Rule Reference](#1112-validation-rule-reference)
12. [Traceability Rules and Domain Conventions](#12-traceability-rules-and-domain-conventions)
    - 12.1 [OSLC Link Direction Convention](#121-oslc-link-direction-convention)
    - 12.2 [Requirement Breakdown and ADRs](#122-requirement-breakdown-and-adrs)
    - 12.3 [Leaf-Level Assignment Rule](#123-leaf-level-assignment-rule)
    - 12.4 [Parent Requirements Cannot Be Assigned](#124-parent-requirements-cannot-be-assigned)
    - 12.5 [Requirement Domain Classification](#125-requirement-domain-classification)
    - 12.6 [Hardware/Software Architecture Independence](#126-hardwaresoftware-architecture-independence)
    - 12.7 [Safety/Security Integrity Level Propagation](#127-safetysecurity-integrity-level-propagation)
    - 12.8 [Implementation Trace](#128-implementation-trace)

---

## 1 Overview and Design Principles

### 1.1 Core Concept

Every SysMLv2 model element is represented as a single `.md` file. The file has two parts:

- **YAML frontmatter** (between `---` delimiters): declares the element's type, identity, and all structural/relational metadata.
- **Markdown body**: the documentation comment (`doc` annotation in SysML terms). This is the human-readable description of the element.

The **directory hierarchy** encodes namespace containment. A directory corresponds to a `package`. The `_index.md` file inside each directory carries that package's own metadata.

### 1.2 Design Principles

1. **One element, one file.** Every named SysML definition or usage that merits independent identity gets its own `.md` file. Exceptions are noted in Section 7.
2. **Path is identity.** An element's qualified name is derived algorithmically from its file path relative to the model root. The `name:` field in frontmatter overrides the display name but does not change the qualified name.
3. **YAML for structure, Markdown for prose.** Relationships, types, multiplicities, and flags go in frontmatter. Human-readable descriptions go in the body.
4. **camelCase for all frontmatter keys.** Consistent with SysML's own abstract syntax naming convention.
5. **Qualified names use `::` separator.** This matches SysML's textual notation.
6. **Defaults should match SysML defaults.** Where SysML specifies a default (e.g., multiplicity `1..1` for owned usages, `isAbstract: false`), the Markdown-SysML format silently applies that default when the field is absent.

### 1.3 Relationship to SysMLv2

Markdown-SysML is a **concrete syntax** for SysMLv2 semantics, alternative to the official SysML textual notation (`.sysml` files). It trades the full expressiveness of SysML expressions for readability and LLM-friendliness. The following SysML features are fully represented:

- All definition and usage kinds (part, item, port, connection, interface, action, state, calculation, constraint, requirement, case, view, metadata, allocation, flow, enumeration, attribute, occurrence, individual)
- Specialization relationships (subclassification, subsetting, redefinition)
- Multiplicity, direction, abstract, variation, variant flags
- Port conjugation
- Requirements with subjects, satisfactions, verifications, concerns
- Actions with parameters, succession flows, control nodes
- State machines with transitions, triggers, guards
- Metadata annotation and user-defined keywords
- Import and alias declarations
- Viewpoint/view/rendering

The following are **out of scope** for this format (they require a formal expression language):
- KerML/OCL constraint bodies (stored as opaque strings, not interpreted)
- Calculation body expressions (stored as opaque strings)
- Import filter conditions (stored as opaque strings)

---

## 2 Element Type Inventory

Every `.md` file must have a `type:` field drawn from the following table. The `sysml_keyword` column shows the corresponding SysML textual notation keyword sequence.

### 2.1 Definition Types

| `type:` value | SysML keyword(s) | Description |
|---|---|---|
| `PartDef` | `part def` | Classifies physical or logical system components |
| `ItemDef` | `item def` | Classifies things that flow through a system |
| `PortDef` | `port def` | Classifies interaction points on parts |
| `ConnectionDef` | `connection def` | Classifies structural links between ports |
| `InterfaceDef` | `interface def` | Classifies compatible connection ends |
| `ActionDef` | `action def` | Classifies behavioral steps |
| `CalculationDef` | `calc def` | Classifies parameterized expressions with return values |
| `ConstraintDef` | `constraint def` | Classifies boolean-valued conditions |
| `RequirementDef` | `requirement def` | Classifies textual/formal requirements |
| `ConcernDef` | `concern def` | Classifies stakeholder concerns |
| `CaseDef` | `case def` | Classifies case definitions (base for analysis/verification/use case) |
| `AnalysisCaseDef` | `analysis def` | Classifies analysis procedures |
| `VerificationCaseDef` | `verification def` | Classifies verification procedures |
| `UseCaseDef` | `use case def` | Classifies system use cases |
| `OccurrenceDef` | `occurrence def` | Classifies things with temporal extent |
| `EventOccurrenceDef` | `event occurrence def` | Classifies momentary, instantaneous occurrences (no duration) |
| `IndividualDef` | `individual def` | Classifies specific individual occurrences |
| `FlowDef` | `flow def` | Classifies flow connections carrying items |
| `SuccessionDef` | `succession def` | Classifies temporal ordering between occurrences |
| `StateDef` | `state def` | Classifies state machine nodes |
| `AttributeDef` | `attribute def` | Classifies data values (scalars, quantities) |
| `EnumerationDef` | `enum def` | Classifies discrete-valued attributes |
| `AllocationDef` | `allocation def` | Classifies allocation relationships |
| `MetadataDef` | `metadata def` | Classifies annotation structures |
| `ViewDef` | `view def` | Classifies model views |
| `ViewpointDef` | `viewpoint def` | Classifies stakeholder-oriented viewpoints |
| `RenderingDef` | `rendering def` | Classifies rendering methods for views |

### 2.2 Usage Types

| `type:` value | SysML keyword(s) | Description |
|---|---|---|
| `Part` | `part` | Usage of a PartDef |
| `Item` | `item` | Usage of an ItemDef |
| `Port` | `port` | Usage of a PortDef |
| `Connection` | `connection` | Usage of a ConnectionDef |
| `Interface` | `interface` | Usage of an InterfaceDef |
| `Action` | `action` | Usage of an ActionDef |
| `Calculation` | `calc` | Usage of a CalculationDef |
| `Constraint` | `constraint` | Usage of a ConstraintDef |
| `Requirement` | `requirement` | Usage of a RequirementDef |
| `Concern` | `concern` | Usage of a ConcernDef |
| `Case` | `case` | Usage of a CaseDef |
| `AnalysisCase` | `analysis` | Usage of an AnalysisCaseDef |
| `VerificationCase` | `verification` | Usage of a VerificationCaseDef |
| `UseCase` | `use case` | Usage of a UseCaseDef |
| `Occurrence` | `occurrence` | Usage of an OccurrenceDef |
| `EventOccurrence` | `event occurrence` | Usage of an EventOccurrenceDef; momentary observation or signal emission. `direction: in` = observed, `direction: out` = emitted |
| `Individual` | `individual` | Usage of an IndividualDef (time-slice/snapshot) |
| `Flow` | `flow` | Usage of a FlowDef |
| `Succession` | `succession` | Temporal ordering between actions/occurrences |
| `BindingConnector` | `binding` (connector) | Equality binding between two features |
| `State` | `state` | Usage of a StateDef |
| `ExhibitState` | `exhibit state` | Referential perform-action usage that exhibits a StateDef |
| `Attribute` | `attribute` | Usage of an AttributeDef |
| `Enumeration` | `enum` | Usage of an EnumerationDef |
| `Allocation` | `allocation` | Usage of an AllocationDef |
| `Metadata` | `metadata` | Application of a MetadataDef to an element |
| `View` | `view` | Usage of a ViewDef |
| `Rendering` | `rendering` | Usage of a RenderingDef |
| `TestCase` | *(native)* | Native test-case element with stable ID, lifecycle status, and Gherkin scenarios. See §8.12.5. |
| `FeatureDef` | *(PLE native)* | Feature model node — a named, selectable characteristic with grouping semantics and optional typed parameters. See §9.6. |
| `Configuration` | *(PLE native)* | A complete feature selection with parameter bindings; produces one concrete product variant. See §9.8. |

### 2.6 Record Types

Record elements are project-management and process artifacts that live in the model directory tree for traceability. They have no SysML counterpart.

| `type:` value | Description |
|---|---|
| `ADR` | Architecture Decision Record documenting a model-level design decision. See §8.17. |

### 2.3 Namespace/Package Types

| `type:` value | SysML keyword(s) | Description |
|---|---|---|
| `Package` | `package` | Named container for model elements |
| `LibraryPackage` | `library package` | Package marked as a model library |
| `Namespace` | (implicit) | Root namespace; `_index.md` at model root with no explicit parent |

### 2.4 Relationship Types

Relationship elements are directed relationships between named elements. Unlike definitions and usages, they are not classifiers or features — they live at package level, have no `typedBy:` field, and cannot be owned as sub-features of another element.

| `type:` value | SysML keyword | Description |
|---|---|---|
| `Dependency` | `dependency` | Directed relationship from one or more client elements to one or more supplier elements |

### 2.5 Diagram Types

Diagrams are visual representations of model elements. They are not SysML language constructs but a format extension for storing LLM-generated SVG diagrams with full model traceability.

| `type:` value | Description |
|---|---|
| `Diagram` | An SVG diagram with a frontmatter manifest linking shapes and edges to model elements |

**Specializations** of `Dependency` (e.g., `Realization`, `Derivation`) are expressed via `supertype:` referencing library types rather than requiring distinct `type:` values:

```yaml
---
type: Dependency
name: EngineControlRealization
supertype: SysML::Dependencies::Realization
clients:
  - LogicalArch::EngineControl
suppliers:
  - PhysicalArch::EngineControlUnit
---
The logical engine control function is realized by the physical ECU.
```

**Dependency-specific fields:**

| Field | YAML type | Required | Description |
|---|---|---|---|
| `clients` | list of strings | **Required** | Qualified names of the dependent (client) elements |
| `suppliers` | list of strings | **Required** | Qualified names of the elements depended upon (suppliers) |

**Placement convention:** Dependency files live either alongside the primary client element or in a dedicated `Dependencies/` package when they cross package boundaries.

---

## 3 Common Frontmatter Schema

This section defines every frontmatter field that may appear on any element file. Element-specific additional fields are defined in Section 8.

All fields are **optional unless marked Required**. Defaults listed apply when the field is absent.

### 3.1 Identity Fields

| Field | YAML type | Required | Default | SysML mapping |
|---|---|---|---|---|
| `type` | string | **Required** | — | Kind of element (Section 2) |
| `name` | string | optional | filename stem | Element name (display name); used in qualified name resolution if present |
| `shortName` | string | optional | absent | Short name (written `<shortName>` in SysML textual notation); abbreviated identifier |
| `qualifiedName` | string | optional | derived from path | Override the derived qualified name; use only when the file cannot be located at the canonical path |
| `visibility` | string | optional | `public` | Membership visibility: `public`, `protected`, `private`. This is a property of the *membership* — it controls whether this element is visible to namespaces outside its owner, not a property of the element itself. `private` means visible only within the owning package; `protected` means visible within the owning package and its specializations. |
| `extRef` | string or list of strings | optional | absent | **External reference(s)** — this element represents an artifact managed in another tool (a requirement in DOORS Next, an element in a SysML tool, a ticket, …). See *External references* below. |

**Name resolution rule:** The element's qualified name segment is the `name:` field value if present, otherwise the filename stem (filename without `.md`). The full qualified name is the `::` join of all ancestor package names and the element's own name, starting from the model root (excluding the model root's own name unless it is a named package).

**External references (`extRef`).** A Syscribe element often mirrors an artifact that lives in another tool. `extRef:` records that correspondence as one or more **opaque** strings — a URI (`https://dng.example/resources/4521`) or a tool-qualified token (`DNG:4521`, `cameo://model/Engine#id-99`) — given either as a single string or a list. The field is optional and its syntax is unconstrained (external systems use widely varying identifier schemes). `extRef` is an *external* pointer: it is **not** a model cross-reference and is never a valid target for `supertype:`, `verifies:`, `derivedFrom:`, connections, etc. Look up the element(s) that carry a given reference with `syscribe -m <root> extref <ref>`. The same `extRef` value on two or more elements is permitted but warned (`W028`), since an external artifact normally maps to a single element.

```yaml
# any element — represents DNG requirement 4521 and a Cameo block
extRef:
  - "DNG:4521"
  - "cameo://model/Engine#id-99"
```

### 3.2 Classification Flags

| Field | YAML type | Required | Default | SysML mapping |
|---|---|---|---|---|
| `isAbstract` | bool | optional | `false` | `abstract` keyword; definition or usage is abstract |
| `isVariation` | bool | optional | `false` | `variation` keyword; element is a variation point |
| `isVariant` | bool | optional | `false` | `variant` keyword; element is one variant within a variation |
| `isIndividual` | bool | optional | `false` | `individual` keyword on occurrence *usages*; declares that this usage represents a specific unique individual, not an abstract role. Applies to usages only (for definitions use `type: IndividualDef`). Implies multiplicity `"0..1"` at most; a parser should warn if multiplicity upper bound exceeds 1. |
| `isReadonly` | bool | optional | `false` | `readonly` keyword; usage value cannot be changed after initialization |
| `isDerived` | bool | optional | `false` | `derived` keyword; value is fully determined by an expression |
| `isEnd` | bool | optional | `false` | Marks a connection end feature |
| `isPortion` | bool | optional | `false` | `portion` kind on occurrences; marks this usage as a temporal sub-extent of its owning occurrence |
| `portionKind` | string | optional | `timeslice` | Required when `isPortion: true`; `timeslice` (contiguous temporal extent) or `snapshot` (zero-duration instantaneous point) |
| `isReference` | bool | optional | `false` | `ref` keyword; usage is referential (not composite) |
| `isComposite` | bool | optional | `true` (for part/item usages owned by a def/usage) | Composite vs. referential ownership |
| `isConstant` | bool | optional | `false` | `constant` keyword; value fixed for the lifetime of the featuring occurrence |
| `isOrdered` | bool | optional | `false` | `ordered` keyword; multiple values are ordered (indexed) |
| `isNonunique` | bool | optional | `false` | `nonunique` keyword; same value may appear multiple times |

**Normative note on `isReference` and compositeness:**

`isReference: true` is equivalent to the SysML `ref` keyword and marks a usage as referential — the containing element does not own the referenced element, it merely holds a reference to one that exists independently.

Default compositeness by context:
- Usages owned inside a `PartDef`, `ItemDef`, `ActionDef`, or similar definition → composite (`isReference: false`) by default.
- Usages directly owned by a `Package` → referential by default.

Certain usage kinds **imply** `isReference: true` regardless of the field value. A conformant parser must enforce this and emit a warning if `isReference: false` is explicitly set on these kinds:

| Usage kind | Reason |
|---|---|
| `ExhibitState` | A part exhibits a state; it never owns it |
| Sub-action with `kind: PerformAction` | Perform always references an independently defined ActionDef |
| `Metadata` | Metadata applications are always annotations, never owned instances |

### 3.3 Typing and Specialization

| Field | YAML type | Required | Default | SysML mapping |
|---|---|---|---|---|
| `supertype` | string or list of strings | optional | absent | Subclassification (`:>` or `specializes`) for definitions; may be a single string or list |
| `typedBy` | string or list of strings | optional | absent | Feature typing (`defined by` or `:`) for usages; names the definition(s) |
| `subsets` | list of strings | optional | absent | Subsetting (`:>` or `subsets`) for usages; list of subsetted usage qualified names |
| `redefines` | list of strings | optional | absent | Redefinition (`:>>` or `redefines`) for usages; list of redefined usage qualified names |
| `conjugates` | string | optional | absent | Port conjugation (`~`); qualified name of the definition being conjugated |

**Note on `supertype` vs `typedBy`:** Definitions use `supertype` (subclassification). Usages use `typedBy` (feature typing). Both support a single string or a YAML list for multiple supertypes/definitions.

### 3.4 Multiplicity

| Field | YAML type | Required | Default | SysML mapping |
|---|---|---|---|---|
| `multiplicity` | string | optional | `"1"` for owned usages inside a def/usage; `"0..*"` otherwise | SysML MultiplicityRange; see Section 6 |

### 3.5 Direction

| Field | YAML type | Required | Default | SysML mapping |
|---|---|---|---|---|
| `direction` | string | optional | absent (undirected) | `in`, `out`, or `inout`; applies to ports, parameters, action features |

### 3.6 Inline Features

| Field | YAML type | Required | Default | SysML mapping |
|---|---|---|---|---|
| `features` | list of feature maps | optional | absent | Inline owned attribute or port usages; see Section 7 and 3.6.1 |

#### 3.6.1 Inline Feature Schema

Each entry in `features:` is a map with these fields:

| Sub-field | YAML type | Required | Default | Description |
|---|---|---|---|---|
| `name` | string | **Required** | — | Feature name |
| `type` | string | optional | absent | `type:` value (e.g., `Attribute`, `Port`) — defaults to `Attribute` when `typedBy` is a scalar type |
| `typedBy` | string | optional | absent | Qualified name of the definition typing this feature |
| `multiplicity` | string | optional | `"1"` | Multiplicity range string |
| `direction` | string | optional | absent | `in`, `out`, or `inout` |
| `isAbstract` | bool | optional | `false` | Abstract flag |
| `isReadonly` | bool | optional | `false` | Readonly flag |
| `isDerived` | bool | optional | `false` | Derived flag |
| `isReference` | bool | optional | `false` | Reference (non-composite) flag |
| `isOrdered` | bool | optional | `false` | `ordered` keyword; values are index-ordered |
| `isNonunique` | bool | optional | `false` | `nonunique` keyword; duplicate values are permitted |
| `value` | string | optional | absent | Literal default value or expression body (opaque string) |
| `valueKind` | string | optional | `bound` | Binding semantics: `bound` (`=`), `initial` (`:=`), `default-bound` (`default =`), `default-initial` (`default :=`) |
| `unit` | string | optional | absent | Shorthand: qualified name of the unit for quantity attributes |
| `subsets` | list of strings | optional | absent | Qualified names of subsetted features |
| `redefines` | list of strings | optional | absent | Qualified names of redefined features |
| `metadata` | list of metadata application maps | optional | absent | Metadata annotations on this inline feature |

### 3.7 Packaging and Import Fields

| Field | YAML type | Required | Default | SysML mapping |
|---|---|---|---|---|
| `imports` | list of import maps or strings | optional | absent | Import declarations; see 3.7.1 |
| `aliases` | list of alias maps | optional | absent | Alias declarations; see 3.7.2 |
| `filterCondition` | string | optional | absent | KerML boolean expression for package filter condition (opaque string) |

#### 3.7.1 Import Schema

Each entry in `imports:` may be a plain string (shorthand) or a map:

**Shorthand:** a qualified name string, optionally suffixed with `::*` (namespace import) or `::**` (recursive import).

```yaml
imports:
  - VehicleSystem::*
  - ISQ::**
```

**Full map form:**

| Sub-field | YAML type | Required | Default | Description |
|---|---|---|---|---|
| `target` | string | **Required** | — | Qualified name, optionally suffixed `::*` or `::**` |
| `isPublic` | bool | optional | `false` | `true` makes imported names re-exported from this namespace (`public import`); `false` is a private import visible only within this namespace |
| `filter` | string | optional | absent | KerML boolean filter expression; only members satisfying the condition are imported (opaque string) |

**Examples:**

```yaml
imports:
  # shorthand — private import of all direct members
  - VehicleSystem::*

  # public import — re-exports ISQ members downstream
  - target: ISQ::**
    isPublic: true

  # filtered import — only abstract members
  - target: VehicleSystem::*
    filter: "isAbstract"

  # filtered import — only members annotated as normative
  - target: ISQ::**
    filter: "#(ModelingMetadata::Normative)"
```

**Note on `isPublic`:** A `public import` in SysML makes the imported names visible to any namespace that imports the importing namespace. A `private import` (the default) makes them visible only within the importing namespace itself.

#### 3.7.2 Alias Schema

Each entry in `aliases:` is a map:

| Sub-field | YAML type | Required | Default | Description |
|---|---|---|---|---|
| `name` | string | **Required** | — | Alias name within this namespace |
| `shortName` | string | optional | absent | Alias short name |
| `for` | string | **Required** | — | Qualified name of the aliased element |
| `visibility` | string | optional | `public` | Membership visibility of the alias |

**Note:** `aliases:` may appear on **any** element, not just packages. When declared on a non-package element (e.g., a `PartDef`), it creates a local alias name for another element within the scope of the declaring element's namespace — useful for shortening deep cross-references within a definition body.

### 3.8 Metadata Application

| Field | YAML type | Required | Default | SysML mapping |
|---|---|---|---|---|
| `metadata` | list of metadata application maps | optional | absent | Annotations applied to this element |

Each entry in `metadata:` is a map:

| Sub-field | YAML type | Required | Default | Description |
|---|---|---|---|---|
| `type` | string | **Required** | — | Qualified name of the `MetadataDef` being applied |
| `<attribute_name>` | any | optional | absent | Attribute values of the metadata instance; key is the attribute name, value is a literal |

Example:

```yaml
metadata:
  - type: ModelingMetadata::StatusInfo
    status: approved
    approver: "Jane Smith"
```

### 3.9 Dependency

| Field | YAML type | Required | Default | SysML mapping |
|---|---|---|---|---|
| `dependsOn` | list of strings | optional | absent | Dependency relationships to supplier elements (qualified names); client is the containing element |

### 3.10 Annotation Fields

These fields control how the Markdown body is interpreted as a documentation annotation.

| Field | YAML type | Required | Default | SysML mapping |
|---|---|---|---|---|
| `about` | list of strings | optional | absent | Qualified names of elements this file's Markdown body annotates; when present, the body is a cross-element comment rather than a doc on the file's own element |
| `locale` | string | optional | absent | BCP 47 language tag (e.g., `en`, `de`, `fr`) tagging the Markdown body for multi-language documentation |

**`about:` usage** — for cross-cutting notes that apply to multiple elements without duplicating text:

```yaml
---
type: Package        # a comment-carrier package, or any element type
name: SafetyNote
about:
  - VehicleSystem::Engine
  - VehicleSystem::Transmission
  - VehicleSystem::Brakes
---
This subsystem group requires safety analysis per ISO 26262. All three elements
share the same hazard classification and must be reviewed together.
```

**`locale:` usage** — for multi-language documentation, create one file per language. Use `qualifiedName:` to make all locale variants point to the same model element:

```yaml
# model/VehicleSystem/Engine.md  (English — primary file, also defines the element)
---
type: PartDef
name: Engine
---
The engine converts fuel energy into mechanical power.
```

```yaml
# model/VehicleSystem/Engine.de.md  (German — documentation-only variant)
---
type: PartDef
qualifiedName: VehicleSystem::Engine   # points to the same element
locale: de
---
Der Motor wandelt Kraftstoffenergie in mechanische Leistung um.
```

**Parser contract for `about:` and `locale:`:**
- A file with `about:` contributes its Markdown body as an additional annotation on the listed elements; it does not define a new model element.
- A file with `locale:` and a `qualifiedName:` override contributes a locale-tagged `doc` annotation to the referenced element; it does not redefine the element's structure.
- Multiple locale files for the same element are all valid; a parser collects them as a map of `locale → doc string`.

### 3.11 Constraint Fields

These fields may appear on **any** element, not only on `RequirementDef`/`Requirement`. When used outside a requirement context, they declare informal constraints on the element.

| Field | YAML type | Required | Default | SysML mapping |
|---|---|---|---|---|
| `requires` | list of strings or constraint maps | optional | absent | Qualified names of `ConstraintDef`s or opaque boolean expressions that must hold; when entries are maps, they follow the constraint clause schema (§8.11.2) |
| `assume` | list of strings or constraint maps | optional | absent | Qualified names of `ConstraintDef`s or expressions assumed to hold (preconditions); map entries follow §8.11.2 |

**Note:** On `RequirementDef` and `Requirement`, these fields have full structural semantics (formal constraint clauses; §8.11.1). On other element types, they are treated as informal annotations and do not affect typing or redefinition.

### 3.12 Representation Field

| Field | YAML type | Required | Default | SysML mapping |
|---|---|---|---|---|
| `rep` | string | optional | absent | Textual representation annotation; formal notation string associated with this element, used when round-tripping to/from SysML textual notation (opaque string) |

### 3.13 Product Line Conditioning Field

| Field | YAML type | Required | Default | Description |
|---|---|---|---|---|
| `appliesWhen` | string or list of strings | optional | absent | Qualified name of a `FeatureDef` (or AND-list of `FeatureDef` names) that must be selected in a `Configuration` for this element to be included in the projected model. Absent = unconditionally included. See §9.10. |

`appliesWhen:` may appear on **any** element type, including `Requirement`, `PartDef`, `Part`, `TestCase`, `Allocation`, `ActionDef`, `Connection`, `Diagram`, and all others — and on a **`Package`**, where it applies transitively to the whole subtree (the *effective condition*; §9.10, error `E228` enforces one declaration per path). It is the sole mechanism by which model elements are conditioned on feature selections. A string value is a reference to a single `FeatureDef`; a list means all listed features must be selected (AND semantics). OR semantics are expressed in the feature model itself via `groupKind: or` (see §9.6).

### 3.14 Domain Classification

These fields encode the engineering domain of an element and are used to enforce hardware/software architecture independence (§12.6).

| Field | YAML type | Required | Default | Description |
|---|---|---|---|---|
| `domain` | string | optional | `system` | Engineering domain: `system` (domain-agnostic), `hardware`, or `software`. Applies to any element; most meaningful on `PartDef`, `Part`, and `Package` elements. |
| `isDeploymentPackage` | bool | optional | `false` | Marks a `PartDef` or `Part` as a deployable software artifact that must be physically allocated to a hardware element. Only meaningful on elements with `domain: software`. |

When `domain:` is not set, `system` is assumed — the element is domain-agnostic and can interoperate with both hardware and software elements.

`isDeploymentPackage: true` implies `domain: software`; a parser should warn (`W306`) if `isDeploymentPackage: true` is combined with `domain: hardware`.

---

## 4 Directory and Namespace Conventions

### 4.1 Model Root

The model root is a directory chosen by the user (e.g., `model/`). All qualified name derivation is relative to this root. A parser is initialized with the path to the root.

The root directory **may** contain an `_index.md` with `type: Namespace` or `type: Package`. If absent, the root is treated as an anonymous root namespace.

### 4.2 Package Directories

A directory maps to a SysML `package`. The package's name is the directory name unless overridden by `name:` in the directory's `_index.md`.

```
model/
  _index.md              # type: Namespace  — root namespace
  VehicleSystem/
    _index.md            # type: Package, name: VehicleSystem
    Chassis.md           # type: PartDef   — qname: VehicleSystem::Chassis
    Powertrain/
      _index.md          # type: Package
      Engine.md          # qname: VehicleSystem::Powertrain::Engine
```

### 4.3 `_index.md` Schema

An `_index.md` file may carry any fields from Section 3, plus the following package-specific fields:

| Field | YAML type | Description |
|---|---|---|
| `type` | string | Must be `Package`, `LibraryPackage`, or `Namespace` |
| `name` | string | Override the directory name as the package name |
| `imports` | list | Import declarations (Section 3.7.1) |
| `aliases` | list | Alias declarations (Section 3.7.2) |
| `filterCondition` | string | Package filter condition (opaque KerML expression) |

### 4.4 Library Packages

A library package uses `type: LibraryPackage`. Library packages are treated identically to regular packages for qualified name resolution, but tools may display them differently and they implicitly export all their members publicly.

```yaml
# model/ISQ/_index.md
---
type: LibraryPackage
name: ISQ
---
International System of Quantities domain library.
```

### 4.5 Name Collision Rules

1. The `name:` field in frontmatter, if present, is the element's declared name and takes precedence over the filename stem for display and cross-reference purposes.
2. The filename stem is always used as the unique file identifier for file-system operations.
3. If `name:` differs from the filename stem, the qualified name uses `name:`. The filename may be anything valid for the OS.
4. Two elements in the same directory must not have the same effective `name:` (after applying the `name:` override). This is a validation error.
5. A file named `_index.md` is never assigned its own qualified name segment; it represents the containing directory's package.

### 4.6 Visibility Within Directories

By default, all elements in a directory are public members of that package. Set `visibility: private` in frontmatter to restrict membership visibility.

### 4.7 Standard Library Package Inventory

The following packages are part of the SysML v2 standard library. A parser must not error on references to these packages even if their `.md` files are not present in the model directory — they are resolved from the built-in library.

**Auto-imported at model root (always available without explicit import):**

| Package | Contents |
|---|---|
| `ScalarValues` | Primitive scalar types: `Integer`, `Real`, `String`, `Boolean`, `Natural` |
| `Base` | Base types: `Anything`, `DataValue` |

**Available via explicit import:**

| Package | Contents |
|---|---|
| `ISQ` | International System of Quantities |
| `SI` | SI units and unit symbols |
| `Parts` | Base `Part` definition |
| `Items` | Base `Item` definition |
| `Ports` | Base `Port` definition |
| `Actions` | Base `Action` definition |
| `States` | Base `StateAction` definition |
| `Calculations` | Base `Calculation` definition |
| `Constraints` | Base `Constraint` definition |
| `Requirements` | Base `Requirement` definition |
| `Allocations` | Base `Allocation` definition |
| `Connections` | Base `Connection` and `Interface` definitions |
| `Transfers` | Base `Transfer` definition |
| `Flows` | Base `FlowConnection` definition |
| `Occurrences` | Base `Occurrence` and `EventOccurrence` definitions |
| `Events` | Event definitions |
| `UseCases` | Base `UseCase` definition |
| `AnalysisCases` | Base `AnalysisCase` definition |
| `VerificationCases` | Base `VerificationCase` definition; includes `VerdictKind` enumeration |
| `Views` | Base `View`, `Viewpoint`, `Rendering` definitions |
| `Metadata` | Base `SemanticMetadata` definition |
| `ModelingMetadata` | Standard annotation metadata: `StatusInfo`, `Issue`, `Rationale`, `Refinement` |

---

## 5 Cross-Reference Syntax

### 5.1 Absolute Qualified Names

An absolute qualified name starts from a top-level package name:

```yaml
typedBy: VehicleSystem::Powertrain::Engine
supertype: Requirements::FunctionalRequirementDef
```

### 5.2 Relative References

A simple unqualified name resolves within the current element's containing package first, then outward through enclosing namespaces:

```yaml
typedBy: Engine        # resolves within current package first
```

A `./`-prefixed name explicitly means "sibling in the same directory":

```yaml
subsets: [./drivetrainPort]
```

### 5.3 Wildcard and Namespace Imports

In `imports:` fields:

| Syntax | Meaning |
|---|---|
| `PackageName` | Import a single named element (membership import) |
| `PackageName::*` | Import all visible members of PackageName (namespace import) |
| `PackageName::**` | Recursively import PackageName and all sub-namespaces |

```yaml
imports:
  - ISQ::*
  - ScalarValues::**
```

### 5.4 Aliases

Declare an alias to create a short local name:

```yaml
aliases:
  - name: MV
    for: ISQ::MassValue
```

Then reference it as `MV` within the same package.

### 5.5 Self-Reference

Use `self` to refer to the element defined by the current file, within constraint or expression bodies (opaque strings only; not used in structural YAML fields).

### 5.6 Feature Chains

Feature chains (dot-notation paths through usages) are written using `.` within qualified names that appear in `connections:`, `flow_connections:`, and `succession_connections:` entries. For example:

```yaml
connections:
  - from: engine.exhaustPort
    to: exhaustSystem.inletPort
```

---

## 6 Multiplicity and Cardinality Syntax

The `multiplicity:` field and the `multiplicity` sub-field inside `features:` entries are **quoted strings** conforming to the following grammar:

```
multiplicity  ::= bound | bound ".." bound
bound         ::= integer | "*" | qualified-name
integer       ::= [0-9]+
qualified-name ::= name ("::" name)*
name          ::= [A-Za-z_][A-Za-z0-9_]*
```

Common forms:

| String | Lower bound | Upper bound | SysML equivalent |
|---|---|---|---|
| `"1"` | 1 | 1 | `[1]` or `[1..1]` |
| `"0"` | 0 | 0 | `[0]` |
| `"*"` | 0 | ∞ | `[*]` or `[0..*]` |
| `"0..1"` | 0 | 1 | `[0..1]` |
| `"1..*"` | 1 | ∞ | `[1..*]` |
| `"0..*"` | 0 | ∞ | `[0..*]` |
| `"2..4"` | 2 | 4 | `[2..4]` |
| `"0..maxSeats"` | 0 | feature ref | `[0..maxSeats]` |
| `"minN..maxN"` | feature ref | feature ref | `[minN..maxN]` |

**Rules:**
- The string must be quoted in YAML to avoid misparse (e.g., `"1"` not bare `1`).
- If only a single token is given (`"4"` or `"maxItems"`), both bounds equal that value.
- `*` is the only shorthand for an unbounded upper; it may not appear as the lower bound.
- The lower bound must resolve to a non-negative integer at evaluation time.
- When a bound is not a valid integer literal or `*`, it is treated as a qualified-name reference and resolved during the same cross-reference resolution pass as all other qualified names (§11.3). A parser must report an unresolved bound reference as a validation error.

**Feature-reference bound example:**

```yaml
# Vehicle.md
features:
  - name: passengers
    typedBy: Passenger
    multiplicity: "0..maxPassengers"   # upper bound from a sibling feature

  - name: maxPassengers
    typedBy: ScalarValues::Integer
    value: "8"
```

---

## 7 Inline vs File-per-Element Rules

### 7.1 Use Inline `features:` When

- The feature is a simple scalar attribute (a single `typed_by` with a primitive or library type, no sub-features of its own).
- The feature has no documentation body worth a separate file.
- The feature has no metadata of its own beyond what fits on one or two lines.
- The feature is used only within this one definition and is not referenced by qualified name from other files.

**Examples of inline-suitable features:**

```yaml
features:
  - name: mass
    typedBy: ISQ::MassValue
    unit: SI::kg
  - name: length
    typedBy: ISQ::LengthValue
    unit: SI::m
  - name: status
    typedBy: VehicleStatus    # an EnumerationDef in the same package
```

### 7.2 Use a Separate `.md` File When

- The feature has its own sub-features (nested parts, ports, attributes).
- The feature has significant documentation.
- The feature is a `PortDef`, `ActionDef`, `StateDef`, or other definition reused across multiple parent definitions.
- The feature is a usage that itself owns usages (a composed sub-part, a sub-action, a sub-state).
- The feature needs its own metadata annotations.
- The feature must be reference-able by qualified name from other files.

### 7.3 Attribute Definitions

Simple `AttributeDef` elements that define a named data type (e.g., `Temperature`, `Pressure`) always get their own file so they can be typed-by from multiple locations.

Primitive inline literals may be expressed as:

```yaml
features:
  - name: maxTemperature
    typedBy: ScalarValues::Real
    value: "200.0"
    unit: SI::degC
```

---

## 8 Element-Specific Schemas

### 8.1 Package and Namespace Elements

#### 8.1.1 `_index.md` for Package

```yaml
---
type: Package               # or LibraryPackage, Namespace
name: VehicleSystem         # optional; defaults to directory name
isAbstract: false
imports:
  - ISQ::*
  - ScalarValues::*
aliases:
  - name: Real
    for: ScalarValues::Real
filterCondition: ""         # optional KerML filter expression (opaque)
metadata:
  - type: ModelingMetadata::StatusInfo
    status: draft
---
The VehicleSystem package contains all definitions and usages
for modelling a generic vehicle.
```

### 8.2 Structural Elements: Part, Item, Occurrence, Individual

#### 8.2.1 `PartDef` — Part Definition

Defines a class of physical or logical system components.

**Type-specific additional fields:**

| Field | YAML type | Default | Description |
|---|---|---|---|
| `supertype` | string or list | absent | Subclassification of other PartDefs |
| `features` | list | absent | Inline owned attributes and ports |
| `connections` | list | absent | Internal connections between sub-parts (IBD edges) |
| `flowConnections` | list | absent | Flow connection specifications |
| `successionConnections` | list | absent | Succession flow between sub-actions |
| `bindingConnections` | list | absent | Binding connector specifications |
| `performs` | list of strings or maps | absent | Actions this part performs; string shorthand or full map form (see below) |
| `exhibitsStates` | list of strings | absent | Shorthand: qualified names of StateDefs this part exhibits (no bindings, multiplicity `"1"`). For named exhibit-state usages with bindings or redefinition, use `type: ExhibitState` in `features:` or as a separate file; see §8.8.4 |

**`performs:` field — string shorthand and full map form:**

A string entry expands to `{typedBy: <string>}` with all other fields at their defaults.
The full map form supports all perform-action usage properties:

| Sub-field | YAML type | Required | Default | Description |
|---|---|---|---|---|
| `name` | string | optional | absent | Named perform-action usage |
| `typedBy` | string | **Required** (full form) | — | Qualified name of the ActionDef being performed |
| `multiplicity` | string | optional | `"1"` | Cardinality |
| `redefines` | list of strings | optional | absent | Qualified names of perform-action usages this redefines |
| `bindingConnections` | list | optional | absent | Binding connectors that bind action parameters to part features; same schema as §8.4.3 |

```yaml
performs:
  - VehicleBehavior::ProvidePower          # shorthand — expands to {typedBy: ...}

  - name: driveAction                      # full form
    typedBy: VehicleBehavior::RegulateSpeed
    multiplicity: "1"
    redefines: [Parts::Part::performedActions]
    bindingConnections:
      - left: driveAction.inputSpeed
        right: self.currentSpeed
```

**Example** (`model/VehicleSystem/Vehicle.md`):

```yaml
---
type: PartDef
name: Vehicle
isAbstract: true
supertype: Parts::Part
features:
  - name: mass
    typedBy: ISQ::MassValue
    unit: SI::kg
  - name: speed
    typedBy: ISQ::SpeedValue
    unit: SI::m_per_s
    isDerived: true
  - name: fuelCapacity
    typedBy: ISQ::VolumeValue
    unit: SI::L
connections:
  - from: engine.powerOut
    to: drivetrain.powerIn
    typedBy: PowerConnection
performs:
  - VehicleBehavior::providePower
---
Abstract definition of a vehicle with mass, speed, and fuel capacity.
All concrete vehicle types specialize this definition.
```

#### 8.2.2 `Part` — Part Usage

A usage of a PartDef in a specific context.

**Type-specific additional fields:**

| Field | YAML type | Default | Description |
|---|---|---|---|
| `typedBy` | string or list | absent | The PartDef(s) this part is an instance of |
| `subsets` | list | absent | Subsetted part usages |
| `redefines` | list | absent | Redefined part usages |
| `multiplicity` | string | `"1"` | Cardinality |
| `isReference` | bool | `false` | `ref` — referential rather than composite |
| `isPortion` | bool | `false` | Time-slice portion of the owning occurrence |
| `connections` | list | absent | Internal connections (if this part usage owns sub-parts) |
| `features` | list | absent | Additional features scoped to this usage |
| `performs` | list of strings | absent | Actions performed in this usage context |

**Example** (`model/VehicleSystem/vehicle_b.md`):

```yaml
---
type: Part
name: vehicle_b
typedBy: Vehicle
multiplicity: "1"
features:
  - name: engine
    type: Part
    typedBy: Powertrain::FourCylinderEngine
    multiplicity: "1"
  - name: transmission
    type: Part
    typedBy: Powertrain::AutomaticTransmission
    multiplicity: "1"
connections:
  - from: engine.powerOut
    to: transmission.powerIn
---
Specific vehicle configuration B with a four-cylinder engine
and automatic transmission.
```

#### 8.2.3 `ItemDef` and `Item`

Items classify things that may flow through or be contained in a system (as opposed to the structural parts). Schema is identical to `PartDef`/`Part` with `type: ItemDef` / `type: Item`.

**Notable difference:** Items are always referential; `isReference` defaults to `true` for item usages inside attribute definitions and usages.

#### 8.2.4 `OccurrenceDef` and `Occurrence`

Occurrences model things with temporal extent. An `OccurrenceDef` can define time-slice structure.

**Note:** `OccurrenceDef` and `Occurrence` are the *base* occurrence types. Use them for abstract occurrence hierarchies that do not specialize any of the more specific kinds (`PartDef`, `ItemDef`, `ActionDef`, `StateDef`, etc.). All more specific definition types in SysML ultimately specialize `OccurrenceDef` at the KerML level.

**Additional fields for `OccurrenceDef`:**

| Field | YAML type | Default | Description |
|---|---|---|---|
| `timeSlices` | list of inline feature maps | absent | Portion/time-slice usages |
| `snapshots` | list of inline feature maps | absent | Snapshot individual usages |

**Example** (`model/Lifecycle/VehicleLifecycle.md`):

```yaml
---
type: OccurrenceDef
name: VehicleLifecycle
supertype: Occurrences::Occurrence
timeSlices:
  - name: operatingPhase
    typedBy: VehicleSystem::Vehicle
    isPortion: true
  - name: maintenancePhase
    typedBy: VehicleSystem::Vehicle
    isPortion: true
---
Models the full lifecycle of a vehicle from manufacture to disposal.
```

#### 8.2.5 `IndividualDef` and `Individual`

Individuals represent specific instances with a fixed identity.

```yaml
---
type: Individual
name: myVehicle2026
typedBy: VehicleSystem::Vehicle
isIndividual: true
features:
  - name: vehicleId
    typedBy: ScalarValues::String
    value: '"VIN-1234567890ABCDEF"'
  - name: manufactureDate
    typedBy: ISQ::DateTime
---
The specific vehicle with VIN 1234567890ABCDEF manufactured in 2026.
```

### 8.3 Port and Interface Elements

#### 8.3.0 Concepts and decision guide

Ports and interfaces are the most error-prone area to model. Orient first, then use the schemas below.

**Mental model (SysML v2).** A **`PortDef`** is a reusable *kind* of connection point that carries **directed features** (`in`/`out`/`inout`). A **`Port`** is a *usage* of a `PortDef` on a part — it lives in the part's `features:` list with `type: Port` and `typedBy:` the PortDef. A connection wires two ports together. An **`InterfaceDef` is a kind of `ConnectionDef`** — in SysML v2, *"an interface is simply a connection all of whose ends are ports"* (§7.14) — so use it to package a reusable, compatible pairing of two ports; a plain **`ConnectionDef`** connects arbitrary features/parts, not necessarily ports.

**Which construct → when:**

| You want to… | Use |
|---|---|
| expose an interaction point on a part | a `Port` (in `features:`) typed by a `PortDef` |
| define a reusable compatible pairing of two ports | `InterfaceDef` (ends typed by PortDefs) |
| connect arbitrary features/parts | `ConnectionDef` |
| wire two specific ports inside a part | a connection usage — `connections:` with `from`/`to` feature chains, optionally `typedBy:` the InterfaceDef |
| move items between connected ports | `FlowDef` / `flowConnections:` |
| equate two features | `bindingConnections:` |

**Conjugation.** The receiving end of a connection is the **conjugate** of the sending end: every directed feature flips direction (`in`↔`out`; `inout` is self-conjugate — SysML v2 §7.12.3, *"each port definition has a conjugated port definition whose directed features are conjugate to those of the original"*). Express it either with a dedicated conjugate `PortDef` (`conjugates: <the source PortDef>`) for reuse, or with `isConjugated: true` on the `Port` usage / interface end for a one-off. Connected directed features must be conjugate-compatible — that is how a supplier's `out` lines up with a consumer's `in`.

**End-to-end shape.** A supplier `PartDef` exposes a `Port` typed by a source `PortDef`; a consumer `PartDef` exposes a `Port` typed by the **conjugate** PortDef; an `InterfaceDef` ties the two PortDefs as its ends; a parent `PartDef` holds both as sub-parts and wires them in `connections:` (`from: supplier.outPort`, `to: consumer.inPort`, `typedBy:` the InterfaceDef). A `FlowDef`/`flowConnections:` may carry the items that move across.

#### 8.3.1 `PortDef` — Port Definition

Defines an interaction point. Ports can be conjugated.

**Type-specific additional fields:**

| Field | YAML type | Default | Description |
|---|---|---|---|
| `supertype` | string or list | absent | Subclassification of other PortDefs |
| `conjugates` | string | absent | Qualified name of the PortDef this conjugates (`~PortDefName`). When set, this PortDef has reversed flow directions for all features. |
| `features` | list | absent | Inline owned flows/attributes carried through this port |

**Example** (`model/Interfaces/PowerPortDef.md`):

```yaml
---
type: PortDef
name: PowerPortDef
features:
  - name: voltage
    typedBy: ISQ::VoltageValue
    direction: out
    unit: SI::V
  - name: current
    typedBy: ISQ::ElectricCurrentValue
    direction: out
    unit: SI::A
---
A port definition representing an electrical power interface.
```

**Conjugated Port Example** (`model/Interfaces/PowerPortReceiverDef.md`):

```yaml
---
type: PortDef
name: PowerPortReceiverDef
conjugates: Interfaces::PowerPortDef
---
Conjugated port definition for receiving electrical power.
All features have their directions reversed relative to PowerPortDef.
```

#### 8.3.2 `Port` — Port Usage

**Type-specific additional fields:**

| Field | YAML type | Default | Description |
|---|---|---|---|
| `typedBy` | string or list | absent | The PortDef(s) defining this port |
| `direction` | string | absent | `in`, `out`, `inout` — overrides direction at the usage level |
| `isConjugated` | bool | `false` | Shorthand: if `true`, the port is treated as conjugated of its PortDef without requiring a separate conjugate PortDef file |
| `multiplicity` | string | `"1"` | Cardinality |

**Example** (`model/VehicleSystem/Engine.md` features section):

```yaml
features:
  - name: powerOut
    type: Port
    typedBy: Interfaces::PowerPortDef
    direction: out
  - name: fuelIn
    type: Port
    typedBy: Interfaces::FuelPortDef
    direction: in
```

#### 8.3.3 `InterfaceDef` and `Interface`

An interface definition specifies the required compatibility between two connection ends.

**Type-specific additional fields for `InterfaceDef`:**

| Field | YAML type | Default | Description |
|---|---|---|---|
| `supertype` | string or list | absent | Supertype InterfaceDefs |
| `ends` | list of inline feature maps | absent | The two (or more) end features; each typed by a PortDef |
| `features` | list | absent | Additional owned features |
| `constraints` | list of constraint maps | absent | Constraint usages asserted across this interface's ends (e.g., conservation laws) |

Each entry in `constraints:`:

| Sub-field | YAML type | Required | Description |
|---|---|---|---|
| `name` | string | optional | Named constraint usage |
| `typedBy` | string | optional | Qualified name of a `ConstraintDef` |
| `expression` | string | optional | Inline boolean expression (opaque string) |
| `isAsserted` | bool | optional (default `true`) | Whether the constraint is asserted to hold |

**Example** (Kirchhoff's current law on an electrical interface):

```yaml
constraints:
  - name: kcl
    expression: "positive.current + negative.current = 0"
    isAsserted: true
```

**Example** (`model/Interfaces/PowerInterface.md`):

```yaml
---
type: InterfaceDef
name: PowerInterface
ends:
  - name: sourceEnd
    typedBy: Interfaces::PowerPortDef
    direction: out
    isEnd: true
  - name: receiverEnd
    typedBy: Interfaces::PowerPortReceiverDef
    direction: in
    isEnd: true
---
Interface defining a valid power connection between a source and receiver.
```

#### 8.3.4 Operations on Ports and Interfaces

The `operations:` field declares callable operations (synchronous) or receptions (asynchronous) on a definition element. It is valid on `PortDef`, `InterfaceDef`, `ConnectionDef`, and any other definition element where behavioral contracts need to be specified. In SysMLv2 terms, each entry corresponds to an `OperationDef` (synchronous) or a `ReceptionDef` (when `isAsync: true`) owned by the enclosing definition.

**Operation entry schema:**

| Sub-field | YAML type | Required | Default | Description |
|---|---|---|---|---|
| `name` | string | **Required** | — | Operation name (must be a valid identifier) |
| `doc` | string | optional | absent | Inline documentation for this operation |
| `isQuery` | bool | optional | `false` | If `true`, the operation is side-effect free (read-only) |
| `isAsync` | bool | optional | `false` | If `true`, this is a reception — caller does not block waiting for a result; `returnType` must be absent |
| `parameters` | list | optional | `[]` | Typed, directional parameters (see table below) |
| `returnType` | string | optional | absent | Qualified name of the element typing the return value; mutually exclusive with `isAsync: true` |

**Parameter entry schema** (each entry in `parameters:`):**

| Sub-field | YAML type | Required | Default | Description |
|---|---|---|---|---|
| `name` | string | **Required** | — | Parameter name |
| `typedBy` | string | optional | absent | Qualified name of the type; validated by W404 |
| `direction` | string | optional | `in` | `in`, `out`, or `inout` |
| `multiplicity` | string | optional | `"1"` | Cardinality |
| `unit` | string | optional | absent | Unit (for scalar quantities) |

**Cross-reference rule:** If `typedBy` on a parameter or `returnType` does not resolve to a known element in the model, the validator emits **W404** (warning, not error, because standard library types such as `ScalarValues::Boolean` and `ScalarValues::Integer` may be unregistered external type libraries).

**Mutual-exclusion rule:** `isAsync: true` and `returnType:` are mutually exclusive. Specifying both is a modelling error.

**Example 1** (`model/Interfaces/TelemetryPortDef.md`) — PortDef with synchronous query and async reception:

```yaml
---
type: PortDef
name: TelemetryPortDef
supertype: Ports::Port
features:
  - name: packet
    typedBy: Items::TelemetryPacket
    direction: out
    multiplicity: "0..*"
operations:
  - name: requestLatest
    doc: "Synchronously return the most recent telemetry packet."
    isQuery: true
    isAsync: false
    parameters: []
    returnType: Items::TelemetryPacket
  - name: subscribe
    doc: "Register for periodic telemetry push at the given interval."
    isAsync: true
    parameters:
      - name: intervalMs
        typedBy: ScalarValues::Integer
        direction: in
---
Port definition for outgoing telemetry data. Components emitting telemetry expose this port.
```

**Example 2** — InterfaceDef with operations spanning both ends:

```yaml
---
type: InterfaceDef
name: CommandInterface
ends:
  - name: commander
    typedBy: Interfaces::ControlPortDef
  - name: executor
    typedBy: Interfaces::ControlPortDef
    isConjugated: true
operations:
  - name: executeCommand
    isAsync: false
    parameters:
      - name: cmd
        typedBy: Items::ControlCommand
        direction: in
    returnType: ScalarValues::Boolean
  - name: abort
    isAsync: true
    parameters: []
---
Interface definition for command dispatch between a commander and executor.
```

### 8.4 Connection, Binding, and Succession Elements

#### 8.4.1 Connection Schema

The `connections:` field on `PartDef`, `Part`, `ActionDef`, and `Action` elements specifies internal block connections. Each entry is a map.

**Binary shorthand** — use `from:` and `to:` for the common two-end case:

| Sub-field | YAML type | Required | Default | Description |
|---|---|---|---|---|
| `name` | string | optional | absent | Named connection usage |
| `typedBy` | string | optional | absent | Qualified name of the ConnectionDef (or InterfaceDef) typing this connection |
| `from` | string | **Required*** | — | Source feature chain; *required unless `ends:` is used* |
| `to` | string | **Required*** | — | Target feature chain; *required unless `ends:` is used* |
| `multiplicity` | string | optional | `"1"` | Cardinality |
| `metadata` | list | optional | absent | Metadata applications |

**N-ary form** — use `ends:` for connections with three or more ends, or when end names matter:

| Sub-field | YAML type | Required | Default | Description |
|---|---|---|---|---|
| `name` | string | optional | absent | Named connection usage |
| `typedBy` | string | optional | absent | Qualified name of the ConnectionDef typing this connection |
| `ends` | list of end-binding maps | **Required*** | — | *Required when `from:`/`to:` are absent* |
| `multiplicity` | string | optional | `"1"` | Cardinality |
| `metadata` | list | optional | absent | Metadata applications |

Each entry in `ends:` (connection usage end-binding):

| Sub-field | YAML type | Required | Description |
|---|---|---|---|
| `end` | string | **Required** | Name of the end feature on the `ConnectionDef` being bound |
| `binds` | string | **Required** | Feature chain to the port/usage being connected |

**Rule:** `from:` / `to:` and `ends:` are mutually exclusive within a single connection entry. `from:`/`to:` is syntactic sugar that expands to `ends: [{end: source, binds: <from>}, {end: target, binds: <to>}]`.

#### 8.4.2 `ConnectionDef` File

A `ConnectionDef` declares its ends using a top-level `ends:` field.

| Sub-field | YAML type | Required | Default | Description |
|---|---|---|---|---|
| `ends` | list of end-declaration maps | optional | absent | Named end features; two implicit unnamed ends exist if absent |

Each entry in `ends:` (connection definition end declaration):

| Sub-field | YAML type | Required | Default | Description |
|---|---|---|---|---|
| `name` | string | **Required** | — | End feature name |
| `typedBy` | string | optional | absent | PortDef or ItemDef typing this end |
| `multiplicity` | string | optional | `"1"` | End cardinality |
| `isAbstract` | bool | optional | `false` | Abstract end |
| `crossFeatures` | list of cross-feature maps | optional | absent | Features navigable across to the other end(s); see below |

Each entry in `crossFeatures:`:

| Sub-field | YAML type | Required | Description |
|---|---|---|---|
| `name` | string | **Required** | Cross feature name |
| `typedBy` | string | optional | Definition typing the cross feature |
| `crosses` | string | **Required** | Name of the opposite end this feature navigates to |
| `direction` | string | optional | `in` or `out` relative to this end; `out` means the item flows away from this end toward the crossed end |
| `multiplicity` | string | optional | Multiplicity of the cross feature |

**Binary example** (`model/Power/PowerConnection.md`):

```yaml
---
type: ConnectionDef
name: PowerConnection
supertype: Connections::BinaryConnection
ends:
  - name: source
    typedBy: Interfaces::PowerPortDef
  - name: receiver
    typedBy: Interfaces::PowerPortReceiverDef
---
A binary connection transmitting electrical power.
```

**N-ary example** (`model/Hydraulics/HydraulicJunction.md`):

```yaml
---
type: ConnectionDef
name: HydraulicJunction
ends:
  - name: supply
    typedBy: HydraulicPortDef
    multiplicity: "1"
  - name: return
    typedBy: HydraulicPortDef
    multiplicity: "1"
  - name: drain
    typedBy: HydraulicPortDef
    multiplicity: "0..1"
---
A three-way hydraulic junction connecting supply, return, and optional drain lines.
```

**Cross-feature example** (`model/Fluid/FluidPath.md`):

```yaml
---
type: ConnectionDef
name: FluidPath
ends:
  - name: source
    typedBy: FluidPortDef
    crossFeatures:
      - name: flow
        typedBy: FluidItemDef
        crosses: target
        direction: out
  - name: target
    typedBy: FluidPortDef
---
A directed fluid path carrying a FluidItem from source to target.
```

**N-ary usage** (inside a PartDef):

```yaml
connections:
  - typedBy: HydraulicJunction
    ends:
      - end: supply
        binds: pump.outPort
      - end: return
        binds: tank.inPort
      - end: drain
        binds: reservoir.drainPort
```

#### 8.4.3 Binding Connectors

The `bindingConnections:` field specifies equality bindings. Each entry:

| Sub-field | YAML type | Required | Description |
|---|---|---|---|
| `name` | string | optional | Named binding usage |
| `left` | string | **Required** | Feature chain for the left-hand side |
| `right` | string | **Required** | Feature chain for the right-hand side |

```yaml
bindingConnections:
  - left: engine.speed
    right: transmission.inputSpeed
```

#### 8.4.4 Successions

The `successionConnections:` field specifies ordering between occurrences/actions. Each entry:

| Sub-field | YAML type | Required | Description |
|---|---|---|---|
| `name` | string | optional | Named succession usage |
| `after` | string | **Required** | Feature chain of the preceding action/occurrence |
| `before` | string | **Required** | Feature chain of the succeeding action/occurrence |
| `guard` | string | optional | Boolean guard expression (opaque string) |
| `effect` | list of strings | optional | Qualified names of actions executed on the transition |

```yaml
successionConnections:
  - after: startEngine
    before: engageTransmission
    guard: "engine.rpm > 600"
```

### 8.5 Attribute and Enumeration Elements

#### 8.5.1 `AttributeDef`

Defines a data value type.

```yaml
---
type: AttributeDef
name: Temperature
supertype: ISQ::ThermodynamicTemperatureValue
features:
  - name: unit
    typedBy: SI::ThermodynamicTemperatureUnit
    value: SI::K
---
A thermodynamic temperature value defaulting to Kelvin.
```

#### 8.5.2 `EnumerationDef`

Defines a discrete-valued attribute. The enumerated values are listed in `values:`.

**Type-specific additional fields:**

| Field | YAML type | Required | Default | Description |
|---|---|---|---|---|
| `supertype` | string or list | optional | absent | Subclassification of another AttributeDef (not another EnumerationDef) |
| `values` | list of enumeration value maps | **Required** | — | The enumerated values |

Each entry in `values:`:

| Sub-field | YAML type | Required | Description |
|---|---|---|---|
| `name` | string | **Required** | Enumeration literal name |
| `value` | string | optional | Bound expression (when specializing a non-enum AttributeDef) |
| `valueKind` | string | optional | Binding semantics (default: `bound`); see §3.6.1 |
| `unit` | string | optional | Unit for quantity-valued enums |
| `metadata` | list | optional | Metadata on this literal |

**Example** (`model/VehicleSystem/VehicleStatus.md`):

```yaml
---
type: EnumerationDef
name: VehicleStatus
values:
  - name: parked
  - name: idling
  - name: moving
  - name: maintenance
---
Operational status of a vehicle.
```

**Example with bound values** (`model/Interfaces/StandardVoltage.md`):

```yaml
---
type: EnumerationDef
name: StandardVoltage
supertype: ISQ::VoltageValue
values:
  - name: lowVoltage
    value: "12.0"
    unit: SI::V
  - name: highVoltage
    value: "48.0"
    unit: SI::V
---
Standard vehicle bus voltage levels.
```

#### 8.5.3 `Attribute` Usage

```yaml
---
type: Attribute
name: vehicleStatus
typedBy: VehicleSystem::VehicleStatus
multiplicity: "1"
---
Current operational status of this vehicle instance.
```

### 8.6 Flow Elements

#### 8.6.1 `FlowDef`

Defines a flow connection type (items moving between ports).

**Type-specific additional fields:**

| Field | YAML type | Default | Description |
|---|---|---|---|
| `supertype` | string or list | absent | Supertype FlowDefs |
| `itemType` | string | absent | Shorthand: qualified name of the ItemDef carried by this flow; see payload note below |
| `ends` | list | absent | End port features (same schema as InterfaceDef ends) |

**Payload shorthand note:** `itemType: Fuel` is syntactic sugar for a `payload` feature that redefines `Transfers::Transfer::payload`:

```yaml
# shorthand
itemType: VehicleSystem::Fuel

# equivalent full form (use when explicit redefinition chain is needed)
features:
  - name: payload
    redefines: [Transfers::Transfer::payload]
    typedBy: VehicleSystem::Fuel
```

A parser that expands `itemType:` to the full form can participate in type-compatibility checking on connected ports. When specializing another `FlowDef`, use the full form with `redefines:` pointing to the supertype's `payload` feature to preserve the redefinition chain.

```yaml
---
type: FlowDef
name: FuelFlowDef
supertype: Flows::FlowConnection
itemType: VehicleSystem::Fuel
ends:
  - name: sourcePort
    typedBy: Interfaces::FuelPortDef
    direction: out
    isEnd: true
  - name: sinkPort
    typedBy: Interfaces::FuelPortReceiverDef
    direction: in
    isEnd: true
---
A flow connection carrying fuel between a fuel source and consumer.
```

#### 8.6.2 `flowConnections:` Schema

The `flowConnections:` field on `PartDef` or `Part` elements:

| Sub-field | YAML type | Required | Default | Description |
|---|---|---|---|---|
| `name` | string | optional | absent | Named flow usage |
| `typedBy` | string | optional | absent | Qualified name of FlowDef |
| `kind` | string | optional | `streaming` | Flow kind: `message`, `streaming`, or `succession` |
| `from` | string | **Required** | — | Source port feature chain |
| `to` | string | **Required** | — | Target port feature chain |
| `item` | string | optional | absent | Qualified name of item type flowing (overrides `FlowDef.itemType`) |
| `multiplicity` | string | optional | `"1"` | Cardinality |

**Flow kind semantics:**

| `kind` | SysML keyword | Ordering constraint |
|---|---|---|
| `message` | `message flow` | No ordering; item is transferred asynchronously |
| `streaming` | `flow` | Continuous while source and target are both active (default) |
| `succession` | `succession flow` | Source completes → transfer completes → target starts; combines data flow with sequencing |

### 8.7 Action and Behavior Elements

#### 8.7.1 `ActionDef`

Defines a behavioral step. Actions can have parameters (in/out/inout/return) and contain sub-actions connected by succession.

**Type-specific additional fields:**

| Field | YAML type | Default | Description |
|---|---|---|---|
| `supertype` | string or list | absent | Supertype ActionDefs |
| `parameters` | list of parameter maps | absent | Action parameters; see 8.7.2 |
| `returnType` | string | absent | Qualified name of the return type (for functions/calculations) |
| `features` | list | absent | Owned sub-actions, attribute usages, etc. (inline) |
| `subActions` | list of sub-action maps | absent | Named sub-action usages (alternative to file-per-subaction); see 8.7.3 |
| `successionConnections` | list | absent | Ordering between sub-actions; see 8.4.4 |
| `bindingConnections` | list | absent | Parameter bindings |
| `controlNodes` | list of control node maps | absent | Fork, join, decision, merge nodes; see 8.7.4 |
| `body` | string | absent | Implementation body in a named language (opaque); mutually exclusive with `subActions` |
| `bodyLanguage` | string | absent | Language name for `body`: `"alf"`, `"ocl"`, `"sysml"`, or user-defined |

#### 8.7.2 Parameter Schema

Each entry in `parameters:`:

| Sub-field | YAML type | Required | Default | Description |
|---|---|---|---|---|
| `name` | string | **Required** | — | Parameter name |
| `typedBy` | string | optional | absent | Typed by a definition |
| `direction` | string | **Required** | — | `in`, `out`, `inout`, or `return` |
| `multiplicity` | string | optional | `"1"` | Cardinality |
| `isReadonly` | bool | optional | `false` | Readonly flag |

#### 8.7.3 Sub-Action Schema

Each entry in `subActions:`:

| Sub-field | YAML type | Required | Default | Description |
|---|---|---|---|---|
| `name` | string | **Required** | — | Sub-action usage name |
| `typedBy` | string | optional | absent | Qualified name of the ActionDef |
| `kind` | string | optional | `Action` | `Action`, `PerformAction`, `SendAction`, `AcceptAction`, `AssignmentAction`, `IfAction`, `LoopAction`, `TerminateAction` |
| `parameters` | list | optional | absent | Parameter bindings for this invocation |
| `guard` | string | optional | absent | Boolean guard expression (opaque) |
| `multiplicity` | string | optional | `"1"` | Cardinality |
| `isParallel` | bool | optional | `false` | For `LoopAction` and parallel composition |

**Sub-action kinds:**

| `kind` value | SysML equivalent | Notes |
|---|---|---|
| `Action` | `action` usage | Generic action usage |
| `PerformAction` | `perform action` | Reuses a separately defined ActionDef |
| `SendAction` | `send ... via ...` | Sends a message/item through a port |
| `AcceptAction` | `accept ... via ...` | Receives trigger or message |
| `AssignmentAction` | `:=` assignment | Assigns a value to a target |
| `IfAction` | `if ... then ... else ...` | Conditional branching |
| `LoopAction` | `loop ...` / `while ...` / `for ...` | Looping |
| `TerminateAction` | `terminate` | Terminates the containing action |
| `DecisionNode` | (control node) | Exclusive branching point |
| `ForkNode` | (control node) | Parallel split |
| `JoinNode` | (control node) | Parallel join |
| `MergeNode` | (control node) | Merge of alternate paths |

#### 8.7.4 Control Node Schema

Each entry in `controlNodes:`:

| Sub-field | YAML type | Required | Description |
|---|---|---|---|
| `name` | string | **Required** | Node name |
| `kind` | string | **Required** | `DecisionNode`, `ForkNode`, `JoinNode`, `MergeNode` |

**Example** (`model/VehicleBehavior/ProvidePower.md`):

```yaml
---
type: ActionDef
name: ProvidePower
parameters:
  - name: requestedPower
    typedBy: ISQ::PowerValue
    direction: in
  - name: actualPower
    typedBy: ISQ::PowerValue
    direction: out
subActions:
  - name: startEngine
    typedBy: VehicleBehavior::StartEngine
    kind: Action
  - name: regulatePower
    typedBy: VehicleBehavior::RegulatePower
    kind: Action
    parameters:
      - name: target
        value: requestedPower
  - name: measureOutput
    typedBy: VehicleBehavior::MeasurePower
    kind: Action
successionConnections:
  - after: startEngine
    before: regulatePower
  - after: regulatePower
    before: measureOutput
bindingConnections:
  - left: measureOutput.measuredPower
    right: actualPower
---
Action definition for providing engine power in response to a power request.
```

#### 8.7.5 `Action` Usage

Same as `ActionDef` but with `type: Action`, plus `typedBy:` referencing the definition:

```yaml
---
type: Action
name: providePower
typedBy: VehicleBehavior::ProvidePower
multiplicity: "1"
---
```

#### 8.7.6 `SendAction` and `AcceptAction`

When `subActions` entries have `kind: SendAction` or `kind: AcceptAction`, the following additional sub-fields apply:

| Sub-field | YAML type | Description |
|---|---|---|
| `payload` | string | Qualified name of the item/message being sent or accepted |
| `via` | string | Feature chain to the port through which the send/accept occurs |
| `trigger` | map | For `AcceptAction` — trigger specification; see 8.7.7 |

When `subActions` entries have `kind: IfAction`, the following additional sub-fields apply:

| Sub-field | YAML type | Required | Description |
|---|---|---|---|
| `condition` | string | **Required** | Boolean condition expression (opaque string) |
| `then` | list of sub-action maps | **Required** | Sub-actions executed when condition is true; same schema as `subActions:` |
| `else` | list of sub-action maps | optional | Sub-actions executed when condition is false |

**Example:**

```yaml
subActions:
  - name: checkFuel
    kind: IfAction
    condition: "fuel.level > 0"
    then:
      - name: startEngine
        kind: PerformAction
        typedBy: VehicleBehavior::StartEngine
    else:
      - name: signalEmpty
        kind: SendAction
        payload: VehicleCommands::FuelEmptySignal
        via: alertPort
```

When `subActions` entries have `kind: LoopAction`, the following additional sub-fields apply:

| Sub-field | YAML type | Required | Default | Description |
|---|---|---|---|---|
| `loopKind` | string | optional | `while` | Loop form: `while`, `until`, or `for` |
| `condition` | string | optional* | — | Boolean expression; *required for `while` and `until` forms |
| `variable` | string | optional* | — | Iteration variable name; *required for `for` form |
| `sequence` | string | optional* | — | Feature chain to the collection being iterated; *required for `for` form |
| `body` | list of sub-action maps | **Required** | — | Sub-actions executed each iteration; same schema as `subActions:` |

**Examples:**

```yaml
subActions:
  # while form
  - name: waitForReady
    kind: LoopAction
    loopKind: while
    condition: "not system.ready"
    body:
      - name: pollStatus
        kind: PerformAction
        typedBy: VehicleBehavior::PollSystemStatus

  # until form
  - name: retryConnect
    kind: LoopAction
    loopKind: until
    condition: "connection.established"
    body:
      - name: attemptConnect
        kind: PerformAction
        typedBy: VehicleBehavior::AttemptConnection

  # for form
  - name: initSensors
    kind: LoopAction
    loopKind: for
    variable: sensor
    sequence: "sensors"
    body:
      - name: initSensor
        kind: PerformAction
        typedBy: VehicleBehavior::InitializeSensor
```

When `subActions` entries have `kind: AssignmentAction`, the following additional sub-fields apply:

| Sub-field | YAML type | Required | Default | Description |
|---|---|---|---|---|
| `target` | string | **Required** | — | Feature chain identifying the occurrence whose feature is being assigned (e.g., `self`, `engine`) |
| `referent` | string | **Required** | — | Feature chain relative to `target` naming the feature to assign |
| `value` | string | **Required** | — | Value expression (opaque string) |
| `valueKind` | string | optional | `bound` | Binding semantics: `bound`, `initial`, `default-bound`, `default-initial`; see §3.6.1 |

**Example:**

```yaml
subActions:
  - name: setThrottle
    kind: AssignmentAction
    target: self
    referent: throttlePosition
    value: "requestedPower / maxPower"
    valueKind: initial
  - name: resetCounter
    kind: AssignmentAction
    target: controller
    referent: cycleCount
    value: "0"
```

#### 8.7.7 Trigger Specification

A `trigger` map within an `AcceptAction` entry:

| Sub-field | YAML type | Description |
|---|---|---|
| `kind` | string | `timeOut`, `message`, `change` |
| `when` | string | For `timeOut`: expression giving the timeout duration (opaque) |
| `payload` | string | For `message`: qualified name of the payload type |
| `condition` | string | For `change`: boolean change condition expression (opaque) |

### 8.8 State Machine Elements

#### 8.8.1 `StateDef`

Defines a state node in a state machine. States can own entry/do/exit behaviors and transitions.

**Type-specific additional fields:**

| Field | YAML type | Default | Description |
|---|---|---|---|
| `supertype` | string or list | absent | Supertype StateDefs |
| `isParallel` | bool | `false` | Parallel (orthogonal) state with concurrent sub-regions |
| `entryAction` | string or map | absent | Entry behavior; qualified name of ActionDef or inline sub-action map |
| `doAction` | string or map | absent | Do behavior; qualified name of ActionDef or inline sub-action map |
| `exitAction` | string or map | absent | Exit behavior; qualified name of ActionDef or inline sub-action map |
| `subStates` | list of sub-state maps | absent | Nested substates; see 8.8.2 |
| `transitions` | list of transition maps | absent | Outgoing transitions; see 8.8.3 |

#### 8.8.2 Sub-State Schema

Each entry in `subStates:`:

| Sub-field | YAML type | Required | Description |
|---|---|---|---|
| `name` | string | **Required** | Substate usage name |
| `typedBy` | string | optional | Qualified name of a StateDef |
| `isInitial` | bool | optional | `true` marks the initial state |
| `isFinal` | bool | optional | `true` marks a final (terminal) state |
| `entryAction` | string or map | optional | Entry behavior |
| `doAction` | string or map | optional | Do behavior |
| `exitAction` | string or map | optional | Exit behavior |
| `transitions` | list | optional | Transitions from this substate |

#### 8.8.3 Transition Schema

Each entry in `transitions:`:

| Sub-field | YAML type | Required | Default | Description |
|---|---|---|---|---|
| `name` | string | optional | absent | Named transition usage |
| `source` | string | optional | absent | Local name or qualified name of the source state; required when the transition is not lexically nested inside its source state |
| `target` | string | **Required** | — | Local name or qualified name of the target state |
| `accept` | string or map | optional | absent | Accepter specification; string shorthand is equivalent to `{payload: <string>}`; see accept sub-schema below |
| `guard` | string | optional | absent | Boolean guard expression evaluated during source state performance (opaque string) |
| `effect` | string or map | optional | absent | Effect action executed during the transition; string is a qualified name shorthand; see effect sub-schema below |

**`accept:` sub-schema** (the transition's AcceptAction):

| Sub-field | YAML type | Required | Description |
|---|---|---|---|
| `payload` | string | optional | Qualified name of the item/message type accepted |
| `via` | string | optional | Feature chain to the port on which the trigger arrives |

**`effect:` sub-schema** (the transition's effect action usage):

| Sub-field | YAML type | Required | Description |
|---|---|---|---|
| `name` | string | optional | Name of the effect action usage |
| `typedBy` | string | optional | Qualified name of the ActionDef typing the effect |
| `bindingConnections` | list | optional | Parameter bindings for the effect action; see §8.4.3 |

**String shorthands:**
- `accept: StartCommand` expands to `accept: {payload: StartCommand}`
- `effect: VehicleBehavior::StartEngine` expands to `effect: {typedBy: VehicleBehavior::StartEngine}`

**Example** (`model/VehicleBehavior/VehicleStates.md`):

```yaml
---
type: StateDef
name: VehicleStates
subStates:
  - name: off
    isInitial: true
    transitions:
      - target: on
        accept:
          payload: VehicleCommands::StartCommand
          via: controlPort
        guard: "fuel.level > 0"
        effect:
          name: doStart
          typedBy: VehicleBehavior::StartEngine
  - name: on
    doAction: VehicleBehavior::ProvidePower
    transitions:
      - target: off
        accept: VehicleCommands::StopCommand
        effect: VehicleBehavior::StopEngine
      - target: fault
        accept:
          payload: VehicleCommands::FaultSignal
        guard: "engine.temperature > maxTemperature"
  - name: fault
    isFinal: false
    entryAction: VehicleBehavior::RecordFault
    transitions:
      - target: off
        accept: VehicleCommands::ResetCommand
---
State machine for the top-level operational states of a vehicle.
```

**Top-level transition** (not nested inside source state):

```yaml
transitions:
  - name: faultRecovery
    source: fault
    target: off
    accept: VehicleCommands::ResetCommand
    guard: "diagnostics.cleared"
    effect: VehicleBehavior::ClearFaultLog
```

#### 8.8.4 `ExhibitState` Usage

An `ExhibitState` usage declares that a part or occurrence **exhibits** a state machine defined elsewhere. It is always referential (`isReference: true` is implied and must not be set to `false`).

**Shorthand** — sufficient when no bindings or redefinition are needed:

```yaml
exhibitsStates:
  - VehicleStates::OperatingState
```

**Inline feature form** — use when naming the usage, adding bindings, or redefining:

```yaml
features:
  - name: operatingMode
    type: ExhibitState
    typedBy: VehicleStates::OperatingState
    multiplicity: "1"
    redefines: [Parts::Part::exhibitedStates]
    bindingConnections:
      - left: operatingMode.speed
        right: self.currentSpeed
```

**File form** (`model/Vehicle/ExhibitOperatingMode.md`):

```yaml
---
type: ExhibitState
name: operatingMode
typedBy: VehicleStates::OperatingState
multiplicity: "1"
bindingConnections:
  - left: operatingMode.speed
    right: self.currentSpeed
---
Exhibits the top-level operating state machine, binding speed to the vehicle's current speed.
```

**Type-specific rules:**
- `isReference` is always `true`; the parser must enforce this and may emit a warning if explicitly set to `false`.
- `typedBy` is required (an exhibit-state usage must reference a `StateDef`).
- `bindingConnections` follow the same schema as §8.4.3.

### 8.9 Calculation Elements

#### 8.9.1 `CalculationDef`

Defines a parameterized calculation that returns a value.

**Type-specific additional fields:**

| Field | YAML type | Default | Description |
|---|---|---|---|
| `supertype` | string or list | absent | Supertype CalculationDefs |
| `parameters` | list | absent | Input/output parameters; same schema as ActionDef parameters (Section 8.7.2) |
| `returnType` | string | absent | Qualified name of the return type |
| `body` | string | absent | Expression body (opaque string in `bodyLanguage`) |
| `bodyLanguage` | string | absent | Language of `body`: `"kerml"`, `"ocl"`, `"alf"`, `"python"`, etc. |

**Example** (`model/Analysis/FuelEconomy.md`):

```yaml
---
type: CalculationDef
name: FuelEconomyCalc
parameters:
  - name: distance
    typedBy: ISQ::LengthValue
    direction: in
  - name: fuelConsumed
    typedBy: ISQ::VolumeValue
    direction: in
  - name: economy
    typedBy: ISQ::FuelEconomyValue
    direction: return
bodyLanguage: kerml
body: |
  economy = distance / fuelConsumed
---
Calculates fuel economy as distance divided by fuel consumed.
```

#### 8.9.2 `Calculation` Usage

A `Calculation` usage invokes a `CalculationDef` in a specific context. It has the same structural fields as `CalculationDef` plus `typedBy:` identifying the definition being invoked. The `parameters:` and `returnType:` are inherited from the typed definition and need not be repeated unless overriding.

| Field | YAML type | Default | Description |
|---|---|---|---|
| `typedBy` | string | absent | Qualified name of the `CalculationDef` this usage invokes |
| `multiplicity` | string | `"1"` | Cardinality |
| `bindingConnections` | list | absent | Binding connectors that supply arguments to the calculation's parameters; same schema as §8.4.3 |

```yaml
---
type: Calculation
name: computeThrust
typedBy: PropulsionCalcs::ThrustCalculation
multiplicity: "1"
bindingConnections:
  - left: computeThrust.massFlow
    right: self.fuelMassFlow
  - left: computeThrust.exhaustVelocity
    right: self.nozzleExitVelocity
---
Invokes the ThrustCalculation with this engine's fuel mass flow and nozzle exit velocity.
```

### 8.10 Constraint Elements

#### 8.10.1 `ConstraintDef`

Defines a boolean-valued condition that can be evaluated.

**Type-specific additional fields:**

| Field | YAML type | Default | Description |
|---|---|---|---|
| `supertype` | string or list | absent | Supertype ConstraintDefs |
| `parameters` | list | absent | Parameters for parameterized constraints |
| `expression` | string | absent | Constraint expression body (opaque string) |
| `expressionLanguage` | string | `"ocl"` | Language for `expression` |

**Example** (`model/Requirements/MassConstraint.md`):

```yaml
---
type: ConstraintDef
name: MassConstraint
parameters:
  - name: actualMass
    typedBy: ISQ::MassValue
    direction: in
  - name: maxMass
    typedBy: ISQ::MassValue
    direction: in
expression: "actualMass <= maxMass"
expressionLanguage: ocl
---
Constraint that the actual mass does not exceed the maximum allowed mass.
```

#### 8.10.2 `Constraint` Usage and `assert`

A `Constraint` usage applies a ConstraintDef within context:

```yaml
---
type: Constraint
name: massCheck
typedBy: Requirements::MassConstraint
---
```

The `assert` keyword in SysML is represented by `isAsserted: true`:

```yaml
---
type: Constraint
name: massCheck
typedBy: Requirements::MassConstraint
isAsserted: true
---
```

| Field | YAML type | Default | Description |
|---|---|---|---|
| `isAsserted` | bool | `false` | When `true`, corresponds to `assert constraint` — the constraint must hold |
| `isNegated` | bool | `false` | When `true`, asserts the negation of the constraint |

### 8.11 Requirement Elements

#### 8.11.1 `RequirementDef`

Defines a class of requirements.

**Type-specific additional fields:**

| Field | YAML type | Default | Description |
|---|---|---|---|
| `supertype` | string or list | absent | Supertype RequirementDefs |
| `subject` | string | absent | Qualified name of the *type* (e.g., a `PartDef`) that the implicit `subject` feature is typed by. The subject feature is implicitly named `subject` with multiplicity `"1"` and represents the element that must satisfy this requirement. |
| `actors` | list of strings | absent | Qualified names of stakeholder parts (actors for use-case style requirements) |
| `stakeholders` | list of strings | absent | Qualified names of `PartDef` elements representing stakeholders |
| `concerns` | list of strings | absent | Qualified names of `ConcernDef` or `Concern` elements addressed by this requirement |
| `framedConcerns` | list of strings | absent | Concerns that this requirement explicitly frames (captures but does not resolve) |
| `requires` | list of constraint maps | absent | Formal constraint clauses; see 8.11.2 |
| `assume` | list of constraint maps | absent | Assumption clauses |
| `parameters` | list | absent | Requirement parameters (typed attributes that parameterize the requirement) |
| `derivedFrom` | list of strings | absent | Qualified names of source requirements this requirement is derived from (`deriveReqt` relationship). A parser creates implicit `Dependency` edges of type `RequirementDerivation` (from the `RequirementsVerification` standard library) for each entry. |

#### 8.11.2 Requirement Constraint Clause Schema

Each entry in `requires:` or `assume:`:

| Sub-field | YAML type | Required | Description |
|---|---|---|---|
| `typedBy` | string | optional | Qualified name of a ConstraintDef |
| `expression` | string | optional | Inline constraint expression (opaque) |
| `expressionLanguage` | string | optional | Language for `expression`; default `"ocl"` |
| `isAsserted` | bool | optional | Whether asserted (default `true` for `requires:`, `false` for `assume:`) |

**Example** (`model/Requirements/MassRequirementDef.md`):

```yaml
---
type: RequirementDef
name: MassRequirementDef
subject: VehicleSystem::Vehicle
parameters:
  - name: maxMass
    typedBy: ISQ::MassValue
    direction: in
requires:
  - typedBy: Requirements::MassConstraint
stakeholders:
  - Stakeholders::VehicleEngineer
concerns:
  - Concerns::VehicleWeight
---
Requirement that a vehicle's mass shall not exceed a specified maximum.
The maximum mass is a parameter of this requirement definition.
```

#### 8.11.3 `Requirement` Usage

```yaml
---
type: Requirement
name: vehicleMassReq
typedBy: Requirements::MassRequirementDef
features:
  - name: maxMass
    typedBy: ISQ::MassValue
    value: "1500.0"
    unit: SI::kg
---
The vehicle mass shall not exceed 1500 kg.
```

#### 8.11.4 Satisfying and Verifying Requirements

**Satisfaction** is declared on the element satisfying the requirement (typically a Part or PartDef):

```yaml
# On Vehicle.md or vehicle_b.md
satisfies:
  - Requirements::vehicleMassReq
```

**Verification** is declared on a VerificationCase:

```yaml
# On a VerificationCase file
verifies:
  - Requirements::vehicleMassReq
```

Top-level satisfaction and verification fields on any element:

| Field | YAML type | Default | Description |
|---|---|---|---|
| `satisfies` | list of strings | absent | Qualified names of Requirement usages this element satisfies |
| `implementedBy` | string or list | absent | Path(s) to the source artifact(s) realising this `Part`/`PartDef`. Resolved like `sourceFile`; missing local paths emit W023 (§12.8) |
| `verifiedBy` | list of strings | absent | Qualified names of VerificationCase usages that verify this requirement |

#### 8.11.5 `ConcernDef` and `Concern`

`ConcernDef` specializes `RequirementDef` in SysML and therefore inherits all `RequirementDef` fields. The `stakeholders:` field identifies who holds the concern. The following fields apply with the same semantics as on `RequirementDef`:

| Field | YAML type | Default | Description |
|---|---|---|---|
| `supertype` | string or list | absent | Supertype ConcernDefs or RequirementDefs |
| `subject` | string | absent | Qualified name of the *type* (e.g., a `PartDef`) that this concern is about; the implicit `subject` feature has multiplicity `"1"` |
| `stakeholders` | list of strings | absent | Qualified names of `PartDef` elements representing the stakeholders who hold this concern |
| `requires` | list of constraint maps | absent | Formal constraint clauses the concern asserts (same schema as §8.11.2) |
| `assume` | list of constraint maps | absent | Assumption clauses (same schema as §8.11.2) |
| `parameters` | list | absent | Concern parameters |

```yaml
---
type: ConcernDef
name: VehicleWeightConcern
subject: VehicleSystem::Vehicle
stakeholders:
  - Stakeholders::VehicleEngineer
  - Stakeholders::CustomerRep
requires:
  - expression: "subject.mass <= 2000.0"
    expressionLanguage: ocl
---
Concern regarding the total weight of the vehicle impacting fuel economy and performance.
```

#### 8.11.6 Native `Requirement` Type

The native `Requirement` type is a **first-class element** designed for safety-critical and regulated engineering contexts where every requirement must carry a stable, opaque identifier that never changes (regardless of file renames or restructuring), a structured lifecycle status, and a normative textual statement in the Markdown body.

This is distinct from the SysML-usage `Requirement` (§8.11.3), which is typed by a `RequirementDef`. Native requirements are dispatched by the parser based on the `id:` field matching the `REQ-*` pattern (§11.5 step 0).

**Frontmatter fields:**

| Field | Type | Required | Description |
|---|---|---|---|
| `type` | literal `Requirement` | **Required** | Discriminator. |
| `id` | string | **Required** | Stable opaque ID matching `^REQ(-[A-Z0-9]{2,12})+-[0-9]{3}$`. Unique across the model. Never changes. |
| `title` | string | **Required** | One-line summary. Max 120 chars. No newlines. |
| `status` | enum | **Required** | Lifecycle state: `draft`, `review`, `approved`, `implemented`, `verified`. |
| `derivedFrom` | list of id-or-qualname | optional | IDs (`REQ-*`) or qualified names of parent Requirements. Absent = stakeholder-level requirement. |
| `silLevel` | integer 1–4 | optional | IEC 61508 SIL level. Mutually exclusive with `asilLevel` — do not set both (W006). |
| `asilLevel` | enum A\|B\|C\|D | optional | ISO 26262 ASIL level. Mutually exclusive with `silLevel` — do not set both (W006). |
| `plLevel` | enum a\|b\|c\|d\|e | optional | ISO 13849-1 Performance Level. Mutually exclusive with `asilLevel`/`silLevel`. |
| `derivedFromSafetyGoal` | string | optional | ID or qualified name of the `SafetyGoal` that motivated this requirement (§8.18.2). When set the SafetyGoal's integrity level must also appear on this element (E841). |
| `derivedFromSecurityGoal` | string | optional | ID or qualified name of the `CybersecurityGoal` that motivated this requirement (§8.18.4). Implies `verificationMethod:` should be set (W807). |
| `verificationMethod` | enum | optional | How this requirement will be verified: `test`, `inspection`, `analysis`, or `demonstration`. Required for ASIL B/C/D requirements (W701). |
| `wcet` | string | optional | WCET claim (opaque). E.g. `"O(1)"`, `"≤ 200 cycles @ 72 MHz"`. |
| `tags` | list of strings | optional | Free labels for filtering/grouping. |
| `reqDomain` | enum | optional | Engineering domain of this requirement: `system`, `hardware`, or `software`. Leaf requirements at `implemented`/`verified` status should be refined to `hardware` or `software` (warning `W302`). |
| `breakdownAdr` | string | optional | `ADR-*` id or qualified name of the ADR documenting the rationale for this requirement's derivation from its parent(s). Required when `derivedFrom:` is non-empty (error `E310`). Also required when the requirement's integrity level is lower than its source's (W808; see §12.7). |

**Status values:**

| Value | Meaning |
|---|---|
| `draft` | Being authored; not yet reviewed. |
| `review` | Under formal review. |
| `approved` | Baseline-approved; may be implemented. |
| `implemented` | Implementation exists. |
| `verified` | Covered by at least one `active` TestCase. |

**ID pattern:** `^REQ(-[A-Z0-9]{2,12})+-[0-9]{3}$`
- Prefix `REQ`, one or more uppercase-alphanumeric segments (2–12 chars each), three-digit suffix.
- Examples: `REQ-SCHED-001`, `REQ-SCHED-BITMAP-001`, `REQ-BRAKE-CTRL-003`

**Body structure:**

The body is free-form Markdown. The **normative statement** is the leading prose — all content before the first `##` heading. The parser extracts this as the canonical requirement text.

Recommended layout:

```
<normative statement — paragraphs containing at least one "shall">

## Rationale
<why this requirement exists; safety argument>

## Notes
<WCET analysis, implementation pointers, related standards clauses>
```

Validation: normative text must be non-empty (`E012`); should contain `shall` (warning `W001` if absent).

**Complete example:**

```markdown
---
type: Requirement
id: REQ-SCHED-BITMAP-001
title: "Bitmap-based O(1) priority selection"
status: approved
silLevel: 4
derivedFrom:
  - REQ-SCHED-001
breakdownAdr: ADR-SW-SCHED-001
verificationMethod: test
wcet: "O(1)"
tags:
  - scheduler
  - timing-critical
---

The scheduler shall maintain a 32-bit priority bitmap and select the
highest-priority ready thread in O(1) time using a count-leading-zeros
operation. This guarantee shall hold regardless of the number of threads
or their distribution across priority levels.

## Rationale

A linear-scan selection algorithm produces a WCET proportional to
`MAX_THREADS`, violating the fixed-time scheduling requirement of
IEC 61508 SIL 4 timing analysis.

## Notes

Implementation: `kernel/src/scheduler/ready_queue.rs` — `ReadyQueue::dequeue_highest`.
```

**Coexistence with SysML elements:** Native `Requirement` files live in the same directory tree as `RequirementDef`, `Part`, and all other SysML element types. The walker dispatches on `type: Requirement` with an `id:` field matching the REQ pattern; files with `type: Requirement` but no `id:` (or with `typedBy:`) are treated as SysML requirement usages (§8.11.3).

### 8.12 Case Elements (Analysis, Verification, Use Case)

#### 8.12.1 Common Case Fields

All case types (`CaseDef`, `AnalysisCaseDef`, `VerificationCaseDef`, `UseCaseDef` and their usages) share these fields in addition to common action fields:

| Field | YAML type | Default | Description |
|---|---|---|---|
| `subject` | string | absent | Qualified name of the subject of the case |
| `actors` | list of strings | absent | Qualified names of actor parts |
| `objectives` | list of strings or inline objective maps | absent | Case objectives |
| `result` | string | absent | Qualified name of the result type for analysis/verification cases |

#### 8.12.2 `AnalysisCaseDef`

```yaml
---
type: AnalysisCaseDef
name: FuelEconomyAnalysis
subject: VehicleSystem::Vehicle
objectives:
  - "Determine fuel economy under standard drive cycle"
result: Analysis::FuelEconomyResult
subActions:
  - name: runSimulation
    typedBy: Analysis::DriveSimulation
    kind: Action
  - name: computeEconomy
    typedBy: Analysis::FuelEconomyCalc
    kind: PerformAction
successionConnections:
  - after: runSimulation
    before: computeEconomy
---
Analysis case for calculating the fuel economy of the vehicle
under a standard EPA drive cycle.
```

#### 8.12.3 `VerificationCaseDef`

**Additional fields:**

| Field | YAML type | Default | Description |
|---|---|---|---|
| `verifies` | list of strings | absent | Qualified names of RequirementDefs/Requirements this case verifies |
| `verdictExpression` | string | absent | Expression evaluating to a pass/fail/inconclusive verdict (opaque) |
| `verdictType` | string | `VerificationCases::VerdictKind` | Qualified name of the enumeration type for the verdict; defaults to the standard library `VerdictKind` (pass/fail/inconclusive/error) |
| `returnType` | string | absent | Qualified name of the result type returned by the case; if absent, defaults to `verdictType` |

```yaml
---
type: VerificationCaseDef
name: MassVerificationTest
subject: VehicleSystem::vehicle_b
verifies:
  - Requirements::vehicleMassReq
subActions:
  - name: measureMass
    typedBy: TestActions::MeasureMass
    kind: Action
  - name: checkMass
    typedBy: Requirements::MassConstraint
    kind: Action
verdictExpression: "measureMass.result <= vehicleMassReq.maxMass"
---
Verification test for the vehicle mass requirement.
The vehicle is placed on a calibrated scale and the measured mass
is compared against the requirement threshold.
```

#### 8.12.4 `UseCaseDef`

**Additional fields:**

| Field | YAML type | Default | Description |
|---|---|---|---|
| `actors` | list of strings | absent | Qualified names of actor parts |
| `includes` | list of strings | absent | Qualified names of included UseCaseDefs |
| `extends` | list of extension maps | absent | Conditional extensions to other use cases |
| `extensionPoints` | list of strings | absent | Named extension points within this use case that other use cases may extend |

Each entry in `extends:`:

| Sub-field | YAML type | Required | Description |
|---|---|---|---|
| `target` | string | **Required** | Qualified name of the use case being extended |
| `extensionPoint` | string | optional | Name of the extension point being targeted on the extended use case |
| `condition` | string | optional | Boolean condition under which the extension applies (opaque string) |

**Example:**

```yaml
extends:
  - target: BasicDrive::DriveVehicle
    extensionPoint: assistanceHook
    condition: "adaptiveCruise.enabled"
extensionPoints:
  - assistanceHook
```

```yaml
---
type: UseCaseDef
name: TransportPassenger
subject: VehicleSystem::Vehicle
actors:
  - Stakeholders::Driver
  - Stakeholders::Passenger
objectives:
  - "Transport passengers from origin to destination safely"
includes:
  - VehicleUseCases::StartVehicle
  - VehicleUseCases::NavigateRoute
subActions:
  - name: boardPassengers
    typedBy: VehicleActions::BoardPassengers
    kind: Action
  - name: drive
    typedBy: VehicleActions::Drive
    kind: Action
  - name: exitPassengers
    typedBy: VehicleActions::ExitPassengers
    kind: Action
successionConnections:
  - after: boardPassengers
    before: drive
  - after: drive
    before: exitPassengers
---
Use case for transporting passengers using the vehicle.
```

#### 8.12.5 Native `TestCase` Type

The native `TestCase` type is a **first-class element** for structured, executable test specifications. It carries a stable ID, a lifecycle status, a verification level, machine-readable Gherkin scenarios, and a link to implementation source. It is the counterpart to the native `Requirement` type (§8.11.6).

**Frontmatter fields:**

| Field | Type | Required | Description |
|---|---|---|---|
| `type` | literal `TestCase` | **Required** | Discriminator. |
| `id` | string | **Required** | Stable opaque ID matching `^TC(-[A-Z0-9]{2,12})+-[0-9]{3}$`. Unique across the model. |
| `title` | string | **Required** | One-line summary. Max 120 chars. |
| `status` | enum | **Required** | Lifecycle state: `draft`, `review`, `approved`, `active`, `retired`. |
| `testLevel` | enum L1–L5 | **Required** | Verification layer (see table below). |
| `verifies` | list of id-or-qualname | **Required** | Requirements this test exercises. At least one entry. All must resolve. |
| `sourceFile` | string | optional | Workspace-relative path to the implementation file. Forward slashes. |
| `testFunctions` | list | optional | Structured references to individual test functions (see below). |
| `tags` | list of strings | optional | Free labels. |

**Status values:**

| Value | Meaning |
|---|---|
| `draft` | Scenarios written; not yet reviewed. |
| `review` | Under review. |
| `approved` | Approved; implementation expected. |
| `active` | Implementation passes in CI. |
| `retired` | Superseded; no longer executed. |

**`testLevel` values:**

| Value | Layer | Typical runner |
|---|---|---|
| `L1` | Unit test — hosted | `cargo test` on x86-64 |
| `L2` | Property-based — hosted | `cargo test` with proptest / bolero |
| `L3` | Formal proof | `cargo kani` |
| `L4` | Integration — emulator | `cargo test --target thumbvNm-none-eabi` + QEMU |
| `L5` | Hardware-in-the-loop | HIL bench + probe-rs |

**ID pattern:** `^TC(-[A-Z0-9]{2,12})+-[0-9]{3}$`
- Examples: `TC-SCHED-BITMAP-001`, `TC-BRAKE-FADE-002`

**`testFunctions` structure:**

Each entry is a plain string (function path) or an object linking a function to a specific Gherkin scenario:

```yaml
testFunctions:
  - "ready_queue_tests::tests::highest_urgency_first"
  - function: "ready_queue_tests::tests::bitmap_cleared_on_last_dequeue"
    scenario: "Bitmap bit is cleared when the last thread at a priority dequeues"
```

- `function` — fully-qualified test path. Required in object form.
- `scenario` — must exactly match a `Scenario:` or `Scenario Outline:` title in a Gherkin block in this file. Unresolved strings are model error `E106`.

**Body structure:**

Two zones in order:

1. **Preamble** (optional) — free-form Markdown before the first Gherkin block.
2. **Gherkin blocks** — one or more fenced blocks tagged ` ```gherkin `. At least one is required (`E011`).

Gherkin block rules:
- Language tag must be exactly `gherkin` (not `cucumber`, `feature`, or blank).
- Each fenced block is an independent Gherkin document.
- `Feature:` is required in the **first** block; subsequent blocks may omit it.
- `Background:` applies only within its own block.
- `Scenario Outline:` must contain at least one `Examples:` table (`E014`).
- Permitted step keywords: `Given`, `When`, `Then`, `And`, `But`. The `*` bullet keyword is not supported.
- Doc strings (`"""`) and data tables (`| col |`) are permitted within steps.
- Gherkin tags (`@tagname`) on `Feature:`, `Scenario:`, or `Scenario Outline:` are surfaced by tooling.

**Complete example:**

````markdown
---
type: TestCase
id: TC-SCHED-BITMAP-001
title: "Bitmap correctly selects highest-priority ready thread"
status: active
testLevel: L1
verifies:
  - REQ-SCHED-BITMAP-001
sourceFile: "tests/host/src/ready_queue_tests.rs"
testFunctions:
  - function: "ready_queue_tests::tests::highest_urgency_first"
    scenario: "Highest-urgency thread wins when multiple priorities are ready"
  - function: "ready_queue_tests::tests::bitmap_cleared_on_last_dequeue"
    scenario: "Bitmap bit is cleared when the last thread at a priority dequeues"
tags:
  - scheduler
---

Unit tests for the priority bitmap on the hosted port (x86-64).

Run: `cargo test -p tests-host --target x86_64-unknown-linux-gnu -- ready_queue_tests`

```gherkin
Feature: Priority bitmap selection

  Background:
    Given the kernel is initialised with the hosted port
    And an empty ready queue with 32 priority levels

  Scenario: Highest-urgency thread wins when multiple priorities are ready
    Given thread A is enqueued at priority 5
    And   thread B is enqueued at priority 2
    And   thread C is enqueued at priority 8
    When  the scheduler selects the next thread
    Then  thread B is returned
    And   the bitmap bit for priority 2 is still set

  Scenario: Bitmap bit is cleared when the last thread at a priority dequeues
    Given thread A is enqueued at priority 5
    When  thread A is dequeued
    Then  the bitmap bit for priority 5 is clear
    And   a subsequent dequeue_highest returns None
```
````

**Coexistence with SysML elements:** Native `TestCase` files live in the same directory tree as `VerificationCaseDef`, `AnalysisCaseDef`, and all other SysML element types. The `type: TestCase` value is not a SysML keyword — files with this type are always routed to the dedicated TestCase handler.

### 8.13 Allocation Elements

#### 8.13.1 `AllocationDef`

Defines a class of allocation relationship.

**Type-specific additional fields for `AllocationDef`:**

| Field | YAML type | Default | Description |
|---|---|---|---|
| `supertype` | string or list | absent | Supertype AllocationDefs |
| `ends` | list | absent | Named end features for the allocation (same schema as ConnectionDef ends; §8.4.2) |
| `allocations` | list of allocation maps | absent | Nested sub-allocation usages owned by this AllocationDef |

Each entry in `allocations:`:

| Sub-field | YAML type | Required | Description |
|---|---|---|---|
| `name` | string | optional | Named allocation usage |
| `typedBy` | string | optional | Qualified name of an `AllocationDef` typing this sub-allocation |
| `allocateFrom` | string | **Required** | Feature chain of the logical/source element |
| `allocateTo` | string | **Required** | Feature chain of the physical/target element |

**Example** (`model/Allocations/FunctionAllocation.md`):

```yaml
---
type: AllocationDef
name: FunctionAllocation
allocations:
  - name: engineControlAlloc
    allocateFrom: LogicalArch::EngineControl
    allocateTo: PhysicalArch::ECU
  - name: transmissionControlAlloc
    allocateFrom: LogicalArch::TransmissionControl
    allocateTo: PhysicalArch::TCU
---
Composite allocation of logical control functions to physical ECUs.
```

```yaml
---
type: AllocationDef
name: FunctionalToPhysical
supertype: Allocations::Allocation
ends:
  - name: allocatedFrom
    typedBy: Actions::Action
    isEnd: true
  - name: allocatedTo
    typedBy: Parts::Part
    isEnd: true
---
Allocation of a logical function (action) to a physical component (part).
```

#### 8.13.2 `Allocation` Usage

The `Allocation` usage file specifies a concrete allocation of one element to another.

**Type-specific additional fields:**

| Field | YAML type | Default | Description |
|---|---|---|---|
| `typedBy` | string | absent | Qualified name of the AllocationDef |
| `allocateFrom` | string | **Required** | Feature chain or qualified name of the allocated element (source) |
| `allocateTo` | string | **Required** | Feature chain or qualified name of the target element |

```yaml
---
type: Allocation
name: controllerToECU
typedBy: Allocations::FunctionalToPhysical
allocateFrom: VehicleBehavior::ProvidePower
allocateTo: PhysicalArch::EngineControlUnit
---
Allocates the ProvidePower action to the Engine Control Unit (ECU).
```

**Bulk allocations** may also be expressed on a container element using `allocations:` field (list of maps with `from:` and `to:` and optional `typedBy:`):

```yaml
# On PhysicalArch/_index.md or a dedicated Allocation file
allocations:
  - from: VehicleBehavior::ProvidePower
    to: PhysicalArch::EngineControlUnit
    typedBy: Allocations::FunctionalToPhysical
  - from: VehicleBehavior::RegulateSpeed
    to: PhysicalArch::TransmissionControlUnit
    typedBy: Allocations::FunctionalToPhysical
```

### 8.14 View, Viewpoint, and Rendering Elements

#### 8.14.1 `ViewpointDef`

Defines a stakeholder viewpoint describing what model information is relevant.

**Type-specific additional fields:**

| Field | YAML type | Default | Description |
|---|---|---|---|
| `supertype` | string or list | absent | Supertype ViewpointDefs |
| `stakeholders` | list of strings | absent | Qualified names of stakeholder PartDefs |
| `concerns` | list of strings | absent | Qualified names of ConcernDefs addressed |
| `methods` | list of strings | absent | Qualified names of ViewDefs or RenderingDefs satisfying this viewpoint |
| `satisfiedBy` | list of strings | absent | ViewDefs/Views that satisfy this viewpoint |

```yaml
---
type: ViewpointDef
name: SafetyViewpoint
stakeholders:
  - Stakeholders::SafetyEngineer
concerns:
  - Concerns::SafetyHazards
  - Concerns::RiskLevel
methods:
  - Views::FaultTreeView
---
Viewpoint addressing safety concerns from the safety engineer's perspective.
```

#### 8.14.2 `ViewDef`

**Type-specific additional fields:**

| Field | YAML type | Default | Description |
|---|---|---|---|
| `supertype` | string or list | absent | Supertype ViewDefs |
| `satisfies` | list of strings | absent | Qualified names of ViewpointDefs satisfied by this view |
| `expose` | list of expose maps | absent | Elements exposed by this view; see 8.14.3 |
| `rendering` | string | absent | Qualified name of the RenderingDef applied to this view |

#### 8.14.3 Expose Schema

Each entry in `expose:`:

| Sub-field | YAML type | Required | Description |
|---|---|---|---|
| `target` | string | **Required** | Qualified name or import pattern of exposed element(s) |
| `isRecursive` | bool | optional | Whether sub-elements are recursively exposed |
| `filter` | string | optional | Filter expression (opaque) |

```yaml
---
type: ViewDef
name: SafetyView
satisfies:
  - Views::SafetyViewpoint
expose:
  - target: VehicleSystem::*
    isRecursive: true
    filter: "@SafetyMetadata and SafetyMetadata::SafetyAnnotation::level >= 2"
rendering: Views::FaultTreeRendering
---
Safety view exposing all safety-annotated elements of the vehicle system.
```

#### 8.14.4 `RenderingDef`

```yaml
---
type: RenderingDef
name: FaultTreeRendering
supertype: Views::Rendering
features:
  - name: diagramType
    typedBy: ScalarValues::String
    value: '"FTA"'
---
Rendering definition for Fault Tree Analysis diagrams.
```

### 8.15 Metadata Elements

#### 8.15.1 `MetadataDef`

Defines a metadata annotation structure (user-defined keyword).

**Type-specific additional fields:**

| Field | YAML type | Default | Description |
|---|---|---|---|
| `supertype` | string or list | absent | Supertype MetadataDefs |
| `features` | list | absent | Owned attribute features of the metadata |
| `annotates` | list of strings | absent | KerML/SysML metaclass names of elements this metadata may annotate (e.g., `["PartDef", "Part"]`); omit for unrestricted |
| `isSemantic` | bool | `false` | `true` if this metadata has user-defined semantic effect on the annotated element |

**Valid metaclass names for `annotates:`** are drawn from the KerML/SysML type hierarchy. Commonly used values: `Element` (any element), `Feature`, `Definition`, `Usage`, `PartDef`, `PartUsage`, `ActionDef`, `ActionUsage`, `RequirementDef`, `RequirementUsage`, `Package`, `Namespace`. More specific metaclass names from the SysML abstract syntax are also valid. A parser should treat unrecognized metaclass names as warnings, not errors, to allow forward compatibility.

**Example** (`model/Metadata/SafetyAnnotation.md`):

```yaml
---
type: MetadataDef
name: SafetyAnnotation
features:
  - name: level
    typedBy: ScalarValues::Natural
  - name: hazardId
    typedBy: ScalarValues::String
  - name: mitigation
    typedBy: ScalarValues::String
annotates:
  - PartDef
  - Part
  - ActionDef
  - Action
---
Metadata for annotating model elements with safety classification information.
Level 1 = minor hazard; Level 2 = major hazard; Level 3 = catastrophic hazard.
```

#### 8.15.2 Applying Metadata

Apply metadata to any element using the `metadata:` field:

```yaml
---
type: PartDef
name: FuelSystem
metadata:
  - type: Metadata::SafetyAnnotation
    level: 3
    hazardId: "HAZ-042"
    mitigation: "Double-walled fuel lines, automatic shutoff valve"
---
Fuel system component with catastrophic hazard classification.
```

#### 8.15.3 Standard Library Metadata

The following metadata types from the SysML Standard Library are referenced by their qualified names:

| Qualified name | Purpose |
|---|---|
| `ModelingMetadata::StatusInfo` | Model element status (draft, approved, deprecated) |
| `ModelingMetadata::Issue` | Open issue on an element |
| `ModelingMetadata::Rationale` | Design rationale annotation |
| `ModelingMetadata::Refinement` | Refinement relationship annotation |
| `RiskMetadata::Risk` | Risk metadata with probability and severity |
| `RiskMetadata::RiskLevel` | Risk level enumeration |

### 8.16 Diagram Elements

#### 8.16.1 Overview

A `Diagram` file stores an LLM-generated SVG diagram alongside a structured frontmatter manifest that links every SVG shape and edge to a model element by qualified name. The frontmatter is the canonical source of traceability; the SVG is the visual geometry. A parser can validate the diagram entirely from the frontmatter without touching the SVG.

The SVG uses a `sysml:` XML namespace on shapes for redundant inline traceability, enabling the SVG to be opened standalone in any viewer.

**Two storage modes are supported.** The frontmatter schema is identical in both; only the Markdown body differs:

| Mode | Body content | GitHub rendering | Files |
|---|---|---|---|
| **Inline** | Fenced `svg` code block | Code only (not rendered) | One `.md` file |
| **Companion** | `<img src="./Name.svg">` HTML tag | SVG rendered as image | `.md` + `.svg` pair |

Choose **inline** when GitHub rendering is not required and keeping everything in one file is preferred. Choose **companion** when the diagram must render on GitHub or in other Markdown viewers.

#### 8.16.2 `Diagram` Frontmatter Schema

**Top-level fields:**

| Field | YAML type | Required | Default | Description |
|---|---|---|---|---|
| `type` | string | **Required** | — | `Diagram` |
| `name` | string | optional | filename stem | Display name for the diagram |
| `kind` | string | **Required** | — | Diagram kind: `BDD`, `IBD`, `Sequence`, `StateMachine`, `Requirement`, `Allocation`, `UseCase`, `Custom` |
| `subject` | string | **Required** | — | Qualified name of the model element this diagram depicts |
| `svgMode` | string | optional | `inline` | Storage mode: `inline` (fenced block in body) or `companion` (separate `.svg` file) |
| `svgFile` | string | optional | `<stem>.svg` | Companion file path relative to the `.md` file; only used when `svgMode: companion` |
| `shapes` | map | optional | absent | Shape manifest; see §8.16.3 |
| `edges` | map | optional | absent | Edge manifest; see §8.16.4 |
| `symbolLib` | string | optional | `_diagram-symbols.svg` | Path (relative to model root) to the shared SVG symbol library |
| `generatedBy` | string | optional | absent | Free-text note on how the SVG was produced (e.g., `"claude-sonnet-4-6"`) |

#### 8.16.3 Shape Manifest

`shapes:` is a map from SVG element `id` to a shape descriptor. The value may be a plain string (shorthand) or a map:

**String shorthand** — the shape represents a single model element:
```yaml
shapes:
  engine-rect: VehicleSystem::Engine
  tranny-rect: VehicleSystem::Transmission
```

**Map form** — for additional metadata on the shape:

| Sub-field | YAML type | Required | Description |
|---|---|---|---|
| `ref` | string | **Required** | Qualified name of the model element this shape represents |
| `kind` | string | optional | Shape role: `block`, `port`, `label`, `compartment`, `annotation` (default: `block`) |
| `parent` | string | optional | SVG `id` of the enclosing/containing shape (for nested ports or compartments) |

```yaml
shapes:
  engine-rect:
    ref: VehicleSystem::Engine
    kind: block
  engine-out:
    ref: VehicleSystem::Engine::powerOut
    kind: port
    parent: engine-rect
  engine-label:
    ref: VehicleSystem::Engine
    kind: label
    parent: engine-rect
```

#### 8.16.4 Edge Manifest

`edges:` is a map from SVG element `id` to an edge descriptor:

| Sub-field | YAML type | Required | Description |
|---|---|---|---|
| `ref` | string | **Required** | Qualified name of the model element (Connection, Flow, Allocation, etc.) this edge represents |
| `source` | string | **Required** | SVG `id` of the shape at the source/tail end |
| `target` | string | **Required** | SVG `id` of the shape at the target/head end |
| `kind` | string | optional | Edge role: `connection`, `flow`, `allocation`, `dependency`, `inheritance`, `association` (default: `connection`) |

```yaml
edges:
  power-conn:
    ref: VehicleSystem::PowerConnection
    source: engine-out
    target: tranny-in
    kind: flow
  fuel-conn:
    ref: VehicleSystem::FuelFlow
    source: tank-fuel-out
    target: engine-fuel-in
    kind: flow
```

#### 8.16.5 SVG Body Conventions

**Inline mode** — the Markdown body contains the SVG as a fenced `svg` code block:

````markdown
```svg
<svg xmlns="http://www.w3.org/2000/svg" ...>
  ...
</svg>
```
````

**Companion mode** — the Markdown body contains an HTML `<img>` tag pointing to the companion `.svg` file. GitHub renders this as an image:

```markdown
<img src="./VehiclePowertrainIBD.svg" alt="Vehicle Powertrain IBD" width="100%"/>
```

The `svgFile:` frontmatter field identifies the companion file. If absent, the parser looks for a file with the same stem as the `.md` file and a `.svg` extension in the same directory (e.g., `Foo.md` → `Foo.svg`).

**In both modes, the SVG itself must conform to the following conventions:**

**Namespace declaration:**
```svg
<svg xmlns="http://www.w3.org/2000/svg"
     xmlns:sysml="urn:sysml-md:1.0"
     viewBox="0 0 W H">
```

**Shape elements** carry a `sysml:ref` attribute matching the `ref` in the frontmatter manifest, and an `id` matching the manifest key:
```svg
<use href="#sym-PartDef" id="engine-rect"
     sysml:ref="VehicleSystem::Engine"
     x="100" y="150" width="160" height="80"/>
```

**Edge elements** carry `sysml:ref`, `sysml:source`, and `sysml:target`:
```svg
<path id="power-conn"
      sysml:ref="VehicleSystem::PowerConnection"
      sysml:source="engine-out"
      sysml:target="tranny-in"
      d="M 260,190 H 400"
      class="connection flow"
      marker-end="url(#arrow-flow)"/>
```

**Symbol library** (`<defs>` or referenced via `symbolLib:`): defines reusable symbols for each SysML element kind. Shapes reference them via `<use href="#sym-<TypeName>">`. A default symbol library is provided at `_diagram-symbols.svg` in the model root.

**CSS classes** on shapes and edges correspond to SysML element types and control visual style:

| CSS class | Applies to |
|---|---|
| `PartDef`, `ItemDef`, `PortDef`, … | Shapes typed to those element kinds |
| `connection`, `flow`, `allocation` | Edge paths |
| `port in`, `port out`, `port inout` | Port shapes with direction |
| `abstract` | Any shape representing an abstract element |

#### 8.16.6 Complete Examples

**Inline mode** (`model/VehicleSystem/Diagrams/VehiclePowertrainIBD.md`):

The frontmatter and SVG body live in one file. GitHub displays code, not an image.

**Companion mode** — two files with identical frontmatter, different bodies:

`model/VehicleSystem/Diagrams/VehiclePowertrainIBD.md`:

````markdown
---
type: Diagram
kind: IBD
name: VehiclePowertrainIBD
subject: VehicleSystem::Vehicle
svgMode: companion
generatedBy: claude-sonnet-4-6
shapes:
  vehicle-boundary: VehicleSystem::Vehicle
  engine-rect:
    ref: VehicleSystem::Engine
    kind: block
    parent: vehicle-boundary
  tranny-rect:
    ref: VehicleSystem::Transmission
    kind: block
    parent: vehicle-boundary
  engine-out:
    ref: VehicleSystem::Engine::powerOut
    kind: port
    parent: engine-rect
  tranny-in:
    ref: VehicleSystem::Transmission::powerIn
    kind: port
    parent: tranny-rect
edges:
  power-conn:
    ref: VehicleSystem::PowerConnection
    source: engine-out
    target: tranny-in
    kind: flow
---

<img src="./VehiclePowertrainIBD.svg" alt="Vehicle Powertrain IBD" width="100%"/>
````

`model/VehicleSystem/Diagrams/VehiclePowertrainIBD.svg` (companion file — the actual SVG geometry):

```svg
<svg xmlns="http://www.w3.org/2000/svg"
     xmlns:sysml="urn:sysml-md:1.0"
     viewBox="0 0 800 500">
  <!-- shapes, edges, etc. with sysml:ref attributes -->
</svg>
```

**Inline mode** (`model/VehicleSystem/Diagrams/VehiclePowertrainIBD.md`) — same frontmatter without `svgMode`, SVG in body:

````markdown
---
type: Diagram
kind: IBD
name: VehiclePowertrainIBD
subject: VehicleSystem::Vehicle
generatedBy: claude-sonnet-4-6
shapes:
  vehicle-boundary: VehicleSystem::Vehicle
  engine-rect:
    ref: VehicleSystem::Engine
    kind: block
    parent: vehicle-boundary
  ...
edges:
  power-conn:
    ref: VehicleSystem::PowerConnection
    source: engine-out
    target: tranny-in
    kind: flow
---

```svg
<svg xmlns="http://www.w3.org/2000/svg"
     xmlns:sysml="urn:sysml-md:1.0"
     viewBox="0 0 800 500">
  ...
</svg>
```
````

**Full inline example with complete SVG** (`model/VehicleSystem/Diagrams/VehiclePowertrainIBD.md`):

````markdown
---
type: Diagram
kind: IBD
name: VehiclePowertrainIBD
subject: VehicleSystem::Vehicle
generatedBy: claude-sonnet-4-6
shapes:
  vehicle-boundary: VehicleSystem::Vehicle
  engine-rect:
    ref: VehicleSystem::Engine
    kind: block
    parent: vehicle-boundary
  tranny-rect:
    ref: VehicleSystem::Transmission
    kind: block
    parent: vehicle-boundary
  tank-rect:
    ref: VehicleSystem::FuelTank
    kind: block
    parent: vehicle-boundary
  engine-out:
    ref: VehicleSystem::Engine::powerOut
    kind: port
    parent: engine-rect
  tranny-in:
    ref: VehicleSystem::Transmission::powerIn
    kind: port
    parent: tranny-rect
  tank-fuel-out:
    ref: VehicleSystem::FuelTank::fuelOut
    kind: port
    parent: tank-rect
  engine-fuel-in:
    ref: VehicleSystem::Engine::fuelIn
    kind: port
    parent: engine-rect
edges:
  power-conn:
    ref: VehicleSystem::PowerConnection
    source: engine-out
    target: tranny-in
    kind: flow
  fuel-conn:
    ref: VehicleSystem::FuelFlow
    source: tank-fuel-out
    target: engine-fuel-in
    kind: flow
---

```svg
<svg xmlns="http://www.w3.org/2000/svg"
     xmlns:sysml="urn:sysml-md:1.0"
     viewBox="0 0 800 500">
  <defs>
    <marker id="arrow-flow" markerWidth="10" markerHeight="7"
            refX="9" refY="3.5" orient="auto">
      <polygon points="0 0, 10 3.5, 0 7" fill="#4a90d9"/>
    </marker>
  </defs>

  <!-- System boundary -->
  <rect id="vehicle-boundary" sysml:ref="VehicleSystem::Vehicle"
        x="40" y="40" width="720" height="420"
        class="PartDef boundary" rx="8"/>
  <text x="400" y="68" class="stereotype label">«part» Vehicle</text>

  <!-- Engine block -->
  <use href="#sym-PartDef" id="engine-rect"
       sysml:ref="VehicleSystem::Engine"
       x="80" y="160" width="160" height="80"/>
  <text x="160" y="206" class="label">Engine</text>

  <!-- Transmission block -->
  <use href="#sym-PartDef" id="tranny-rect"
       sysml:ref="VehicleSystem::Transmission"
       x="400" y="160" width="160" height="80"/>
  <text x="480" y="206" class="label">Transmission</text>

  <!-- Fuel tank block -->
  <use href="#sym-PartDef" id="tank-rect"
       sysml:ref="VehicleSystem::FuelTank"
       x="80" y="340" width="160" height="80"/>
  <text x="160" y="386" class="label">FuelTank</text>

  <!-- Ports -->
  <circle id="engine-out"     sysml:ref="VehicleSystem::Engine::powerOut"
          cx="240" cy="200" r="7" class="port out"/>
  <circle id="tranny-in"      sysml:ref="VehicleSystem::Transmission::powerIn"
          cx="400" cy="200" r="7" class="port in"/>
  <circle id="tank-fuel-out"  sysml:ref="VehicleSystem::FuelTank::fuelOut"
          cx="160" cy="340" r="7" class="port out"/>
  <circle id="engine-fuel-in" sysml:ref="VehicleSystem::Engine::fuelIn"
          cx="160" cy="240" r="7" class="port in"/>

  <!-- Connections -->
  <path id="power-conn"
        sysml:ref="VehicleSystem::PowerConnection"
        sysml:source="engine-out" sysml:target="tranny-in"
        d="M 247,200 H 393"
        class="connection flow" marker-end="url(#arrow-flow)"/>
  <text x="320" y="190" class="edge-label">«flow» power</text>

  <path id="fuel-conn"
        sysml:ref="VehicleSystem::FuelFlow"
        sysml:source="tank-fuel-out" sysml:target="engine-fuel-in"
        d="M 160,333 V 247"
        class="connection flow" marker-end="url(#arrow-flow)"/>
  <text x="170" y="295" class="edge-label">«flow» fuel</text>
</svg>
```
````

#### 8.16.7 Validation Rules

A conformant parser must:

1. **Reference validation** — every `ref` value in `shapes:` and `edges:` must resolve to a model element. Unresolved references are errors.
2. **Type compatibility** — the CSS class on an SVG shape must match the `type:` of the referenced model element (e.g., a shape with `class="PartDef"` whose `ref` resolves to a `RequirementDef` is an error).
3. **ID consistency** — every `id` key in the frontmatter `shapes:` and `edges:` maps must appear as an `id` attribute in the SVG (inline or companion), and vice versa. Orphaned IDs in either direction are warnings.
4. **Edge endpoint validity** — `source` and `target` values in `edges:` must be keys present in `shapes:`.
5. **Subject existence** — the `subject:` qualified name must resolve to a model element.
6. **Companion file existence** — when `svgMode: companion`, the parser must verify the companion `.svg` file exists at the path given by `svgFile:` (or the default stem-matched path). A missing companion file is an error.
7. **Mode consistency** — when `svgMode: inline`, the body must contain a fenced `svg` block. When `svgMode: companion`, the body must contain an `<img>` tag. A mismatch is a warning.
8. **Completeness warnings** (optional, non-blocking) — for `IBD` diagrams, the parser may warn if sub-parts or connections declared in the subject element's `.md` file do not appear in `shapes:` or `edges:`.

---

#### 8.16.8 Kind-Specific Diagram Conventions

Each `kind:` value imposes constraints on which element types are valid for `subject:`, which values are valid for `kind:` in shape and edge descriptors, which CSS classes and SVG primitives to use, and which model elements the parser must check for completeness. The `Custom` kind has no prescribed conventions — all shape and edge `kind:` values are user-defined and no completeness rules are enforced.

---

##### 8.16.8.1 BDD (Block Definition Diagram)

A BDD shows classifiers (definitions) and their relationships in a given package or rooted at a given definition. It is analogous to a UML class diagram restricted to SysML block types. Inheritance hierarchies, attribute compartments, and typed associations are the primary content.

**Valid `subject:` types:** `Package`, `PartDef`, `ItemDef`

**Shape kinds:**

| `kind` value | Description | SVG primitive | CSS class | Symbol |
|---|---|---|---|---|
| `block` | A definition box representing a `PartDef`, `ItemDef`, or similar classifier | `<rect>` | `block <TypeName>` (e.g., `block PartDef`) | `#sym-PartDef`, `#sym-ItemDef` |
| `compartment` | An attribute or feature compartment drawn inside a `block`; its `parent` is the enclosing block's SVG id | `<rect>` + `<line>` divider | `compartment` | — |
| `label` | A text label (stereotype banner, element name, or annotation text) | `<text>` | `label` | — |
| `note` | A comment annotation box (dog-eared rectangle) | `<rect class="note">` + `<line>` fold | `note` | `#sym-note` |

**Edge kinds:**

| `kind` value | Description | SVG primitive | CSS class | Arrowhead |
|---|---|---|---|---|
| `inheritance` | Specialization (`':>'`) — hollow triangle at the supertype end | `<path>` | `edge inheritance` | `url(#arrow-inherit)` (hollow triangle) |
| `association` | Plain typed association between two classifiers | `<line>` or `<path>` | `edge association` | none (or open arrowhead) |
| `composition` | Composite ownership — filled diamond at the whole end | `<path>` | `edge composition` | `url(#arrow-composition)` (filled diamond) |
| `aggregation` | Shared aggregation — hollow diamond at the aggregate end | `<path>` | `edge aggregation` | `url(#arrow-aggregation)` (hollow diamond) |
| `dependency` | Dependency or usage — dashed open arrowhead | `<path stroke-dasharray="6 3">` | `edge dependency` | `url(#arrow-open)` |

**SVG body notes:** block rectangles use `<rect>` with compartment dividers rendered as `<line>` elements; attribute names and types appear in `<text>` elements inside compartment groups; inheritance arrows use `<path>` with a hollow-triangle marker; name labels appear in a `<text>` above or inside the block.

**Minimal YAML manifest example:**

```yaml
type: Diagram
kind: BDD
name: PowertrainBDD
subject: VehicleSystem::Powertrain
shapes:
  engine-block:
    ref: VehicleSystem::Powertrain::Engine
    kind: block
  engine-attrs:
    ref: VehicleSystem::Powertrain::Engine
    kind: compartment
    parent: engine-block
  motor-block:
    ref: VehicleSystem::Powertrain::ElectricMotor
    kind: block
  motor-label:
    ref: VehicleSystem::Powertrain::ElectricMotor
    kind: label
    parent: motor-block
edges:
  engine-inherits:
    ref: VehicleSystem::Powertrain::Engine
    source: engine-block
    target: motor-block
    kind: inheritance
  engine-composed:
    ref: VehicleSystem::Powertrain::EngineMount
    source: engine-block
    target: motor-block
    kind: composition
```

**Completeness rule:** the parser must warn if any `PartDef` or `ItemDef` that is a direct child element of the subject package (or a direct sub-definition of the subject definition) is absent from `shapes:`.

---

##### 8.16.8.2 IBD (Internal Block Diagram)

An IBD shows the internal structure of a single `PartDef` or `Part` — its owned part usages, port usages, and the connections or flows that bind them. The subject's outer boundary is always present as the first shape.

**Valid `subject:` types:** `PartDef`, `Part`

**Shape kinds:**

| `kind` value | Description | SVG primitive | CSS class | Symbol |
|---|---|---|---|---|
| `boundary` | The subject's outer containing box | `<rect>` with `rx` for rounded corners | `boundary` | `#sym-boundary` |
| `block` | An owned part usage inside the boundary | `<rect>` | `block Part` or `block PartDef` | `#sym-PartDef` |
| `port` | A port usage drawn on the edge of its parent block | `<rect>` (half-in, half-out) or `<circle>` | `port in`, `port out`, `port inout` | `#sym-port` |
| `label` | A text label for any shape | `<text>` | `label` | — |
| `note` | A comment annotation box | `<rect class="note">` + fold line | `note` | `#sym-note` |

**Edge kinds:**

| `kind` value | Description | SVG primitive | CSS class | Arrowhead |
|---|---|---|---|---|
| `connection` | Structural connector between two ports or parts | `<path>` | `edge connection` | none |
| `flow` | Flow connection with directional arrow | `<path>` | `edge flow` | `url(#arrow-flow)` (filled triangle) |
| `binding` | Equality binding — dashed line with `=` midpoint label | `<path stroke-dasharray="4 4">` | `edge binding` | none |
| `succession` | Action succession — dashed arrow with filled head | `<path stroke-dasharray="6 3">` | `edge succession` | `url(#arrow-filled)` |

**SVG body notes:** the boundary is a large `<rect>` with `rx="8"` or similar; ports are small `<rect>` or `<circle>` positioned on the edge of their parent block (use `parent:` in the manifest to declare ownership); flow edges use `<path>` with a directional arrowhead marker; labels inside blocks use `<text>` centered within the block rectangle.

**Minimal YAML manifest example:**

```yaml
type: Diagram
kind: IBD
name: EngineInternalIBD
subject: VehicleSystem::Powertrain::Engine
shapes:
  engine-boundary:
    ref: VehicleSystem::Powertrain::Engine
    kind: boundary
  piston-block:
    ref: VehicleSystem::Powertrain::Engine::pistons
    kind: block
    parent: engine-boundary
  crankshaft-block:
    ref: VehicleSystem::Powertrain::Engine::crankshaft
    kind: block
    parent: engine-boundary
  piston-out:
    ref: VehicleSystem::Powertrain::Engine::pistons::powerOut
    kind: port
    parent: piston-block
  crank-in:
    ref: VehicleSystem::Powertrain::Engine::crankshaft::forceIn
    kind: port
    parent: crankshaft-block
edges:
  piston-crank-flow:
    ref: VehicleSystem::Powertrain::Engine::pistonCrankFlow
    source: piston-out
    target: crank-in
    kind: flow
```

**Completeness rule:** the parser must warn if any owned `Part`, `Port`, `Connection`, or `Flow` declared in the subject's `.md` file (in `features:` or `connections:`) is absent from `shapes:` or `edges:`.

---

##### 8.16.8.3 Sequence

A Sequence diagram shows time-ordered message exchanges between lifelines. Time flows top-to-bottom along the Y axis. Lifelines represent parts or actors; messages are horizontal arrows between them.

**Valid `subject:` types:** `ActionDef`, `UseCaseDef`, `InteractionDef`

**Shape kinds:**

| `kind` value | Description | SVG primitive | CSS class | Symbol |
|---|---|---|---|---|
| `lifeline` | A participant — vertical dashed line with a named header box at the top | `<rect>` (header) + `<line stroke-dasharray="4 4">` (stem) | `lifeline` | `#sym-lifeline` |
| `actor` | An external actor — stick figure or labeled box | `<g class="actor">` containing `<circle>` (head) + `<line>` elements (body) | `actor` | `#sym-actor` |
| `activation` | An activation bar — narrow rectangle on a lifeline indicating an active period | `<rect>` | `activation` | — |
| `fragment` | A combined fragment (loop, alt, opt, par) — rectangle with keyword label in top-left corner | `<rect>` + `<rect class="fragment-header">` + `<text>` | `fragment` | `#sym-fragment` |
| `note` | A comment annotation box | `<rect class="note">` + fold line | `note` | `#sym-note` |

**Edge kinds:**

| `kind` value | Description | SVG primitive | CSS class | Arrowhead |
|---|---|---|---|---|
| `message` | A message send — synchronous: solid line with filled arrowhead; asynchronous: solid line with open arrowhead | `<path>` | `edge message sync` or `edge message async` | `url(#arrow-filled)` or `url(#arrow-open)` |
| `return` | A return message — dashed line with open arrowhead | `<path stroke-dasharray="6 3">` | `edge return` | `url(#arrow-open)` |
| `create` | Object creation — dashed arrow to a new lifeline's header | `<path stroke-dasharray="4 4">` | `edge create` | `url(#arrow-open)` |
| `destroy` | Lifeline termination — X mark at the bottom of the lifeline | `<line>` pair forming an X | `destroy` | — |

**SVG body notes:** lifeline stems are `<line>` with `stroke-dasharray`; activation bars are narrow `<rect>` overlaid on the lifeline stem; messages are horizontal `<path>` elements with arrowhead markers; time increases downward so Y-coordinates represent event ordering; fragment boxes enclose the relevant message range with the keyword (`loop`, `alt`, etc.) in a small header rectangle.

**Minimal YAML manifest example:**

```yaml
type: Diagram
kind: Sequence
name: EngineStartSequence
subject: VehicleSystem::Actions::EngineStart
shapes:
  driver-lifeline:
    ref: VehicleSystem::Actors::Driver
    kind: actor
  ecu-lifeline:
    ref: VehicleSystem::ECU
    kind: lifeline
  engine-lifeline:
    ref: VehicleSystem::Powertrain::Engine
    kind: lifeline
  ecu-activation:
    ref: VehicleSystem::ECU
    kind: activation
    parent: ecu-lifeline
edges:
  start-cmd:
    ref: VehicleSystem::Actions::EngineStart::sendStartCommand
    source: driver-lifeline
    target: ecu-lifeline
    kind: message
  ignite-msg:
    ref: VehicleSystem::Actions::EngineStart::sendIgnite
    source: ecu-lifeline
    target: engine-lifeline
    kind: message
  ignite-return:
    ref: VehicleSystem::Actions::EngineStart::igniteAck
    source: engine-lifeline
    target: ecu-lifeline
    kind: return
```

**Completeness rule:** the parser must warn if any `SendAction` or `AcceptAction` in the subject `ActionDef`'s sub-actions (reachable via `subActions:` or `steps:`) is absent from `edges:`.

---

##### 8.16.8.4 StateMachine

A StateMachine diagram shows the states and transitions of a `StateDef`. Initial and final pseudostates, choice pseudostates, and history pseudostates are first-class shape kinds.

**Valid `subject:` types:** `StateDef`

**Shape kinds:**

| `kind` value | Description | SVG primitive | CSS class | Symbol |
|---|---|---|---|---|
| `state` | A named state — rounded rectangle; may have entry/do/exit sub-labels | `<rect rx="12">` | `state` | `#sym-state` |
| `initial` | Initial pseudostate — filled circle | `<circle class="initial">` | `initial` | `#sym-initial` |
| `final` | Final state — bullseye (outer circle + inner filled circle) | `<circle class="final-outer">` + `<circle class="final-inner">` | `final` | `#sym-final` |
| `choice` | Choice pseudostate — diamond | `<polygon>` | `choice` | `#sym-choice` |
| `history` | History pseudostate — circle containing `H` or `H*` | `<circle>` + `<text>H</text>` | `history` | `#sym-history` |
| `note` | A comment annotation box | `<rect class="note">` + fold line | `note` | `#sym-note` |

**Edge kinds:**

| `kind` value | Description | SVG primitive | CSS class | Arrowhead |
|---|---|---|---|---|
| `transition` | A state transition — arrow with optional label formatted as `event [guard] / effect` | `<path>` | `edge transition` | `url(#arrow-filled)` |

**SVG body notes:** state rectangles use `<rect>` with `rx="12"`; sub-labels (entry/do/exit) are `<text>` elements inside the state box below a `<line>` compartment divider; the initial pseudostate is a single `<circle class="initial">` (no arrowhead on the circle itself); the final state is two concentric circles; transition labels follow the format `event [guard] / effect` and are rendered as `<text>` elements near the midpoint of the transition path.

**Minimal YAML manifest example:**

```yaml
type: Diagram
kind: StateMachine
name: EngineStateMachine
subject: VehicleSystem::States::EngineStateDef
shapes:
  init-pseudo:
    ref: VehicleSystem::States::EngineStateDef
    kind: initial
  stopped-state:
    ref: VehicleSystem::States::EngineStateDef::Stopped
    kind: state
  running-state:
    ref: VehicleSystem::States::EngineStateDef::Running
    kind: state
  final-pseudo:
    ref: VehicleSystem::States::EngineStateDef
    kind: final
edges:
  init-to-stopped:
    ref: VehicleSystem::States::EngineStateDef::initTransition
    source: init-pseudo
    target: stopped-state
    kind: transition
  stopped-to-running:
    ref: VehicleSystem::States::EngineStateDef::startTransition
    source: stopped-state
    target: running-state
    kind: transition
  running-to-stopped:
    ref: VehicleSystem::States::EngineStateDef::stopTransition
    source: running-state
    target: stopped-state
    kind: transition
```

**Completeness rule:** the parser must warn if any sub-state listed in the subject `StateDef`'s `subStates:` field is absent from `shapes:`, and if any transition listed in `transitions:` is absent from `edges:`.

---

##### 8.16.8.5 Requirement

A Requirement diagram shows requirements and their inter-relationships within a package or rooted at a `RequirementDef`. All relationship edges are dashed with `«keyword»` labels following SysML requirement relationship notation.

**Valid `subject:` types:** `Package`, `RequirementDef`

**Shape kinds:**

| `kind` value | Description | SVG primitive | CSS class | Symbol |
|---|---|---|---|---|
| `requirement` | A requirement box — rectangle with `«requirement»` stereotype banner and two compartments (name, text) | `<rect>` + `<line>` compartment divider + `<text>` stereotype | `requirement` | `#sym-requirement` |
| `block` | A satisfying element — a system part or definition referenced by `satisfy`/`verify` links | `<rect>` | `block PartDef` | `#sym-PartDef` |
| `testcase` | A verification case box | `<rect>` | `testcase` | `#sym-testcase` |
| `note` | A comment annotation box | `<rect class="note">` + fold line | `note` | `#sym-note` |

**Edge kinds:**

| `kind` value | Description | SVG primitive | CSS class | Arrowhead | Keyword label |
|---|---|---|---|---|---|
| `containment` | Parent/child requirement nesting | `<line>` | `edge containment` | none | — |
| `derive` | Derived requirement — `«deriveReqt»` | `<path stroke-dasharray="6 3">` | `edge derive` | `url(#arrow-open)` | `«deriveReqt»` |
| `satisfy` | Satisfaction by a model element — `«satisfy»` | `<path stroke-dasharray="6 3">` | `edge satisfy` | `url(#arrow-open)` | `«satisfy»` |
| `verify` | Verification by a test case — `«verify»` | `<path stroke-dasharray="6 3">` | `edge verify` | `url(#arrow-open)` | `«verify»` |
| `refine` | Refinement — `«refine»` | `<path stroke-dasharray="6 3">` | `edge refine` | `url(#arrow-open)` | `«refine»` |
| `trace` | Traceability — `«trace»` | `<path stroke-dasharray="6 3">` | `edge trace` | `url(#arrow-open)` | `«trace»` |
| `copy` | Copied requirement — `«copy»` | `<path stroke-dasharray="6 3">` | `edge copy` | `url(#arrow-open)` | `«copy»` |

**SVG body notes:** requirement boxes use `<rect>` with two internal compartments separated by a `<line>`; the top compartment contains a `<text class="stereotype">«requirement»</text>` and the element name; the bottom compartment contains the requirement text; all relationship edges are `<path stroke-dasharray="6 3">` with an open arrowhead marker and a `<text class="edge-label">` keyword.

**Minimal YAML manifest example:**

```yaml
type: Diagram
kind: Requirement
name: SafetyRequirementsDiagram
subject: VehicleSystem::Requirements::Safety
shapes:
  sys-safety-req:
    ref: VehicleSystem::Requirements::Safety::SystemSafety
    kind: requirement
  brake-req:
    ref: VehicleSystem::Requirements::Safety::BrakeResponse
    kind: requirement
  brake-sys-block:
    ref: VehicleSystem::BrakeSystem
    kind: block
  brake-test:
    ref: VehicleSystem::Tests::BrakeResponseTest
    kind: testcase
edges:
  safety-contains-brake:
    ref: VehicleSystem::Requirements::Safety::SystemSafety
    source: sys-safety-req
    target: brake-req
    kind: containment
  brake-satisfied-by:
    ref: VehicleSystem::Requirements::Safety::BrakeResponse::satisfiedBy
    source: brake-sys-block
    target: brake-req
    kind: satisfy
  brake-verified-by:
    ref: VehicleSystem::Requirements::Safety::BrakeResponse::verifiedBy
    source: brake-test
    target: brake-req
    kind: verify
```

**Completeness rule:** the parser must warn if any `Requirement` owned by the subject package or sub-`RequirementDef` is absent from `shapes:`, and if any `satisfies:`, `verifies:`, or `derivedFrom:` link declared in any requirement's `.md` file within the subject scope is absent from `edges:`.

---

##### 8.16.8.6 Allocation

An Allocation diagram shows `«allocate»` relationships between logical/functional elements and physical/hardware elements. The diagram is conventionally divided into two swim lanes — one for logical elements and one for physical elements.

**Valid `subject:` types:** `Package`, `AllocationDef`

**Shape kinds:**

| `kind` value | Description | SVG primitive | CSS class | Symbol |
|---|---|---|---|---|
| `swimlane` | Background lane dividing logical from physical elements; spans full diagram height or width | `<rect class="swimlane">` | `swimlane` | — |
| `block` | An allocated element — logical (function, action) or physical (hardware, node) | `<rect>` | `block <TypeName>` | `#sym-PartDef` or `#sym-ActionDef` |
| `label` | A lane title or annotation text | `<text>` (rotated `transform="rotate(-90)"` for vertical lanes) | `label swimlane-label` | — |
| `note` | A comment annotation box | `<rect class="note">` + fold line | `note` | `#sym-note` |

**Edge kinds:**

| `kind` value | Description | SVG primitive | CSS class | Arrowhead | Keyword label |
|---|---|---|---|---|---|
| `allocation` | Allocation relationship from logical to physical — `«allocate»` | `<path stroke-dasharray="8 4">` | `edge allocation` | `url(#arrow-open)` | `«allocate»` |

**SVG body notes:** swim lanes are `<rect class="swimlane">` drawn first (behind all other elements) spanning the full diagram height or width; lane title labels are `<text transform="rotate(-90)">` for vertical lanes; allocation arrows are `<path stroke-dasharray="8 4">` with an open arrowhead and a `«allocate»` keyword label at the midpoint.

**Minimal YAML manifest example:**

```yaml
type: Diagram
kind: Allocation
name: FunctionToHardwareAllocation
subject: VehicleSystem::Allocations
shapes:
  logical-lane:
    ref: VehicleSystem::Allocations
    kind: swimlane
  physical-lane:
    ref: VehicleSystem::Allocations
    kind: swimlane
  engine-ctrl-fn:
    ref: VehicleSystem::Functions::EngineControl
    kind: block
    parent: logical-lane
  ecu-hw:
    ref: VehicleSystem::Hardware::ECU
    kind: block
    parent: physical-lane
  brake-fn:
    ref: VehicleSystem::Functions::BrakeControl
    kind: block
    parent: logical-lane
edges:
  engine-ctrl-alloc:
    ref: VehicleSystem::Allocations::EngineControlToECU
    source: engine-ctrl-fn
    target: ecu-hw
    kind: allocation
  brake-alloc:
    ref: VehicleSystem::Allocations::BrakeControlToECU
    source: brake-fn
    target: ecu-hw
    kind: allocation
```

**Completeness rule:** the parser must warn if any `allocateFrom`/`allocateTo` pair declared in any `Allocation` or `AllocationDef` within the subject package is absent from `edges:`.

---

##### 8.16.8.7 UseCase

A UseCase diagram shows actors, use cases, the system boundary, and their relationships. Use cases are enclosed within the system boundary rectangle; actors appear outside it.

**Valid `subject:` types:** `UseCaseDef`, `Package`

**Shape kinds:**

| `kind` value | Description | SVG primitive | CSS class | Symbol |
|---|---|---|---|---|
| `system-boundary` | Rectangle enclosing all use cases — the system under consideration | `<rect class="system-boundary">` | `system-boundary` | `#sym-system-boundary` |
| `usecase` | A use case — ellipse with name label inside | `<ellipse>` + `<text>` | `usecase` | `#sym-usecase` |
| `actor` | An external actor — stick figure (`<circle>` head + `<line>` body) with name label below | `<g class="actor">` containing `<circle>` + `<line>` elements | `actor` | `#sym-actor` |
| `note` | A comment annotation box | `<rect class="note">` + fold line | `note` | `#sym-note` |

**Edge kinds:**

| `kind` value | Description | SVG primitive | CSS class | Arrowhead | Keyword label |
|---|---|---|---|---|---|
| `association` | Plain association between an actor and a use case | `<line>` or `<path>` | `edge association` | none | — |
| `include` | Include relationship — dashed arrow from base to included use case — `«include»` | `<path stroke-dasharray="6 3">` | `edge include` | `url(#arrow-open)` | `«include»` |
| `extend` | Extend relationship — dashed arrow from extension to base — `«extend»` | `<path stroke-dasharray="6 3">` | `edge extend` | `url(#arrow-open)` | `«extend»` |
| `generalization` | Inheritance between actors or between use cases — hollow triangle at supertype end | `<path>` | `edge generalization` | `url(#arrow-inherit)` (hollow triangle) | — |

**SVG body notes:** the system boundary is a `<rect class="system-boundary">` drawn first so all use cases appear on top of it; use cases are `<ellipse>` elements with `<text>` labels centered inside; actors are `<g class="actor">` groups containing a `<circle>` for the head and `<line>` elements for the body, arms, and legs, with a `<text>` name label below; all relationship edges are `<path>` with the appropriate marker and, for `include`/`extend`, a `<text class="edge-label">` keyword.

**Minimal YAML manifest example:**

```yaml
type: Diagram
kind: UseCase
name: VehicleUseCaseDiagram
subject: VehicleSystem::UseCases
shapes:
  system-rect:
    ref: VehicleSystem
    kind: system-boundary
  start-engine-uc:
    ref: VehicleSystem::UseCases::StartEngine
    kind: usecase
  check-fuel-uc:
    ref: VehicleSystem::UseCases::CheckFuelLevel
    kind: usecase
  driver-actor:
    ref: VehicleSystem::Actors::Driver
    kind: actor
edges:
  driver-start-assoc:
    ref: VehicleSystem::UseCases::StartEngine::driverAssoc
    source: driver-actor
    target: start-engine-uc
    kind: association
  start-includes-check:
    ref: VehicleSystem::UseCases::StartEngine::includesCheckFuel
    source: start-engine-uc
    target: check-fuel-uc
    kind: include
```

**Completeness rule:** the parser must warn if any `UseCase` owned by the subject, or any actor association, `includes:`, or `extends:` link declared in any use case's `.md` file within the subject scope, is absent from `shapes:` or `edges:`.

---

### 8.17 Architecture Decision Records (ADR)

An `ADR` file is a first-class model element that documents a significant design decision — particularly the rationale behind a requirement breakdown. It is referenced from native `Requirement` elements via the `breakdownAdr:` field (§8.11.6).

#### 8.17.1 Frontmatter Schema

| Field | Type | Required | Description |
|---|---|---|---|
| `type` | literal `ADR` | **Required** | Discriminator. |
| `id` | string | **Required** | Stable opaque ID matching `^ADR(-[A-Z0-9]{2,12})+-[0-9]{3}$`. Unique across the model. Never changes. |
| `title` | string | **Required** | One-line summary. Max 120 chars. |
| `status` | enum | **Required** | Lifecycle state: `proposed`, `accepted`, `deprecated`, `superseded`. |
| `date` | string | optional | ISO-8601 date the decision was made (e.g., `"2026-05-26"`). |
| `deciders` | list of strings | optional | Qualified names of stakeholder `PartDef` elements or free-text names of the decision-makers. |
| `tags` | list of strings | optional | Free labels for filtering/grouping. |

**ID pattern:** `^ADR(-[A-Z0-9]{2,12})+-[0-9]{3}$`
- Prefix `ADR`, one or more uppercase-alphanumeric segments (2–12 chars), three-digit suffix.
- Examples: `ADR-SYS-001`, `ADR-SW-SCHED-001`, `ADR-UAV-PWR-002`

**Status values:**

| Value | Meaning |
|---|---|
| `proposed` | Decision is under review; not yet ratified. An approved Requirement must not reference a `proposed` ADR (warning `W303`). |
| `accepted` | Decision is ratified and in effect. |
| `deprecated` | Decision is outdated but not replaced. |
| `superseded` | Decision has been replaced by another ADR; reference the newer ADR from `breakdownAdr:`. |

#### 8.17.2 Body Structure

Recommended layout — free-form Markdown:

```
## Context

<Why this decision was needed; the forces at play.>

## Decision

<The decision made and the breakdown it describes.>

## Consequences

<What becomes easier or harder; impact on the requirement hierarchy.>
```

#### 8.17.3 Placement Convention

ADR files live in a dedicated `ADRs/` package at or near the model root, or in a sub-package alongside the requirements they concern:

```
model/
  ADRs/
    _index.md         # type: Package, name: ADRs
    SYS-001.md        # type: ADR, id: ADR-SYS-001
    SW-SCHED-001.md   # type: ADR, id: ADR-SW-SCHED-001
```

#### 8.17.4 Complete Example

```markdown
---
type: ADR
id: ADR-SW-SCHED-001
title: "Decompose REQ-UAV-SCHED-001 into scheduler and bitmap sub-requirements"
status: accepted
date: "2026-05-20"
deciders:
  - Stakeholders::SystemsEngineer
  - Stakeholders::SoftwareArchitect
tags:
  - scheduler
  - requirement-breakdown
---

## Context

REQ-UAV-SCHED-001 specifies O(1) real-time scheduling in aggregate. During
software architecture review it became clear that the requirement conflates two
independently verifiable sub-properties: the data structure selection (bitmap)
and the worst-case execution time (WCET) guarantee. Separate requirements allow
independent test coverage and different SIL assignments.

## Decision

REQ-UAV-SCHED-001 is decomposed into:
- REQ-SCHED-BITMAP-001 — bitmap-based priority selection
- REQ-SCHED-WCET-001 — WCET guarantee at the hardware platform clock rate

Both child requirements carry `derivedFrom: [REQ-UAV-SCHED-001]` and
`breakdownAdr: ADR-SW-SCHED-001`. The parent requirement retains
`status: approved` but must not appear in any `satisfies:` list (rule §12.4).

## Consequences

- Each child requirement can be assigned to a distinct software module.
- Verification is split: REQ-SCHED-BITMAP-001 is covered by unit tests (L1/L2);
  REQ-SCHED-WCET-001 requires WCET analysis tooling (L3/L4).
- The parent requirement REQ-UAV-SCHED-001 becomes a leaf-suppressed node in
  the coverage report.
```

---

### 8.18 Safety and Security Analysis Elements

This section defines element types used in functional safety (ISO 26262, IEC 61508, ISO 13849-1) and cybersecurity analysis (ISO/SAE 21434). These are **native** element types — like `Requirement` and `TestCase` they carry stable opaque IDs and are dispatched by the parser on `type:` field.

#### 8.18.1 Tier 2 — HARA Elements

Used in Hazard Analysis and Risk Assessment (HARA) per ISO 26262-3 or IEC 61508.

| Element type | ID pattern | Description |
|---|---|---|
| `HazardousEvent` | `HE-*` | A combination of a hazard and an operational situation; carries ISO 26262 risk parameters (`severity`, `exposure`, `controllability`) or IEC 61508 risk graph parameters (`consequence`, `freqExposure`, `avoidance`, `demandRate`). |
| `SafetyGoal` | `SG-*` | A top-level safety requirement derived from the HARA; carries `asilLevel:` (ISO 26262), `silLevel:` (IEC 61508), or `plLevel:` (ISO 13849-1), and `hazardousEvents:` referencing the events it addresses. |

**Integrity level rules (W801, W806, E841, W808):** A `SafetyGoal` must carry an integrity level (W801). It must reference at least one `HazardousEvent` via `hazardousEvents:` (W806). Any `Requirement` derived from a `SafetyGoal` via `derivedFromSafetyGoal:` must carry the same integrity level field (E841), and may carry a lower level only when `breakdownAdr:` is set (W808; see §12.7).

#### 8.18.2 Tier 2 — TARA Elements

Used in Threat Analysis and Risk Assessment (TARA) per ISO/SAE 21434.

| Element type | ID pattern | Description |
|---|---|---|
| `DamageScenario` | `DS-*` | An adverse consequence to a stakeholder; carries `damageSeverity:` and `impactCategories:`. May carry `hazardRef:` (string or list) linking it to the `HazardousEvent`/`SafetyGoal` it endangers (safety↔security co-engineering). |
| `ThreatScenario` | `TS-*` | A potential attack scenario; carries `attackFeasibility:` and `attackVector:`. References `damageScenarios:`. May carry a direct `hazardRef:` (string or list) to a `HazardousEvent`/`SafetyGoal`, a `riskTreatment:` (`avoid`/`reduce`/`share`/`retain`), and a free-text `residualRisk:`. |
| `CybersecurityGoal` | `CSG-*` | A high-level security requirement; carries `securityProperty:` (`confidentiality`, `integrity`, `availability`, `authenticity`), `calLevel:` (`CAL1`–`CAL4`), and `threatScenarios:` (the `TS-*` threats it counters). |
| `SecurityControl` | `SC-*` | A concrete countermeasure; carries `controlType:` and `implementsGoals:`. |
| `VulnerabilityReport` | `VR-*` | A tracked vulnerability; carries `cvssScore:`, `mitigatedBy:`, and `affectedElements:`. |
| `TARASheet` | `TARA-*` | An Option-B container: a single file whose `damageTable:`, `threatTable:`, `goalTable:`, and `controlTable:` sections are exploded at parse time into the individual Tier 2 element types above. |

**Cross-reference rules:** A `Requirement` motivated by a cybersecurity goal should set `derivedFromSecurityGoal:` to the `CSG-*` ID, and must set `verificationMethod:` (W807). The OSLC link direction applies: the downstream element holds the reference.

**Safety↔security co-engineering (ISO 26262 ⇄ ISO/SAE 21434):** A `DamageScenario`/`ThreatScenario` may declare `hazardRef:` (string or list) pointing to the `HazardousEvent`/`SafetyGoal` it endangers, resolved by `id` or qualified name. A `hazardRef` that does not resolve, or resolves to a non-`HazardousEvent`/non-`SafetyGoal` element, is an error (E844). A `DamageScenario` whose `impactCategories:` includes `safety` but has no `hazardRef` warns W030 (opt-in, gateable with `--deny W030`). The `co-analysis` command (§ CLI) reports, per safety goal/hazard, the cyber threats that can violate it.

**Cybersecurity risk determination (ISO/SAE 21434 §15.8–15.9):** Each `ThreatScenario` has a computed risk level. Severity rank = max `damageSeverity` over its resolved `damageScenarios` (`negligible`=0, `moderate`=1, `major`=2, `severe`=3); feasibility rank from `attackFeasibility` (`very_low`=0, `low`=1, `medium`=2, `high`=3). If either is unknown the risk is **unknown** (listed, not gated); otherwise `score = severity + feasibility` (0..6) → **low** (0–1), **medium** (2–3), **high** (4), **critical** (5–6). A `ThreatScenario` records its risk-treatment decision with `riskTreatment:` (`avoid`/`reduce`/`share`/`retain`; invalid → E845) and an optional free-text `residualRisk:`. A high/critical-risk threat with no `riskTreatment` that is not listed by any `CybersecurityGoal.threatScenarios` warns W031; a `CybersecurityGoal` whose `calLevel` is below the expected CAL for its threats' max risk (low→CAL1 … critical→CAL4) warns W032. Both are gateable with `--deny` and promotable via `[profiles]`. The `cyber-risk` command (§ CLI) lists every threat with its risk and treatment.

**Binding SecurityControls to architecture:** Architecture elements (e.g. `PartDef`) that realise a `SecurityControl` should set `allocatedFrom:` to the control's `SC-*` ID. Both `allocatedFrom:` and `allocatedTo:` accept a single string or a list of strings to support multiple controls per element.

#### 8.18.3 Tier 4 — Fault Tree Analysis (FTA)

| Element type | ID pattern | Description |
|---|---|---|
| `FaultTree` | `FT-*` | Root of a fault tree; references a `SafetyGoal` via `topEvent:`. |
| `FaultTreeGate` | `FTG-*` | Logic gate; `gateType:` is one of `AND`, `OR`, `XOR`, `NOT`, `inhibit`; `inputs:` lists child gate/event IDs. |
| `FaultTreeEvent` | `FTE-*` | Leaf event; `eventKind:` is `basic`, `undeveloped`, or `house`; optional `failureRate:` (λ, /h), `diagnosticCoverage:` (DC), `latentDiagnosticCoverage:` (DCl) — DC/DCl in `0.0`–`1.0` (E846), inputs to the quantitative metrics roll-up (§ above). |

**Nesting rule (W900):** `FaultTreeGate` and `FaultTreeEvent` elements must be placed in a subdirectory named after the `FaultTree` file so their qualified names are prefixed by the tree's qualified name:

```
Safety/FTA/FT-BRAKE-001.md          →  Safety::FTA::FT-BRAKE-001
Safety/FTA/FT-BRAKE-001/
  FTG-BRAKE-001.md                  →  Safety::FTA::FT-BRAKE-001::FTG-BRAKE-001
  FTE-BRAKE-001.md                  →  Safety::FTA::FT-BRAKE-001::FTE-BRAKE-001
```

#### 8.18.4 Tier 4 — FMEA

| Element type | ID pattern | Description |
|---|---|---|
| `FMEASheet` | `FMEA-*` | Container with an `entries:` list; each entry is a failure mode row with `failureMode:`, `effect:`, `cause:`, `fmeaSeverity:`, `occurrence:`, `detection:` (1–10 each), optional `rpn:` (auto-computed when absent), and a `recommendedAction:` mitigation. |

Each row is synthesised at parse time into a virtual `FMEAEntry` element (`FM-*` ID) for cross-reference and validation purposes.

---

## 9 Variability and Variation Points

### 9.1 Variation Definitions

A variation definition is an abstract definition whose direct sub-usages are all variants. Declare with `isVariation: true`:

```yaml
---
type: PartDef
name: EngineChoices
typedBy: VehicleSystem::Engine
isVariation: true
isAbstract: true
---
Variation point for engine selection in the vehicle family.
```

### 9.2 Variant Usages

Each variant is declared with `isVariant: true` and must be a direct usage member of the variation:

```yaml
---
type: Part
name: fourCylEngine
typedBy: Powertrain::FourCylinderEngine
isVariant: true
# This file lives inside EngineChoices/ directory
---
Four-cylinder engine variant.
```

### 9.3 Directory Layout for Variations

Variants live as files within a subdirectory named after the variation:

```
model/VehicleSystem/
  Engine.md                  # type: PartDef — base engine definition
  EngineChoices/
    _index.md                # type: PartDef, isVariation: true, typedBy: Engine
    FourCylinderEngine.md    # type: Part, isVariant: true
    SixCylinderEngine.md     # type: Part, isVariant: true
    ElectricMotor.md         # type: Part, isVariant: true
```

### 9.4 Variant References

A non-variant usage can be declared to act as a variant of a separately declared variation by using the `variantOf:` field:

```yaml
# On a Part or PartDef file that already exists elsewhere
variantOf: VehicleSystem::EngineChoices
```

| Field | YAML type | Default | Description |
|---|---|---|---|
| `variantOf` | string | absent | Qualified name of the variation this element acts as a variant of |

---

## 9.5 Product Line Engineering Overview

Sections 9.1–9.4 cover **structural variation** — SysML's built-in mechanism for declaring alternative implementations of a definition within a single model. This section (§9.5–§9.12) covers **Product Line Engineering (PLE)** — a separate, higher-level concern that governs which combination of features constitutes a valid product, and propagates that selection through every engineering layer.

### Relationship between structural variation and PLE

Structural variation (`isVariation` / `isVariant`) and PLE features are complementary:

| Mechanism | Answers | Expressed as |
|---|---|---|
| `isVariation` / `isVariant` | What alternative implementations exist? | Model structure |
| `FeatureDef` | What can a customer choose? | Problem-space feature model |
| `Configuration` | Which choices define this product? | Feature selection + parameter bindings |
| `appliesWhen:` | Which elements belong to this product? | Condition on any model element |

A variant part (`isVariant: true`) is typically *selected* by a `FeatureDef` — the two mechanisms work together. The `appliesWhen:` field on a variant ties it to the feature that activates it.

### Design principles

1. **Problem space / solution space separation.** The feature model (`FeatureDef` hierarchy under `SystemFeatures/`) is the problem space — it describes what varies from a customer perspective. Architecture, requirements, code, and tests are the solution space. The connection between them is `appliesWhen:` (solution → feature) and `satisfies:` (component configuration → system feature).
2. **Single binding direction.** The solution space points at features; features do not point at the solution space. This keeps the feature model implementation-agnostic.
3. **`appliesWhen:` is always a cross-reference, never an expression.** Feature conditions are structured as references to `FeatureDef` elements so they are typed, resolvable, and graph-traversable.
4. **Parametrization lives on features, not on configurations.** A `FeatureDef` declares what parameters it carries; a `Configuration` assigns values to them. This separates schema from data.
5. **Two-level feature models reduce complexity.** A system-level feature model captures product-visible variability; component-level feature models capture implementation variability. A system `Configuration` selects system features; a component `Configuration` implements them.

---

## 9.6 `FeatureDef` — Feature Model Element

A `FeatureDef` represents one node in a feature model tree. It may be a leaf feature (a concrete selectable characteristic) or a composite group node (organising child features). Feature models are built entirely from `FeatureDef` elements arranged in a directory hierarchy under a dedicated package (e.g., `SystemFeatures/` or `<Package>/Features/`).

### Frontmatter schema

| Field | YAML type | Required | Default | Description |
|---|---|---|---|---|
| `type` | literal `FeatureDef` | **Required** | — | Discriminator. |
| `name` | string | optional | filename stem | Display name of the feature. |
| `groupKind` | enum | optional | `optional` | How this feature's **children** are grouped (see table below). |
| `mandatory` | bool | optional | `false` | **Membership** of this feature relative to its parent, orthogonal to `groupKind`: `true` means the feature is selected whenever its parent is (parent ⇒ feature), or always selected when top-level. A node may be both `mandatory: true` and `groupKind: alternative` — a mandatory XOR group (every product selects exactly one child). The legacy `groupKind: mandatory` is a shorthand for `mandatory: true` on a feature that sets no `mandatory:` field. |
| `cardinality` | string | optional | see groupKind | For `or` and `alternative` groups: how many children may be selected simultaneously, as a multiplicity string (§6). |
| `parentFeature` | string | optional | derived from directory | Qualified name of the parent `FeatureDef`. Normally inferred from the directory hierarchy; set explicitly only when the file cannot be placed at the canonical path. |
| `requires` | list of strings | optional | absent | Qualified names of other `FeatureDef` elements that must also be selected when this feature is selected (cross-tree implication). |
| `excludes` | list of strings | optional | absent | Qualified names of `FeatureDef` elements that must NOT be selected when this feature is selected (mutual exclusion). |
| `defaultValue` | bool or string | optional | `false` for optional; see groupKind | Default selection state when not explicitly bound in a `Configuration`. |
| `parameters` | list of parameter maps | optional | absent | Typed parameters carried by this feature (see §9.7). |
| `contributesTo` | string | optional | absent | Qualified name of a system-level `FeatureDef` that this component-level feature partially realises. For fine-grained two-level traceability (optional; the primary binding is `satisfies:` on `Configuration`). |
| `tags` | list of strings | optional | absent | Free labels. |

### `groupKind` values

`groupKind` describes how a feature's **children** are grouped. Membership of the feature itself (mandatory vs optional relative to its parent) is the separate `mandatory:` field above — the two are orthogonal (`ADR-FM-003`).

| Value | Meaning | Default cardinality | Default for children |
|---|---|---|---|
| `optional` | Children may be selected independently of each other | `"0..1"` | independent |
| `alternative` | Exactly one child must be selected (XOR group) | `"1"` | `"0..1"` each |
| `or` | One or more children may be selected | `"1..*"` | `"0..1"` each |
| `mandatory` | **Legacy shorthand** for `mandatory: true` membership (a leaf that is always selected when its parent is); prefer the `mandatory:` field, especially on group nodes | `"1"` | n/a |

The `cardinality:` field overrides the default. For example, an `or` group requiring at least two children: `cardinality: "2..*"`.

### Feature model directory convention

```
SystemFeatures/
  _index.md                 # Package for the feature model root
  Propulsion/
    _index.md               # FeatureDef, mandatory: true + groupKind: alternative — every product picks exactly one
    QuadRotor.md            # FeatureDef, optional
    HexRotor.md             # FeatureDef, optional
  Safety/
    _index.md               # FeatureDef, groupKind: or — one or more safety features
    DualIMU.md              # FeatureDef, optional
    ASIL_D_FC.md            # FeatureDef, optional; requires: [SystemFeatures::Safety.DualIMU]
  Communication/
    StandardLink.md         # FeatureDef, mandatory
    LongRangeLink.md        # FeatureDef, optional
```

### Complete `FeatureDef` example

```yaml
---
type: FeatureDef
name: HexRotorPropulsion
groupKind: optional
requires: []
excludes:
  - SystemFeatures::Propulsion.QuadRotorPropulsion
parameters:
  - name: numMotors
    type: ScalarValues::Integer
    value: 6
    isFixed: true
  - name: motorKV
    type: ScalarValues::Real
    unit: SI::rpm_per_volt
    range: "900..1200"
    default: 1000.0
    isRequired: true
  - name: propDiameterIn
    type: ScalarValues::Real
    range: "10..15"
    default: 12.0
tags:
  - propulsion
  - heavy-lift
---

Selects the six-rotor hexacopter propulsion configuration, providing higher
thrust margin and payload capacity at the cost of increased mass and power.
```

---

## 9.7 Feature Parametrization

Feature parameters allow a `FeatureDef` to carry typed, named values that must be bound to concrete values in any `Configuration` that selects the feature. This is the mechanism for quantitative variability — not just *whether* a feature is selected, but *how* it is configured.

### Parameter schema

Each entry in a `FeatureDef`'s `parameters:` list is a map with the following fields:

| Field | YAML type | Required | Default | Description |
|---|---|---|---|---|
| `name` | string | **Required** | — | Parameter name, unique within this `FeatureDef`. |
| `type` | string | **Required** | — | Qualified name of the parameter type. Must resolve to a scalar type (`ScalarValues::Real`, `ScalarValues::Integer`, `ScalarValues::String`, `ScalarValues::Boolean`, `ScalarValues::Natural`) or an `EnumerationDef`. |
| `unit` | string | optional | absent | Qualified name of the SI unit. Applies only when `type` is a quantity type. |
| `range` | string | optional | absent | Inclusive bounds as a multiplicity-style string `"min..max"`. Both bounds are literal numbers. Applies to numeric types only. |
| `enumValues` | list of strings | optional | absent | Explicit set of valid string values when `type` is `ScalarValues::String` and finer constraint than a full `EnumerationDef` is desired. |
| `default` | scalar | optional | absent | Default value used when the parameter is not explicitly bound in a `Configuration`. If absent and `isRequired: true`, every selecting `Configuration` must bind it. |
| `isFixed` | bool | optional | `false` | If `true`, the parameter value is fixed by the feature definition itself (the `default` or `value` field) and may not be overridden in a `Configuration`. Validation error if a `Configuration` attempts to bind it. |
| `isRequired` | bool | optional | `false` | If `true` and `isFixed: false`, every `Configuration` that selects this feature must explicitly bind this parameter. Validation warning `W010` if absent. |
| `value` | scalar | optional | absent | The fixed value, used when `isFixed: true`. Equivalent to `default` + `isFixed: true`. |
| `derivedFrom` | string | optional | absent | An opaque expression string (evaluated in the context of other parameters of the same feature) whose value is computed rather than bound. When present, the parameter is automatically `isFixed: true` and may not be bound in a `Configuration`. |
| `bindTo` | string | optional | absent | For component-level `FeatureDef` only: qualified path `SystemFeatures::<Feature>::<paramName>` of a system-level parameter to which this parameter is **propagated**. When a system `Configuration` binds the system parameter, the resolved component parameter inherits the same value. The component parameter may still specify its own `range:` as a narrowing constraint; validation error `E202` if the propagated value falls outside the narrower range. |
| `isArray` | bool | optional | `false` | If `true`, the parameter value is a YAML list of the declared `type`. Used for multi-valued parameters (e.g., a list of rotor positions in degrees). |
| `bindingTime` | enum | optional | absent | **When** the parameter's value is resolved, from the product-line-engineering binding-time triad, ordered earliest→latest: `compile` (build / code generation) · `load` (deployment / installation / startup) · `runtime` (dynamically, while the system runs). See *Binding time* below. An unrecognised value is `E230`. |

### Parameter kinds summary

| Kind | Fields | Behaviour |
|---|---|---|
| **Configurable** | `isFixed: false`, no `derivedFrom` | Must be bound (if `isRequired: true`) or uses `default` in each selecting `Configuration`. |
| **Fixed** | `isFixed: true` or `value:` present | Value is part of the feature definition. `Configuration` must not bind it. |
| **Derived** | `derivedFrom: "<expr>"` | Value is computed from sibling parameters. Implicitly fixed. |
| **Propagated** | `bindTo: "<system param path>"` | Component parameter inherits value from its bound system-level parameter. |
| **Array** | `isArray: true` | Value is a YAML list. `range:` applies per element. |

### Parameter binding in `Configuration`

In a `Configuration` file, parameter values are declared under `parameterBindings:` as a flat map. The key is the **canonical dotted parameter reference**: `<FeatureDef qualified name>.<parameter name>` — `::` separates the feature's namespace/qname segments, and a single `.` separates the parameter (member) from its owning feature, so the feature/parameter boundary is unambiguous. This same dotted form is used everywhere a feature parameter is referenced: `parameterBindings:` keys, `parameterConstraints` expressions, and `bindTo:` targets. A key written in the legacy all-`::` member form is malformed (`E222`).

```yaml
parameterBindings:
  SystemFeatures::Propulsion::HexRotorPropulsion.motorKV: 1050.0
  SystemFeatures::Propulsion::HexRotorPropulsion.propDiameterIn: 13.0
  SystemFeatures::Payload::SurveyCamera.resolutionMpx: 20.0
  SystemFeatures::Communication::LongRangeLink.frequencyBandGHz: 5.8
  SystemFeatures::Mission::Endurance.minDurationMin: 30.0
```

Rules:
- A `Configuration` must not bind a parameter of a feature that is not selected (`features: <feature>: false` or omitted when default is `false`). Validation error `E203`.
- A `Configuration` must not bind a parameter whose `isFixed: true`. Validation error `E204`.
- A bound value must satisfy the parameter's `range:` constraint. Validation error `E205`.
- A bound value must be a member of `enumValues:` (if declared). Validation error `E206`.

### Binding time

A parameter may declare an optional **`bindingTime:`** stating *when* in the product lifecycle its value is resolved — the standard product-line-engineering triad, **ordered** earliest → latest:

| `bindingTime:` | Resolved at |
|---|---|
| `compile` | build / code generation |
| `load` | deployment / installation / startup |
| `runtime` | dynamically, while the system executes |

`bindingTime:` is orthogonal to `isFixed:`/`value:` — those express a value *fixed in the model* (no variability), whereas `bindingTime:` records the lifecycle moment a value that genuinely varies is bound. It is optional; an **absent** `bindingTime:` is *unspecified* and opts the parameter out of every binding-time check below (existing models are unaffected). Rules:

- An unrecognised `bindingTime:` value (not `compile`/`load`/`runtime`) is `E230` (`validate`).
- A parameter computed from a source it depends on — a sibling named in its `derivedFrom:` expression, or its `bindTo:` target — must **not** bind earlier than that source: if both declare a `bindingTime:` and the dependent's is earlier, that is impossible and reported `E229` (`feature-check`).
- A `Configuration` that binds a `bindingTime: runtime` parameter is warned `W027` (`validate`): a runtime value is supplied by the running system, not at configuration time. For the same reason, the `W017` "required parameter unbound" warning is **suppressed** for a `runtime` parameter left unbound by a `Configuration`.

```yaml
parameters:
  - name: motorKV
    type: ScalarValues::Real
    range: "900..=1200"
    bindingTime: load        # chosen at deployment, not baked in at build
```

### Cross-feature parameter constraints

Parameter constraints that involve parameters from more than one feature are declared on the feature model package `_index.md` or on the `Configurations/` package `_index.md` (the configuration authority). Each constraint applies to every `Configuration` that selects all the referenced features.

```yaml
# SystemFeatures/_index.md or Configurations/_index.md
parameterConstraints:
  - id: PC-ENERGY-001
    expression: >
      SystemFeatures::Power::Battery.capacityWh
      >= (SystemFeatures::Mission::Endurance.minDurationMin / 60.0)
         * SystemFeatures::Power::AveragePower.averagePowerW
    severity: error
    rationale: "Battery capacity must cover the minimum mission duration at nominal power draw."
  - id: PC-THRUST-001
    expression: >
      SystemFeatures::Propulsion::HexRotorPropulsion.totalThrustN
      >= 2.0 * (SystemFeatures::Mass::MaxPayload.maxMassKg + 2.5) * 9.81
    severity: error
    appliesWhen:
      - SystemFeatures::Propulsion::HexRotorPropulsion
    rationale: "Hex-rotor thrust-to-weight ratio must be at least 2.0 at maximum payload."
```

`parameterConstraints:` entries:

| Field | YAML type | Required | Description |
|---|---|---|---|
| `id` | string | **Required** | Stable constraint ID, e.g. `PC-ENERGY-001`. Unique within the feature model. |
| `expression` | string | **Required** | A comparison `LHS <op> RHS` (`== != >= <= > <`) over arithmetic (`+ - * /`, parentheses) of numeric literals and dotted parameter references. Evaluated by `feature-check` against every applicable `Configuration`; a violation is `E221` (or `W025` when `severity: warning`). |
| `severity` | enum | optional | `error` (default → `E221`) or `warning` (→ `W025`). |
| `appliesWhen` | string or list | optional | Boolean predicate over `FeatureDef` qualified names (`and` / `or` / `not` / parentheses; a bare name or list = AND). The constraint is evaluated only against configurations whose selections satisfy it. Absent = always checked. |
| `rationale` | string | optional | Human-readable explanation. |

### Two-level parameter propagation

A component-level `FeatureDef` parameter may declare `bindTo:` to receive its value from a system-level parameter:

```yaml
# UAV/Propulsion/Features/SixMotorLayout.md
---
type: FeatureDef
name: SixMotorLayout
contributesTo: SystemFeatures::Propulsion.HexRotorPropulsion
parameters:
  - name: motorKV
    type: ScalarValues::Real
    unit: SI::rpm_per_volt
    range: "900..1200"
    bindTo: SystemFeatures::Propulsion::HexRotorPropulsion.motorKV
    # When the product-line configuration sets motorKV: 1050.0,
    # this component parameter is automatically resolved to 1050.0.
    # The range here is a narrowing constraint — validation error if the
    # inherited value falls outside 900..1200.
  - name: internalCoolingRequired
    type: ScalarValues::Boolean
    derivedFrom: "motorKV > 1100"
    # Derived: automatically true when motorKV exceeds 1100 rpm/V.
    # Not bindable in any Configuration.
  - name: rotorPositionsDeg
    type: ScalarValues::Real
    isArray: true
    value: [0.0, 60.0, 120.0, 180.0, 240.0, 300.0]
    isFixed: true
    # Fixed array parameter: six equally-spaced rotor positions.
---

Six-motor layout for hex-rotor propulsion. Motor KV is propagated from
the system-level feature parameter.
```

### Derived parameter evaluation

`derivedFrom:` is an opaque expression evaluated against the parameters of the same `FeatureDef`. The expression language is intentionally unspecified here — tools may support OCL, a simple arithmetic subset, or a custom language. The key semantics are:

- All referenced names must be parameters of the same `FeatureDef`.
- The expression is evaluated after all configurable parameters are bound, before cross-feature constraints are checked.
- Circular derivation (parameter A derived from B, B derived from A) is a validation error `E207`.

---

## 9.8 `Configuration` — Feature Selection

A `Configuration` is a complete, named feature selection with parameter bindings. It is the unit of product definition — selecting a `Configuration` uniquely determines a projected model slice.

### ID scheme

Configuration IDs follow the pattern `^CONF(-[A-Z0-9]{2,12})+-[0-9]{3}$`:
- System-level: `CONF-UAV-STD-001`, `CONF-UAV-HVY-001`
- Component-level: `CONF-PROP-HEX-001`, `CONF-AVI-DUAL-001`

### Frontmatter schema

| Field | YAML type | Required | Default | Description |
|---|---|---|---|---|
| `type` | literal `Configuration` | **Required** | — | Discriminator. |
| `id` | string | **Required** | — | Stable ID matching `CONF-*` pattern. |
| `name` | string | optional | filename stem | Display name. |
| `title` | string | **Required** | — | One-line summary. Max 120 chars. |
| `status` | enum | **Required** | — | `draft`, `review`, `approved`, `released`, `retired`. |
| `featureModel` | string | **Required** | — | Qualified name of the feature model package this configuration selects from. |
| `features` | map | **Required** | — | Feature selection: `FeatureDef qualified name: true/false`. All features in the model must appear; defaults apply for absent entries (see §9.6). |
| `parameterBindings` | map | optional | absent | Parameter value assignments keyed by the canonical dotted reference: `<FeatureDef qname>.<paramName>: <value>`. |
| `satisfies` | list of strings | optional | absent | For component `Configuration` only: qualified names of system-level `FeatureDef` elements this configuration realises. This is the sole top-down binding in the two-level model. |
| `derivedFrom` | string | optional | absent | Qualified name of a base `Configuration` this one extends. Inherits all `features:` and `parameterBindings:` of the base; entries in this file override the base. |
| `baselineRef` | string | optional | absent | Opaque external baseline reference (e.g., a Git tag, CM baseline ID). |
| `tags` | list of strings | optional | absent | Free labels. |

### Status lifecycle

| Value | Meaning |
|---|---|
| `draft` | Being assembled; feature selection incomplete. |
| `review` | Under formal review; parameter constraints checked. |
| `approved` | Approved for development; all constraints pass. |
| `released` | Shipped product; frozen. |
| `retired` | End-of-life; no longer produced. |

### System-level configuration example

```yaml
---
type: Configuration
id: CONF-UAV-HVY-001
title: "Heavy-lift survey UAV — hex-rotor, dual-IMU, long-range link"
status: approved
featureModel: SystemFeatures
features:
  SystemFeatures::Propulsion.QuadRotorPropulsion: false
  SystemFeatures::Propulsion.HexRotorPropulsion: true
  SystemFeatures::Safety.DualIMU: true
  SystemFeatures::Safety.ASIL_D_FC: false
  SystemFeatures::Communication.StandardLink: true
  SystemFeatures::Communication.LongRangeLink: true
  SystemFeatures::Payload.SurveyCamera: true
  SystemFeatures::Payload.MultispectralCamera: false
parameterBindings:
  SystemFeatures::Propulsion::HexRotorPropulsion.motorKV: 1050.0
  SystemFeatures::Propulsion::HexRotorPropulsion.propDiameterIn: 13.0
  SystemFeatures::Payload::SurveyCamera.resolutionMpx: 20.0
  SystemFeatures::Communication::LongRangeLink.frequencyBandGHz: 5.8
  SystemFeatures::Mission::Endurance.minDurationMin: 25.0
baselineRef: "UAV-HEAVY-LIFT-v2.1-RC3"
---

Heavy-lift variant targeting survey missions with payloads up to 1.5 kg.
```

### Configuration inheritance

A configuration may extend a base configuration, inheriting its feature selections and parameter bindings and overriding only the differing entries:

```yaml
---
type: Configuration
id: CONF-UAV-HVY-LR-001
title: "Heavy-lift UAV — extended-range variant"
status: draft
featureModel: SystemFeatures
derivedFrom: Configurations::HeavyLiftUAV   # inherits all from CONF-UAV-HVY-001
features:
  # Only overrides differ from the base
  SystemFeatures::Communication.LongRangeLink: true  # already true — no change
parameterBindings:
  # Override a single parameter; all others inherited from base
  SystemFeatures::Communication::LongRangeLink.frequencyBandGHz: 2.4
---

Long-range variant using 2.4 GHz band for better penetration in forested areas.
```

---

## 9.9 Two-Level Feature Models

### Motivation

A single flat feature model for a large product line becomes unmanageable. The two-level model partitions variability:

- **Level 1 — System Feature Model** (`SystemFeatures/`): customer-visible, implementation-agnostic. Describes *what* the product does differently between variants.
- **Level 2 — Component Feature Models** (`<Package>/Features/`): implementation-level variability within one subsystem. Describes *how* a component realises a system feature.

### Structural convention

```
model/
  SystemFeatures/          # Level 1 — one per product line
    _index.md              # type: Package
    Propulsion/
      QuadRotorPropulsion.md   # type: FeatureDef
      HexRotorPropulsion.md    # type: FeatureDef
    Safety/
      DualIMU.md
      ASIL_D_FC.md

  Configurations/          # System-level configurations (select Level 1 features)
    _index.md              # type: Package
                           # parameterConstraints: [...] declared here
    StandardSurveyUAV.md   # type: Configuration, featureModel: SystemFeatures
    HeavyLiftUAV.md        # type: Configuration, featureModel: SystemFeatures

  UAV/
    Propulsion/
      Features/            # Level 2 — one per contributing package
        _index.md          # type: Package
        SixMotorLayout.md  # type: FeatureDef, contributesTo: SystemFeatures::Propulsion.HexRotorPropulsion
        FourMotorLayout.md # type: FeatureDef, contributesTo: SystemFeatures::Propulsion.QuadRotorPropulsion
        ESCProtocol/
          DSHOT.md         # type: FeatureDef, groupKind: alternative (internal — no contributesTo)
          PWM.md           # type: FeatureDef, groupKind: alternative (internal)
      Configurations/      # Component-level configurations
        _index.md          # type: Package
        HexConfig.md       # type: Configuration, satisfies: [SystemFeatures::Propulsion.HexRotorPropulsion]
        QuadConfig.md      # type: Configuration, satisfies: [SystemFeatures::Propulsion.QuadRotorPropulsion]
```

### Binding rules

1. **A system `Configuration` selects system features.** It does not reference component configurations directly.
2. **A component `Configuration` declares `satisfies:`** listing the system-level `FeatureDef` elements it realises.
3. **Resolution:** when a system `Configuration` is evaluated, the resolver scans all component `Configuration` files across all packages, collects those whose `satisfies:` intersects the set of selected system features, and includes them in the resolved product description.
4. **Uniqueness constraint:** for any given system feature that is selected, exactly one component `Configuration` per package may satisfy it. If zero component configurations satisfy a selected system feature, validation error `E210`. If more than one in the same package satisfy it, validation error `E211`.
5. **Internal component features** (no `contributesTo:`) are invisible to the system level. They are resolved entirely within the component's own feature model.
6. **Cross-package feature dependencies are forbidden at Level 2.** A component `FeatureDef` must not reference `FeatureDef` elements from another component's feature model. All cross-component constraints live in the Level 1 feature model via `requires:` and `excludes:`.

### Parameter propagation in the two-level model

When a system `Configuration` binds a system parameter, and a component `FeatureDef` parameter declares `bindTo:` pointing to that system parameter:

1. The resolver reads the system parameter value from the system `Configuration.parameterBindings`.
2. It writes that value into the component parameter binding as if it were explicitly declared in the component `Configuration`.
3. The component's `range:` constraint is checked against the propagated value (error `E202` on violation).
4. The component's `derivedFrom:` expressions are evaluated after propagation.

The component `Configuration` does not need to redeclare propagated parameter values — they flow automatically.

### Component-level configuration example

```yaml
---
type: Configuration
id: CONF-PROP-HEX-001
title: "Six-rotor hex propulsion — DSHOT ESC protocol"
status: approved
featureModel: UAV::Propulsion::Features
features:
  UAV::Propulsion::Features::SixMotorLayout: true
  UAV::Propulsion::Features::FourMotorLayout: false
  UAV::Propulsion::Features::ESCProtocol.DSHOT: true
  UAV::Propulsion::Features::ESCProtocol.PWM: false
# motorKV is NOT bound here — it is propagated from SystemFeatures::Propulsion::HexRotorPropulsion.motorKV
# via SixMotorLayout.motorKV.bindTo
parameterBindings: {}
satisfies:
  - SystemFeatures::Propulsion.HexRotorPropulsion
---

Hex-rotor configuration using DSHOT ESC protocol. Motor KV is propagated
from the system-level configuration.
```

---

## 9.10 `appliesWhen:` — Conditional Model Elements

`appliesWhen:` is a universal field (§3.13) that conditions any model element on one or more feature selections. It is the mechanism by which the full model is **projected** onto a specific `Configuration`.

### Value forms

`appliesWhen:` accepts any of the following; all reduce to a boolean expression over `FeatureDef` qualified names:

1. **absent** — the element is unconditional (always active).
2. **bare qualified name** — `appliesWhen: Features::Wdt`. Active iff that feature is selected.
3. **list of qualified names** — `appliesWhen: [Features::A, Features::B]`. AND semantics (all selected). *(Legacy form; kept for back-compatibility.)*
4. **boolean expression** — `and`, `or`, `not`, and parentheses over qualified names:

```yaml
appliesWhen: "Features::CortexM and Features::Mpu"
appliesWhen: "Features::Async or Features::Bus"
appliesWhen: "not Features::Smp"
appliesWhen: "(Features::A or Features::B) and not Features::A"
```

Operator precedence is `not` > `and` > `or`; parentheses override. Every operand qualified name must resolve to a `FeatureDef` — an unresolved/non-`FeatureDef` operand, or a malformed expression, is error `E209`.

### Evaluation

Given a resolved `Configuration`, an element's `appliesWhen:` expression is evaluated against the configuration's `features:` selections (a feature absent from `features:` takes its `FeatureDef` default). The element is **included** iff the expression evaluates true (or is absent).

```
project(configuration, allElements) →
  for each element in allElements:
    include  if  element.appliesWhen is absent
             or  eval(element.appliesWhen, configuration.features) == true
    else exclude
```

The projected model is the input to all downstream tools (diagram generators, requirement coverage reports, integration test selection).

### Transitive package `appliesWhen` (the *effective* condition)

A **`Package`** (a namespace `_index.md`) may itself declare `appliesWhen:`. The condition then applies **transitively** to every element in that package's subtree — directly contained and through nested sub-packages — so a whole cohesive variant subtree (requirements + architecture + tests) can be enabled or disabled with a single declaration.

Every element therefore has an **effective condition**, evaluated by every projection/coverage/escaping consumer:

- the element's **own** `appliesWhen:` if it declares one; otherwise
- the `appliesWhen:` of the **nearest ancestor package** that declares one; otherwise
- always active.

Effective conditions are never combined. To keep them unambiguous, **at most one** node on any root-to-leaf path may declare `appliesWhen:`. Violations are error **`E228`**:

- a **nested** declaration — an element or sub-package `_index.md` that declares `appliesWhen:` while an ancestor package already does (even an identical restatement);
- a **forbidden target** — because the feature model and configurations are identified by *element type* (there is no dedicated package type), `appliesWhen:` may not be declared on a `FeatureDef` or a `Configuration`, a package declaring `appliesWhen:` may not contain any `FeatureDef`/`Configuration` in its subtree, and the **model-root** package may not declare it (gating the whole model).

A package that declares `appliesWhen:` but contains no projectable element gates nothing — warning **`W026`**.

Because a gated subtree moves together, references *between* elements inside it never escape; only a reference from outside into the gated subtree escapes when the condition is off (`E226`/`W019`). A package with `appliesWhen:` is the all-or-nothing tool; condition individual elements (where the enclosing package does **not** declare `appliesWhen:`) when you need a strict subset.

### `TestCase` variant membership — no `runsIn` field

`appliesWhen:` is also the sole mechanism that ties a `TestCase` to the variants it runs in: **a `TestCase` runs in a `Configuration` iff its `appliesWhen:` is satisfied by that configuration's `features:`.** A `TestCase` with no `appliesWhen:` is *configuration-agnostic* and runs in every `Configuration`. There is deliberately no separate `runsIn` field — membership is computed, not stored. (Where two configurations would otherwise have identical selections — e.g. an emulator vs a physical rig — model the distinguishing axis as a feature, such as an `ExecEnv` alternative group.)

This drives two downstream capabilities (both dormant unless the variability dimension is active — §9.10.1):

- the **`syscribe matrix`** command — a Requirement × Configuration coverage grid whose columns are the model's `Configuration` elements and whose cells are *N/A* (requirement not active in that variant), *covered*, or *gap*;
- the per-`Configuration` coverage rule **`W015`** (§9.11) — a requirement active in a configuration with no covering in-configuration `TestCase`.

### 9.10.1 Opt-in (the variability dimension is dormant by default)

The variability dimension is **active** only when both (a) at least one `FeatureDef` exists, and (b) at least one element links to it (a `Configuration`, or any element/`TestCase` carrying `appliesWhen:`). When it is dormant:

- validation output is identical to a model with no PLE elements (no `W015`, no projection);
- an `appliesWhen:` with no feature model present is inert — *except* that an unresolved reference is still `E209`;
- `syscribe matrix` prints a "no feature model present" notice and falls back to a flat requirement↔testcase view.

This lets a project adopt variability incrementally without disturbing existing validation.

### Usage across layers

```yaml
# Requirement — only applies to hex-rotor products
type: Requirement
id: REQ-UAV-THRUST-002
appliesWhen: SystemFeatures::Propulsion.HexRotorPropulsion

# Architecture variant
type: PartDef
name: HexRotorConfig
isVariant: true
variantOf: UAV::Propulsion::PropulsionSystem
appliesWhen: SystemFeatures::Propulsion.HexRotorPropulsion

# Test case — only run for hex-rotor products
type: TestCase
id: TC-UAV-THRUST-002
appliesWhen: SystemFeatures::Propulsion.HexRotorPropulsion
verifies:
  - REQ-UAV-THRUST-002

# Allocation — only applies when both hex-rotor AND dual-IMU are selected
type: Allocation
name: HexMixingAlgorithmAlloc
appliesWhen:
  - SystemFeatures::Propulsion.HexRotorPropulsion
  - SystemFeatures::Safety.DualIMU
```

### `appliesWhen:` at Level 2

Within a component package, `appliesWhen:` references component-level `FeatureDef` elements. This is resolved against the component `Configuration` (not the system configuration directly).

```yaml
# UAV/Propulsion/SixMotorMixing.md
type: ActionDef
name: SixMotorMixing
appliesWhen: UAV::Propulsion::Features::SixMotorLayout
sourceFile: "src/flight/mixing_hex.rs"
```

---

## 9.11 PLE Validation Rules

### Parse-time errors (per-element)

| Code | Condition |
|---|---|
| `E200` | `Configuration.id` does not match `CONF-*` pattern |
| `E201` | `FeatureDef` or `Configuration` missing a required field (`id`, `title`, `status`, `featureModel`) |
| `E202` | A propagated parameter value (via `bindTo:`) falls outside the component parameter's `range:` |
| `E203` | `Configuration.parameterBindings` binds a parameter for a feature not selected in this configuration |
| `E204` | `Configuration.parameterBindings` binds a parameter declared `isFixed: true` |
| `E205` | Bound parameter value violates `range:` constraint |
| `E206` | Bound parameter value is not a member of `enumValues:` |
| `E207` | Circular `derivedFrom:` dependency between parameters of the same `FeatureDef` |

### Model-time errors (cross-element)

| Code | Condition |
|---|---|
| `E208` | Duplicate `Configuration.id` across all files |
| `E209` | `appliesWhen:` references a qualified name that does not resolve to a `FeatureDef` |
| `E210` | A selected system feature has no component `Configuration` in some package satisfying it |
| `E211` | A selected system feature is satisfied by more than one component `Configuration` in the same package |
| `E212` | A `FeatureDef.requires:` or `excludes:` entry does not resolve to another `FeatureDef` |
| `E213` | A cross-feature `parameterConstraints` expression references a parameter path that does not resolve |
| `E214` | A `FeatureDef.contributesTo:` does not resolve to a `FeatureDef` in a system feature model |
| `E215` | A `Configuration.derivedFrom:` base configuration is not in `approved` or `released` status |
| `E216` | A `Configuration.features` map omits a `mandatory` feature (or sets it to `false`) |
| `E217` | A `Configuration.features` map selects both sides of an `alternative` group |
| `E218` | A `Configuration.features` map violates an `or` group's `cardinality:` constraint |
| `E219` | A `FeatureDef.requires:` constraint is violated by the selected features in a `Configuration` |
| `E220` | A `FeatureDef.excludes:` constraint is violated by the selected features in a `Configuration` |
| `E221` | A cross-feature `parameterConstraints` expression evaluates to `false` for a `Configuration` whose `appliesWhen:` predicate holds (default severity). Emitted by `feature-check`. |
| `E228` | Invalid `appliesWhen:` placement (§9.10): a declaration nested under a package that already declares `appliesWhen:`; or on a `FeatureDef`/`Configuration`, a package whose subtree contains one, or the model-root package. |
| `E229` | A parameter's `bindingTime:` is **earlier** than that of a `derivedFrom`/`bindTo` source it depends on — an impossible ordering (§9.7). Checked only when both endpoints declare a `bindingTime:`. Emitted by `feature-check`. |
| `E230` | A parameter declares a `bindingTime:` value that is not one of `compile`/`load`/`runtime` (§9.7). Emitted by `validate`. |

### Warnings

| Code | Condition |
|---|---|
| `W010` | A `Configuration` does not bind a parameter declared `isRequired: true` on a selected feature |
| `W011` | A `FeatureDef` with `groupKind: optional` is selected in zero `Configuration` files (possibly dead feature) |
| `W012` | A `FeatureDef` with `groupKind: optional` is selected in every `Configuration` (should be `mandatory`) |
| `W013` | A component `FeatureDef` has no `contributesTo:` and no `excludes:` referencing any system feature — internal feature not visible from system level (informational) |
| `W014` | A `parameterConstraint` has `appliesWhen:` that references a feature not in any `Configuration` |
| `W015` | A requirement is **active** in a `Configuration` (its `appliesWhen:` holds for that configuration's `features:`) but no non-draft `TestCase` that runs in that `Configuration` (§9.10) verifies it. Emitted only when the variability dimension is active (§9.10.1); draft requirements/tests are suppressed; gate with `--deny W015`. |
| `W016` | A `Configuration` parsed **zero** feature selections while a `FeatureDef` exists in the model — e.g. it used an unrecognized `selections:` key instead of the `features:` map (§9.8). Surfaces the otherwise-silent failure that yields an all-N/A coverage matrix. |
| `W017` | A selected feature declares a parameter `isRequired: true` (not fixed, no `default:`) that the `Configuration` does not bind. (This is §9.7's nominal `W010`; the validator uses `W017` because `W010` is taken by test-result ingestion.) |
| `W024` | An **orphan** `FeatureDef` — referenced by no element's `appliesWhen:` and selected `true` by no `Configuration`, so it gates nothing and ships in nothing. Emitted by `feature-check` only; gate with `--deny W024`. |
| `W025` | A `parameterConstraints` violation (as `E221`) where the constraint declares `severity: warning`. Emitted by `feature-check`; gate with `--deny W025`. |
| `W026` | A `Package` declares `appliesWhen:` but its subtree contains no projectable element (it gates nothing). Gate with `--deny W026`. |
| `W027` | A `Configuration` binds a parameter whose `bindingTime: runtime` — resolved by the running system, not at configuration time (§9.7). Gate with `--deny W027`. |
| `W028` | The same `extRef` external reference is declared by two or more elements (§3). One finding per duplicated value. Gate with `--deny W028`. |

> **Implementation note.** Rules split across commands/modes:
> - **`validate`** (per-element, always on) enforces the single-level parameter binding rules `E203`–`E206`, the unresolved-path error `E222`, `W017`, and the binding-time rules `E230` (invalid value) and `W027` (Configuration binds a `runtime` parameter; `W017` is suppressed for `runtime`).
> - **`feature-check`** (explicit, holistic) enforces the feature-model-wide rules: `E212` (requires/excludes resolution), `E219`/`E220` (requires/excludes satisfaction), `E207` (circular `derivedFrom:`), `E202` (`bindTo:` propagation range), `E229` (binding-time ordering across `derivedFrom`/`bindTo`), `E213` (unresolved **or `::`-member** `parameterConstraints` path), `E221`/`W025` (`parameterConstraints` expression evaluation), `W011`/`W012`/`W014`, and `W024` (orphan feature). It **also** re-runs the parameter-binding rules (`E203`–`E206`/`E222`/`W017`) so a product line checked holistically gets the same range/binding enforcement as `validate`.
> - **`feature-check --deep`** (SAT-backed, over a propositional encoding of the Boolean layer; deterministic; engine is batsat (pure-Rust CDCL) — see `ADR-FM-002`) adds whole-configuration-space analysis: `E223` void model, `E224` dead feature, `E225` invalid configuration (full group/cardinality semantics), `W018` false-optional, plus a reported set of *core* features and a conflict-set explanation for each unsatisfiability.
>
> Not yet implemented: group-cardinality *findings* on `feature-check` without `--deep` (`E216`/`E217`/`E218` — `--deep` enforces the group semantics via `E225`), two-level satisfies completeness (`E210`/`E211`), and general numeric/parameter (SMT) reasoning beyond the comparison/arithmetic grammar `E221` evaluates. `E222`–`E225`/`E229`/`E230` and `W017`/`W018`/`W024`/`W025`/`W027`/`W028` are implementation codes beyond the spec table.

---

## 9.12 PLE Graph Edges

After all files are loaded, the model graph (§11.8) is extended with the following PLE-specific edge types:

| Edge kind | Source | Target | Description |
|---|---|---|---|
| `ConditionalOn` | Any element with `appliesWhen:` | `FeatureDef` | Element is conditionally included when this feature is selected |
| `Selects` | `Configuration` | `FeatureDef` (selected=true) | Configuration selects this feature |
| `Deselects` | `Configuration` | `FeatureDef` (selected=false) | Configuration explicitly deselects this feature |
| `BindsParameter` | `Configuration` | `FeatureDef` | Configuration binds at least one parameter of this feature |
| `Satisfies` | Component `Configuration` | System `FeatureDef` | Component configuration realises this system feature |
| `ContributesTo` | Component `FeatureDef` | System `FeatureDef` | Fine-grained realization link between feature levels |
| `FeatureRequires` | `FeatureDef` | `FeatureDef` | Cross-tree implication (requires:) |
| `FeatureExcludes` | `FeatureDef` | `FeatureDef` | Mutual exclusion (excludes:) |
| `ParameterBoundTo` | Component param | System param | Parameter propagation via bindTo: |
| `DerivedFrom` | `Configuration` | Base `Configuration` | Configuration inheritance |

These edges enable the following standard queries without full model traversal:

- **All elements for a configuration**: follow `Selects` edges from the `Configuration`, then collect all elements with matching `ConditionalOn` targets.
- **Requirement coverage within a configuration**: filter the `Verifies` edges (§11.8) to those whose source and target are both included in the projected configuration.
- **Feature impact analysis**: given a `FeatureDef`, find all elements reachable via incoming `ConditionalOn` edges — the complete set of model elements that change when this feature is toggled.
- **Dead feature detection**: `FeatureDef` nodes with no incoming `Selects` edges from any `Configuration` with `status: approved` or higher.

---

## 10 Worked Examples

### 10.1 Vehicle System Structure (BDD + IBD)

**Directory layout:**

```
model/
  _index.md
  VehicleSystem/
    _index.md
    Vehicle.md
    Engine.md
    Transmission.md
    FuelTank.md
    vehicle_b.md
  Interfaces/
    _index.md
    PowerPortDef.md
    FuelPortDef.md
    PowerInterface.md
  Powertrain/
    _index.md
    FourCylinderEngine.md
    AutomaticTransmission.md
```

**`model/_index.md`:**

```yaml
---
type: Namespace
imports:
  - ISQ::*
  - SI::*
  - ScalarValues::*
---
Root namespace for the Simple Vehicle Model.
```

**`model/VehicleSystem/_index.md`:**

```yaml
---
type: Package
name: VehicleSystem
---
Contains all top-level definitions for the vehicle system.
```

**`model/VehicleSystem/Vehicle.md`:**

```yaml
---
type: PartDef
name: Vehicle
isAbstract: true
features:
  - name: mass
    typedBy: ISQ::MassValue
    unit: SI::kg
    multiplicity: "1"
  - name: powerOut
    type: Port
    typedBy: Interfaces::PowerPortDef
    direction: out
  - name: fuelIn
    type: Port
    typedBy: Interfaces::FuelPortDef
    direction: in
  - name: engine
    type: Part
    typedBy: VehicleSystem::Engine
    multiplicity: "1"
  - name: fuelTank
    type: Part
    typedBy: VehicleSystem::FuelTank
    multiplicity: "1"
connections:
  - from: fuelTank.fuelOut
    to: engine.fuelIn
    typedBy: Interfaces::FuelInterface
performs:
  - VehicleBehavior::ProvidePower
exhibitsStates:
  - VehicleBehavior::VehicleStates
---
Abstract definition of a vehicle. Concrete vehicle types specialize this.
```

**`model/VehicleSystem/Engine.md`:**

```yaml
---
type: PartDef
name: Engine
isAbstract: true
features:
  - name: displacement
    typedBy: ISQ::VolumeValue
    unit: SI::L
  - name: maxPower
    typedBy: ISQ::PowerValue
    unit: SI::W
  - name: powerOut
    type: Port
    typedBy: Interfaces::PowerPortDef
    direction: out
  - name: fuelIn
    type: Port
    typedBy: Interfaces::FuelPortDef
    direction: in
---
Abstract engine definition. Concrete engine types must specialize this.
```

**`model/VehicleSystem/vehicle_b.md`:**

```yaml
---
type: Part
name: vehicle_b
typedBy: VehicleSystem::Vehicle
features:
  - name: engine
    type: Part
    typedBy: Powertrain::FourCylinderEngine
    redefines: [VehicleSystem::Vehicle::engine]
  - name: fuelTank
    type: Part
    typedBy: VehicleSystem::FuelTank
    features:
      - name: capacity
        typedBy: ISQ::VolumeValue
        value: "60.0"
        unit: SI::L
metadata:
  - type: ModelingMetadata::StatusInfo
    status: approved
    approver: "Chief Engineer"
---
Concrete vehicle configuration B with a four-cylinder gasoline engine
and a 60 L fuel tank.
```

### 10.2 Requirements Package

**Directory layout:**

```
model/
  Requirements/
    _index.md
    MassRequirementDef.md
    SpeedRequirementDef.md
    vehicleSpec/
      _index.md
      vehicleMassReq.md
      vehicleSpeedReq.md
  Stakeholders/
    _index.md
    VehicleEngineer.md
    SafetyEngineer.md
    Customer.md
  Concerns/
    _index.md
    VehicleWeight.md
    MaxSpeed.md
```

**`model/Requirements/MassRequirementDef.md`:**

```yaml
---
type: RequirementDef
name: MassRequirementDef
subject: VehicleSystem::Vehicle
parameters:
  - name: maxMass
    typedBy: ISQ::MassValue
    direction: in
requires:
  - expression: "subject.mass <= maxMass"
    expressionLanguage: ocl
stakeholders:
  - Stakeholders::VehicleEngineer
concerns:
  - Concerns::VehicleWeight
---
Requirement that the vehicle total mass shall not exceed the specified maximum.
```

**`model/Requirements/vehicleSpec/vehicleMassReq.md`:**

```yaml
---
type: Requirement
name: vehicleMassReq
typedBy: Requirements::MassRequirementDef
features:
  - name: maxMass
    typedBy: ISQ::MassValue
    value: "1500.0"
    unit: SI::kg
---
The vehicle mass shall not exceed 1500 kg in the unladen configuration.
```

**`model/VehicleSystem/Vehicle.md`** (updated to include satisfaction):

```yaml
satisfies:
  - Requirements::vehicleSpec::vehicleMassReq
```

### 10.3 Action Tree (Use Case)

**`model/VehicleBehavior/TransportPassenger.md`:**

```yaml
---
type: UseCaseDef
name: TransportPassenger
subject: VehicleSystem::vehicle_b
actors:
  - Stakeholders::Driver
objectives:
  - "Safely transport a passenger to the requested destination"
subActions:
  - name: startVehicle
    typedBy: VehicleBehavior::StartEngine
    kind: Action
  - name: navigate
    typedBy: VehicleBehavior::DriveToDestination
    kind: Action
  - name: stopVehicle
    typedBy: VehicleBehavior::StopEngine
    kind: Action
successionConnections:
  - after: startVehicle
    before: navigate
  - after: navigate
    before: stopVehicle
---
Use case for transporting a passenger from origin to destination.
```

### 10.4 Allocation (Logical to Physical)

**`model/Allocations/_index.md`:**

```yaml
---
type: Package
name: Allocations
---
```

**`model/Allocations/FunctionalAllocation.md`:**

```yaml
---
type: Allocation
name: functionalAllocation
typedBy: Allocations::FunctionalToPhysical
allocateFrom: VehicleBehavior::ProvidePower
allocateTo: PhysicalArch::EngineControlUnit
metadata:
  - type: ModelingMetadata::Rationale
    text: "ECU is the only processing unit with real-time control authority over engine"
---
Allocates the ProvidePower behavioral function to the Engine Control Unit.
```

### 10.5 Metadata Annotation Example

**`model/Metadata/StatusInfo.md`:**

```yaml
---
type: MetadataDef
name: StatusInfo
features:
  - name: status
    typedBy: ScalarValues::String
  - name: approver
    typedBy: ScalarValues::String
  - name: approvalDate
    typedBy: ScalarValues::String
annotates:
  - PartDef
  - Part
  - RequirementDef
  - Requirement
  - ActionDef
  - Action
---
Metadata for tracking the approval status of model elements.
```

**Applied to a PartDef:**

```yaml
---
type: PartDef
name: FuelSystem
metadata:
  - type: Metadata::StatusInfo
    status: approved
    approver: "Jane Smith"
    approvalDate: "2026-03-15"
---
Fuel system component.
```

---

## 11 Parser and Tool Contract

This section defines the normative behavior required of a conformant Markdown-SysML parser.

### 11.1 File Discovery

1. The parser is initialized with a **model root** directory path.
2. The parser **recursively** walks all subdirectories of the model root.
3. All files with the `.md` extension are candidate element files.
4. Files and directories matching any of the following patterns are ignored:
   - `.git/`, `.github/`, `node_modules/`, `target/`, `dist/`
   - Files/directories whose names begin with `.` (hidden files)
   - A `.sysmlignore` file at the model root may list additional ignore glob patterns, one per line, using standard gitignore syntax.
5. A file named `_index.md` represents the package for its containing directory.
6. Files whose YAML frontmatter is absent or cannot be parsed MUST produce a parser warning and be skipped; they do not cause a fatal error.

### 11.2 Frontmatter Extraction

1. The YAML frontmatter is delimited by two `---` lines at the beginning of the file. The opening `---` must be the first line (no preceding whitespace).
2. The content between the two `---` delimiters is parsed as YAML 1.2.
3. The remaining content after the closing `---` is the Markdown documentation body.
4. Files with no frontmatter are skipped with a warning.
5. Files where the frontmatter does not contain a `type:` field are skipped with a warning.

### 11.3 Qualified Name Derivation Algorithm

Given a file at path `<root>/<seg1>/<seg2>/.../<segN>/<filename>.md`:

1. Collect the path segments from the model root to the file, exclusive of the root itself.
2. For each intermediate directory segment, the package name is:
   - The `name:` field in `<segN>/_index.md` if present.
   - Otherwise the directory name itself.
3. For the file itself, the element name segment is:
   - The `name:` field in the file's frontmatter if present.
   - Otherwise the filename stem (filename without `.md`).
   - Exception: `_index.md` contributes no name segment; it represents the directory's package.
4. The qualified name is `seg1::seg2::...::segN::elementName`.
5. The root namespace itself has no name segment.

**Example derivation:**

- File: `model/VehicleSystem/Powertrain/Engine.md`
- Model root: `model/`
- Intermediate directories: `VehicleSystem` → `Powertrain`
- Filename stem: `Engine`
- Qualified name: `VehicleSystem::Powertrain::Engine`

If `model/VehicleSystem/_index.md` contains `name: VS`, the qualified name becomes `VS::Powertrain::Engine`.

### 11.4 Implicit Supertype Rules

When no `supertype:` is given on a definition element, the parser implicitly applies the following base library supertypes. These defaults match the SysML standard library implicit specialization rules (§7.6.8 of the SysML spec). When `supertype:` is explicitly provided, it replaces (not supplements) the implicit default.

| Element `type:` | Implicit supertype |
|---|---|
| `PartDef` | `Parts::Part` |
| `ItemDef` | `Items::Item` |
| `PortDef` | `Ports::Port` |
| `ActionDef` | `Actions::Action` |
| `StateDef` | `States::StateAction` |
| `CalculationDef` | `Calculations::Calculation` |
| `ConstraintDef` | `Constraints::Constraint` |
| `RequirementDef` | `Requirements::Requirement` |
| `AttributeDef` | `Base::DataValue` |
| `EnumerationDef` | `Base::DataValue` |
| `FlowDef` | `Transfers::Transfer` |
| `AllocationDef` | `Allocations::Allocation` |
| `ConnectionDef` | `Connections::Connection` |
| `InterfaceDef` | `Connections::Interface` |
| `OccurrenceDef` | `Occurrences::Occurrence` |
| `EventOccurrenceDef` | `Occurrences::EventOccurrence` |
| `ViewDef` | `Views::View` |
| `ViewpointDef` | `Views::Viewpoint` |
| `RenderingDef` | `Views::Rendering` |
| `MetadataDef` | `Metadata::SemanticMetadata` |
| `UseCaseDef` | `UseCases::UseCase` |
| `AnalysisCaseDef` | `AnalysisCases::AnalysisCase` |
| `VerificationCaseDef` | `VerificationCases::VerificationCase` |

### 11.5 Cross-Reference Resolution Order

For a reference string `R` encountered within element `E` in package `P`:

1. If `R` begins with a valid top-level package name or the model root name, resolve as an **absolute qualified name**: split on `::`, resolve each segment as a namespace member from the global root.
2. If `R` begins with `./`, strip `./` and resolve as a sibling of `E` within package `P`.
3. Otherwise, attempt **relative resolution**:
   a. Look up `R` as a member of package `P`.
   b. If not found, look in each enclosing package outward.
   c. Then look in each package imported (directly or via `imports:`) by `P` and its ancestors.
4. If resolution fails at all levels, emit a **reference error** with the source location. Do not panic; continue parsing remaining elements.

### 11.6 Circular Reference Handling

1. The parser builds a **dependency graph** of cross-references after the first pass.
2. Cycles in the subclassification (`supertype` → `E016`), typing (`typedBy` → `E107`), subsetting (`subsets` → `E018`), or requirement-derivation (`derivedFrom` → `E017`) graphs are **semantic errors** (SysML forbids them). Report each cycle once. A **self-reference** (a length-1 cycle, e.g. an element whose `typedBy:` or `supertype:` names itself) is a cycle and is reported the same way — an element cannot be its own type or supertype.
3. Cycles in `satisfies`, `verifies`, `metadata`, or `allocations` are permitted (they are non-hierarchical) and do not produce errors.
4. Cycles in package `imports` are permitted (packages may mutually import) but the parser must detect them and process each import exactly once to avoid infinite loops.

### 11.7 Validation Requirements

A conformant parser MUST report errors for:

- `type:` values not in the defined inventory (Section 2).
- `multiplicity:` strings that do not conform to the syntax in Section 6.
- `direction:` values other than `in`, `out`, `inout`, or `return` (where `return` is valid only for parameters).
- `visibility:` values other than `public`, `protected`, or `private`.
- Two elements in the same directory with the same effective `name:`.
- `supertype:` used on a usage (should be `typedBy:` and/or `subsets:`).
- `typedBy:` used on a definition (should be `supertype:`).
- `isVariant: true` on an element not owned by a variation element (an element with `isVariation: true`).
- `EnumerationDef` with a `supertype:` that resolves to another `EnumerationDef`.
- `values:` absent on `EnumerationDef`.

A conformant parser MUST emit warnings for:

- Unknown frontmatter fields (for forward compatibility, unknown fields are preserved but warned about).
- References to elements that do not exist in the model.
- Elements with `isAbstract: false` (or defaulted as concrete) that directly subtype an abstract definition without providing concrete instantiation elsewhere.

### 11.8 Built Graph Structure

After parsing, a conformant tool must build an in-memory model graph with the following node and edge types:

**Nodes:** one node per parsed element, carrying all frontmatter fields, documentation text, source file path, and derived qualified name.

**Edges:**
- `OWNED_BY` (containment): from element to its owning package or definition/usage
- `SUBCLASSIFIES` (supertype): from definition to its supertypes
- `TYPED_BY` (typing): from usage to its definition(s)
- `SUBSETS` (subsetting): from usage to subsetted usage(s)
- `REDEFINES` (redefinition): from usage to redefined usage(s)
- `CONJUGATES` (conjugation): from conjugated PortDef to original PortDef
- `CONNECTS` (connection): from connection to source/target port features
- `SATISFIES`: from element to requirement(s) it satisfies
- `VERIFIES`: from verification case to requirement(s) it verifies
- `ALLOCATES`: from allocation to source and target elements
- `IMPORTS`: from package to imported namespace(s)
- `ALIASES`: from alias membership to aliased element
- `METADATA_OF`: from metadata application to annotated element

### 11.9 File Watch Mode

A conformant tool supporting live updates MUST:

1. Watch all `.md` files under the model root for create, modify, and delete events.
2. On any change, re-parse the affected file(s) only.
3. Re-run cross-reference resolution for any element whose referenced targets may have changed.
4. Emit a structured change event over the model update channel (e.g., WebSocket) describing the changed elements and edge updates.

### 11.10 Native-Element ID-Based Cross-Reference Resolution

Native `Requirement` (§8.11.6) and `TestCase` (§8.12.5) elements are referenced by their stable opaque `id:` field in addition to their qualified name. The reference resolution algorithm for `verifies:` and `derivedFrom:` fields is extended as follows:

**Step 0 (ID match — runs before steps 1–4 in §11.5):**

For a reference string `R` in a `verifies:` or `derivedFrom:` list:

1. If `R` matches `^REQ(-[A-Z0-9]{2,12})+-[0-9]{3}$`, search all loaded elements for a native `Requirement` whose `id:` equals `R`. If found, bind the reference. If not found, emit model error `E102` (for `verifies:`) or `E103` (for `derivedFrom:`).
2. If `R` matches `^TC(-[A-Z0-9]{2,12})+-[0-9]{3}$`, search all loaded elements for a native `TestCase` whose `id:` equals `R`. If found, bind the reference. If not found, emit `E102`.
3. If `R` matches neither ID pattern, fall through to the standard qualified-name resolution (§11.5 steps 1–4).

**Additional cross-reference validation:**

- If `verifies:` resolves to an element that is not a native `Requirement`, emit `E104`.
- If `derivedFrom:` resolves to an element that is not a native `Requirement`, emit `E105`.
- A `TestCase` with an empty `verifies:` list emits `E013`.

### 11.11 Computed Reverse Indices and Coverage

After all files are loaded and cross-references resolved, a conformant tool MUST build the following in-memory indices. These are never written to disk.

**`verifiedBy: Map<RequirementId, List<TestCaseId>>`**
For each native `Requirement`, the list of native `TestCase` ids whose `verifies:` includes this requirement's id.

**`derivedChildren: Map<RequirementId, List<RequirementId>>`**
For each native `Requirement`, the list of native `Requirement` ids whose `derivedFrom:` includes this requirement's id.

**Coverage check:**

A native `Requirement` is **covered** when `verifiedBy` is non-empty and at least one entry has `status: active`. The following warnings apply:

- `W002`: A `Requirement` with `status: approved` or `status: implemented` has no `active` TestCase in `verifiedBy`.
- `W003`: A `Requirement` with `status: verified` has an empty `verifiedBy` or all entries have `status: retired`.

### 11.12 Validation Rule Reference

This section defines the normative set of parse-time errors, model-time errors, and warnings that a conformant tool MUST emit.

#### Parse-time errors (emitted while reading a single file)

| Code | Condition |
|---|---|
| `E001` | File does not begin with `---` (missing frontmatter delimiter) |
| `E002` | YAML frontmatter is not valid YAML 1.2 |
| `E003` | Frontmatter contains an unrecognised key (strict mode only; in lenient mode, emit `W007` and preserve) |
| `E004` | A required field is absent |
| `E005` | `type:` value is not in the element type inventory (§2) |
| `E006` | `id:` is present but does not match the required pattern for the element type |
| `E007` | `status:` value is not in the allowed enum for the element type |
| `E008` | `testLevel:` value is not in `L1`–`L5` |
| `E009` | `silLevel:` value is not an integer in 1–4 |
| `E010` | `asilLevel:` value is not in `A`–`D` |
| `E011` | Native `TestCase` body has no ` ```gherkin ` fenced block |
| `E012` | Native `Requirement` body has no normative text (text before the first `##` heading is empty or whitespace only) |
| `E013` | `verifies:` list is present but empty |
| `E014` | A `Scenario Outline:` block has no `Examples:` table |
| `E015` | The first Gherkin block in a `TestCase` has no `Feature:` line |
| `E300` | `ADR.id` does not match the `ADR-*` pattern |
| `E301` | `ADR` is missing a required field (`id`, `title`, or `status`) |
| `E302` | `reqDomain:` value is not `system`, `hardware`, or `software` |
| `E303` | `domain:` value is not `system`, `hardware`, or `software` |
| `E304` | `ADR.status` value is not in `proposed | accepted | deprecated | superseded` |

#### Model-time errors (emitted during cross-reference resolution and graph building)

| Code | Condition |
|---|---|
| `E101` | Two elements have the same `id:` value |
| `E102` | A reference in `verifies:` cannot be resolved (no element with matching id or qualified name) |
| `E103` | A reference in `derivedFrom:` cannot be resolved |
| `E104` | A `verifies:` reference resolves to an element that is not a native `Requirement` |
| `E105` | A `derivedFrom:` reference resolves to an element that is not a native `Requirement` |
| `E106` | A `testFunctions[].scenario` string does not match any `Scenario:` or `Scenario Outline:` title in this file's Gherkin blocks |
| `E310` | Native `Requirement` has `derivedFrom:` entries but no `breakdownAdr:` |
| `E311` | `breakdownAdr:` cannot be resolved, or resolves to an element that is not an `ADR` |
| `E312` | A parent `Requirement` (one with `derivedChildren`) appears in a `satisfies:` list |
| `E313` | A `satisfies:` link connects an architecture element and a requirement whose `domain` / `reqDomain` values are incompatible (e.g., a `software` element satisfying a `hardware` requirement) |
| `E314` | A `Part` or `PartDef` with `isDeploymentPackage: true` has no `Allocation` to a `hardware` element |
| `E315` | An element with `domain: software` has a `supertype:` or `typedBy:` reference that resolves to an element with `domain: hardware`, or vice versa — cross-domain direct reference; use `Allocation` instead |

#### Warnings

| Code | Condition |
|---|---|
| `W001` | Native `Requirement` normative text contains no `shall` |
| `W002` | Native `Requirement` with `status: approved` or `status: implemented` has no `active` TestCase in `verifiedBy` |
| `W003` | Native `Requirement` with `status: verified` but `verifiedBy` is empty or all entries have `status: retired` |
| `W004` | A **local** `sourceFile:` path does not exist on disk. For a `TestCase`, emitted only when `status: active` (see *TestCase drift scoping*). Remote-URI sourceFiles are accepted and not checked locally (see *sourceFile location semantics*). |
| `W005` | Native `Requirement` has neither `derivedFrom:` entries nor `derivedChildren` (possible orphan not connected to any requirement hierarchy) |
| `W006` | Both `silLevel:` (IEC 61508) and `asilLevel:` (ISO 26262) are set on the same element — incompatible standards; use only one |
| `W007` | Frontmatter contains an unrecognised key (lenient mode; key is preserved in the element's extra-fields map) |
| `W009` | A `testFunctions[].function` does not resolve to a definition in its (existing) `sourceFile` — function-level traceability drift (renamed/deleted test). Emitted only for `TestCase`s with `status: active` (see *TestCase drift scoping*). See *Function matchers* below. |
| `W010` | An `active` `TestCase`'s `testFunctions[].function` last failed, was ignored/skipped, or was absent in the ingested test results. See *Test result ingestion* below. Inert unless results have been ingested. |
| `W300` | Leaf `Requirement` at `status: approved` or `status: implemented` has no satisfying architecture element (no element has `satisfies:` pointing to it) |
| `W301` | Leaf `Requirement` is satisfied by more than one architecture element — only one expected at leaf level |
| `W302` | Leaf `Requirement` at `status: implemented` or `status: verified` still has `reqDomain: system` — refine to `hardware` or `software` |
| `W303` | `breakdownAdr:` references an ADR with `status: proposed`, but the `Requirement` itself has `status: approved` or higher |
| `W304` | `isDeploymentPackage: true` combined with `domain: hardware` — deployment packages must be software |
| `W305` | Parent `Requirement` (has `derivedFrom` children) at `status: approved`, `implemented`, or `verified` has no active `TestCase` at `testLevel: L3`, `L4`, or `L5` — leaf-level tests on derived requirements are insufficient to verify emergent composed behaviour |

#### Integrity-level propagation errors (E841–E843)

Once any element in the traceability chain carries `asilLevel:`, `silLevel:`, or `plLevel:`, all downstream elements reachable via `derivedFromSafetyGoal:`, `derivedFrom:`, or `satisfies:` must also carry the same field. See §12.7.

| Code | Condition |
|---|---|
| `E841` | Element with `derivedFromSafetyGoal:` is missing `asilLevel`/`silLevel` when the referenced `SafetyGoal` carries one |
| `E842` | Element with `derivedFrom:` is missing `asilLevel`/`silLevel` when the parent element carries one |
| `E843` | Element with `satisfies:` is missing `asilLevel`/`silLevel` when the satisfied `Requirement` carries one |

#### Integrity-level propagation warnings

| Code | Condition |
|---|---|
| `W808` | Element's integrity level is strictly lower than its source (`derivedFromSafetyGoal:`, `derivedFrom:`, or `satisfies:`) but no `breakdownAdr:` is set — add an ADR documenting the ASIL/SIL decomposition rationale |

#### Safety / security analysis warnings (W800–W808)

| Code | Condition |
|---|---|
| `W800` | `HazardousEvent` is not referenced by any `SafetyGoal.hazardousEvents` |
| `W801` | `SafetyGoal` has no integrity level — set `asilLevel` (ISO 26262), `silLevel` (IEC 61508), or `plLevel` (ISO 13849-1) |
| `W802` | `CybersecurityGoal` is not implemented by any `SecurityControl.implementsGoals` |
| `W803` | `VulnerabilityReport` has `status: open` |
| `W804` | `CybersecurityGoal` has no `Requirement` with `derivedFromSecurityGoal:` pointing to it |
| `W805` | `SafetyGoal` has no `Requirement` with `derivedFromSafetyGoal:` pointing to it |
| `W806` | `SafetyGoal` has no `hazardousEvents:` — not grounded in any hazard analysis |
| `W807` | `Requirement` with `derivedFromSecurityGoal:` has no `verificationMethod:` |

#### Safety↔security co-engineering (E844, W030)

| Code | Severity | Condition |
|---|---|---|
| `E844` | Error | A `hazardRef:` value on a `DamageScenario`/`ThreatScenario` does not resolve, or resolves to an element that is not a `HazardousEvent`/`SafetyGoal` |
| `W030` | Warning | A `DamageScenario` whose `impactCategories:` includes `safety` has no `hazardRef:` (the cross-domain gap). Opt-in (safety-tagged only); gateable with `--deny W030` |

#### Cybersecurity risk determination (E845, W031, W032)

| Code | Severity | Condition |
|---|---|---|
| `E845` | Error | `ThreatScenario.riskTreatment:` is not one of `avoid`/`reduce`/`share`/`retain` |
| `W031` | Warning | A `ThreatScenario` whose computed risk is `high`/`critical` has no `riskTreatment:` and is not addressed by any `CybersecurityGoal.threatScenarios`. Gateable with `--deny W031`; promotable via `[profiles]` |
| `W032` | Warning | A `CybersecurityGoal`'s `calLevel:` is below the expected minimum CAL for the max risk of its listed threats (low→CAL1, medium→CAL2, high→CAL3, critical→CAL4). Fires only when at least one linked threat has a computable risk; gateable with `--deny W032` |

#### Quantitative HW safety metrics (E846, W033)

ISO 26262-5 §8–9 hardware architectural metrics, rolled up per `SafetyGoal` from the `FaultTreeEvent`s under the `FaultTree`(s) whose `topEvent` resolves to it. Each event may carry `diagnosticCoverage:` (DC) and `latentDiagnosticCoverage:` (DCl), both in `0.0`–`1.0`. Over the contributing events that declare a `failureRate:` (λ, /h): `Σλ = Σ λ_i`; `λ_RF = Σ λ_i·(1−DC_i)`; `SPFM = 1 − λ_RF/Σλ`; `λ_MPFL = Σ λ_i·DC_i·(1−DCl_i)` (events declaring DCl); `LFM = 1 − λ_MPFL/(Σλ−λ_RF)`; `PMHF = λ_RF + λ_MPFL`. Metrics are computed and gated **only** when at least one contributing event declares `diagnosticCoverage` (opt-in; otherwise `n/a`, never gated). Targets — ASIL SPFM ≥ {B 0.90, C 0.97, D 0.99}, LFM ≥ {B 0.60, C 0.80, D 0.90}, PMHF < {B/C 1e-7, D 1e-8}/h; SIL gates PMHF/PFH < {SIL2 1e-6, SIL3 1e-7, SIL4 1e-8}/h only. This is a **first-order FMEDA approximation** and must be independently verified. The `metrics` command (§ CLI) reports per-goal SPFM/LFM/PMHF and pass/fail.

| Code | Severity | Condition |
|---|---|---|
| `E846` | Error | `diagnosticCoverage:` or `latentDiagnosticCoverage:` is outside `0.0`–`1.0` |
| `W033` | Warning | A `SafetyGoal` with diagnosticCoverage data has a computed SPFM, LFM, or PMHF that misses its ASIL/SIL target. Gateable with `--deny W033`; promotable via `[profiles]` |

#### Freedom From Interference (W034)

ISO 26262-9 §7 dependent-failure analysis. Two elements **share a resource** when both are allocated to the same target element; allocation edges `(source → target)` are collected from `allocatedTo:` (source = the element), `allocatedFrom:` (target = the element), and `Allocation` elements' `allocatedFrom`/`allocatedTo`, resolved via the resolver and inverted into a `target → { sources }` map. Each element's integrity tag is `asilLevel`, else `silLevel` (→ `SIL<n>`), else `QM`; two sources on a target are mixed-criticality when their tags differ (including classified vs `QM`). A mixed pair is excused when the target **or** at least one of the two sources declares a non-empty `ffiRationale:` string, or carries a `breakdownAdr:` resolving to an `accepted` ADR. The check is **opt-in** — dormant unless some element declares `asilLevel` or `silLevel`. (The cross-domain "attack surface" co-analysis bonus from the originating issue is deferred.)

| Code | Severity | Condition |
|---|---|---|
| `W034` | Warning | For an allocation target with ≥2 sources, a mixed-criticality source pair has no freedom-from-interference argument (one finding per `(target, sourceA, sourceB)`, naming both sources and their tags). Gateable with `--deny W034`; promotable via `[profiles]` |

The full set of Tier 2 (E800–E846) and Tier 4 (E900–E941, W900–W905) validation codes is defined in the validation rule reference document (`docs/validation/rules.md`). §8.18 defines the element schemas.

#### sourceFile location semantics

A `sourceFile:` value is interpreted by its form, so each element can choose how its path is resolved:

| Form | Resolves to |
|---|---|
| `path` (bare relative) | model root + `path` (default) |
| `model:<path>` | model root + `path` (explicit) |
| `repo:<path>` | repository root + `path` |
| `/abs/path` | the absolute path as-is |
| `file://…` | the local path encoded in the file URI |
| `scheme://…` (any other scheme) | a **remote** location — not resolved or read locally |

The repository root is taken from `repo_root` in `<model_root>/.syscribe.toml` (resolved against the model root when relative), or auto-detected as the nearest ancestor directory containing `.git`.

> **`.syscribe.toml` also marks the model root.** When a command is given no `-m`/`--model` flag and no `SYSCRIBE_MODEL` environment variable, the tool walks upward from the current working directory and uses the nearest ancestor that contains a `.syscribe.toml` as the model root (falling back to the literal `model` directory if none is found). An empty `.syscribe.toml` is a valid root marker. This is a tooling locator only — it does not affect qualified-name resolution or the implicit root namespace.

Local forms are subject to `W004` (existence) and `W009` (function resolution). Remote URIs are treated as external: `W004` is not emitted and `W009` is skipped, since the file cannot be read during validation — **unless** a download hook is enabled (see below).

##### Remote download hook

A model may configure a hook that fetches remote sourceFiles so they can be verified like local ones:

```toml
[remote]
download = "curl -fsSL {url} -o {dest}"
# cache_dir = ".syscribe/cache"   # optional (default shown)
```

The `download` command is run via POSIX `sh`; `{url}` and `{dest}` are substituted as shell-quoted values, and `SYSCRIBE_URL` / `SYSCRIBE_DEST` are set in the environment. Fetched files are cached under `cache_dir` keyed by URL (extension preserved so the matcher still selects the language).

For safety the hook is **opt-in**: defining it has no effect until the user runs `validate --fetch-remote`, so validating an untrusted model never executes configured commands. With `--fetch-remote`, a successfully fetched remote file is subject to `W009`; a URL the hook cannot retrieve raises `W004`.

#### TestCase drift scoping

The source-drift checks `W004` (sourceFile existence) and `W009` (testFunction resolution) are scoped to a `TestCase`'s lifecycle, because drift is only meaningful once a test is claimed to exist. The `TestCase` `status` vocabulary and its drift treatment:

| `status` | Meaning | Drift treatment |
|---|---|---|
| `draft` | specified skeleton, not implemented | **planned** — informational `I010`, no `W004`/`W009` |
| `review` | spec under review | **planned** — `I010` |
| `approved` | spec approved, not yet implemented | **planned** — `I010` |
| `active` | implemented and live | **live** — `W004`/`W009` apply |
| `retired` | decommissioned | suppressed — no `W004`/`W009`/`I010` |

A non-`TestCase` element that carries a `sourceFile:` is always checked (`W004`), independent of any status. This lets a model honestly carry planned verification (e.g. future HIL/WCET cases) without fake stubs or warning noise, while `validate --deny W004,W009` gates the live verification set cleanly.

#### Informational codes

Informational findings surface a fact without failing validation. They never cause a non-zero exit on their own, but can be selected explicitly with `--deny <code>`.

| Code | Condition |
|---|---|
| `I010` | A planned `TestCase` (`status` `draft`/`review`/`approved`) has a `sourceFile`/`testFunctions[].function` that is not present yet (the planned-verification counterpart of `W004`/`W009`). |

#### Function matchers (`W009`)

`W004` confirms a local `TestCase`'s `sourceFile:` exists; `W009` additionally confirms each `testFunctions[].function` resolves to a real test/function definition in that file. The function name is the last segment of the `function` string after splitting on `::`, `.`, `#`, or `/`.

Built-in, language-aware matchers (keyed by file extension):

| Language | Extensions | Recognises |
|---|---|---|
| Rust | `.rs` | `fn name` (incl. `#[test]`, `#[kani::proof]`) |
| Java | `.java` | method declarations |
| C / C++ | `.c .h .cpp .cc .cxx .hpp .hh .hxx .ino` | function definitions; GoogleTest `TEST` / `TEST_F` / `TEST_P` |
| Kotlin | `.kt .kts` | `fun name`; backtick test names |
| Shell | `.sh .bash .bats .zsh .ksh` | POSIX `name()`; `function name`; bats `@test "..."` |

Any other file type (e.g. `.robot`, `.feature`, `.txt`, generated test manifests) uses a **generic whole-token fallback**: the test name must appear as a complete token in the file. This lets any file that represents a test participate in traceability while still catching deletions.

Projects can override or add per-extension patterns via a `[matchers]` table in `<model_root>/.syscribe.toml`, where each key is an extension (no dot) and each value is a list of regexes whose **capture group 1** is the defined test name:

```toml
[matchers]
py = ['def\s+(test_[A-Za-z0-9_]*)\s*\(']
feature = ['Scenario:\s*(.+)']
```

An override **replaces** the built-in patterns for that extension. `W009` severity is overridable via the CI gating flags (`--deny W009`).

#### Test result ingestion (`W010`)

`syscribe ingest-results --format <cargo-json|junit> <file>` parses an external test report (libtest JSON or JUnit XML), reduces it to a per-test verdict keyed by the test's leaf name, and writes a sidecar at `<model_root>/.syscribe/results.json`. When that sidecar is present (or results are supplied ad-hoc with `validate --results <file>`), the validator emits `W010` for every `active` `TestCase` whose `testFunctions[].function` last **failed**, was **ignored/skipped**, or was **missing** from the run — so "verified" can mean "covered by a test that actually passed". Passing functions are silent, and `W010` is inert when no results have been ingested. Gate on it in CI with `--deny W010`.

#### Exit-code contract (CI gating)

The `validate` subcommand exposes a stable exit-code contract so it can be used directly as a CI gate:

| Exit code | Meaning |
|---|---|
| `0` | No `Error`-severity findings and no gate failure |
| `1` | One or more `Error`-severity findings present |
| `2` | One or more `Warning`-severity findings tripped a configured gate |

Gating is opt-in via flags on `validate`:

- `--deny <CODES>` — comma-separated warning codes promoted to gate failures (e.g. `--deny W004,W009`).
- `--max-warnings <N>` — fail when the total warning count exceeds `N`.
- `--warnings-as-errors` — promote every warning to a gate failure.
- `--profile <name>` — apply a named, optionally integrity-level-scoped gating policy declared in `<model_root>/.syscribe.toml` (see below).

`Error` findings always dominate (exit `1`) regardless of gating flags. With no gating flag, warning-only models exit `0`. This enables a phased rollout: warn during burndown, then `--deny` to enforce.

##### Named severity profiles (`[profiles.*]`)

Reusable gating policies are declared as `[profiles.<name>]` tables in `<model_root>/.syscribe.toml` and selected with `validate --profile <name>`. A profile promotes the warning codes in its `promote` list to gate failures, optionally **scoped** to the integrity level / status / tag of the element each finding concerns:

```toml
[profiles.safety]
promote = ["W002", "W015", "W300"]   # warning codes promoted to gate failures
# OPTIONAL scope — promotion applies only to findings on an element matching ALL fields:
sil    = "4"          # element's silLevel stringifies to "4" OR asilLevel == "4"
status = "approved"   # element's status:
tag    = "safety"     # element's tags: contains this
```

A finding trips the gate when its `code` is listed in `promote` **and** either the profile declares no scope fields, or the element whose `file_path` equals the finding's file matches **all** of the provided scope fields (a finding mapping to no element is not promoted when a scope is set). Multiple profiles may coexist, and the `[profiles.*]` tables parse alongside the existing `[matchers]` / `[remote]` tables and the `repo_root` key. An undefined profile name (or a missing `.syscribe.toml`) is a usage error and exits `1`. `--profile` composes additively with `--deny` / `--max-warnings` / `--warnings-as-errors`.

---

## Appendix A: Complete Frontmatter Field Reference

The following table is a consolidated index of all frontmatter fields defined in this specification.

| Field | Applies to | Type | Default | Section |
|---|---|---|---|---|
| `type` | All | string | — (required) | 3.1 |
| `name` | All | string | filename stem | 3.1 |
| `shortName` | All | string | absent | 3.1 |
| `qualifiedName` | All | string | derived | 3.1 |
| `visibility` | All | string | `public` | 3.1 |
| `extRef` | All | string or list | absent | 3.1 |
| `isAbstract` | All | bool | `false` | 3.2 |
| `isVariation` | Def/Usage | bool | `false` | 3.2 |
| `isVariant` | Usage | bool | `false` | 3.2 |
| `isIndividual` | Occurrence | bool | `false` | 3.2 |
| `isReadonly` | Usage | bool | `false` | 3.2 |
| `isDerived` | Usage | bool | `false` | 3.2 |
| `isEnd` | Usage | bool | `false` | 3.2 |
| `isPortion` | Occurrence usage | bool | `false` | 3.2 |
| `isReference` | Usage | bool | `false` | 3.2 |
| `isComposite` | Usage | bool | `true` | 3.2 |
| `isConstant` | Usage | bool | `false` | 3.2 |
| `isOrdered` | Usage | bool | `false` | 3.2 |
| `isNonunique` | Usage | bool | `false` | 3.2 |
| `supertype` | Def | string or list | absent | 3.3 |
| `typedBy` | Usage | string or list | absent | 3.3 |
| `subsets` | Usage | list | absent | 3.3 |
| `redefines` | Usage | list | absent | 3.3 |
| `conjugates` | PortDef | string | absent | 3.3 |
| `multiplicity` | Usage | string | `"1"` or `"0..*"` | 3.4, 6 |
| `direction` | Port, Parameter | string | absent | 3.5 |
| `features` | Def/Usage | list | absent | 3.6 |
| `imports` | Package | list | absent | 3.7 |
| `aliases` | All | list | absent | 3.7 |
| `filterCondition` | Package | string | absent | 3.7 |
| `metadata` | All | list | absent | 3.8 |
| `dependsOn` | All | list | absent | 3.9 |
| `requires` | All | list | absent | 3.11 |
| `assume` | All | list | absent | 3.11 |
| `rep` | All | string | absent | 3.12 |
| `connections` | PartDef/Part | list | absent | 8.4.1 |
| `flowConnections` | PartDef/Part | list | absent | 8.6.2 |
| `successionConnections` | ActionDef/Action | list | absent | 8.4.4 |
| `bindingConnections` | Def/Usage | list | absent | 8.4.3 |
| `performs` | PartDef/Part | list | absent | 8.2.1 |
| `exhibitsStates` | PartDef/Part | list | absent | 8.2.1 |
| `parameters` | ActionDef/CalcDef/etc. | list | absent | 8.7.2 |
| `returnType` | CalculationDef/VerificationCaseDef | string | absent | 8.9.1, 8.12.3 |
| `body` | CalculationDef/ActionDef | string | absent | 8.7.1, 8.9.1 |
| `bodyLanguage` | CalculationDef/ActionDef | string | `"ocl"` | 8.7.1, 8.9.1 |
| `subActions` | ActionDef/Action/CaseDef | list | absent | 8.7.3 |
| `controlNodes` | ActionDef/Action | list | absent | 8.7.4 |
| `entryAction` | StateDef/State | string or map | absent | 8.8.1 |
| `doAction` | StateDef/State | string or map | absent | 8.8.1 |
| `exitAction` | StateDef/State | string or map | absent | 8.8.1 |
| `subStates` | StateDef/State | list | absent | 8.8.2 |
| `transitions` | StateDef/State | list | absent | 8.8.3 |
| `isParallel` | StateDef/State | bool | `false` | 8.8.1 |
| `isAsserted` | Constraint | bool | `false` | 8.10.2 |
| `isNegated` | Constraint | bool | `false` | 8.10.2 |
| `expression` | ConstraintDef | string | absent | 8.10.1 |
| `expressionLanguage` | ConstraintDef | string | `"ocl"` | 8.10.1 |
| `subject` | Req/Case | string | absent | 8.11.1, 8.12.1 |
| `actors` | Req/UseCase | list | absent | 8.11.1, 8.12.4 |
| `stakeholders` | Req/Viewpoint | list | absent | 8.11.1, 8.14.1 |
| `concerns` | Req/Viewpoint | list | absent | 8.11.1, 8.14.1 |
| `framedConcerns` | RequirementDef | list | absent | 8.11.1 |
| `derivedFrom` | RequirementDef/Requirement | list | absent | 8.11.1 |
| `satisfies` | Part/PartDef/etc. | list | absent | 8.11.4 |
| `implementedBy` | Part/PartDef | string or list | absent | 8.11.4 / 12.8 |
| `verifiedBy` | Requirement | list | absent | 8.11.4 |
| `verifies` | VerificationCase | list | absent | 8.12.3 |
| `verdictExpression` | VerificationCase | string | absent | 8.12.3 |
| `verdictType` | VerificationCaseDef | string | `VerificationCases::VerdictKind` | 8.12.3 |
| `objectives` | CaseDef | list | absent | 8.12.1 |
| `result` | CaseDef | string | absent | 8.12.1 |
| `includes` | UseCaseDef | list | absent | 8.12.4 |
| `extends` | UseCaseDef | list | absent | 8.12.4 |
| `extensionPoints` | UseCaseDef | list | absent | 8.12.4 |
| `allocateFrom` | Allocation | string | — | 8.13.2 |
| `allocateTo` | Allocation | string | — | 8.13.2 |
| `allocations` | AllocationDef/Package/PartDef | list | absent | 8.13.1, 8.13.2 |
| `constraints` | InterfaceDef | list | absent | 8.3.3 |
| `expose` | ViewDef | list | absent | 8.14.2 |
| `rendering` | ViewDef | string | absent | 8.14.2 |
| `satisfiedBy` | ViewpointDef | list | absent | 8.14.1 |
| `methods` | ViewpointDef | list | absent | 8.14.1 |
| `values` | EnumerationDef | list | — (required) | 8.5.2 |
| `annotates` | MetadataDef | list | absent (unrestricted) | 8.15.1 |
| `isSemantic` | MetadataDef | bool | `false` | 8.15.1 |
| `ends` | ConnDef/IntfDef | list | absent | 8.3.3, 8.4.2 |
| `timeSlices` | OccurrenceDef | list | absent | 8.2.4 |
| `snapshots` | OccurrenceDef | list | absent | 8.2.4 |
| `variantOf` | Part/Usage | string | absent | 9.4 |
| `isConjugated` | Port | bool | `false` | 8.3.2 |
| `itemType` | FlowDef | string | absent | 8.6.1 |
| `id` | native Requirement/TestCase | string | — (required) | 8.11.6, 8.12.5 |
| `title` | native Requirement/TestCase | string | — (required) | 8.11.6, 8.12.5 |
| `status` | native Requirement/TestCase | string | — (required) | 8.11.6, 8.12.5 |
| `testLevel` | native TestCase | string | — (required) | 8.12.5 |
| `silLevel` | native Requirement | integer | absent | 8.11.6 |
| `asilLevel` | native Requirement | string | absent | 8.11.6 |
| `plLevel` | native Requirement / SafetyGoal | string | absent | 8.11.6, 8.18.1 |
| `derivedFromSafetyGoal` | native Requirement | string | absent | 8.11.6, 8.18.1 |
| `derivedFromSecurityGoal` | native Requirement | string | absent | 8.11.6, 8.18.2 |
| `verificationMethod` | native Requirement | string | absent | 8.11.6 |
| `wcet` | native Requirement | string | absent | 8.11.6 |
| `allocatedFrom` | Any element | string or list | absent | 8.18.2 |
| `allocatedTo` | Any element | string or list | absent | 8.18.2 |
| `ffiRationale` | Any element | string | absent | 11.12 (W034) — freedom-from-interference / partitioning rationale; excuses a mixed-criticality shared-allocation pair |
| `hazardRef` | DamageScenario / ThreatScenario | string or list | absent | 8.18.2 |
| `riskTreatment` | ThreatScenario | enum (`avoid`/`reduce`/`share`/`retain`) | absent | 8.18.2 |
| `residualRisk` | ThreatScenario | string | absent | 8.18.2 |
| `sourceFile` | native TestCase | string | absent | 8.12.5 |
| `testFunctions` | native TestCase | list | absent | 8.12.5 |
| `tags` | native Requirement/TestCase | list of strings | absent | 8.11.6, 8.12.5 |

---

## Appendix B: Mapping of SysML Textual Keywords to `type:` Values

| SysML textual keyword | Markdown-SysML `type:` |
|---|---|
| `part def` | `PartDef` |
| `item def` | `ItemDef` |
| `port def` | `PortDef` |
| `connection def` | `ConnectionDef` |
| `interface def` | `InterfaceDef` |
| `action def` | `ActionDef` |
| `calc def` | `CalculationDef` |
| `constraint def` | `ConstraintDef` |
| `requirement def` | `RequirementDef` |
| `concern def` | `ConcernDef` |
| `case def` | `CaseDef` |
| `analysis def` | `AnalysisCaseDef` |
| `verification def` | `VerificationCaseDef` |
| `use case def` | `UseCaseDef` |
| `occurrence def` | `OccurrenceDef` |
| `individual def` | `IndividualDef` |
| `flow def` | `FlowDef` |
| `succession def` | `SuccessionDef` |
| `state def` | `StateDef` |
| `attribute def` | `AttributeDef` |
| `enum def` | `EnumerationDef` |
| `allocation def` | `AllocationDef` |
| `metadata def` | `MetadataDef` |
| `view def` | `ViewDef` |
| `viewpoint def` | `ViewpointDef` |
| `rendering def` | `RenderingDef` |
| `part` | `Part` |
| `item` | `Item` |
| `port` | `Port` |
| `connection` | `Connection` |
| `interface` | `Interface` |
| `action` | `Action` |
| `calc` | `Calculation` |
| `constraint` | `Constraint` |
| `requirement` | `Requirement` |
| `concern` | `Concern` |
| `case` | `Case` |
| `analysis` | `AnalysisCase` |
| `verification` | `VerificationCase` |
| `use case` | `UseCase` |
| `occurrence` | `Occurrence` |
| `individual` | `Individual` |
| `flow` | `Flow` |
| `succession` | `Succession` |
| `state` | `State` |
| `attribute` | `Attribute` |
| `enum` | `Enumeration` |
| `allocation` | `Allocation` |
| `metadata` | `Metadata` |
| `view` | `View` |
| `rendering` | `Rendering` |
| `package` | `Package` |
| `library package` | `LibraryPackage` |
| *(native — no SysML keyword)* | `TestCase` |

---

## 12 Traceability Rules and Domain Conventions

This section defines mandatory traceability rules that govern how requirements, architecture elements, and design decisions are linked. These rules are normative — a conformant tool MUST enforce them via the validation codes defined in §11.12.

---

### 12.1 OSLC Link Direction Convention

All traceability links in Markdown-SysML follow OSLC (Open Services for Lifecycle Collaboration) upstream-link semantics: **the artifact that is derived from, implements, or verifies another always holds the link field pointing to the upstream artifact.**

| Link field | Direction | Meaning |
|---|---|---|
| `derivedFrom:` | child → parent | This requirement was broken down from the parent |
| `verifies:` | test → requirement | This test case verifies the requirement |
| `satisfies:` | architecture element → requirement | This element implements the requirement |
| `allocatedTo:` / `allocatedFrom:` | downstream → upstream | The architecture element (downstream, realising party) holds `allocatedFrom:` referencing the upstream logical or security artifact; `allocatedTo:` is used on `Allocation` elements |
| `breakdownAdr:` | requirement → ADR | This requirement's breakdown is documented in the ADR |

No reverse links are stored in model files. Reverse indices (`verifiedBy`, `derivedChildren`, `satisfiedBy`) are computed by the parser at load time and never written to disk.

---

### 12.2 Requirement Breakdown and ADRs

When a requirement is broken down into multiple child requirements, the breakdown must be documented in an Architecture Decision Record (ADR, §8.17).

**Rule R-002:** Every native `Requirement` that has one or more `derivedFrom:` entries **must** set `breakdownAdr:` to the `id` or qualified name of an `accepted` ADR that documents the rationale for the breakdown. (Error `E310` if absent; warning `W303` if the referenced ADR is still `proposed`.)

**Breakdown procedure:**
1. Author the ADR (`type: ADR`, `status: proposed`) documenting context, decision, and consequences.
2. Get the ADR reviewed and set `status: accepted`.
3. Create the child requirements with `derivedFrom: [PARENT-ID]` and `breakdownAdr: ADR-ID`.
4. The parent requirement's `status` remains unchanged; it must not be moved to `verified` unless all child requirements are verified.

**Example:**
```yaml
---
type: Requirement
id: REQ-SCHED-BITMAP-001
title: "Bitmap-based O(1) priority selection"
status: approved
derivedFrom:
  - REQ-UAV-SCHED-001
breakdownAdr: ADR-SW-SCHED-001
...
```

---

### 12.3 Leaf-Level Assignment Rule

A requirement is a **leaf requirement** when no other requirement has `derivedFrom:` pointing to it (i.e., the computed `derivedChildren` index is empty for this requirement's id).

**Rule R-003:** A leaf requirement at `status: approved` or higher must be assigned to exactly one architecture element — meaning exactly one `Part` or `PartDef` element must have `satisfies:` referencing this requirement.

- Zero satisfying elements → warning `W300`
- More than one satisfying element → warning `W301`

The assignment can evolve iteratively:
1. Initially, assign to a higher-level block (`satisfies: UAV::FlightController`).
2. When the internal design of that block is defined, update the `satisfies:` to the specific sub-element (`satisfies: UAV::FlightController::SchedulingModule`).
3. A requirement's `satisfies:` should point to the **deepest** known architecture element that is responsible for fulfilling it.

---

### 12.4 Parent Requirements Cannot Be Assigned

**Rule R-004:** A requirement that has been broken down (has a non-empty `derivedChildren` index) **must not** appear in any element's `satisfies:` list.

Only leaf requirements may be assigned to architecture elements. Assigning a parent requirement is an error (`E312`) because it conflates the requirement with its children and makes traceability ambiguous.

The parent requirement may still carry `status: approved`, `status: implemented` (when all children are implemented), or `status: verified` (when all children are verified) to track aggregate status.

---

### 12.5 Requirement Domain Classification

Native `Requirement` elements carry a `reqDomain:` field (§8.11.6) indicating which engineering domain they govern.

| `reqDomain` value | Meaning |
|---|---|
| `system` | Domain-agnostic. Used for top-level stakeholder requirements before domain allocation. |
| `hardware` | The requirement governs a hardware element (physical component, board, sensor, actuator). |
| `software` | The requirement governs a software element (module, binary, firmware, algorithm). |

**Rule R-005a:** A leaf requirement must be satisfied only by an architecture element whose `domain:` matches the requirement's `reqDomain:`, unless either is `system`. (Error `E313`.)

**Rule R-005b:** A leaf requirement at `status: implemented` or `status: verified` that still has `reqDomain: system` should be refined to `hardware` or `software`. (Warning `W302`.)

During top-down decomposition, `reqDomain:` naturally migrates from `system` to `hardware` or `software` as the design progresses. This migration is an explicit model edit, not an automatic inference.

---

### 12.6 Hardware/Software Architecture Independence

The hardware and software architectures are **independent hierarchies**. They interact only through `Allocation` elements.

**Rule R-006a (no direct cross-domain references):** An element with `domain: software` must not have `supertype:` or `typedBy:` referencing an element with `domain: hardware`, and vice versa. Cross-domain direct references are errors (`E315`). The correct pattern is an explicit `Allocation`.

**Rule R-006b (deployment allocation):** A `Part` or `PartDef` with `isDeploymentPackage: true` must have at least one `Allocation` element whose `allocateFrom:` is this element and whose `allocateTo:` references an element with `domain: hardware`. (Error `E314`.)

**Correct cross-domain pattern:**
```yaml
# SW element
---
type: PartDef
name: SchedulerModule
domain: software
isDeploymentPackage: true
---

# HW element
---
type: PartDef
name: FlightComputer
domain: hardware
---

# Allocation — the only permitted cross-domain link
---
type: Allocation
name: schedulerToFC
allocateFrom: Software::SchedulerModule
allocateTo: Hardware::FlightComputer
---
```

**Rule R-006c (requirement domain consistency):** Hardware requirements must be satisfied by hardware architecture elements; software requirements must be satisfied by software architecture elements. This is enforced by Rule R-005a (§12.5).

#### ADR for deployment decisions

When a software component is first allocated to a hardware element, or when the hardware allocation changes, an ADR should document the rationale. Reference it from the deployment `Allocation` using the `metadata:` field:

```yaml
---
type: Allocation
name: schedulerToFC
allocateFrom: Software::SchedulerModule
allocateTo: Hardware::FlightComputer
metadata:
  - type: ModelingMetadata::Rationale
    text: "See ADR-HW-DEPLOY-001 for platform selection rationale"
---
```

---

### 12.7 Safety/Security Integrity Level Propagation

**Rule R-007:** Once any element in the traceability chain carries a safety or security integrity level (`asilLevel:`, `silLevel:`, or `plLevel:`), **all downstream elements** reached via `derivedFromSafetyGoal:`, `derivedFrom:`, or `satisfies:` links must also carry the same field. An element that omits the field when its upstream source has one is an error (E841, E842, or E843 depending on the link kind).

**Level constraint:** The downstream element's level may be the same as or lower than the upstream element's. A lower level indicates an ASIL/SIL decomposition (ISO 26262-9, IEC 61508-2 §7.4.9): the model asserts that the downstream component achieves the weaker target through architectural independence or redundancy arguments.

**ADR requirement for decomposition:** When a downstream element carries a lower level than its source, `breakdownAdr:` must reference an `accepted` ADR documenting the decomposition rationale (W808 if absent).

| Link | Enforced by | Missing field | Lower level without ADR |
|---|---|---|---|
| `derivedFromSafetyGoal:` → `SafetyGoal` | R-007 | E841 | W808 |
| `derivedFrom:` → parent Requirement | R-007 | E842 | W808 |
| `satisfies:` → Requirement | R-007 | E843 | W808 |

**Level comparison rules:**

- `asilLevel:` ranks A < B < C < D.
- `silLevel:` ranks 1 < 2 < 3 < 4.
- `plLevel:` (ISO 13849-1) has values `a`–`e` but is not numerically compared with ASIL or SIL.
- Mixing `asilLevel:` on one element with `silLevel:` on another is architecturally unusual and not validated cross-element; W006 flags the case where both appear on the *same* element.

**Example — ASIL D requirement decomposed to ASIL B:**

```yaml
# Parent SafetyGoal
---
type: SafetyGoal
id: SG-BRAKE-001
title: "Prevent unintended brake release"
asilLevel: D
hazardousEvents: [HE-BRAKE-001]
---

# Derived requirement — decomposed to ASIL B via redundancy argument
---
type: Requirement
id: REQ-BRAKE-HYD-001
title: "Maintain hydraulic pressure within 50 ms"
asilLevel: B
derivedFromSafetyGoal: SG-BRAKE-001
breakdownAdr: ADR-BRAKE-DECOMP-001    # documents the decomposition rationale
---
```

### 12.8 Implementation Trace

**Rule R-008:** A `Part` or `PartDef` may carry an optional `implementedBy:` field linking the architecture element to the source artifact(s) that realise it. This closes the downstream leg of the V-model:

```
Requirement ─satisfies→ Architecture ─implementedBy→ Code ─verifies→ Test
```

`implementedBy:` accepts a single string or a list of strings. Each value is a path into the codebase, resolved with the **same rules as a TestCase's `sourceFile`** (§8.12.5, §11.12): model-root-relative (the default for bare paths), `model:`-prefixed (model-root-relative), `repo:`-prefixed (repository-root-relative), absolute, `file://`, and remote `scheme://` URIs. Local paths are checked on disk; remote URIs are accepted as external pointers and not verified locally.

**Validation (W023):** When a non-`draft` `Part`/`PartDef` declares `implementedBy:` and a **local** path does not exist on disk, the tool emits **W023** (one finding per missing path) — the architecture-to-code analog of `W004` for `sourceFile`. The rule is:

- **Opt-in** — an element with no `implementedBy:` is never flagged.
- **Draft-suppressed** — elements with `status: draft` are skipped (the implementation may not exist yet).
- **Remote-tolerant** — remote (`scheme://`) targets are not verified locally.
- **Gateable** — `validate --deny W023` exits non-zero when any W023 is present.

The link is discoverable through tooling: `syscribe links <element>` lists `implementedBy` paths as outbound relationships, and `syscribe refs <path-or-dir>` reverse-maps a source path (or directory prefix) back to the declaring architecture element(s).

**Example:**

```yaml
---
type: PartDef
name: Scheduler
domain: software
satisfies: [REQ-SCHED-001]
implementedBy:
  - src/scheduler/mod.rs
  - repo:src/scheduler/bitmap.rs
---
```
