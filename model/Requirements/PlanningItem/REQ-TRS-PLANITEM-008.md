---
type: Requirement
id: REQ-TRS-PLANITEM-008
name: "A PlanningItem may be assigned to a single Unix-style username, with the display name and the declared roster both configured in .syscribe.toml"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-PLANITEM-000]
breakdownAdr: Decisions::PlanningItemADR
tags:
  - planning
---

A `PlanningItem` shall accept an optional `assignedTo: <username>` field naming a single user
responsible for it. The value shall be in the form of a Unix-style username: lowercase, starting
with a letter or underscore, followed by lowercase letters/digits/underscore/hyphen, 1–32
characters — checked **unconditionally**, independent of any roster configuration. A project shall
be able to declare its roster of valid users in a `[users]` table of `<model_root>/.syscribe.toml`,
each entry mapping a username to its display (real) name. When that roster is non-empty, an
`assignedTo:` value not present in it shall be reported as a validation error. When the roster is
empty or the `[users]` table is absent, roster membership shall not be checked — the roster is
opt-in, following the same posture every other `.syscribe.toml`-configured table in this codebase
already uses (`[repos]`, `[ids.prefixes]`, `[matchers]`, …); the username-format check itself is
**not** opt-in and applies regardless.

## Rationale

`assignedTo:` is a plain declared-string field, not a model cross-reference — a user is not a
`RawElement` the resolver can find, so it needs its own format + roster-membership checks rather
than the existing id/qname resolution machinery `parent:`/`achieves:`/`blockedBy:` all share.
Requiring a Unix-style shape catches obviously-wrong values (a display name typed directly into
`assignedTo:`, stray whitespace, mixed case) even before any roster exists to check against — the
same reasoning that makes an id pattern check useful independent of whether the id also resolves
to anything. Requiring the roster itself to be declared in project configuration (rather than
accepting any string, or maintaining a separate `type: User` element) keeps user identity centrally
defined once per project, and pairing each username with its display name in that same declaration
means the model's own frontmatter only ever needs the short, stable form.

## Scope

- `assignedTo:` is a **single scalar**, not a list — deliberately unlike `achieves:`/`blockedBy:`,
  mirroring `parent:`'s "one at a time" shape. Multi-assignee support is a rejected-for-now
  alternative (see the ADR addendum) — nothing here precludes broadening it to a list later if a
  concrete multi-assignee need arises.
- The username format check (Unix-account-like: `^[a-z_][a-z0-9_-]{0,31}$`) applies to **both**
  `assignedTo:` values and the declared `[users]` table's own keys, and is checked regardless of
  whether a roster is configured at all. A malformed `assignedTo:` value is reported once (not
  doubled up with a separate "not declared" finding for the same defect); a malformed `[users]` key
  is reported as its own, config-level finding and excluded from the effective roster — one bad
  roster entry does not take down the check for every other, well-formed one.
- The `[users]` table's values (display/real names) are free prose, unconstrained — like `name:`
  fields elsewhere in this format.
- Whether an unassigned `PlanningItem` (no `assignedTo:` at all) should ever be flagged is out of
  scope — an absent `assignedTo:` never raises anything, regardless of whether a roster is
  configured or how large it is.
- CLI filtering/listing by assignee (e.g. "show me everything assigned to alice") is not addressed
  by this requirement — `assignedTo:` is schema + validation only in this phase, the same posture
  `ADR-SYS-PLANITEM-001` already established for the rest of `PlanningItem`. `show`'s field dump
  is the one display-side change: it resolves and prints the declared display name alongside the
  raw username when the roster is configured.
