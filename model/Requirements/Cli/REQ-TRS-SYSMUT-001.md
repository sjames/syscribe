---
type: Requirement
id: REQ-TRS-SYSMUT-001
name: "The scenarios of a composite can be judged by mutation analysis of each part"
status: draft
reqDomain: software
reqClass: system
tags:
  - execution
  - composition
  - mutation
---

`mutate <composite|TestCase>` shall work for a composite subject (GH #299, part of #297/#272): the machine of each part is mutated one fault at a time with the operators of REQ-TRS-SMMUT-001, and every mutant is judged by running all the composite's scenarios with that one part's machine replaced. A mutant is killed when a scenario fails, survives when none does and the scenarios behaved differently from the baseline, and is an equivalent suspect when the mutated element was never exercised. The scenarios must pass on the unmutated composite, else the analysis stops with an error.

## Behavior

- **Per part.** The report has one block per part with a machine (label `<composite>.<part> (<machine>)`), each with its own score, killed, survived and equivalent-suspect counts, mutants and uncovered states; and an aggregate score over all parts. `--part <name>` restricts the analysis to one part.
- **Gate.** `--min-score <pct>` is checked for every part; a part below it is named and the exit code is 2.
- **Plants.** A composite with FMU plant parts is supported: every mutant run instantiates fresh plants. In a build without the `fmi` feature such a composite is refused (exit 1) with the reason.
- **Output.** Text lists the survivors and equivalent suspects per part; `--json` prints `{composite, hash, scenarios, score, killed, survived, equivalentSuspect, parts: [...]}`.
- **Determinism.** Mutants are generated and run in a fixed order; the same model and scenarios give the same report.
- **Operators.** Those of REQ-TRS-SMMUT-001, including removing an action, which covers `send` effects (a part whose message is never sent is a mutant the receiving part's scenarios must catch).
- **Gate honesty.** With `--min-score`, a part for which no mutant could be judged (every mutant is an equivalent suspect because the scenarios never exercise the part) fails the gate and is named. `--part` naming no part with a machine, and a target that is not an executable composite, are usage errors (exit 1); exit 2 is a failed gate or a failing baseline.
- **Budget.** A run sends at most 250,000 messages; a mutant that exceeds it ends in an error and counts as killed.
- **Not in v1:** mutation of the wiring (connections, bindings, delays) and of plant parameters.
