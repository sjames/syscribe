# mutate — judge the quality of a machine's scenarios by seeding faults

## SYNOPSIS
    syscribe mutate <StateDef|TestCase> [--min-score <percent>] [--json]

## DESCRIPTION
Seeds one fault at a time into the machine's IR (the model files are never touched) and
re-runs the machine's Gherkin scenarios (see `syscribe help simulate`). A scenario set that
cannot tell a mutant from the original is not testing that part of the machine: it may only
restate the model it was derived from, which is how an LLM writes weak tests.

Mutation operators: negate a guard or `when`; replace a relational operator (`<`↔`<=`,
`>`↔`>=`, `==`↔`!=`); move a numeric literal in a condition by one unit of its last digit;
retarget a transition to another state of the target's region; remove a transition; remove
an action (assign or effect, transition or state entry/exit); assign a variable to itself
instead of its expression; scale an `after` delay by 0.5 and 2; swap two transitions of one
source with the same trigger.

A mutant is **killed** when any scenario then fails or errors, **survived** when none does,
and **equivalent-suspect** when its traces are identical to the original's and the mutated
element was never covered (reported separately, not counted). Score = killed / (killed +
survived). Survivors are listed with the mutation, its source location and the scenario that
came closest to killing it. The scenarios must all pass on the unmutated machine.

## EXIT STATUS
0 done (and above `--min-score`) · 1 usage, or no scenarios · 2 below `--min-score`, a
lowering diagnostic, or a scenario failing on the unmutated machine.
