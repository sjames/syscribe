---
id: REQ-TRS-SAFE-006
type: Requirement
name: Freedom From Interference for mixed-criticality shared resources
title: Tool shall flag mixed-criticality elements sharing a target without an FFI / partitioning argument
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** detect mixed-criticality model elements that **share a resource**
without a documented freedom-from-interference (FFI) / partitioning argument, per the
dependent-failure analysis of ISO 26262-9 §7, and **shall** raise warning **W034** for
each unguarded mixed-criticality sharing.

## Shared-resource definition

Two elements **share a resource** when both are **allocated to the same target element**.
The tool **shall** collect allocation edges `(source -> target)` from ALL of the following,
resolving every reference through the `Resolver`:

- an element carrying `allocatedTo: [T, ...]` — `source` = the element, `target` = each `T`;
- an element carrying `allocatedFrom: [S, ...]` — `target` = the element, `source` = each `S`;
- an `Allocation` element carrying `allocatedFrom: S` + `allocatedTo: T` — `source` = `S`,
  `target` = `T`. (The issue's `allocateFrom`/`allocateTo` spelling maps onto the existing
  `allocatedFrom`/`allocatedTo` fields used by `Allocation` elements throughout the model.)

The edges **shall** be inverted into a `target -> { sources }` map. A target with fewer than
two distinct resolved sources cannot host a sharing and is not examined.

## Integrity tag

Each element is given a single integrity tag:

1. `asilLevel` if present (e.g. `"D"`), else
2. `silLevel` if present (e.g. `3` → `"SIL3"`), else
3. `QM` (no classification).

## Mixed-criticality test

Two sources allocated to the same target are **mixed-criticality** when their integrity
tags **differ** — including the case of one classified (ASIL/SIL) source versus one `QM`
source. Equal tags are **not** mixed; two `QM` sources are **not** mixed.

## FFI argument

A mixed-criticality pair is **excused** (no finding) when an FFI / partitioning rationale is
present in any one of these forms:

- the **target** element declares a non-empty `ffiRationale:` string; OR
- **at least one** of the two source elements declares a non-empty `ffiRationale:` string; OR
- the **target** or **at least one** of the two source elements carries a `breakdownAdr:`
  that resolves (via the `Resolver`) to an ADR (`type: ADR`) whose `status:` is `accepted`.

The new field `ffiRationale:` (string) **shall** be accepted on any element's frontmatter
(`RawFrontmatter.ffi_rationale`).

## W034

| Code | Severity | Condition |
|---|---|---|
| `W034` | warning | For a target with ≥2 allocated sources, a mixed-criticality source pair has no FFI argument. One finding per offending `(target, sourceA, sourceB)`, naming the two sources and their integrity tags. |

`W034` is a **warning** (per codebase convention so unannotated models stay at exit 0), is
gateable via `--deny W034`, and is profile-promotable.

## Opt-in rule

The whole check is **dormant** unless at least one element in the model declares `asilLevel`
or `silLevel`. A model with no ASIL/SIL classification emits zero W034 and keeps unchanged
exit codes.

## Deferred

The issue's "cross-domain bonus" — surfacing the shared resources as candidate cybersecurity
attack surfaces for the co-analysis view — is **deferred** and not implemented here.

## Acceptance criteria

- Two software `PartDef`s of differing ASIL (e.g. `asilLevel: D` and unclassified/QM) both
  `allocatedTo` the same hardware `PartDef`, with no FFI argument anywhere, yield exactly one
  **W034** naming both sources and their tags.
- Adding `ffiRationale:` to one source or to the target clears the W034; a `breakdownAdr:`
  that resolves to an `accepted` ADR likewise clears it. Both models `validate` with no errors.
- Two sources with the **same** ASIL allocated to one target yield **no** W034.
- `--deny W034` exits non-zero when an unguarded mixed-criticality sharing is present.
- A model with no `asilLevel`/`silLevel` anywhere emits zero W034 (opt-in / dormant).
</content>
</invoke>
