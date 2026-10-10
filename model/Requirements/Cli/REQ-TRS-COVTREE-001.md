---
type: Requirement
id: REQ-TRS-COVTREE-001
name: "coverage tree rolls test coverage up the derivation tree"
status: draft
reqDomain: software
reqClass: system
tags:
  - cli
  - coverage
---

`syscribe coverage tree <req>` shall show, for a requirement and every requirement derived below it, how well the tree is verified (GH #252, v1: read-only command; `matrix --rollup`, audit percentages, diagrams and the policy of #253 follow).

## Behavior

- Per node, never stored: a leaf is `verified` (an active verifying TestCase), `planned` (only planned TestCases) or `uncovered`, using the same classifier as `matrix` and `coverage`. A parent aggregates its leaf descendants (`active/total`, `planned`, `uncovered`) and the number of direct verifying TestCases.
- Verdict glyph: `●` all leaves verified and the node has a direct test (a leaf: verified); `○` nothing verified, planned or direct; `◐` anything between.
- Leaves not applicable in every configuration (`na`) are excluded from the counts. A cycle in `derivedFrom` terminates.
- `--json` emits the same tree. An unresolvable root exits 1. The rule does not replace direct tests: a parent covered only through children shows `◐`.
