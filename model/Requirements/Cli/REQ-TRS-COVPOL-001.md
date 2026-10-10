---
type: Requirement
id: REQ-TRS-COVPOL-001
name: "A [coverage] policy sets how a parent requirement is judged, per class, tag and integrity level"
status: draft
reqDomain: software
reqClass: system
tags:
  - cli
  - coverage
---

`.syscribe.toml` shall accept a `[coverage]` table whose ordered rules choose how a parent requirement is judged complete by `coverage tree` (GH #253, v1; the validator findings and `matrix --rollup` follow).

## Behavior

- `default` is `both` (a direct active test **and** every leaf verified), `direct` (a direct active test) or `rollup` (every leaf verified); absent means `both`.
- `[[coverage.rule]]` entries select requirements by `reqClass`, `requirementKind` and `status` (any of; a requirement without a kind never matches a `requirementKind` selector), `tag` (any of), `asil` (any of A–D), `cal` (any of CAL1–CAL4) and `sil` (`"QM"` = no ASIL/CAL/SIL, or integers) — all given selectors must match; the first matching rule wins; `parent_rule` is its verdict rule.
- A rule that would loosen (`direct` or `rollup`) an integrity-rated requirement (ASIL A–D, CAL1–4 or SIL ≥ 1) is a configuration error: `coverage tree` names the requirement and rule, prints nothing else and exits 1. An invalid value or unknown selector is also an error.
- The applied rule is printed on every parent line (`(rule: rollup)`) and carried in `--json` as `rule`. With no `[coverage]` table the output only gains that suffix.
