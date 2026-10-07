---
type: Requirement
id: REQ-TRS-SYSMLV2-069
name: "Action step fields with no SysML v2 text form travel in a SyscribeStep annotation: via, referent, valueKind"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-000]
breakdownAdr: Decisions::SysmlV2SubmodelADR
tags:
  - sysmlv2
---

The pinned 0.54 parser has no `via` on an action `accept`/`send` step and no `referent`/`valueKind` on an `assign`. A native `subActions:` entry carrying `via:` (AcceptAction/SendAction) or `referent:`/`valueKind:` (AssignmentAction) shall export with a `@SyscribeStep { via = '…'; referent = '…'; valueKind = '…'; }` metadata annotation inside the step's own body, and ingestion shall read that annotation back into the same entry fields. The annotation is a Syscribe extension, ignored by other SysML v2 tools, in the same family as the existing `@Syscribe*` annotations.
