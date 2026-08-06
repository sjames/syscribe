---
type: Requirement
id: REQ-TRS-PLANITEM-005
name: "PlanningItem evidence is a list of element-reference or file/doc entries, each independently waivable by rationale"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-PLANITEM-000]
breakdownAdr: Decisions::PlanningItemADR
tags:
  - planning
  - traceability
---

A `PlanningItem` shall accept an `evidence:` field — a list of entries, each one of:

- **`ref: <stable-id-or-qualified-name>`** — a reference to an existing model element (an
  architecture element, a `TestCase`, or any other element the resolver can find; not restricted to
  a fixed allowed-kind list).
- **`path: <local-path-or-URI>`** — a reference to a file or document created as proof of the work,
  resolved the same way `Requirement`/`Part`'s `implementedBy:` already resolves a path (§12.8): a
  local path is checked to exist, a remote URI is accepted as external without a local check.

Either form may additionally carry **`rationale: <string>`**, which documents *and waives* that
specific entry's existence/resolution check — mirroring the established `ffiRationale` pattern
(the HW/SW freedom-from-interference waiver), not a new suppression mechanism.

## Rationale

Duck-typing by which key is present (`ref:` vs `path:`) rather than an explicit `type:`
discriminator matches this codebase's existing `features:`-list idiom (an `Allocation`'s
`allocatedFrom`/`allocatedTo` pair is recognised as an edge by which keys are present, not by a
tag). A per-entry, co-located `rationale:` keeps the waiver next to the thing it excuses, auditable
in the same diff that adds it, rather than a separate global suppression list a reviewer would have
to cross-reference.

## Scope

- `ref:` accepts any resolvable id or qualified name; the target's element type is not checked
  against an allowed-kind list (`ADR-SYS-PLANITEM-001`'s Decision 3).
- An unresolved `ref:` or a non-existent local `path:` is a validation finding **unless** that same
  entry also carries `rationale:`, in which case the check is skipped for that entry only — every
  other entry in the same `evidence:` list is still checked normally.
- A `path:` entry's local/remote classification and existence-check semantics reuse
  `implementedBy:`'s existing resolution logic rather than reimplementing it.
- Whether a `PlanningItem` is *required* to have at least one `evidence:` entry, and under what
  condition, is `REQ-TRS-PLANITEM-006`, not this requirement.
