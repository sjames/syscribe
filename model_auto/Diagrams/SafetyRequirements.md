---
type: Diagram
name: Safety Requirement Traceability
diagramKind: Requirement
pumlMode: companion
pumlFile: ./SafetyRequirements.puml
subject: Requirements::Safety
shapes:
  s-sys:   {ref: "Requirements::Safety::REQ-ENG-SAFE-000",  kind: Requirement}
  s-safe:  {ref: "Requirements::Safety::REQ-ENG-SAFE-001",  kind: Requirement}
  s-stall: {ref: "Requirements::Safety::REQ-ENG-SAFE-003",  kind: Requirement}
  s-sm:    {ref: "System::Software::SafetyMonitor",          kind: PartDef}
  s-esm:   {ref: "System::Software::EngineStallMonitor",     kind: PartDef}
  s-tc1:   {ref: "Verification::TC-ENG-SAFE-001",            kind: TestCase}
edges:
  e-safe-sys:   {source: s-safe,  target: s-sys,   kind: derivedFrom}
  e-stall-sys:  {source: s-stall, target: s-sys,   kind: derivedFrom}
  e-tc1-sys:    {source: s-tc1,   target: s-sys,   kind: verifies}
  e-sm-safe:    {source: s-sm,    target: s-safe,  kind: satisfies}
  e-esm-stall:  {source: s-esm,   target: s-stall, kind: satisfies}
---

![Safety Requirement Traceability](./SafetyRequirements.svg)

Traceability from the top-level engine safety requirement `REQ-ENG-SAFE-000` down
to derived requirements and up to verification.  `SafetyMonitor` (ASIL D) satisfies
the 100 ms fault-detection requirement; `EngineStallMonitor` (ASIL B) satisfies
the stall-detection requirement.  `TC-ENG-SAFE-001` (HIL, L5) covers the parent
end-to-end.
