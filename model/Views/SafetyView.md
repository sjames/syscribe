---
type: View
name: SafetyView
viewpoint: Viewpoints::SafetyEngineerViewpoint
expose:
  - Behavior::FlightStates
  - UAV::Avionics::FlightController
  - Requirements::FaultTolerantFCReq
  - Requirements::SafeLandingReq
  - Verification::FCFaultInjectionTest
  - Verification::SafeLandingTest
---

Safety view exposing the state machine, safety requirements, and their verification methods.
