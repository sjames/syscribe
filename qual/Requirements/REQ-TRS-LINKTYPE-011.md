---
id: REQ-TRS-LINKTYPE-011
type: Requirement
name: "User-defined links shall participate in suspect-link detection unless the type opts out"
status: draft
reqDomain: software
verificationMethod: test
---

Custom link targets **shall** be treated as trace links by suspect-link detection
(`ADR-SYS-SUSLINK-001`): `suspect list` lists them, `suspect accept` baselines them into
`traceBaselines:`, and validation raises `W090` when a baselined target changes. A link type
declaring `suspect = false` **shall** be excluded from all of these.

**Acceptance criteria:** a baselined custom link raises `W090` after its target changes; the
same with `suspect = false` does not and is absent from `suspect list`.

**Source:** `REQ-TRS-LINKTYPE-011` (product model), `ADR-SYS-LINKTYPE-001`.
