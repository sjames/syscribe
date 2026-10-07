---
id: REQ-TRS-SYSMLV2-070
type: Requirement
name: "Action step fields with no SysML v2 text form travel in a SyscribeStep annotation: trigger and until loops"
status: verified
reqDomain: software
verificationMethod: test
---

A native AcceptAction `trigger: {kind, condition}` and a `LoopAction` with `loopKind: until` (and its `condition:`) shall export through the same `@SyscribeStep` annotation (`triggerKind`, `triggerCondition`; `loopKind = 'until'`, `condition`), the `until` loop being written as an unconditioned `loop { … }` plus the annotation, and ingestion shall rebuild the identical entries. An entry whose extra fields are not one of the documented set, or whose values are not plain text, still degrades to a comment.

**Source:** `REQ-TRS-SYSMLV2-070` (product model), `ADR-SYS-SYSMLV2-002`.
