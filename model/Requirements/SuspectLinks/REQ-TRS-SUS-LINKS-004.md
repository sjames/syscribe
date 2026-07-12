---
type: Requirement
id: REQ-TRS-SUS-LINKS-004
name: "Stale baseline emits warning W090; unbaselined links stay silent"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-SUS-LINKS-000]
breakdownAdr: Decisions::SuspectLinksADR
tags:
  - traceability
  - suspect-links
---

For each trace link that **has** a stored baseline (REQ-TRS-SUS-LINKS-001), validation
shall resolve the target, recompute its projection hash (REQ-TRS-SUS-LINKS-002), and
compare it to the stored value. On mismatch the validator shall emit warning **W090**
identifying the source element, the target, and the link kind.

## Opt-in behaviour

- A trace link with **no** stored baseline shall **not** be flagged during validation —
  no warning of any kind. Only baselined links are checked. This keeps the feature purely
  additive: existing models emit no new findings until a link is baselined.

## Severity and gating

- W090 is a **Warning**. It participates in the standard gate machinery and can be made
  fatal in CI via `--deny W090` (exit code 2 when tripped).

## Unresolvable target

- If a baselined target cannot be resolved, the existing unresolved-cross-reference
  handling applies; W090 (a content-mismatch signal) shall not be emitted for a target
  that does not resolve, to avoid conflating "missing" with "changed".
