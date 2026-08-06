---
id: REQ-TRS-SYSMLV2-003
type: Requirement
name: A SysMLv2 element's native satisfy/verify relationship shall be able to target a native Requirement
status: draft
reqDomain: software
verificationMethod: test
---

A SysMLv2 element's native `satisfy`/`verify` relationship **shall** be able to target a native
Syscribe `Requirement`, addressed either by its quoted `REQ-*` stable id (SysML v2's quoted-name
syntax, e.g. `satisfy 'REQ-SCHED-001';`, needed because a bare SysML v2 identifier cannot contain
a hyphen) or by its Syscribe qualified name. The mapper **shall** carry the target string
verbatim into the synthesized element's `satisfies:`/`verifies:` field; resolution **shall** use
the existing id-or-qname resolver unchanged, with no SysMLv2-specific resolution logic.

**Source:** `REQ-TRS-SYSMLV2-003` (product model).

**Acceptance criteria:** a SysMLv2 element's `satisfy 'REQ-X';` (quoted id) resolves cleanly
against a real `REQ-X` and suppresses that requirement's "no satisfying element" warning; a
SysMLv2 element's `satisfy Pkg::'REQ-Y';` (qualified name) resolves the same way against a
different requirement; a SysMLv2 `requirement` usage's own `verify 'REQ-Z';` resolves against a
third requirement with no dangling-reference finding.
