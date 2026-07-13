---
type: Requirement
id: REQ-TRS-BL-007
name: "baseline list and show"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-BL-000]
breakdownAdr: Decisions::BaselineADR
tags:
  - baseline
  - cli
---

Syscribe shall provide read-only inventory subcommands over baselines.

- `syscribe -m <root> baseline list` shall enumerate every `Baseline` element with its id,
  name, status, date, and scope summary, ordered deterministically (a stable order such as
  most-recent-release-first is acceptable, provided it is reproducible across runs).
- `syscribe -m <root> baseline show <BL-id>` shall print a baseline's full provenance:
  `name`, `date`, `approver`, `gitTag`/`gitCommit`, the resolved `frozenScope`,
  `elementCount`, `aggregateHash`, `supersedes`, and the location of its manifest.
- `list` and `show` shall be read-only; they shall not modify any model file.
