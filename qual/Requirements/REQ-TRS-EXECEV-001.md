---
id: REQ-TRS-EXECEV-001
type: Requirement
name: Tool shall report the executable evidence for each requirement and each architecture element
status: draft
reqDomain: software
verificationMethod: test
---

A TestCase is executable when it is `active`, names a state machine, an executable action or a composite as its `subject:` and has Gherkin scenarios (spec §12.12).

The tool **shall**:

- run the scenarios of every executable TestCase in-process, each verifying its TestCase's `verifies:` requirements narrowed by its `@verifies` tags and `# verifies:` step comments;
- count a scenario as evidence for a requirement only when its subject exercises an architecture element that satisfies it;
- grade each requirement `executable`, `executable-weak` (a failing scenario, or coverage or mutation score below a given threshold), `descriptive-only`, `unverified` or `not-applicable` (`requirement-coverage`);
- show for each software and hardware architecture element whether it is executable itself, modelled by executable parts or not modelled, the scenarios that exercise it and its requirements by verdict (`exec-coverage`);
- read mutation scores recorded by `mutate --record`, marking those whose machine changed since;
- exit 2 when a `--min-executable`, `--min-modelled` or `--require` gate is missed;
- report a `@verifies` claim outside the TestCase's `verifies:` as `W994`, and a `verifies:` requirement that no scenario claims when every scenario is tagged as `W995`.

**Source:** GH #333 and #344, from the grid-tie inverter model (`model_gti/`): its traceability matrix showed a tick for a requirement whether a scenario ran or a bench test was described.

**Acceptance criteria:**

- On a model with a tagged machine TestCase, a descriptive TestCase and an inspection requirement, every verdict and element row is as specified, and the gates exit 2 when missed.
- Each tag defect gives its code.
