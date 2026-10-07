---
id: REQ-TRS-SYSMLV2-057
type: Requirement
name: "export-sysml emits state def/state bodies that ingestion reads back: entry/do/exit, substates, initial/final markers and transitions with accept, guard and effect"
status: verified
reqDomain: software
verificationMethod: test
---

`StateDef`/`State` elements shall export `entryAction:`/`doAction:`/`exitAction:` as `entry action n;`/`do action n;`/`exit action n;`, `subStates:` as nested `state n [: T] {…}` (recursively), `isInitial`/`isFinal` as `then n;`/`final n;` siblings and `transitions:` as `transition first s accept a [if g] [do effect] then t;` (inside a substate the source is implicit and `first` is omitted), so REQ-TRS-SYSMLV2-018's ingestion maps the text back to equal `subStates:`/`transitions:`. Parse-back round-trip tests shall cover each construct.

**Source:** `REQ-TRS-SYSMLV2-057` (product model), `ADR-SYS-SYSMLV2-002`.
