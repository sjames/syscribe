---
id: REQ-TRS-SYSMLV2-015
type: Requirement
name: A dotted connect endpoint keeps its full path, so connect endpoints no longer raise W542 (truncation retired by GH #206)
status: verified
reqDomain: software
verificationMethod: test
---

A `connect` endpoint's dotted chain **shall** map to the full `::` path under the owner and
**shall not** be truncated to its head, so no `connect` endpoint raises `W542`. A tail that is
neither declared nor inherited through the head's `typedBy:`/`supertype:` chain is reported by
validation as `W056` (or `E127` for a missing head). `W542` remains for `allocate` endpoints.

**Source:** `REQ-TRS-SYSMLV2-015` (product model), `ADR-SYS-SYSMLV2-001` addendum.
