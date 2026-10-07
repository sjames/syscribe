---
type: Requirement
id: REQ-TRS-SYSMLV2-076
name: "A bare package-level attribute, port or item is read as the usage it is, with its value and unit"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-000]
breakdownAdr: Decisions::SysmlV2SubmodelADR
tags:
  - sysmlv2
---

A bare package-level `attribute x : T = v [unit];`, `port p : T;` and `item i : T;` shall be ingested as `Attribute`, `Port` and `Item` elements with `typedBy:` (not as definitions with `supertype:`), and an attribute usage's literal value and unit shall map to `value:`/`unit:` at package level exactly as inside a part body.
