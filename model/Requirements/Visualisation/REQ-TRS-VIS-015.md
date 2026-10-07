---
type: Requirement
id: REQ-TRS-VIS-015
name: "Follow-on: State, Action, Requirement, Sequence and Allocation generators on the same IR"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-VIS-000]
breakdownAdr: Decisions::VisualisationADR
tags:
  - diagram
  - visualisation
---

After BDD and IBD, derived generators shall be added on the same IR, in this order, each with
its own golden tests and writer support: `StateMachine` (states, nested regions, transitions
with trigger/guard/effect labels), `Action` (action steps and successions), `Requirement`
(derivation tree with satisfying and verifying elements), `Sequence` (lifelines and messages,
consistent with the `W080` completeness rule) and `Allocation` (allocation map). Until a kind's
generator lands, a `Diagram` of that kind is manifest-sourced only.

## Rationale

The user chose structure first. Recording the rest here keeps the scope visible and the IR
honest: every enum and layout hint added for BDD/IBD must leave room for these.

## Scope

- Order may change with user priority; each generator is its own planning item when scheduled.
- No ELK option beyond the layered and fixed algorithms is assumed.
