# Syscribe Element Types

## Definition Types (§2.1)

| `type:` | SysML keyword | Description |
|---|---|---|
| `PartDef` | `part def` | Physical or logical system component classifier |
| `ItemDef` | `item def` | Classifies things that flow through a system |
| `PortDef` | `port def` | Interaction point classifier |
| `ConnectionDef` | `connection def` | Structural link classifier |
| `InterfaceDef` | `interface def` | Compatible connection-end classifier |
| `ActionDef` | `action def` | Behavioral step classifier |
| `CalculationDef` | `calc def` | Parameterized expression with return value |
| `ConstraintDef` | `constraint def` | Boolean-valued condition classifier |
| `RequirementDef` | `requirement def` | Textual/formal requirement classifier (SysML usage) |
| `ConcernDef` | `concern def` | Stakeholder concern classifier |
| `CaseDef` | `case def` | Base for analysis/verification/use-case definitions |
| `AnalysisCaseDef` | `analysis def` | Analysis procedure classifier |
| `VerificationCaseDef` | `verification def` | Verification procedure classifier |
| `UseCaseDef` | `use case def` | System use-case classifier |
| `OccurrenceDef` | `occurrence def` | Temporal-extent classifier |
| `EventOccurrenceDef` | `event occurrence def` | Momentary, instantaneous occurrence classifier |
| `IndividualDef` | `individual def` | Specific individual occurrence classifier |
| `FlowDef` | `flow def` | Flow connection carrying items |
| `SuccessionDef` | `succession def` | Temporal ordering classifier |
| `StateDef` | `state def` | State machine node classifier |
| `AttributeDef` | `attribute def` | Data value classifier (scalars, quantities) |
| `EnumerationDef` | `enum def` | Discrete-value classifier |
| `AllocationDef` | `allocation def` | Allocation relationship classifier |
| `MetadataDef` | `metadata def` | Annotation structure classifier |
| `ViewDef` | `view def` | Model view classifier |
| `ViewpointDef` | `viewpoint def` | Stakeholder viewpoint classifier |
| `RenderingDef` | `rendering def` | Rendering method classifier |

## Usage Types (§2.2)

| `type:` | SysML keyword | Description |
|---|---|---|
| `Part` | `part` | Usage of a PartDef |
| `Item` | `item` | Usage of an ItemDef |
| `Port` | `port` | Usage of a PortDef |
| `Connection` | `connection` | Usage of a ConnectionDef |
| `Interface` | `interface` | Usage of an InterfaceDef |
| `Action` | `action` | Usage of an ActionDef |
| `Calculation` | `calc` | Usage of a CalculationDef |
| `Constraint` | `constraint` | Usage of a ConstraintDef |
| `Requirement` | `requirement` | SysML usage of a RequirementDef |
| `Concern` | `concern` | Usage of a ConcernDef |
| `Case` | `case` | Usage of a CaseDef |
| `AnalysisCase` | `analysis` | Usage of an AnalysisCaseDef |
| `VerificationCase` | `verification` | Usage of a VerificationCaseDef |
| `UseCase` | `use case` | Usage of a UseCaseDef |
| `Occurrence` | `occurrence` | Usage of an OccurrenceDef |
| `EventOccurrence` | `event occurrence` | Momentary observation or signal (`direction: in` = observed, `out` = emitted) |
| `Individual` | `individual` | Time-slice/snapshot of an IndividualDef |
| `Flow` | `flow` | Usage of a FlowDef |
| `Succession` | `succession` | Temporal ordering between actions/occurrences |
| `BindingConnector` | `binding` | Equality binding between two features |
| `State` | `state` | Usage of a StateDef |
| `ExhibitState` | `exhibit state` | Referential usage exhibiting a StateDef |
| `Attribute` | `attribute` | Usage of an AttributeDef |
| `Enumeration` | `enum` | Usage of an EnumerationDef |
| `Allocation` | `allocation` | Usage of an AllocationDef |
| `Metadata` | `metadata` | Application of a MetadataDef |
| `View` | `view` | Usage of a ViewDef |
| `Rendering` | `rendering` | Usage of a RenderingDef |
| `TestCase` | *(native)* | Native test case — stable `TC-*` ID, Gherkin, lifecycle |
| `FeatureDef` | *(PLE native)* | Feature model node (Product Line Engineering) |
| `Configuration` | *(PLE native)* | Complete feature selection producing one product variant |

## Record and infrastructure types

| `type:` | Category | Description |
|---|---|---|
| `ADR` | Record | Architecture Decision Record — stable `ADR-*` ID |
| `Package` | Namespace | Named container; `_index.md` in a directory |
| `LibraryPackage` | Namespace | Package marked as a model library |
| `Namespace` | Namespace | Root namespace; `_index.md` at model root |
| `Dependency` | Relationship | Directed client → supplier link; fields: `clients:`, `suppliers:` |
| `Diagram` | Diagram | SVG diagram with model-element manifest |

## Safety/security native types — see `syscribe spec safety`

`HazardousEvent` (HE-*) · `SafetyGoal` (SG-*) · `DamageScenario` (DS-*) ·
`ThreatScenario` (TS-*) · `CybersecurityGoal` (CSG-*) · `SecurityControl` (SC-*) ·
`VulnerabilityReport` (VR-*) · `TARASheet` (TARA-*) · `FaultTree` (FT-*) ·
`FaultTreeGate` (FTG-*) · `FaultTreeEvent` (FTE-*) · `FMEASheet` (FMEA-*) ·
`FMEAEntry` (row of an FMEASheet) · `AttackTree` (AT-*) ·
`AttackTreeGate` (ATG-*) · `AttackStep` (ATS-*) ·
`ConfirmationMeasure` (CM-*) — confirmation review / FS audit & assessment /
cybersecurity assessment, with independence level I1–I3 ·
`Argument` (ARG-*) — a GSN node (claim/strategy/solution) ·
`AssumptionOfUse` (AOU-*) — safety-related application condition (SRAC)

## Key per-type schemas

### PartDef / Part

```yaml
type: PartDef            # or Part
name: FlightController
supertype: Parts::Part   # (PartDef) specialization
typedBy: Avionics::FlightControllerDef  # (Part) type
domain: software         # system | hardware | software
isAbstract: false
isDeploymentPackage: false
asilLevel: C             # propagated from satisfied requirements
satisfies: [REQ-UAV-FC-001]
features:
  - name: powerIn
    type: Port
    typedBy: Interfaces::PowerPortDef
    direction: in
connections:
  - from: fc.controlOut
    to: motors.controlIn
performs: [Behavior::MissionExecution]
exhibitsStates: [Behavior::FlightStates]
```

### PortDef / Port

```yaml
type: PortDef
name: PowerPortDef
supertype: Ports::Port
conjugates: ./PowerPortDef   # conjugate port def for the other end
ends:                        # (InterfaceDef) typed ends
  - name: supplier
    typedBy: ./VoltageSupplyDef
  - name: consumer
    typedBy: ./CurrentDrawDef
---
type: Port
name: powerIn
typedBy: Interfaces::PowerPortDef
direction: in    # in | out | inout
isConjugated: false
ports:           # nested sub-ports
  - name: voltage
    direction: in
```

### Ports & Interfaces — decision guide

Mental model (SysML v2): a **`PortDef`** is a reusable *kind* of connection point carrying directed features; a **`Port`** is a usage of one on a part (in `features:`, `type: Port`, `typedBy:` the PortDef). An **`InterfaceDef` is a kind of `ConnectionDef`** — *a connection whose ends are ports* — used to package a reusable, compatible pairing; a **`ConnectionDef`** connects arbitrary features/parts.

Which construct → when:

| You want to… | Use |
|---|---|
| expose an interaction point on a part | a `Port` (in `features:`) typed by a `PortDef` |
| define a reusable compatible pairing of two ports | `InterfaceDef` (ends typed by PortDefs) |
| connect arbitrary features/parts (not necessarily ports) | `ConnectionDef` |
| wire two specific ports inside a part | a connection usage: `connections:` with `from`/`to` feature chains (optionally `typedBy:` the InterfaceDef) |
| move items between connected ports | `FlowDef` / `flowConnections:` |
| equate two features | `bindingConnections:` |

**Conjugation:** the receiving end is the **conjugate** of the sending end — every directed feature flips (`in`↔`out`; `inout` is self-conjugate). Express it with `conjugates:` on a dedicated receiver `PortDef`, or `isConjugated: true` on a `Port` usage / interface end. Connected directed features must be conjugate-compatible. (See `syscribe spec safety`-style depth in format spec §8.3.)

### Allocation

```yaml
type: Allocation
name: schedulerToFC
allocatedFrom: Software::SchedulerModule   # upstream source
allocatedTo: Hardware::FlightComputer      # downstream target
```

On architecture elements: `allocatedFrom: SC-001` (or list) references upstream controls.

### native Requirement

```yaml
type: Requirement
id: REQ-UAV-FC-001           # required; REQ(-[A-Z0-9]{2,12})+-[0-9]{3}
title: "..."                 # required
status: approved             # draft | review | approved | implemented | verified
reqDomain: software          # system | hardware | software
asilLevel: B                 # A–D  OR  silLevel: 1–4  (not both — W006)
plLevel: c                   # a–e (ISO 13849-1)
verificationMethod: test     # test | inspection | analysis | demonstration
derivedFrom: [REQ-UAV-SAFE-000]
breakdownAdr: ADR-UAV-001    # required when derivedFrom is set (E310)
derivedFromSafetyGoal: SG-BRAKE-001
derivedFromSecurityGoal: CSG-001
tags: [safety, contingency]
```
Body must contain at least one `shall` statement (W001 if absent).

### native TestCase

```yaml
type: TestCase
id: TC-UAV-FC-001            # required; TC(-[A-Z0-9]{2,12})+-[0-9]{3}
title: "..."                 # required
status: active               # draft | active | retired
testLevel: L2                # L1–L5
verifies: [REQ-UAV-FC-001]
sourceFile: tests/test_fc.py
testFunctions:
  - function: test_safe_landing
    scenario: "Safe landing on battery critical"
```
Body must contain at least one ` ```gherkin ` fenced block (E011).

### ADR

```yaml
type: ADR
id: ADR-SW-SCHED-001         # required; ADR(-[A-Z0-9]{2,12})+-[0-9]{3}
title: "..."                 # required
status: accepted             # proposed | accepted | deprecated | superseded
```
Body sections: `## Context`, `## Decision`, `## Consequences`.
