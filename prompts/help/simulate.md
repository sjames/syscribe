# simulate — run a state machine's scenarios, or a script, and print the trace

## SYNOPSIS
    syscribe simulate <TestCase|StateDef> [--scenario <title>] [--trace] [--record [--run <id>] [--config <C>]] [--strict] [--json]
    syscribe simulate <StateDef> --do "<script>" [--strict] [--json]

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

`And`/`But` continue the previous keyword. `Scenario Outline` with `Examples:` expands to one
scenario per row, named `<title> [row N]`. A scenario is `pass`, `fail` (a Then did not hold,
with expected and actual) or `error` (a run error, an unknown state, an unusable step).
`Given ... is in state` sets the state directly without running actions.

`--record` writes the verdicts into the results sidecar as session-log evidence (an outline's own
title carries the worst of its rows; verdicts of scenarios this run did not cover are kept) and the
traces under `.syscribe/traces/`; `--run` adds them to the run history, `--config` records them for
a Configuration.

## EXIT STATUS
0 ran, or every scenario passed · 1 usage or bad script · 2 lowering diagnostic, run error or a scenario that did not pass.
