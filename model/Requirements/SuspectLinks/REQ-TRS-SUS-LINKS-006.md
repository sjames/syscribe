---
type: Requirement
id: REQ-TRS-SUS-LINKS-006
name: "suspect list reports suspect and unbaselined links"
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

Syscribe shall provide a `suspect list` subcommand that reports the state of baselined and
baselineable trace links across the model.

- `syscribe -m <root> suspect list` shall report every **suspect** link (a baselined link
  whose target's current projection hash differs from the stored baseline —
  REQ-TRS-SUS-LINKS-004), giving for each: the source element, the target, and the link
  kind.
- It shall additionally be able to report trace links that have **no** baseline yet
  (candidates for `suspect accept`). Because validation is silent on unbaselined links
  (REQ-TRS-SUS-LINKS-004), this command is how opt-in coverage gaps are made discoverable
  on demand.
- The report shall be deterministic in ordering so that its output is stable across runs
  and suitable for diffing.

The command is read-only; it shall not modify any model file.
