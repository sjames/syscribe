---
type: Diagram
name: RequirementsDerived
diagramKind: Requirement
subject: Requirements
include:
  - REQ-UAV-SAFE-000
  - REQ-UAV-FC-001
  - REQ-UAV-SAFE-001
  - REQ-UAV-PERF-000
  - REQ-UAV-ENDUR-001
  - REQ-UAV-NAV-001
  - REQ-UAV-COMM-001
  - UAV::Avionics::FlightController
  - UAV::Power::BatteryPack
  - UAV::Avionics::GPSReceiver
  - TC-UAV-FC-001
  - TC-UAV-SAFE-001
  - TC-UAV-ENDUR-001
  - TC-UAV-NAV-001
  - TC-UAV-COMM-001
---

Requirement diagram **derived from the model** (`REQ-TRS-VIS-020`): the safety and mission
performance requirement trees of the `Requirements` package with their `«deriveReqt»` legs
(child below parent), the architecture elements whose `satisfies:` names them (`«satisfy»`) and
the test cases whose `verifies:` names them (`«verify»`). The `Requirements` package also holds
this tool's own `REQ-TRS-*` requirements, so `include:` narrows the view to the UAV trees — by
stable id for the requirements and test cases, by qualified name for the parts. Compare
`SafetyRequirementsD`, the hand-listed manifest form of the safety half.
