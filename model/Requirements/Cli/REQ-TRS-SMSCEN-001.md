---
type: Requirement
id: REQ-TRS-SMSCEN-001
name: "Executable Gherkin scenarios on a TestCase run against a state machine and become verification evidence"
status: draft
reqDomain: software
reqClass: system
tags:
  - execution
  - state-machine
---

A native `TestCase` whose Gherkin block uses the executable step vocabulary shall be runnable against its state machine, and its verdict and trace shall be recorded as verification evidence (GH #275, part of #272).

## Behavior

- **Subject.** The machine under test is named by the TestCase's `subject:` (a `StateDef` reference), else inferred from the single `StateDef` its `verifies:` requirements are satisfied by.
- **Closed step vocabulary** (case-insensitive keywords, `And`/`But` continue the previous keyword): `Given <machine> is in state <state>` (also `<region>/<state>` and a dotted path); `Given <variable> is <expression>` and `When <variable> becomes <expression>` (assignment from outside; `becomes` is an environment change and may arm change triggers); `When <event> is injected [with <field> = <expression>[, …]]`; `When <duration> elapse[s]`; `Then <machine> is in state <state>`; `Then <expression>` (a boolean over variables); `Then <event or action> was not … ` is not part of the vocabulary. A step outside the vocabulary is a finding on the TestCase (`E886`), never a silent skip.
- **Scenario Outline** with `Examples:` tables expands to one scenario per row (`<column>` substituted before parsing); each expanded scenario has its own verdict, named `<scenario title> [row N]`.
- **Verdict** per scenario: `pass`, `fail` (a `Then` did not hold, with expected vs actual), `error` (a run error of REQ-TRS-SMEXEC-001, or a step that cannot be applied, e.g. an unknown state). Never `unknown`.
- **CLI.** `syscribe simulate <TestCase|StateDef> [--scenario <title>] [--trace] [--json] [--strict]` runs one TestCase (or all scenario TestCases of a machine) and exits `0` when all pass, `2` otherwise, `1` on a usage error. `--config <C>` projects the model and applies the Configuration's parameters to variable initial values written as placeholders.
- **Evidence.** `simulate --record [--run <id>] [--config <C>]` writes the verdicts into the results sidecar in the session-log shape — `(TestCase, scenario)` keys, per-configuration and in run history exactly like `ingest-results` — so `trace`, `matrix`, `audit` and the safety case treat a passing scenario TestCase as executed evidence. The trace is kept beside it (`.syscribe/traces/`), referenced from `results failures`.
