---
type: Requirement
id: REQ-TRS-SYSMLV2-046
name: "A package-level SysMLv2 satisfy ... by subject statement lifts into the subject element's satisfies: when the subject resolves"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-007]
breakdownAdr: Decisions::SysmlV2SubmodelADR
tags:
  - sysmlv2
  - traceability
---

A `satisfy <requirement> by <subject>;` statement in a package body shall append the requirement
reference to the `satisfies:` list of the ingested element the `subject` expression resolves to
(innermost-scope-first from the package, as allocation endpoints resolve). A statement whose subject
does not resolve to an ingested element, the bare `satisfy <requirement>;` shorthand (no subject),
a negated one, and an inline `satisfy requirement ...` stay counted by `W543`, reported per file
as `satisfy` with the unresolved ones counted rather than silently dropped.
