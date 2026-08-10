---
type: Requirement
id: REQ-TRS-PLANITEM-009
name: "The template subcommand knows PlanningItem and can print a ready-to-fill skeleton for it"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-PLANITEM-000]
breakdownAdr: Decisions::PlanningItemADR
tags:
  - planning
---

`syscribe template PlanningItem` shall print a ready-to-fill frontmatter skeleton for the
`PlanningItem` type instead of failing with `Unknown type '...'`. The skeleton shall include a
`PI-*`-shaped `id`, a `name`, and a `status` (with the four accepted values listed as a comment),
plus commented-out examples of `itemType`, `parent`, `blockedBy`, `assignedTo`, and `evidence`
covering both the top-level (`achieves:`) and child (`parent:`) shapes established by
`ADR-SYS-PLANITEM-001`. `PlanningItem` shall also appear in the "Native elements" line of the
"Known types" listing shown for an unrecognized `template <type>` argument.

## Rationale

`PlanningItem` is a fully working native type — `list`, `types`, `show`, and `validate` all
handle it, and this repository's own `model/Planning/` uses it to track its own feature work —
but `template`'s type dispatch was never updated when the type shipped (`ADR-SYS-PLANITEM-001`).
`template` is the documented, recommended first step for authoring any new element (`spec
types`/`spec fields`/`--agent-instructions` all say so), so a type it silently doesn't know about
is a real authoring-workflow gap, not just a cosmetic omission. The MCP server's `template` tool
(`REQ-TRS-MCP-013`) delegates to the same underlying lookup, so fixing it here closes the gap on
both surfaces at once.

## Scope

- This is a `template`-only fix. `list`/`types`/`validate`/`show` already handle `PlanningItem`
  correctly and are unaffected.
- The skeleton's placeholder values (`PI-PREFIX-001`, `REQ-PREFIX-001`, …) are illustrative, the
  same convention every other native-type template already uses — they are not expected to
  resolve or validate cleanly as-is.
