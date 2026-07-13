---
type: Requirement
id: REQ-TRS-BL-005
name: "Drift detection and validator-frozen release lifecycle"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-BL-000]
breakdownAdr: Decisions::BaselineADR
tags:
  - baseline
  - validation
---

Validation shall detect when a baseline's sealed scope no longer matches the current model,
and shall freeze released baselines.

## Drift detection (status-graded)

For each `Baseline` element, validation shall re-resolve its scope (REQ-TRS-BL-003),
recompute the in-scope aggregate hash (REQ-TRS-BL-002), and compare it to the stored
`seal.aggregateHash`. On mismatch the severity shall follow the baseline's status:

- `draft` — silent (still being assembled).
- `approved` — **`W520`** (warning).
- `released` — **`E520`** (error, gates CI): a released baseline is frozen and must not
  drift.
- `superseded` — skipped (a historical record; its content is expected to have moved on).

## Seal integrity

- If a baseline's `seal.aggregateHash` disagrees with the aggregate recorded in its manifest,
  validation shall emit **`E521`** — the seal has been hand-edited (tampered), independent of
  drift.

## Immutability via supersession

- A `released` baseline shall not be changed in place. A model change that alters a released
  baseline's in-scope content shall be resolved by creating a **new** baseline whose
  `supersedes:` names the old one (and marking the old one `superseded`), never by re-sealing
  the released baseline.
- A `supersedes:` reference shall resolve to an existing `Baseline`; an unresolved reference
  shall be **`E522`**. `supersedes:` is resolver-checked but is **not** a suspect-tracked
  trace link, so re-sealing or editing the superseded baseline does not flag its successor.

`E520`/`W520`/`E521`/`E522` participate in the standard gate machinery and can be made fatal
or waived per code via `--deny` / severity profiles.
