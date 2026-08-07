---
type: PlanningItem
id: PI-HPLE-SUBCONFIG-001
name: "subConfigurations: field, resolution, and peer-Configuration validity gate"
status: done
itemType: task
parent: PI-HPLE-001
achieves: [REQ-TRS-HPLE-001]
evidence:
  - path: "repo:crates/syscribe-model/tests/hple_subconfigurations.rs"
  - path: "repo:crates/syscribe-model/tests/hple_review3_adversarial.rs"
tags:
  - variability
  - multi-repo
---

Add `subConfigurations:` to `Configuration`'s schema; resolve each entry to a real `Configuration`
(local or `repoImports:`-mounted); require the resolved `Configuration` to itself be internally
valid (SAT-clean) before it can be consolidated.

Landed across 7 commits (`a1e5feb`..`464410c`) and three independent adversarial review rounds —
the first two each found a real, confirmed bug before merge: a genuine cross-repo circular
`subConfigurations` chain could crash the process with a stack overflow on realistic small-stack
threads (fixed by running peer-validity recursion on a dedicated, large-stack thread), and a
fixed-snapshot approach to closing the local-vs-peer validation asymmetry only propagated one
level transitively (fixed by processing local targets in topological dependency order with a
growing findings accumulator, via Kahn's algorithm). The third round, probing the previously-untested
mixed local+peer case and deeper chains, found nothing further. 25 tests total.
