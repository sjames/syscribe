---
type: Requirement
id: REQ-TRS-REQTRACE-001
name: "The Requirements Explorer highlights the trace from an element to tests, architecture, features, safety or security elements"
status: draft
reqDomain: software
reqClass: system
tags:
  - server
---

`syscribe-server`'s Requirements Explorer shall let the user highlight how the selected element (else the root) connects to a category of other elements, and jump from a node to the page that owns its kind (GH #269, phase 4).

## Behavior

- Categories classify element types: `tests` (TestCase, TestPlan, TestEnvironment, VerificationCase, VerificationCaseDef), `architecture` (PartDef, Part, ItemDef, PortDef, InterfaceDef, ConnectionDef, Allocation, AllocationDef, ActionDef, StateDef), `features` (FeatureDef, Configuration, FeatureModel), `safety` (HazardousEvent, SafetyGoal, SafetyMechanism, ConfirmationMeasure, DependentFailureAnalysis, AssumptionOfUse, Argument, FaultTree, FaultTreeGate, FaultTreeEvent, FMEASheet, FMEAEntry) and `security` (DamageScenario, ThreatScenario, CybersecurityGoal, SecurityControl, VulnerabilityReport, Asset, AttackTree, AttackTreeGate, AttackStep, Zone, Conduit, TARASheet).
- The trace is a pure function of the drawn graph: from the start node it follows the edges in either direction and returns every node and edge lying on a shortest path to a node of the category (the start itself is never a target). When nothing in the drawn graph matches, the result is empty and the page says so, suggesting a greater depth or more relations. The page dims everything off the trace and marks the targets; clearing restores the view.
- Selecting a Feature, Configuration or FeatureModel node offers a link to `/features`; a PlanningItem offers `/planning`. Links are fixed paths — model text is never placed in an `href`.
