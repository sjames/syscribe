---
type: Requirement
id: REQ-TRS-SMSCXML-001
name: "A state machine exports to SCXML so an independent engine can replay its scenarios"
status: draft
reqDomain: software
reqClass: system
tags:
  - execution
  - state-machine
  - export
---

`syscribe export-scxml <StateDef>` shall write a W3C SCXML document equivalent to the machine, and `simulate --crosscheck <engine>` shall compare an independent SCXML engine's trace with ours (GH #275, part of #272).

## Behavior

- Mapping: substates → `<state>` (`isInitial` → `initial=`; `isFinal` → `<final>`; `isParallel` → `<parallel>`); `variables` → `<datamodel>` with SI-base numbers; `accept` payload → `event=` (`<port>.<payload>` with dots); `after` → `<send event="timer.<id>" delay="…ms"/>` in `<onentry>` with `<cancel>` in `<onexit>` and a transition on that event; `when` → SCXML eventless transitions are level-triggered, so the SysML v2 one-shot observation is reproduced with an `armed_<id>` flag set in `<onentry>`: an eventless targetless transition `cond="armed_<id> && (<when>)"` clears the flag and `<raise>`s `signal.<id>`, and the real transition is taken on that event with the guard as its `cond` (so a failing guard still consumes the signal); `done` → `done.state.<id>`; `internal: true` → `type="internal"` transition without target; `assign` → `<assign>`; guards and assignments → ECMAScript over SI numbers (units flattened, `:=` → `=`, `and/or/not` → `&&/||/!`).
- Constructs with no SCXML form (`doAction` running concurrently, typed payload dimensions) are flagged in the document as comments and in the command output; the export never silently changes behaviour.
- `--crosscheck <command>` runs the SCXML document with an external engine under a documented driver contract (event list in, configuration trace out) and reports any step where the active-state sequence differs from ours. Engines are configured in `.syscribe.toml`; none is bundled.
