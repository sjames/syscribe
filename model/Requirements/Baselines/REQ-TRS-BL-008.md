---
type: Requirement
id: REQ-TRS-BL-008
name: "baseline verify proves content and git consistency"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-BL-000]
breakdownAdr: Decisions::BaselineADR
tags:
  - baseline
  - cli
---

Syscribe shall provide a `baseline verify` subcommand that proves a baseline is intact — the
load-bearing check an assessor or CI gate relies on.

- `syscribe -m <root> baseline verify <BL-id>` shall perform two checks and exit non-zero if
  either fails:
  1. **Content proof** — re-resolve the scope, recompute the in-scope aggregate hash
     (REQ-TRS-BL-002), and confirm it equals the element's `seal.aggregateHash` **and** the
     aggregate recorded in the manifest. A mismatch is a verification failure (aligned with
     the drift/tamper findings of REQ-TRS-BL-005).
  2. **Git consistency** — when the `gitTag` exists, confirm it resolves to the recorded
     `gitCommit`; a mismatch is a verification failure. A missing tag shall be reported as a
     warning, not a hard failure (the tag may not yet be pushed).
- `verify` shall be read-only; it shall not modify any model file.
- `verify` shall support verifying all baselines in one invocation (e.g. `--all`) for use as
  a CI gate.
