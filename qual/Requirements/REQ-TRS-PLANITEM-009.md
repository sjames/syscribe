---
id: REQ-TRS-PLANITEM-009
type: Requirement
name: The template subcommand knows PlanningItem and can print a ready-to-fill skeleton for it
status: draft
reqDomain: software
verificationMethod: test
---

`syscribe template PlanningItem` **shall** print a ready-to-fill frontmatter skeleton for the
`PlanningItem` type — a `PI-*`-shaped `id`, a `name`, and a `status` — instead of failing with
`Unknown type '...'`. `PlanningItem` **shall** also appear in the "Native elements" line of the
"Known types" listing shown for an unrecognized `template <type>` argument.

**Source:** `REQ-TRS-PLANITEM-009` (product model), `ADR-SYS-PLANITEM-001`.
