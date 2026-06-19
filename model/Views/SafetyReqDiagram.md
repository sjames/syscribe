---
type: Diagram
name: SafetyReqDiagram
diagramKind: Requirement
svgMode: companion
pumlMode: companion
pumlFile: ./SafetyReqDiagram.puml
subject: Requirements::SafetyReqs
shapes:
  s-safe:    {ref: "Requirements::SafetyParentReq",        kind: Requirement}
  s-land:    {ref: "Requirements::SafeLandingReq",         kind: Requirement}
  s-fc:      {ref: "Requirements::FaultTolerantFCReq",     kind: Requirement}
  s-review:  {ref: "Verification::SafetyCaseReview",       kind: TestCase}
  s-ltest:   {ref: "Verification::SafeLandingTest",        kind: TestCase}
  s-fitest:  {ref: "Verification::FCFaultInjectionTest",   kind: TestCase}
  s-flightc: {ref: "UAV::Avionics::FlightController",      kind: Part}
edges:
  e-land-safe:   {source: s-land,   target: s-safe,    kind: derivedFrom}
  e-fc-safe:     {source: s-fc,     target: s-safe,    kind: derivedFrom}
  e-review-safe: {source: s-review, target: s-safe,    kind: verifies}
  e-ltest-land:  {source: s-ltest,  target: s-land,    kind: verifies}
  e-fitest-fc:   {source: s-fitest, target: s-fc,      kind: verifies}
  e-flightc-fc:  {source: s-flightc, target: s-fc,     kind: satisfies}
---

<img src="SafetyReqDiagram.svg" alt="Safety Requirement Diagram" width="100%">

![SafetyReqDiagram](./SafetyReqDiagram.svg)

Requirement tree rooted at REQ-UAV-SAFE-000, the top-level safety goal for the UAV. The two derived requirements (REQ-UAV-FC-001, REQ-UAV-SAFE-001) are shown with their test cases and the satisfying FlightController architecture element.

`«derive»` edges connect each child requirement upward to its parent. `«verify»` edges connect each TestCase to the requirement it covers. The `«satisfy»` edge shows that FlightController is the architectural element responsible for the fault-tolerance and safe-landing requirements.
