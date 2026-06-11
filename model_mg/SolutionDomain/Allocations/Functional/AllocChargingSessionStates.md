---
type: Allocation
name: AllocChargingSessionStates
features:
  - name: source
    allocatedFrom: ProblemDomain::WhiteBox::FunctionalAnalysis::ChargingSessionStates
    allocatedTo: ProblemDomain::WhiteBox::LogicalSubsystems::EnergyControlSubsystem
---

Functional allocation: the **ChargingSessionStates** state machine is run by the
**EnergyControlSubsystem**, which sequences the session from Idle through
Charging to Completed, with the fault branch.
