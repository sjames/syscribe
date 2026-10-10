---
type: Requirement
id: REQ-TRS-DOCFIX-001
name: "Evidence-status rules, plan coverage, W039 wording and the ASIL guide are stated where they apply"
status: draft
reqDomain: software
reqClass: system
tags:
  - docs
---

Documentation and output shall state the rules readers otherwise have to guess (GH #248, #249).

## Behavior

- `verification-depth` states which TestCase statuses it counts, and so do the help pages of `who-verifies`, `trace`, `audit` and `behavioral-coverage`.
- `testplan` prints what its Coverage percentage is a percentage of.
- The `W039` message for a `CAL3` item names "I2 or higher".
- The ISO 26262 guide says that `W811` and `hara matrix` derive and cross-check the ASIL; the `W039` rows cover `CAL3`; the `Allocation` naming convention for `W042` and the precedence between `requirementKind` and `reqClass` are documented.
