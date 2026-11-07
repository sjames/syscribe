---
type: Requirement
id: REQ-TRS-AMEND-003
name: "A scope-change amendment names the owner's decision"
status: draft
reqDomain: software
reqClass: system
tags:
  - adr
  - amendments
---

A `kind: scope-change` amendment shall carry an owner reference, else `validate` shall raise `W325` (GH #454).

## Behavior

- An owner reference is a `ref` naming an answered PlanningItem question (`status: answered` with an answer), a `ReviewRecord`, or a commit made by a person.
- A commit is a person's when its message carries `Syscribe-As: human`, or it has no `Syscribe-Actor` trailer and its author matches none of `[agents] actors`; a commit that cannot be looked up is accepted.
- No `ref`, an open or deferred question, another element type, or a requirement item is not an owner reference. A `clarification` needs none.
