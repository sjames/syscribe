---
type: Requirement
id: REQ-TRS-AMEND-001
name: "ADRs and requirements record amendments as data"
status: draft
reqDomain: software
reqClass: system
tags:
  - adr
  - amendments
---

An `ADR` and a native `Requirement` shall accept an `amendments:` list recording each change made after acceptance, and the tool shall check its shape (GH #454).

## Behavior

- Each entry is a mapping `{date, by, reason, ref?, kind}`: `date` an ISO date `YYYY-MM-DD`, `by` and `reason` non-empty, `kind` one of `clarification` or `scope-change`, `ref` optional.
- `ref` resolves, in order, as a question or item reference (`PI-X#Qn`, `REQ-X#slug`), an element (stable id or qualified name), or a commit (7–40 hex) of the repository containing the model; a commit is accepted unchecked when the repository cannot be asked.
- A list that is not a list of mappings, an entry missing `date`/`by`/`reason`/`kind`, an unknown key, a bad date or kind, or a `ref` that resolves to nothing is `E330`.
- `amendments:` on any other element type is an unrecognized field (`W047`).
- `amendments` is excluded from the suspect normative projection, so adding an entry never makes a link suspect.
