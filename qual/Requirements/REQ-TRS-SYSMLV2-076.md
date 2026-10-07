---
id: REQ-TRS-SYSMLV2-076
type: Requirement
name: "A bare package-level attribute, port or item is read as the usage it is, with its value and unit"
status: verified
reqDomain: software
verificationMethod: test
---

A bare package-level `attribute x : T = v [unit];`, `port p : T;` and `item i : T;` shall be ingested as `Attribute`, `Port` and `Item` elements with `typedBy:` (not as definitions with `supertype:`), and an attribute usage's literal value and unit shall map to `value:`/`unit:` at package level exactly as inside a part body.

**Source:** `REQ-TRS-SYSMLV2-076` (product model), `ADR-SYS-SYSMLV2-001`.
