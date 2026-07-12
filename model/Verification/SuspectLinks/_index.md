---
type: Package
name: SuspectLinks
---

Test cases for suspect-link detection (`REQ-TRS-SUS-LINKS-*`): the `traceBaselines`
schema field, BLAKE3 content projection/hashing, opt-in suspect detection and warning
W090, the `suspect accept` / `suspect list` subcommands, and implicit one-hop
propagation.

Leaf test cases (L1/L2) verify individual `REQ-TRS-SUS-LINKS-001..007`; integration test
cases (L3) verify the stakeholder parent `REQ-TRS-SUS-LINKS-000` end-to-end.
