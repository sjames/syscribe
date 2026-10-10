---
type: Requirement
id: REQ-TRS-AUDITCLS-001
name: "audit reports requirement coverage per requirement class, not one flat figure"
status: draft
reqDomain: software
reqClass: system
tags:
  - cli
  - audit
---

`audit` shall break requirement coverage down by `reqClass`, using the derivation-tree roll-up (GH #252, audit part).

## Behavior

- A section "Coverage by Requirement Class" lists, per `reqClass` (`-` for a requirement without one), the number of requirements whose roll-up verdict is complete, partial, none or n/a, and the percentage complete of those applicable. The verdicts are those of `matrix --rollup` / `coverage tree` including the `[coverage]` policy.
- `--json` carries `coverageByClass{class: {complete, partial, none, na, percentComplete}}`. The view honours `--config` and `--plan` like the other sections. The verdict (PASS/FAIL) is unchanged by this section.
- An invalid `[coverage]` table makes the section report the policy problem instead of numbers (the table's own `E898` already fails the verdict).
