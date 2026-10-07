---
id: REQ-TRS-SYSMLV2-020
type: Requirement
name: "A SysMLv2 view def/view maps to the native ViewDef/View schema — rendering, and (usage only) expose/viewpoint"
status: verified
reqDomain: software
verificationMethod: test
---

A `view def`/`view` usage shall be synthesized into a native `ViewDef`/`View` element carrying the
same `expose:`/`viewpoint:`/`rendering:` shape a hand-authored one uses
(`model/Views/SystemArchitectureView.md`'s existing convention), so a SysMLv2-authored view
participates fully in the existing `W500`/`W502` cross-reference checks, with zero validator
changes. A `view def` synthesizes a `ViewDef`; a `view` usage synthesizes a `View`.

**Source:** `REQ-TRS-SYSMLV2-020` (product model), `ADR-SYS-SYSMLV2-001` addendum.
