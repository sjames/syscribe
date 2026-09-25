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

### Gating in the configuration lens (issue #126)

The gating options (`--deny`, `--max-warnings`, `--warnings-as-errors`, `--profile`) **shall** apply identically under the configuration lens:

- `validate --config <C>` **shall** evaluate the gate (including a selected `--profile`) over the projected variant's findings, and **shall** honour `--file <path>` by reporting and gating only the findings whose file matches, exactly as whole-model `validate` does.
- `validate --all-configs` **shall** evaluate the gate **per variant** (each stored `Configuration`'s findings are gated on their own, so `--max-warnings N` is a per-variant budget) and exit with the **worst** per-variant code under the order `1` > `2` > `0`: `1` if any variant has an `Error` finding, else `2` if any variant tripped a gate, else `0`.

### Usage errors

A `validate` invocation that cannot run as written — an undefined `--profile` name, a malformed `--max-warnings` value, or a `--config` argument that does not resolve ([[REQ-TRS-PROJ-001]]) — is a **usage error**: it **shall** print a message to stderr, print nothing to stdout, and exit `1`. Exit `2` **shall** be reserved exclusively for a tripped warning gate, so a CI job can always read `2` as "the model is valid but a gate failed".

**Source:** Issue #3 (CI severity-gating flags + documented exit codes); §11.12

**Acceptance criteria:** `validate --deny <W>` exits `2` when any `<W>` finding is present and `0` otherwise; `--max-warnings 0` exits `2` when warnings exist; `--warnings-as-errors` exits `2` when warnings exist; a model with an error exits `1` even when gating flags are set; `validate --config C --warnings-as-errors` exits `2` on a variant with warnings; `validate --all-configs` with any gating flag exits `2` when a variant trips it and `1` when any variant has an error; a scoped `--profile` trips only in the variant whose element matches its scope; `validate --config C --file <path>` reports only that file's findings; `validate --config <unresolvable>` and `validate --profile <undefined>` exit `1` with a stderr message and empty stdout.
