---
id: REQ-TRS-OUT-001
type: Requirement
name: Tool shall output a validation report in Markdown format to stdout
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** write its validation report to standard output (`stdout`) in Markdown format when invoked with a model directory. The report **shall** be machine-parseable and human-readable.

## Report title

The report's first line **shall** be a level-1 heading naming the model: `# <Name> Validation
Report`, where `<Name>` is the `name:` of the model root package (the root `_index.md`).
When the model has no root `_index.md`, or it carries no non-empty `name:`, the heading
**shall** be the neutral `# Model Validation Report`. The title **shall never** name a
particular bundled example (such as "UAV") for a model that is not that example. (GH #174.)

**Source:** §11.7; report title GH #174.

**Acceptance criteria:** Redirecting stdout to a file produces a valid Markdown document. Stderr is not mixed into the report. A model whose root `_index.md` has `name: ValidModel` produces the first line `# ValidModel Validation Report`; a model with no root `_index.md` produces `# Model Validation Report`; neither mentions "UAV".
