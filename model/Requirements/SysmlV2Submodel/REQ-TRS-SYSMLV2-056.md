---
type: Requirement
id: REQ-TRS-SYSMLV2-056
name: "export-sysml emits action def/action bodies that ingestion reads back: sub-actions, control nodes, successions and control-flow constructs"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-000]
breakdownAdr: Decisions::SysmlV2ExportADR
tags:
  - sysmlv2
  - export
---

`ActionDef`/`Action` elements shall export the native `subActions:` tree as SysML text that REQ-TRS-SYSMLV2-019's ingestion maps back to an equal tree: `PerformAction` as `action n [: T];`, `AcceptAction`/`SendAction` as `accept|send n [: payload];`, `AssignmentAction` as `assign target := value;`, `LoopAction` as `while c {…}`/`loop {…}`/`for v in s {…}`, `IfAction` as `if c {…} else {…}` and `TerminateAction` as `terminate [target];`; `controlNodes:` as `fork|join|decide|merge n;` and `successionConnections:` as `first a then b;`. Parse-back round-trip tests shall cover each construct.
