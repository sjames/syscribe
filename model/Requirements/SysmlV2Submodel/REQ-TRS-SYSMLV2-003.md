---
type: Requirement
id: REQ-TRS-SYSMLV2-003
name: "A SysMLv2 element's native satisfy/verify relationship can target a Syscribe Requirement"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-000]
breakdownAdr: Decisions::SysmlV2SubmodelADR
tags:
  - sysmlv2
  - traceability
---

A SysMLv2 element's native `satisfy`/`verify` relationship shall be able to target a native
Syscribe `Requirement`, by its `REQ-*` id — using SysML v2's quoted-name syntax for the hyphenated
id, e.g. `satisfy 'REQ-SCHED-001';` — or by its Syscribe qualified name. The mapper carries the
target string verbatim into the synthesized element's `satisfies:`/`verifies:` field; resolution
then uses the existing id-or-qname resolver unchanged.

## Rationale

`satisfy` and `verify` are SysML v2's own OSLC-shaped relationship keywords — reusing them keeps
the `.sysml` source standards-compliant and directly usable by real SysML v2 tooling (`spec42` and
others), rather than requiring a Syscribe-specific annotation for a relationship the language
already expresses natively.

## Scope

- A `satisfy`/`verify` target that resolves to neither a real id nor a real qname is a
  dangling-reference finding — the same class already raised for any other unresolved
  `satisfies:`/`verifies:` today. No new diagnostic code is introduced for this.
- Direction follows the existing OSLC link-direction rule (`CLAUDE.md` §12.1): the SysMLv2 artifact
  holds the reference; the target `Requirement` does not reference back.
- This requirement covers `Requirement` targets only; a `TestCase` verifying a SysMLv2 element is
  the reverse direction, `REQ-TRS-SYSMLV2-004`.
