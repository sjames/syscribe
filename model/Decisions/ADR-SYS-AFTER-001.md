---
type: ADR
id: ADR-SYS-AFTER-001
name: "A timer's duration is an expression of the machine's variables, read when the timer is armed"
status: accepted
tags:
  - state-machine
  - execution
---

## Context

GH #511: `accept: {after: 2 s}` must be a literal. `ir.rs` evaluates it once, at lowering, in an empty
environment, so a variable is an `E882` ("does not evaluate"). A machine whose deadline comes from configuration
(a per-component prepare deadline, a value set by an earlier event) cannot say so. The reporter models each such
deadline as an event injected by the scenario, which removes the timing from the machine and from every check that
reasons about it.

Everything else a machine computes is already an expression over its variables, typed and dimension-checked
(`expr`, `typecheck`, REQ-TRS-STYPE-001). `after:` is the one place that is not.

## Decision

1. **`after:` takes an expression of dimension time.** It is written in the existing expression language and may
   name the machine's variables: `after: prepareDeadline`, `after: base + 2 * margin`. A literal stays a literal.
2. **Read when the timer is armed.** The expression is evaluated each time the timer is armed: on entry to the
   source state, and when an internal transition re-arms it. The result fixes that timer's due time. A later change
   to a variable does not move an armed timer; leaving the state cancels it as today (a timer is never re-timed in
   place). This is the SCXML `<send delayexpr>` semantics.
3. **Typing is static, the value is dynamic.** A variable or expression that is not of dimension time is `E935`,
   like any other dimension error. A literal or constant expression is still checked at lowering for being a
   non-negative duration (`E882`). A negative value found at run time stops the run with the same error as any other
   failed expression; zero fires at once. No new validation code.
4. **No event payload in the expression.** A timer belongs to a state, not to an event, so a payload field is not in
   scope. A deadline that arrives in an event is stored first (`assign: "deadline := d"` on the accepting
   transition) and read by the timer of the state that follows.
5. **IR.** `Trigger::Timer` carries either the constant (`delaySeconds`, unchanged, so the hash of every machine with
   literal timers is unchanged) or an `IrExpr` (`delay`). The two are mutually exclusive.
6. **Every consumer treats an expression timer like a literal one where it can, and says so where it cannot.**
   - `simulate`, co-simulation and `Given … is in state` set-up arm it through the same evaluation.
   - `mutate`'s `scale-after` multiplies the expression by 0.5 and 2.
   - Coverage labels it `after <expression>`.
   - SCXML export emits `delayexpr` with the translation already used for guards; a delay that cannot be translated
     is a finding in the export report, like other constructs SCXML cannot carry.
   - An `AcceptAction` with `after:` inside an `ActionDef` follows the same rule over the action's variables and
     parameters (a later slice; the grammar is identical).

## Alternatives considered

- **Keep literals and add a `timer` variable assigned by `assign:`.** Moves the timing out of the transition's
  trigger, so coverage, mutation and the SCXML export would not see it as a timer.
- **A structured form (`after: {variable: x}`).** A second grammar for something the expression language already
  states, and no way to write `base + margin`.
- **Variables only, no expressions.** The reporter's own examples need arithmetic, and the type checker already
  handles expressions; restricting to a name saves nothing.
- **Re-time an armed timer when its variable changes.** Makes a run depend on assignment order within a macrostep and
  has no SCXML counterpart; the owner of a deadline that must move can leave and re-enter the state.

## Consequences

- Timing that depends on configuration or earlier events stays in the machine, so `simulate`, `mutate --coverage`
  and the SCXML cross-check all see it.
- A run's timer due times are no longer known before it starts: `next_timer` reports only armed timers, which it
  already does.
- Static timing analysis that assumed constant delays (none today) would have to bound the variable.
- Follow-ups, each its own slice: REQ/TC for the machine form; the `ActionDef` form; SCXML `delayexpr`; the
  `after:` row of the executable-machine guide and the `E882`/`E935` catalogue text.
