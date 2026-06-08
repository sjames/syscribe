---
id: REQ-TRS-OUT-011
type: Requirement
name: Verification-Depth and Independence Report
title: Tool shall report per-requirement verification-level depth and gate single-method high-integrity coverage
status: draft
reqDomain: software
verificationMethod: test
---

Diversity / independence of verification is a core high-integrity (SIL-4 / ASIL-D) expectation, but today it is only visible by running `trace` on one requirement at a time. The tool **shall** provide a fleet-wide verification-depth report and a gate.

## `verification-depth` command

`syscribe -m <root> verification-depth [--sil <v>] [--status <s>] [--min-levels N] [--json]`

- For each native `Requirement`, the report **shall** show the set of **distinct verification levels** (the `testLevel` values: `L1`–`L5`) among the **active** TestCases that verify it, the count of distinct levels, and a **depth flag**:
  - `none` — no active TestCase verifies it (zero verification);
  - `hil-only` — the only level is `L5` (hardware-in-the-loop only);
  - `single` — exactly one distinct level (and not `hil-only`);
  - `ok` — two or more distinct levels.
- `--sil <v>` **shall** restrict the report to requirements whose `silLevel` stringifies to `<v>` or whose `asilLevel` equals `<v>` (same semantics as `list --sil`); `--status <s>` restricts by lifecycle status.
- `--json` **shall** emit a machine-readable array (`id`, `silLevel`, `asilLevel`, `levels`, `count`, `flag`).
- `--min-levels N` **shall** turn the report into a **gate**: the tool exits non-zero when any reported requirement has fewer than `N` distinct verification levels. Combined with `--sil <v>` it gates only that integrity tier (e.g. `verification-depth --sil 4 --min-levels 2`).

**Source:** GH #16. Reuses the `verified_by` reverse index and TestCase `testLevel`/`status`; complements the per-requirement `trace` view and the ASIL-D HIL rule `W702`.

**Acceptance criteria:** `verification-depth` lists each requirement with its distinct level set, count, and flag; a requirement verified only by one L5 test is flagged `hil-only`, one with no active test `none`, one with L3+L5 `ok`; `--sil`/`--status` filter the rows; `--json` emits the array; `verification-depth --sil 4 --min-levels 2` exits non-zero when a SIL-4 requirement has fewer than two distinct levels and zero when all satisfy it.
