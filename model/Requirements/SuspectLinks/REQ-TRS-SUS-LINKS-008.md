---
type: Requirement
id: REQ-TRS-SUS-LINKS-008
name: "suspect accept --all-unbaselined onboards links that have no baseline"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-SUS-LINKS-000]
breakdownAdr: Decisions::SuspectLinksADR
tags:
  - traceability
  - suspect-links
  - cli
---

Syscribe shall provide a `suspect accept --all-unbaselined` mode that captures a baseline
for **every trace link that currently has no baseline**, as a deliberate one-time
onboarding step for adopting suspect-link detection across an existing model.

- `syscribe -m <root> suspect accept --all-unbaselined` shall, for each trace link whose
  target resolves and for which the source holds **no** `traceBaselines` entry
  (the un-baselined set of REQ-TRS-SUS-LINKS-006), compute the target's current projection
  hash (REQ-TRS-SUS-LINKS-002) and write it into the source's `traceBaselines` map
  (REQ-TRS-SUS-LINKS-001).
- It shall **not** modify links that already carry a baseline: it never overwrites an
  existing entry and therefore never silently clears an outstanding suspect flag. This
  distinguishes it from `--all` (REQ-TRS-SUS-LINKS-005), which re-baselines the **suspect**
  set. The two flags are mutually exclusive; supplying both is a usage error.
- A link whose target does not resolve shall be skipped (nothing to hash), consistent with
  REQ-TRS-SUS-LINKS-004.
- The command shall report how many links were baselined and be idempotent: a second
  immediate run baselines nothing further and emits no error.

## Rationale

Initial adoption is intentionally separated from routine review. `--all` re-baselines only
the suspect (W090) set so that clearing a suspect flag always reflects a human review;
auto-baselining every link on demand would assert "all links are reviewed" without review.
`--all-unbaselined` makes wholesale adoption a single explicit, auditable action distinct
from that review flow, and — because it never touches an existing baseline — it can never
mask a link that has already gone suspect.
