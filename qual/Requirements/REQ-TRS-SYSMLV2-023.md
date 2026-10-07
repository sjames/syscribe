---
id: REQ-TRS-SYSMLV2-023
type: Requirement
name: "A SysMLv2 concern def/concern maps to the native ConcernDef/Concern schema — subject, stakeholders"
status: verified
reqDomain: software
verificationMethod: test
---

A `concern def`/`concern` usage shall be synthesized into a native `ConcernDef`/`Concern` element
carrying `subject:`/`stakeholders:`, so `ViewpointDef.concerns:`/`RequirementDef.concerns:` — which
already exist as native fields — have something real to eventually reference, and so a
SysMLv2-authored concern participates in browsing/cross-reference the same way a hand-authored one
would (were one to exist — none currently does anywhere in `model/`).

**Source:** `REQ-TRS-SYSMLV2-023` (product model), `ADR-SYS-SYSMLV2-001` addendum.
