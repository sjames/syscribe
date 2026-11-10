# mutate — judge the quality of a machine's scenarios by seeding faults

## SYNOPSIS
    syscribe mutate <StateDef|TestCase|composite> [--part <name>] [--min-score <percent>] [--record] [--config <C>] [--json]

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

For an executable `ActionDef` the operators act on the action graph: negate a guard or
condition, change a relational operator or literal, remove or retarget a succession, swap the
targets of two decision branches, remove an assignment or assign a variable to itself, make a
node complete without effect.

For a composite `Part`/`PartDef` (or a TestCase whose subject is one), the machine of each part is
mutated in turn and every mutant is judged by running all the composite scenarios with that one part
replaced; plant parts get fresh plants for every run. The report has one block per part
(`<composite>.<part> (<machine>)`) and an aggregate score; `--part <name>` restricts the analysis to one
part and `--min-score` is checked for every part, naming the one below it.

`--record` keeps each machine's score, with the hash of the machine it was computed on, in
`<model>/.syscribe/mutation.json` (a record replaces the earlier one of the same machine, subject and hash; the newest
100 per machine and subject are kept, so a product line with many Configurations keeps one per Configuration, and an
older one reads as stale). A `mutation.json` that cannot be read is never overwritten: `--record` exits 1 saying to
fix or remove it, and `requirement-coverage` and `exec-coverage` report it in their errors. They read the file; a score whose
machine has since changed is shown as stale. A part none of whose
mutants was judged is not recorded.

## OPTIONS
    --part <name>        composite only: mutate just this part
    --min-score <pct>    gate: exit 2 when the score (of any part, for a composite) is below it
    --config <C>         run on the model projected onto this Configuration: feature parameters written as {{Features::F.param}} in guards, assignments and durations take its values
    --record             keep the score(s) in .syscribe/mutation.json for requirement-coverage and exec-coverage
    --json               machine-readable report

## EXIT STATUS
0 done (and above `--min-score`) · 1 usage, no scenarios, or the scores could not be recorded (an unreadable
`mutation.json` is never overwritten) · 2 below `--min-score`, a
lowering diagnostic, or a scenario failing on the unmutated machine.
