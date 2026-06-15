---
type: Requirement
id: REQ-TRS-BUILD-002
name: "FeatureDef parameter entries shall support an optional buildVar field"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-BUILD-001]
breakdownAdr: Decisions::BuildExportADR
tags:
  - schema
  - build-integration
---

Each entry in a `FeatureDef`'s `parameters:` list shall support an optional `buildVar:`
field whose value is the name of the build variable that carries the parameter's resolved
value in generated output.

When `buildVar:` is present and the parameter has a bound value (via the active
`Configuration`'s `parameterBindings:`), the resolver shall emit `<buildVar> = <bound value>`
in the output. When the parameter has no bound value and declares a `default:`, the
default value shall be used. When neither binding nor default is present, the variable
shall be omitted.

`buildVar:` is optional; omitting it leaves the parameter outside the build variable
set without affecting model validity.
