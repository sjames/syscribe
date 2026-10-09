---
id: REQ-TRS-FTA-003
type: Requirement
name: "Tool shall analyse a fault tree: minimal cut sets, top-event probability, importance measures and beta-factor CCF"
status: draft
reqDomain: software
verificationMethod: test
---

`fault-tree analyze <FT> [--json] [--max-order N] [--no-ccf]` **shall** evaluate the gate logic of one `FaultTree` and report: the minimal cut sets with their order and probability; the exact top-event probability (plus rare-event and min-cut-upper-bound approximations) over the tree's `missionTime` (event probability = explicit `probability:` or `1 - exp(-lambda*t)`); per-event Fussell-Vesely, Birnbaum and RAW importance and a role (`single_point`, `dual_point`, `multi_point`, `irrelevant`, `unreachable`, `house`); and, for events sharing a `ccfGroup:` with a `ccfBeta:`, the beta-factor common-cause expansion. A `house` event is a constant. The analysis is a public library API (`syscribe_model::fta`) validated against a brute-force truth-table oracle.

**Source:** GitHub issue #211.

**Acceptance criteria:** For `OR(AND(a,b),c)` with p = 0.1, 0.2, 0.05 the cut sets are `{c}` and `{a,b}` and the top probability is 0.069; a two-member `ccfGroup` with beta 0.1 yields an order-1 `CCF:` cut set.
