# export-scxml — write a state machine as W3C SCXML

## SYNOPSIS
    syscribe export-scxml <StateDef> [--output <file>]

## DESCRIPTION
Lowers the machine (see `syscribe help compile`) and writes an SCXML document equivalent to it, so
independent engines and tools can run or check it. Substates map to `<state>`, `<parallel>` and
`<final>`; variables to a `<datamodel>` of SI base-unit numbers; guards and assignments to
ECMAScript (units flattened); `after` to `<send delay>` in `<onentry>` with `<cancel>` in
`<onexit>`; `done` to `done.state.<id>`; an internal transition to a targetless one.

A `when` change trigger is a SysML v2 one-shot observation, while SCXML eventless transitions are
level-triggered, so the export arms an `armed_<id>` flag on entry and raises a `signal.<id>` event
when the condition holds; the real transition is taken on that signal.

Constructs with no SCXML form (behaviour references, `doAction`, attribute paths) are written as XML
comments and listed as `note:` lines on stderr; the export never silently changes behaviour.

Cross-check with an external engine: `syscribe simulate <StateDef> --do "<script>" --crosscheck <engine>`.
`<engine>` is a name configured as `[scxml.engines.<name>] command = "…"` in `.syscribe.toml`, or the
command itself. The command is run with `sh -c`, reads `{"scxml": "<doc>", "commands": [{"event",
"data"} | {"advance": seconds} | {"set": {var: value}}]}` on stdin and prints `{"configurations":
[[state ids], …]}` — the active states after start and after each command. Any difference from
syscribe's own trace is reported (exit 2). No engine is bundled.

## EXIT STATUS
0 written · 1 usage · 2 lowering diagnostics.
