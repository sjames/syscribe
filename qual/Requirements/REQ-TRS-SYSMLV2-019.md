---
id: REQ-TRS-SYSMLV2-019
type: Requirement
name: "A SysMLv2 action def/action maps to the native ActionDef/Action schema — subActions, controlNodes, successionConnections, real if/while/loop/for recursion, fork/join/decide/merge as name-only control nodes"
status: verified
reqDomain: software
verificationMethod: test
---

An `action def`/`action` usage shall be synthesized into a native `ActionDef`/`Action` element
carrying the same `subActions:`/`controlNodes:`/`successionConnections:` shape a hand-authored one
uses (`model/Behavior/{TakeoffAction,LandingAction,WaypointNavAction,MissionExecution}.md`'s
existing convention), going as deep as the pinned parser (`sysml-v2-parser = "0.54.0"`) actually
retains: `if`/`while`/`loop`/`for` recurse for real; `fork`/`join`/`decide`/`merge` become flat,
name-only control nodes with no recoverable internal content, since the parser itself discards their
block-body contents. A top-level `action`/`action def` is its own real, qname-addressable element;
an action-body construct found nested inside another `ActionDef`/`ActionUsage`'s own body becomes
inline `subActions:`/`controlNodes:` data only, never a separate element.

**Source:** `REQ-TRS-SYSMLV2-019` (product model), `ADR-SYS-SYSMLV2-001` addendum.
