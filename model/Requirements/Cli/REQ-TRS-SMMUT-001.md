---
type: Requirement
id: REQ-TRS-SMMUT-001
name: "Mutation analysis of a state machine measures whether its scenarios can detect a wrong behaviour"
status: draft
reqDomain: software
reqClass: system
tags:
  - execution
  - state-machine
---

`syscribe mutate <StateDef>` shall judge the quality of a machine's scenarios by seeding faults into the machine and re-running them (GH #275, part of #272). It is the objective check on scenarios written by an LLM, which may only restate the model they were derived from.

## Behavior

- **Mutants**, one fault each, generated deterministically: negate a guard or `when`; replace a relational operator (`<`→`<=`, `>`→`>=`, `==`→`!=`) and move a numeric literal in a guard by one unit of its last digit; retarget a transition to another state of its region; remove a transition; remove an `assign` or an `effect`; change an assignment's right-hand side to its left operand; scale an `after` duration by 0.5 and 2; swap two transitions' document order where it can matter.
- Each mutant is run against all scenarios. It is **killed** if any scenario's verdict changes from pass; **survived** otherwise; **equivalent-suspect** when its trace is identical to the original's on every scenario *and* the mutated element was never covered (reported separately, not counted against the score).
- Report: the mutation score (killed ÷ (killed + survived)), the survivors with the mutation applied, the scenario run that came closest, and the uncovered elements. `--min-score <percent>` exits `2` below the threshold; `--json` for tools.
- The original model files are never modified: mutants are applied to in-memory copies.
