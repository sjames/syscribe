---
type: Requirement
id: REQ-TRS-SYSMLV2-039
name: "Every name and reference in the SysML v2 export is a valid SysML identifier, quoted with single quotes when it is not a basic name or is a reserved word"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-000]
breakdownAdr: Decisions::SysmlV2ExportADR
tags:
  - sysmlv2
  - export
  - naming
---

Every emitted declaration name and every `::`-separated segment of an emitted reference shall be a
valid SysML v2 identifier: a basic name (`^[A-Za-z_][A-Za-z0-9_]*$`) that is not a reserved word is
emitted bare; anything else (stable ids such as `REQ-TRS-001`, names with spaces or hyphens, reserved
words such as `part`) is emitted quoted with single quotes, escaping `\` and `'`. A reference that is a
stable id of an exported element renders as that element's qualified name; any other reference renders
verbatim, segment by segment.
