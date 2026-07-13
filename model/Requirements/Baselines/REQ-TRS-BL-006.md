---
type: Requirement
id: REQ-TRS-BL-006
name: "baseline diff compares two baselines"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-BL-000]
breakdownAdr: Decisions::BaselineADR
tags:
  - baseline
  - cli
---

Syscribe shall provide a `baseline diff` subcommand that compares two baselines.

- `syscribe -m <root> baseline diff <BL-A> <BL-B>` shall report, keyed by **stable id**
  (falling back to qualified name so the comparison survives file moves):
  - **added** — elements in `<BL-B>`'s scope but not `<BL-A>`'s,
  - **removed** — elements in `<BL-A>` but not `<BL-B>`,
  - **changed** — elements in both whose per-element hash differs,
  grouped by element type. Meta differences (scope, tool version, approver) shall also be
  surfaced.
- The element-level (hash) comparison shall work **offline** from the two manifests
  (REQ-TRS-BL-004); no source-control access shall be required to list what changed.
- `baseline diff <BL-A> <BL-B> --detail` shall additionally show the field/body-level change
  for each changed element, reconstructing content from source control
  (`git show <gitCommit>:<file>` for each side). Where a baseline's commit or a file is not
  retrievable, the tool shall degrade gracefully to the hash-level result for that element.
- Output shall be deterministic and suitable for diffing/reporting.
