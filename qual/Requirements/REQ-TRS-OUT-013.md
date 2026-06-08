---
id: REQ-TRS-OUT-013
type: Requirement
name: Safety-Readiness Audit Dashboard
title: Tool shall provide a read-only safety-readiness audit dashboard with a configurable PASS/FAIL verdict
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** provide a read-only `audit` subcommand
(`syscribe -m <root> audit [--json] [--profile <name>]`) that aggregates existing
model data into a single safety-readiness dashboard plus a configurable PASS/FAIL
verdict. The command **shall reuse** existing validation, coverage and profile
logic and **shall not** reimplement validation or coverage.

This requirement refines the structured-output and exit-code contract of
REQ-TRS-OUT-006 and reuses the named severity profiles of REQ-TRS-OUT-012.

## Dashboard sections

The text report **shall** present, and `--json` **shall** mirror, the following
sections:

1. **Requirement status split** — counts of native `Requirement` elements by
   `status:` (draft / review / approved / implemented / verified), reported
   **overall** and **per top-level package** (the first `::` segment of the
   qualified name / the top-level directory under the model root).
2. **SIL / ASIL distribution** — counts of native `Requirement` elements by
   `silLevel` and by `asilLevel`, including a `QM/none` bucket for requirements
   that declare neither.
3. **Per-configuration coverage %** — `covered` / `gap` / `N-A` per
   `Configuration` and overall, computed by **reusing the `matrix` coverage
   computation** (one definition shared with `matrix`; no duplicated coverage
   logic). When no feature model is present, the flat requirement→TestCase
   coverage view **shall** be shown instead.
4. **Orphans** — counts and ids of:
   - native `Requirement`s with no active verifying `TestCase`;
   - native `Requirement`s that no element `satisfies:`;
   - `TestCase`s whose `verifies:` is empty or resolves to nothing (dangling);
   - `Requirement`s with neither `derivedFrom` nor `derivedChildren`.
5. **Readiness verdict** — a single PASS/FAIL line that names **why** it failed
   (the triggering codes / counts).

## Policy and exit code

The tool **shall** run validation via `validator::validate_with_config` and
compute the verdict as follows. The audit **FAILS** (exit code `2`) when **any**
of the following hold:

- any **Error**-severity finding exists; or
- (default policy) any **W306** finding (the unsatisfied-safety-mechanism gate)
  exists; or
- when `--profile <name>` is given, any finding **promoted** by that
  `[profiles.<name>]` policy in `<model_root>/.syscribe.toml` is present (reusing
  the REQ-TRS-OUT-012 / issue #18 profile loader and promotion logic — not a
  reimplementation).

Otherwise the audit **PASSES** (exit code `0`). An undefined `--profile` name (or
a missing `.syscribe.toml`) is a usage error and **shall** exit `1`.

## JSON output

With `--json`, the whole rollup **shall** be emitted as one document with the
shape:

```json
{
  "statusSplit": { "...": "..." },
  "integrityDistribution": { "...": "..." },
  "coverage": { "...": "..." },
  "orphans": { "...": "..." },
  "verdict": { "pass": true, "reasons": [] }
}
```

**Source:** Issue #15 (safety-readiness dashboard); refines REQ-TRS-OUT-006;
reuses REQ-TRS-OUT-012.

**Acceptance criteria:**

- `syscribe -m <root> audit` prints the five sections and a PASS/FAIL verdict.
- A "ready" model (approved requirements, satisfied and covered, no W306, no
  errors) audits PASS and exits `0`.
- A "not-ready" model (a SIL-4 / ASIL-D requirement that is draft or
  unsatisfied, tripping W306) audits FAIL, names W306 in the verdict, and exits
  `2`.
- `audit --json` emits one document carrying `statusSplit`, `integrityDistribution`,
  `coverage`, `orphans` and `verdict`, and is valid JSON.
- The per-configuration coverage section reuses the `matrix` coverage
  computation, and the existing `matrix` tests continue to pass unchanged.
