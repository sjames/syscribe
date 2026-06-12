---
id: REQ-TRS-MG-013
type: Requirement
title: magicgrid --audit shall roll up the MagicGrid findings, readiness, and a PASS/FAIL verdict
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** provide a MagicGrid **audit** mode on the existing `magicgrid` command —
`magicgrid --audit [--json]` — that runs validation with the MagicGrid gate active and
reports, in one dashboard, the MagicGrid warnings and errors, a readiness summary, and a
PASS/FAIL verdict. Without `--audit`, `magicgrid` is unchanged (the grid report of
[[REQ-TRS-MG-003]]). The audit is read-only; its **verdict mirrors the `validate --profile
magicgrid` gate outcome** ([[REQ-TRS-OUT-012]]) but presents it as a rollup.

### MagicGrid findings (the core)

- The audit **shall** run the gated MagicGrid validation (as if `--profile magicgrid`) and
  collect the **MagicGrid-relevant findings**: the `MG###` family (`MG010`–`MG013`,
  `MG020`–`MG021`, `MG030`–`MG033`, `MG040`–`MG042`, `MG050`–`MG052`, `MG060`–`MG062`,
  `MG070`) plus the base `E316` and `W307`.
- It **shall** print the **error and warning counts**, a **per-code table** (code, count,
  and the category below), and then the **individual error lines and warning lines**
  (each naming the file and message) — i.e. it lists out the warnings and errors.
- Codes **shall** be grouped into categories: **Grid** (`MG020`/`MG021`), **Refines**
  (`E316`/`W307`), **Context** (`MG010`–`MG013`), **SoI** (`MG060`–`MG062`), **MoE**
  (`MG030`–`MG033`), **MoP** (`MG050`–`MG052`), **Layer** (`MG040`–`MG042`), **Variant**
  (`MG070`).

### Readiness summary

- The audit **shall** also report: **grid completeness** (the count of populated vs empty
  cells, reusing the [[REQ-TRS-MG-003]] grid); the **System of Interest** status (present
  and unique, none, or ambiguous); and the counts of **MoEs** (`mg_moe`), **MoPs**
  (`mg_mop`), and trade-study **Configurations**.

### Verdict and output

- The audit **shall** end with a **PASS/FAIL verdict** and exit code: **FAIL (exit 2)** when
  the MagicGrid gate would fail — any MagicGrid error or any promoted `W307`; **PASS
  (exit 0)** otherwise. The verdict line **shall** name the triggering code(s)/count(s).
- `--json` **shall** emit the structured audit: counts, the per-code/per-category rollup,
  the findings, the readiness summary, and the verdict.

**Source:** user feature request — a MagicGrid-profile audit listing the warnings and errors.
Extends the `magicgrid` command of [[REQ-TRS-MG-003]]; reuses the gated validation pass and
the gate semantics of [[REQ-TRS-OUT-012]]; complements the safety-readiness `audit` command.

**Acceptance criteria:**

- On a clean MagicGrid model, `magicgrid --audit` prints zero errors, the readiness summary
  (grid completeness, a unique SoI, MoE/MoP/Configuration counts), and **`Verdict: PASS`**,
  exiting 0.
- On a model with a MagicGrid error (e.g. a `UseCaseDef` with no actor → `MG013`), the audit
  lists that error under its category, the per-code table shows `MG013`, and it prints
  **`Verdict: FAIL`** naming `MG013`, exiting 2.
- `magicgrid` without `--audit` still prints the grid report and does not run the verdict.
- `magicgrid --audit --json` emits an object carrying the counts, per-code rollup, findings,
  readiness, and verdict.
