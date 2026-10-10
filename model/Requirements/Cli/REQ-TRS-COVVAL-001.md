---
type: Requirement
id: REQ-TRS-COVVAL-001
name: "validate applies the [coverage] policy to W305 and rejects an invalid policy"
status: draft
reqDomain: software
reqClass: system
tags:
  - validation
  - coverage
---

`validate` shall judge the parent-requirement integration-test check (`W305`) by the `[coverage]` policy and report an invalid policy (GH #253, validator part).

## Behavior

- `W305` names the rule that applied, `(rule: both)` by default. Under `parent_rule = "rollup"` a parent whose every leaf descendant has an active verifying TestCase raises no `W305`; a parent with an unverified leaf still does. `direct` and `both` behave as before (a parent needs its own active L3–L5 TestCase).
- `E898` (attached to `.syscribe.toml`) — the `[coverage]` table is invalid (unknown key, bad rule or selector value, unparseable file), and `E898` on a requirement — an integrity-rated (ASIL/CAL/SIL) parent that a rule would loosen below `both`.
