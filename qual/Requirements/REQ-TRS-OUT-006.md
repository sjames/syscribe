---
id: REQ-TRS-OUT-006
type: Requirement
name: Tool shall support configurable warning gating with a documented exit-code contract
status: draft
reqDomain: software
verificationMethod: test
---

The `validate` subcommand **shall** support configurable severity gating so the tool can be used as a CI gate, and **shall** honour the following exit-code contract:

| Exit code | Meaning |
|---|---|
| `0` | No `Error`-severity findings and no gate failure |
| `1` | One or more `Error`-severity findings present |
| `2` | One or more `Warning`-severity findings tripped a configured gate |

The tool **shall** provide the following gating options on `validate`:

- `--deny <CODES>` — treat each listed warning code (comma-separated) as a gate failure.
- `--max-warnings <N>` — fail when the total number of warnings exceeds `N`.
- `--warnings-as-errors` — treat every warning as a gate failure.

`Error`-severity findings **shall** dominate: a model containing any error exits `1` regardless of gating flags. When no gating flag is supplied, warning-only models **shall** continue to exit `0` (preserving REQ-TRS-OUT-005).

**Source:** Issue #3 (CI severity-gating flags + documented exit codes); §11.12

**Acceptance criteria:** `validate --deny <W>` exits `2` when any `<W>` finding is present and `0` otherwise; `--max-warnings 0` exits `2` when warnings exist; `--warnings-as-errors` exits `2` when warnings exist; a model with an error exits `1` even when gating flags are set.
