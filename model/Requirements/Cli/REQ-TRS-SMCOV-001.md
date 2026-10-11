---
type: Requirement
id: REQ-TRS-SMCOV-001
name: "Scenario execution measures state, transition, guard-condition and requirement coverage of the model"
status: draft
reqDomain: software
reqClass: system
tags:
  - execution
  - state-machine
---

Running scenarios shall report how much of the machine they exercised (GH #275, part of #272). This replaces the traceability proxy of `behavioral-coverage` with measured coverage for machines that have scenarios.

## Behavior

- `simulate --all --coverage [--json]` reports, over all scenarios run for a machine: **states** entered, **transitions** taken (listing the uncovered ones by trigger and guard), **transition pairs** (consecutive transitions through a state) as an additional measure, **guard conditions** (for each guard or `when` expression, whether each atomic condition was seen both true and false, and, unless the expression has a single condition, an MC/DC-style demonstration that each condition independently decides the outcome), **timers** (each `after` both fired and cancelled), and **requirement coverage** (requirements verified by a TestCase all of whose scenarios pass: a TestCase with a failing scenario verifies nothing).
- A coverage figure is computed per machine and per Configuration (the variables' initial values may differ), and can be gated: `--min-states`, `--min-transitions`, `--min-guards <percent>` exit `2` when missed.
- The report states its basis: the scenarios run, the Configuration, the tool version.
