---
type: StateDef
name: ChargingSessionStates
subStates:
  - name: Idle
  - name: Authorising
  - name: Negotiating
  - name: Charging
  - name: Faulted
  - name: Completed
transitions:
  - from: Idle
    to: Authorising
    trigger: plugIn
  - from: Authorising
    to: Negotiating
    trigger: authorised
  - from: Negotiating
    to: Charging
    trigger: contractAgreed
  - from: Charging
    to: Completed
    trigger: targetSocReached
  - from: Charging
    to: Faulted
    trigger: faultDetected
  - from: Faulted
    to: Idle
    trigger: reset
  - from: Completed
    to: Idle
    trigger: unplug
refines:
  - ProblemDomain::WhiteBox::SystemRequirements::Availability
custom_fields:
  mg_cell: W2
---

State machine of a charging session, from Idle through authorisation, contract
negotiation and charging to completion, with a fault branch that isolates energy
and returns to Idle on reset.
