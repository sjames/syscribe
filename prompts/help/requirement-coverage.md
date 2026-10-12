# requirement-coverage — which requirements have executable evidence, and how strong it is

## SYNOPSIS
    syscribe requirement-coverage [<REQ>...] [--config <C>] [--min-executable <percent>] [--require <REQ>]... [--min-states <percent>] [--min-transitions <percent>] [--min-guards <percent>] [--min-score <percent>] [--mutate] [--json]

## DESCRIPTION
Runs, in-process, the Gherkin scenarios of every `active` TestCase whose `subject:` is a state machine, an
executable action or a composite part (see `syscribe help simulate`), and grades each requirement by the evidence:

* **executable**: a passing scenario verifies it and its subject exercises an architecture element that satisfies it
  (the check `W992` makes), above every threshold given;
* **executable-weak**: such scenarios exist, but one fails, or the coverage they give of the machines that carry the
  requirement, or those machines' mutation score, is below a threshold;
* **descriptive-only**: only TestCases with no executable subject or no scenarios verify it;
* **unverified**: nothing verifies it;
* **not-applicable**: its `verificationMethod:` is `inspection` or `analysis`, or it is a parent requirement all of
  whose children are executable or not applicable, and no executable scenario verifies it.

A scenario verifies the requirements of its TestCase's `verifies:`. When it is tagged `@verifies REQ-A,REQ-B` (or
`@verifies(REQ-A)`; a tag before `Feature:` applies to every scenario), or has a `# verifies: REQ-A` comment line
before a step, it verifies just those of them. A scenario whose claims are all invalid verifies every requirement of its
TestCase. A scenario whose subject exercises nothing that satisfies the requirement is listed as *claimed but not
exercised* and is not evidence.

Coverage is the states, transitions and guard conditions (MC/DC) of the machines that carry the requirement (the
machine of a machine subject; the parts of a composite whose definition satisfies the requirement or models an
element that does), merged over the scenarios that verify it. Environment parts (`role: environment`) never count.
The mutation score is the one `mutate <subject> --record` kept for the machine and subject (shown as stale when the
machine changed since), or with `--mutate` it is computed now over all the subject's scenarios; a machine used by
several parts has one score summed over them. A requirement with no score is weak only when `--min-score` is given,
and one with no state-machine coverage (an action subject) is weak under any threshold.

Without arguments every requirement is listed, with the reasons for each that is not executable; with requirement ids,
those are shown in detail (their scenarios, coverage, mutation scores, satisfiers and descriptive tests). Scenarios that
use plants are not run here and count as not run.

## OPTIONS
    --config <C>             evaluate the model projected onto this Configuration
    --min-executable <pct>   gate: exit 2 when fewer of the applicable requirements are executable
    --require <REQ>          gate: exit 2 unless this requirement is executable (repeatable)
    --min-states <pct>       threshold: states covered of each carrying machine
    --min-transitions <pct>  threshold: transitions covered
    --min-guards <pct>       threshold: guard conditions covered
    --min-score <pct>        threshold: mutation score of each carrying machine
    --mutate                 compute mutation scores now instead of reading the recorded ones
    --json                   machine-readable report

## EXIT STATUS
0 done (and every gate met) · 1 usage, an unknown requirement or Configuration · 2 a gate missed.
