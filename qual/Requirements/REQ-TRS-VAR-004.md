---
id: REQ-TRS-VAR-004
type: Requirement
title: Tool shall provide a feature-model-driven Requirement x Configuration matrix command
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** provide a `matrix` command that emits a grid whose **rows** are requirements and whose **columns** are the model's `Configuration` elements (never free-text strings).

Each cell (R, C) **shall** be classified as:

| Cell | Condition |
|---|---|
| `—` N/A | R's `appliesWhen:` is **not** satisfied by C's selections (R does not exist in this variant) |
| `✓` covered | R is active in C **and** some `TestCase` T exists with T's `appliesWhen:` satisfied by C **and** `R ∈ T.verifies` |
| `✗` gap | R is active in C but no such `TestCase` exists |

A `Configuration` with no covering tests **shall** yield an empty column; a feature with no `Configuration` simply has no column (so untested ports are a visible model fact).

The command **shall** support `--json` (reusing the `export` schema style and carrying a `schemaVersion`) and a `--tag` filter on rows (see [[REQ-TRS-TAG-001]]). With no feature model linked, it **shall** print a notice and fall back to a flat requirement↔testcase view without error (see [[REQ-TRS-VAR-001]]).

**Source:** Issue #9

**Acceptance criteria:** Columns are exactly the model's `Configuration` elements and rows the requirements; cells correctly classify N/A vs covered vs gap from `appliesWhen:` + selections + `verifies:`; `--json` emits documented structured output carrying `schemaVersion`; a dormant model produces a graceful flat fallback with no error.
