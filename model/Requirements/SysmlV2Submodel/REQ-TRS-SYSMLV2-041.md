---
type: Requirement
id: REQ-TRS-SYSMLV2-041
name: "The SysML v2 export is deterministic and its output re-parses through the SysML v2 ingestion with the supported kinds and qualified names intact"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-000]
breakdownAdr: Decisions::SysmlV2ExportADR
tags:
  - sysmlv2
  - export
  - roundtrip
---

Exporting the same model twice shall yield byte-identical text (elements ordered by qualified name,
no timestamps or host-specific data). Parsing the exported text with the existing `sysml-v2-parser`
ingestion (as a `sysmlSubmodel: true` package) shall succeed with no `W541`, and for the supported set
every exported element shall come back with the same qualified name (relative to the importing
package) and element kind, the single documented exception being native `Requirement`, which comes back
as `RequirementDef`. Exporting `examples/sysmlv2-submodel/` and a native-only fixture shall satisfy this.
