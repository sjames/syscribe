---
type: Requirement
id: REQ-TRS-SUS-LINKS-005
name: "suspect accept captures and refreshes baselines"
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

Syscribe shall provide a `suspect accept` subcommand that records the current state of a
link as reviewed by writing the target's current projection hash into the source's
`traceBaselines` map.

- `syscribe -m <root> suspect accept <source> <target>` shall compute the current
  projection hash of `<target>` (REQ-TRS-SUS-LINKS-002) and create or overwrite the entry
  for `<target>` in `<source>`'s `traceBaselines` map (REQ-TRS-SUS-LINKS-001). `<source>`
  and `<target>` may be given as stable id or qualified name.
- `syscribe -m <root> suspect accept --all` shall (re-)baseline **every** currently
  suspect link across the model (the W090 set of REQ-TRS-SUS-LINKS-004) in one pass.
- Writing shall modify only the `traceBaselines` field, preserving the source file's
  existing frontmatter formatting and field order as far as the writer allows.
- It shall be an error to `accept` a `<target>` that is not actually referenced by any
  trace link on `<source>` (nothing to baseline).

Accepting a link is the sole mechanism for **clearing** a suspect flag: after acceptance,
the stored hash matches the target's current projection, so W090 no longer fires until the
target's projection changes again.
