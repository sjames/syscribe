---
type: Requirement
id: REQ-TRS-SYSMLV2-070
name: "Action step fields with no SysML v2 text form travel in a SyscribeStep annotation: trigger and until loops"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-000]
breakdownAdr: Decisions::SysmlV2SubmodelADR
tags:
  - sysmlv2
---

A native AcceptAction `trigger: {kind, condition}` and a `LoopAction` with `loopKind: until` (and its `condition:`) shall export through the same `@SyscribeStep` annotation (`triggerKind`, `triggerCondition`; `loopKind = 'until'`, `condition`), the `until` loop being written as an unconditioned `loop { … }` plus the annotation, and ingestion shall rebuild the identical entries. An entry whose extra fields are not one of the documented set, or whose values are not plain text, still degrades to a comment.
