---
type: Requirement
id: REQ-TRS-SYSMLV2-078
name: "A time or change trigger on an accept is ingested and exported as native syntax"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-000]
breakdownAdr: Decisions::SysmlV2SubmodelADR
tags:
  - sysmlv2
---

A payload-less `accept after <e>;`, `accept when <c>;` and `accept at <e>;` (also `action n accept ...;`) in an action body shall be ingested as an `AcceptAction` entry (name synthesized as `accept_N` when anonymous) with `trigger: {kind: timeOut, when: <e>}`, `{kind: change, condition: <c>}` and `{kind: at, when: <e>}` respectively (`at` is a new native trigger kind, documented in spec 8.7.7), and `export-sysml` shall write those entries as the same statements. An `AcceptAction` that has both a `payload` and a `trigger`, or a trigger kind with no native syntax, keeps the deprecated `@SyscribeStep` annotation; any other shape still degrades to a comment.
