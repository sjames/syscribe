---
type: StateDef
name: ChargingSessionStates
allocatedTo: ProblemDomain::WhiteBox::LogicalSubsystems::EnergyControlSubsystem
subStates:
  - name: Idle
    isInitial: true
  - name: Authorising
  - name: Negotiating
  - name: Charging
  - name: Faulted
  - name: Completed
transitions:
  - source: Idle
    target: Authorising
    accept: plugIn
  - source: Authorising
    target: Negotiating
    accept: authorised
  - source: Negotiating
    target: Charging
    accept: contractAgreed
  - source: Charging
    target: Completed
    accept: targetSocReached
  - source: Charging
    target: Faulted
    accept: faultDetected
  - source: Faulted
    target: Idle
    accept: reset
  - source: Completed
    target: Idle
    accept: unplug
refines:
  - ProblemDomain::WhiteBox::SystemRequirements::Availability
custom_fields:
  mg_cell: W2
---

State machine of a charging session, from Idle through authorisation, contract
negotiation and charging to completion, with a fault branch that isolates energy
and returns to Idle on reset.
