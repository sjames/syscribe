---
id: REQ-TRS-SYSMLV2-022
type: Requirement
name: "A SysMLv2 rendering def/rendering maps to the native RenderingDef/Rendering schema"
status: verified
reqDomain: software
verificationMethod: test
---

A `rendering def`/`rendering` usage shall be synthesized into a native `RenderingDef`/`Rendering`
element, so a `view`'s `render` clause (`REQ-TRS-SYSMLV2-020`'s `rendering:` field) can reference a
real, browsable element rather than a dangling name.

**Source:** `REQ-TRS-SYSMLV2-022` (product model), `ADR-SYS-SYSMLV2-001` addendum.
