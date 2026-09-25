---
id: REQ-TRS-ADR-001
type: Requirement
name: Tool shall accept the ADR deciders field and display it in show
status: draft
reqDomain: software
verificationMethod: test
---

Spec §8.17.1 defines an optional `deciders:` field on an `ADR`: a list of
strings, each the qualified name of a stakeholder `PartDef` or the free-text
name of a decision-maker. The tool **shall**:

- accept `deciders:` on an `ADR` as a recognized schema field, so it raises no
  `W047` (a single string is accepted as a one-entry list);
- treat each entry as opaque display metadata — **not** a cross-reference —
  so a free-text name that resolves to no model element raises no finding
  (the spec permits free-text names; `[users]` roster checking applies only to
  `PlanningItem.assignedTo:`, not to `deciders:`);
- print the deciders (and the ADR's `date:`) in the `show` field table;
- continue to report `deciders:` on any element type other than `ADR` as an
  unrecognized field (`W047`), since the spec defines it only for `ADR`.

**Source:** GH issue #159; spec §8.17.1, §8.17.4.

**Acceptance criteria:** the §8.17.4 ADR example validates with no `W047`;
`show` on it lists both deciders; a `PartDef` carrying `deciders:` still raises
`W047`.
