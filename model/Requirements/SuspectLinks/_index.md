---
type: Package
name: SuspectLinks
---

Requirements for suspect-link detection: the `traceBaselines` schema field, BLAKE3
content projection/hashing of trace-link targets, opt-in suspect detection and its
warning code, the `suspect` CLI subcommand (`accept` / `list`), and implicit one-hop
propagation semantics.

All requirements derive from `REQ-TRS-SUS-LINKS-000` and are governed by
`ADR-SYS-SUSLINK-001`.
