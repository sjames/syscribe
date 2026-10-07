---
type: Requirement
id: REQ-TRS-SYSMLV2-062
name: "Export never emits a succession whose endpoint entry was not exported"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-000]
breakdownAdr: Decisions::SysmlV2SubmodelADR
tags:
  - sysmlv2
---

A \`successionConnections:\` entry whose \`after\` or \`before\` names a \`subActions:\`/\`controlNodes:\` entry of the same body that was written as a comment shall itself be written as a comment naming the missing endpoint, never as \`first a then b;\` referring to an absent step. Successions whose endpoints are not entries of the body (parameters, nested or foreign names) are unaffected.
