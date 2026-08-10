---
type: Requirement
id: REQ-TRS-SYSMLV2-012
name: "A named connection usage's own trailing doc /* ... */ body shall lift into the synthesized Connection element's doc field"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-000]
breakdownAdr: Decisions::SysmlV2SubmodelADR
tags:
  - sysmlv2
---

A named `connection name : Type connect a to b { doc /* ... */ }` usage's own trailing body shall
have its `doc /* ... */` member(s) lifted into the synthesized `Connection` element's `doc` field
— the same lift `REQ-TRS-SYSMLV2-009` already gives a `connection def { }`'s own body, applied to
the sibling *usage* code path that requirement didn't reach.

## Rationale

`ConnectionUsageMember.body: ConnectionDefBody` is a real, already-parsed field on the same struct
`REQ-TRS-SYSMLV2-010`'s endpoint lift reads `connect_from`/`connect_to`/`connect_extra_ends`
from — `convert_connection_usage` never touches it at all, not even to recurse into it. A
`connection` usage's own explanatory comment (naming which specific interface it represents, for
instance) is silently discarded with no diagnostic, the exact class of gap `REQ-TRS-SYSMLV2-009`
closed for every *other* body-bearing SysMLv2 construct.

## Scope

- Reuses the existing `connection_def_doc` helper unchanged — `ConnectionUsageMember.body` is the
  same `ConnectionDefBody`/`ConnectionDefBodyElement` shape `convert_connection_def` already reads
  its own body through, so no new AST-reading logic is needed, only a new call site.
- Scoped to the named-usage form `convert_connection_usage` already handles — the anonymous
  binary-connector form stays out of scope, matching `REQ-TRS-SYSMLV2-010`'s own precedent.
- A connection usage with no trailing body (`connection c : SomeDef connect a to b;`, the only
  form that worked before this requirement) is unaffected — `doc: ""`, no regression.
- `W600`/`W601`-style empty-doc-body checks do not apply to `type: Connection` today (they're
  scoped to `PartDef`/`Part`), so this lift has no validation-visible effect beyond `show`/`export`
  surfacing the text — consistent with what a hand-authored `Connection` element with a body
  already gets.

**Acceptance criteria:** `connection c : SomeDef connect a to b { doc /* Explanation. */ }` lifts
`doc: "Explanation."` onto the synthesized `Connection` element, retrievable via `show`/`export`;
a connection usage with no trailing body behaves exactly as it does today.
