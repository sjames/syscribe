---
type: Diagram
name: SafetyRequirementsD
diagramKind: Requirement
svgMode: companion
svgFile: ./SafetyRequirementsD.svg
pumlMode: companion
pumlFile: ./SafetyRequirementsD.puml
subject: Requirements::SafetyReqs
shapes:
  s-safety-root:
    ref: Requirements::SafetyReqs
    kind: RequirementDef
  s-fc-req:
    ref: Requirements::FaultTolerantFCReq
    kind: Requirement
  s-land-req:
    ref: Requirements::SafeLandingReq
    kind: Requirement
  s-fc-test:
    ref: Verification::FCFaultInjectionTest
    kind: TestCaseDef
  s-land-test:
    ref: Verification::SafeLandingTest
    kind: TestCaseDef
  s-fc-hw:
    ref: UAV::Avionics::FlightController
    kind: PartDef
  s-uav-sys:
    ref: UAV::UAVSystem
    kind: PartDef
edges:
  e-fc-derive:
    ref: Requirements::FaultTolerantFCReq
    source: s-fc-req
    target: s-safety-root
    kind: derivedFrom
  e-land-derive:
    ref: Requirements::SafeLandingReq
    source: s-land-req
    target: s-safety-root
    kind: derivedFrom
  e-fc-verify:
    ref: Verification::FCFaultInjectionTest
    source: s-fc-test
    target: s-fc-req
    kind: verifies
  e-land-verify:
    ref: Verification::SafeLandingTest
    source: s-land-test
    target: s-land-req
    kind: verifies
  e-fc-alloc:
    ref: Allocations::RequirementAllocation
    source: s-fc-req
    target: s-fc-hw
    kind: allocatedTo
  e-land-alloc:
    ref: Allocations::RequirementAllocation
    source: s-land-req
    target: s-uav-sys
    kind: allocatedTo
layout:
  s-safety-root:
    x: 270
    y: 20
    w: 240
    h: 56
  s-fc-req:
    x: 131
    y: 140
    w: 240
    h: 70
  s-land-req:
    x: 495
    y: 115
    w: 240
    h: 70
  s-fc-test:
    x: -21
    y: 308
    w: 200
    h: 56
  s-land-test:
    x: 336
    y: 284
    w: 200
    h: 56
  s-fc-hw:
    x: 280
    y: 390
    w: 160
    h: 46
  s-uav-sys:
    x: 557
    y: 384
    w: 160
    h: 46
---

![SafetyRequirementsD](./SafetyRequirementsD.svg)

Safety requirements traceability showing derivation from the abstract safety root,
allocation to hardware elements, and verification by test cases.
