# simulate — run a state machine's scenarios, or a script, and print the trace

## SYNOPSIS
    syscribe simulate <TestCase|StateDef> [--scenario <title>] [--trace] [--record [--run <id>] [--config <C>]] [--coverage [--min-states <pct>] [--min-transitions <pct>] [--min-guards <pct>]] [--strict] [--json]
    syscribe simulate <StateDef> --do "<script>" [--crosscheck <engine>] [--strict] [--json]

## DESCRIPTION
Lowers the state machine (see `syscribe help compile`) and runs a script on the
interpreter, which follows SCXML run-to-completion with the SysML v2 / KerML
change and time triggers. The clock is virtual: it moves only on `advance`.

Script commands, separated by `;`:

    send <event> [k=v ...]    inject an external event with payload fields
    advance <duration>        move the virtual clock, firing due `after` timers
    set <var>=<value>         write a variable from outside (an environment input)

    syscribe simulate Door --do "send Cmd::Open; advance 3 s; set level=5"

The trace lists every microstep: virtual time, cause (event, timer, change, done,
eventless), transition, states exited and entered, actions run and variables
changed. `--strict` makes an unhandled event and equal-priority conflicts errors.
`--json` prints the machine hash, final configuration, variables and the trace.

## SCENARIOS
A `TestCase` whose `subject:` is a state machine (else the single `StateDef` that satisfies the
requirements it verifies) is run from its Gherkin block. A machine argument runs every scenario
TestCase whose subject it is. The step vocabulary is closed; a step outside it is `E886` and an
`error` verdict, never skipped:

    Given <machine> is in state <state>          (also <region>/<state>, a dotted path)
    Given <variable> is <expression>
    When  <event> is injected [with <field> = <expression>[, ...]]
    When  <variable> becomes <expression>        (an environment change; may arm `when` triggers)
    When  <duration> elapse[s]
    Then  <machine> is in state <state>
    Then  <boolean expression over the variables>

A `TestCase` whose `subject:` is a composite `Part`/`PartDef` (parts that exhibit executable state machines,
joined by `connections:` and `bindingConnections:`) runs all the parts on one virtual clock. Its steps name the
part; `--coverage` reports each part as `<composite>.<part>` plus an aggregate (gates apply to every part):

    Given <part> is in state <state>
    Given <part>.<variable> is <expression>
    When  <part> receives <event> [via <port>] [with <field> = <expression>[, ...]]
    When  <part>.<variable> becomes <expression>
    When  <duration> elapse[s]
    Then  <part> is in state <state>
    Then  <boolean expression over <part>.<variable> names>
    Then  <part> sent <event> [via <port>]
    Then  <part> did not send <event> [via <port>]
    Given|When fault <id> is injected      (an FMEAEntry or FaultTreeEvent with an `injection:`)
    When  fault <id> is cleared            (a part that failed silent cannot be cleared)

`And`/`But` continue the previous keyword. `Scenario Outline` with `Examples:` expands to one
scenario per row, named `<title> [row N]`. A scenario is `pass`, `fail` (a Then did not hold,
with expected and actual) or `error` (a run error, an unknown state, an unusable step).
`Given ... is in state` sets the state directly without running actions.

`--record` writes the verdicts into the results sidecar as session-log evidence (an outline's own
title carries the worst of its rows; verdicts of scenarios this run did not cover are kept) and the
traces under `.syscribe/traces/`; `--run` adds them to the run history, `--config` records them for
a Configuration.

## COVERAGE
`--coverage` (with a machine and `--all`, or a TestCase) reports what the scenarios exercised, merged
per machine: states entered; transitions taken (the uncovered ones listed by trigger and guard);
transition pairs (consecutive transitions through a state); guard and `when` conditions (each atomic
condition seen both true and false, plus a unique-cause MC/DC demonstration for expressions with
several conditions); timers (each `after` both fired and cancelled); and requirements verified by a
TestCase all of whose scenarios passed. The report states its basis: scenarios run, machine hash,
Configuration, tool version. `--min-states`, `--min-transitions` and `--min-guards` (percent) exit 2
when missed.

## OPTIONS
    --scenario <title>   Run one scenario (or all rows of one outline).
    --all                Run every scenario TestCase of the machine (the default for a machine).
    --do "<script>"      Script mode: send / advance / set commands.
    --crosscheck <eng>   Script mode: compare an external SCXML engine (see `syscribe help export-scxml`).
    --trace, --record, --run, --config, --coverage, --min-states, --min-transitions,
    --min-guards, --min-nodes, --min-edges, --strict, --json   as described above.

## ACTIONS
An `ActionDef` is run the same way (see REQ-TRS-ACTEXEC-001). Script mode: `set <name>=<value>`
(an input parameter or variable), `run`, `send <event> [k=v ...]`, `advance <duration>`. Scenario
vocabulary for an action subject: `Given <name> is <expression>`, `When the action runs`,
`When <event> is injected [with ...]`, `When <name> becomes <expression>`, `When <duration> elapses`,
`Then <boolean expression>` over parameters, variables and outputs, `Then <node> ran`,
`Then <node> did not run`, `Then the action completed | is waiting | terminated`. Coverage reports
nodes fired, successions taken, decision branches, loops run zero and at least once, and guard and
condition coverage with MC/DC; gates `--min-nodes`, `--min-edges`, `--min-guards`.

## EXIT STATUS
0 ran, or every scenario passed · 1 usage or bad script · 2 lowering diagnostic, run error or a scenario that did not pass.
