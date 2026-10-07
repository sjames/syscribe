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

After BDD and IBD, derived generators shall be added on the same IR for the remaining SysMLv2
view kinds, each specified by its own requirement: `StateMachine` (`REQ-TRS-VIS-018`), `Action`
(`REQ-TRS-VIS-019`, a new `diagramKind`), `Requirement` (`REQ-TRS-VIS-020`), `Sequence`
(`REQ-TRS-VIS-021`) and `Allocation` (`REQ-TRS-VIS-022`). Each generator follows the rules of
`REQ-TRS-VIS-003` (source selection by frontmatter, `include:`/`exclude:`, deterministic shape
ids, `W417`/`W418`), has golden IR tests, and is drawable by every writer (`REQ-TRS-VIS-009`,
`-010`) and the browser (`REQ-TRS-VIS-006`). Until a kind's generator lands, a `Diagram` of that
kind is manifest-sourced only.

## Rationale

The user chose structure first. Recording the rest here keeps the scope visible and the IR
honest: every enum and layout hint added for BDD/IBD must leave room for these.

## Scope

- The child requirements name the exact model fields each generator reads; this requirement only
  fixes the set of kinds and the shared rules.
- No ELK option beyond the layered and fixed algorithms is assumed; the Sequence generator places
  its own content (`REQ-TRS-VIS-021`).
