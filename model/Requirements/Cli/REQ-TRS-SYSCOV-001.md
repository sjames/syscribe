---
type: Requirement
id: REQ-TRS-SYSCOV-001
name: "Coverage of the scenarios of a composite is reported per part and in aggregate"
status: draft
reqDomain: software
reqClass: system
tags:
  - execution
  - composition
  - coverage
---

`simulate <TestCase|composite> --coverage` shall work for a composite subject (GH #299, part of #297/#272). Each part that has a state machine gets the report of REQ-TRS-SMCOV-001 (states, transitions, pairs, guard conditions with MC/DC, timers, requirements) computed from what that part's run exercised, merged over the scenarios that ran; the report is labelled `<composite>.<part>`. The output also carries an aggregate of states, transitions and guard conditions over all parts. `--min-states`, `--min-transitions` and `--min-guards` are gates applied to every part: a part below a threshold fails the gate (exit 2) and is named. Coverage is independent of whether the scenarios pass, as for a single machine.

## Behavior

- **Per part.** Coverage is kept per part instance, not per machine definition: two parts of one definition are reported separately.
- **JSON.** `coverage` is `{composite, hash, parts: [<REQ-TRS-SMCOV-001 report> + part], aggregate: {states, transitions, guards: {covered, total}}}`.
- **Text.** One block per part (header names composite, part and machine), then one aggregate line.
- **Requirements.** A requirement is covered in a part's report when every scenario of one verifying TestCase passed, as for a single machine.
- **Not in v1:** coverage of plant parts (a plant has no machine), coverage of messages (which connections carried traffic).
