---
type: Requirement
id: REQ-TRS-SYSMLV2-100
name: "No package-level member is silently dropped: calc usage shapes map and a qualified-name package is counted"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-000]
breakdownAdr: Decisions::SysmlV2SubmodelADR
tags:
  - sysmlv2
---

Every package-level member the 0.57 parser produces shall be either ingested or counted in `W543`. A package-level `calc` usage shape that the parser does not read as a `calc def` (`calc estimate [1];`) shall map to a `Calculation` element exactly as the same usage inside a part body does. A `package`/`library package`/`namespace` declared with a qualified name (`package A::B { ... }`, accepted by the parser but given no meaning by the language) shall be counted once per declaration as `qualified package` in the file's `W543`, its members not ingested, instead of vanishing. `docs/model-guide/sysmlv2-submodel.md` section 6 shall state the `W543` kinds exactly as counted after `REQ-TRS-SYSMLV2-098`..`-100`, shall name members inside a mapped definition's body that are outside the mapped set as skipped and not counted (`ref`, `bind`, `assert constraint`, `exhibit`, a nested `package`/`alias`/`import`/`metadata def`, a `require`/`frame` constraint, ...), and shall no longer list an anonymous `alias` (the parser rejects one) as a counted kind.
