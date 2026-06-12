---
id: REQ-TRS-DISC-007
type: Requirement
name: Tool shall add status/SIL filters, JSON output and gap/coverage views to list and matrix
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** extend the `list <type>` and `matrix` discoverability commands with status/integrity-level filters, gap-only views, machine-readable JSON output, and a coverage rollup, so that an engineer can slice elements and the coverage grid by lifecycle status and safety integrity.

## `list` additions

The `list <type>` command **shall** accept:

- `--status <s>` — keep only elements whose `status:` equals `<s>` exactly;
- `--sil <v>` — keep only elements whose integrity level matches `<v>`: an element matches when its `silLevel:` (integer) stringifies to `<v>` (e.g. `--sil 4`) **or** its `asilLevel:` equals `<v>` (e.g. `--sil D`). One flag covers both SIL and ASIL;
- `--json` — emit a JSON array of the (filtered) elements, each object carrying `qualifiedName`, `type`, `name`, `id`, `status`, `silLevel`, `asilLevel` (absent fields omitted or null).

All three **shall** compose with the existing `--tag`, `--feature`, and `--config` selectors (AND semantics) and **shall** apply to the JSON output as well as the text table.

## `matrix` additions

The `matrix` command **shall** accept:

- `--status <s>` — restrict requirement ROWS to those whose `status:` equals `<s>`, in both text and `--json`;
- `--gaps-only` — drop rows that are fully covered or all-N/A, keeping only rows that have at least one `gap` cell, in both text and `--json`;
- a coverage-% FOOTER — per-configuration and overall coverage computed as `covered / applicable`, where `applicable = covered + gap` (N/A cells excluded). In text the footer prints one line per configuration plus an overall line; in `--json` a `coverage` object is added: `{ "perConfig": { "<cfgId>": {"covered":N,"applicable":M,"pct":P}, ... }, "overall": {"covered":N,"applicable":M,"pct":P} }`, where `pct` is `covered*100/applicable` rounded to one decimal, or `null` when `applicable == 0`.

## Out of scope

SIL-weighted coverage is explicitly out of scope — only plain (unweighted) coverage is computed.

## Rationale

[[REQ-TRS-DISC-004]] added a `--feature` filter to `list`; safety-critical reviews also slice the inventory and coverage grid by lifecycle `status:` and integrity level, and need a JSON form for CI gates. Folding SIL and ASIL into one `--sil` flag keeps the surface small while covering both IEC 61508 and ISO 26262 projects. The coverage footer turns the existing covered/gap/N-A grid into an at-a-glance metric without a separate command.

**Source:** §9 (PLE) and §11.11 (coverage); discoverability commands; extends `list <type>` and `matrix`.

**Acceptance criteria:** `list <type> --status <s>` lists only elements with that status; `list <type> --sil 4` matches a `silLevel: 4` element and `list <type> --sil D` matches an `asilLevel: D` element; `list <type> --json` emits a JSON array reflecting any active filter; `matrix --status <s>` restricts rows to that status; `matrix --gaps-only` keeps only rows with a gap cell; `matrix` prints a per-config and overall coverage percentage footer (and a `coverage` object under `--json`).
