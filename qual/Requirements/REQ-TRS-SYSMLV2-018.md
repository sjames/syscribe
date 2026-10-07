---
id: REQ-TRS-SYSMLV2-018
type: Requirement
name: "A SysMLv2 state def/state maps to the native StateDef/State schema — subStates, transitions with guard/accept/effect, entry/do/exit action names, isInitial/isFinal"
status: verified
reqDomain: software
verificationMethod: test
---

A `state def`/`state` usage shall be synthesized into a native `StateDef`/`State` element carrying
the same `subStates:`/`transitions:`/`entryAction:`/`doAction:`/`exitAction:` shape a hand-authored
one uses (`docs/model-guide/state-machines.md`'s canonical schema), so a SysMLv2-authored state
machine participates fully in the existing `W070`–`W080` completeness checks and in `satisfies:`/
`verifies:` traceability, with zero validator changes. A top-level `state`/`state def` (declared
directly in a package or part) is its own real, qname-addressable element; a `state` nested inside
another `StateDef`/`StateUsage`'s own body becomes an inline `subStates:` entry only, never a
separate element — matching how a hand-authored composite state machine is already written.

**Source:** `REQ-TRS-SYSMLV2-018` (product model), `ADR-SYS-SYSMLV2-001` addendum.
