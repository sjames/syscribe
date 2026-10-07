---
type: Requirement
id: REQ-TRS-SYSMLV2-057
name: "export-sysml emits state def/state bodies that ingestion reads back: entry/do/exit, substates, initial/final markers and transitions with accept, guard and effect"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-000]
breakdownAdr: Decisions::SysmlV2ExportADR
tags:
  - sysmlv2
  - export
---

`StateDef`/`State` elements shall export `entryAction:`/`doAction:`/`exitAction:` as `entry action n;`/`do action n;`/`exit action n;`, `subStates:` as nested `state n [: T] {…}` (recursively), `isInitial`/`isFinal` as `then n;`/`final n;` siblings and `transitions:` as `transition first s accept a [if g] [do effect] then t;` (inside a substate the source is implicit and `first` is omitted), so REQ-TRS-SYSMLV2-018's ingestion maps the text back to equal `subStates:`/`transitions:`. Parse-back round-trip tests shall cover each construct.
