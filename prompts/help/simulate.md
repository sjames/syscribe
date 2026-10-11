# simulate — run a state machine and print its trace

## SYNOPSIS
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

## EXIT STATUS
0 ran · 1 usage or bad script · 2 lowering diagnostic or run error.
