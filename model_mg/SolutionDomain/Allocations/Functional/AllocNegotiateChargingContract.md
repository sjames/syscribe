---
type: Allocation
name: AllocNegotiateChargingContract
features:
  - name: source
    type: Allocation
    allocatedFrom: ProblemDomain::WhiteBox::FunctionalAnalysis::NegotiateChargingContract
    allocatedTo: ProblemDomain::WhiteBox::LogicalSubsystems::EnergyControlSubsystem
---

Functional allocation: the **NegotiateChargingContract** action is performed by
the **EnergyControlSubsystem**, which exchanges limits with the vehicle and
derives the agreed setpoint.
