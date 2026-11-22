---
type: Requirement
id: REQ-TRS-AFTER-001
name: "A timer's duration is an expression of the machine's variables, read when the timer is armed"
status: implemented
reqDomain: software
reqClass: system
tags:
  - state-machine
  - execution
---

`accept: {after: <duration>}` shall take any expression of dimension time over the machine's variables, not only a literal (GH #511, `ADR-SYS-AFTER-001`).

- **When it is read.** The expression is evaluated each time the timer is armed: on entry to the source state, and when an internal transition re-arms it. The result fixes that timer's due time. A later change to a variable does not move an armed timer; leaving the state cancels it.
- **Static checks.** A constant is still checked at lowering for being a non-negative duration (`E882`). A name that is not a declared variable is `W933`; an expression that is not of dimension time is `E935`. A negative value found at run time stops the run with the expression named; zero fires at once.
- **Scope.** An event's payload fields are not in scope; a deadline that arrives in an event is stored in a variable on the accepting transition first.
- **IR.** A machine whose timers are all literals keeps its IR and hash; an expression timer adds a `delay` expression to the trigger.
- **Consumers.** `simulate` and co-simulation arm it through the same evaluation; `mutate`'s `scale-after` multiplies the expression by 0.5 and 2; coverage labels it `after <expression>`; the SCXML export writes `delayexpr` in milliseconds.
- **Actions.** An `AcceptAction` with `after:` in an `ActionDef` follows the same rule over the action's parameters and variables, read when the accept starts waiting.
