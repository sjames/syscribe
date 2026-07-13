---
type: ADR
id: ADR-SYS-BASELINE-001
name: "Release baselines as first-class elements with git-anchored content seals"
status: accepted
tags:
  - traceability
  - baseline
  - release-management
---

## Context

Safety and systems assessment turns on a precise question: *what exactly was released or
assessed, and can it be proven not to have changed?* Today Syscribe has no formal answer.
A git tag is commit-granularity — it neither enumerates which elements were in scope nor
provides a portable, tool-checkable proof of content. The suspect-link mechanism
(`ADR-SYS-SUSLINK-001`) freezes an individual reviewed *relationship*, but nothing freezes
a whole release: the set of work products an assessor is asked to point at.

Assessors (ISO 26262 functional-safety assessor, DO-178C DER, IEC 61508) need a stable
anchor — "the safety case as of REL-2026-07" — that enumerates its scope, records who
approved it and when, ties to a reproducible source state, and can be re-verified and
compared to a later release.

## Decision

Introduce a **`Baseline`** first-class element and a `baseline` command family.

- **Element and identity.** A `Baseline` lives under `model/Baselines/` and carries `name`,
  `date`, `gitTag`, `gitCommit`, `approver`, a `frozenScope` selector, a generated `seal`,
  an optional `supersedes`, and a lifecycle `status` (`draft | approved | released |
  superseded`). Its stable id is `BL-*`, using a FEAT-style relaxed pattern
  `^BL(-[A-Z0-9]{2,12})+$` (no forced numeric suffix), so a release-style id such as
  `BL-2026-07` is valid. The `id` (model identity) is **distinct** from the free-form git
  tag string (`gitTag: REL-2026-07`). Baselines accumulate as a release history;
  `supersedes:` forms the audit chain.
- **Seal = full canonical content hash.** `baseline create` walks the in-scope graph and
  computes, for each element, a BLAKE3 hash of its **full canonical content** — all
  frontmatter *and* body. This reuses the suspect-link canonicalization by **refactoring
  `projection_hash` to take an exclusion set**; the baseline pass supplies a near-empty set.
  The freeze is byte-exact: any in-scope edit, editorial or normative, changes the seal.
- **Baselines are excluded from scope.** `Baseline` elements are themselves excluded from
  scope resolution and from any seal, so a whole-model baseline does not hash other
  baselines (which would make every baseline drift the moment the next one is sealed). Only
  work-product elements are frozen.
- **Git is the content anchor; the hash is the portable proof.** `create` captures the HEAD
  `gitCommit` (expecting a clean working tree; `--allow-dirty` to override) as the
  authoritative sealed state, with the content hash riding on top as a VCS-agnostic
  verification layer. Creating the git **tag** is a release action the user owns; `create`
  records the intended tag name, and `verify` checks tag↔commit consistency when the tag
  exists. `baseline diff … --detail` reconstructs field/body changes via
  `git show <commit>:<path>`.
- **Lean, committed manifest.** `create` writes `<git-root>/baselines/<id>.manifest.json` —
  per-element hashes + metadata + `gitCommit` + a **readiness snapshot** (the validation
  error/warning counts and in-scope element counts by type at seal time). It is committed
  evidence the assessor points at directly; being JSON, the model walker ignores it.
- **Scope (v1).** Whole model (default), a package subtree, and simple `types` / `status` /
  `tags` filters (composed as AND). Configuration-projected and trace-closure scopes are
  deferred; the `frozenScope` schema stays forward-compatible with adding them without
  changing the seal format.
- **Validator-frozen release.** Validation recomputes the in-scope aggregate and compares it
  to the seal, with status-graded severity: `released` drift is `E520` (error, gates CI),
  `approved` drift is `W520` (warning), `draft` is silent, `superseded` is skipped. A seal
  that disagrees with its manifest (a hand-edit) is `E521`. An unresolved `supersedes:`
  reference is `E522`. Changing a released baseline's content requires a new **superseding**
  baseline, never an in-place edit.
- **Review-aware creation.** `create` warns when an in-scope trace link is suspect or has no
  baseline at all; `--require-reviewed` upgrades that to a refusal, so a release can assert
  both *content frozen* and *every relationship reviewed*.

## Consequences

- Assessors get a single referenceable, reproducible artifact (element + manifest + git
  tag), and a portable content proof that survives repository moves.
- The feature is additive and reuses the suspect-link hashing/canonicalization (via a small
  refactor to parameterize the exclusion set) and the validator post-pass pattern; the new
  dependency surface is nil.
- Immutability is enforced by lifecycle + supersede, not by locking files: history stays in
  git and in the supersede chain.
- Baseline seals and suspect baselines answer different questions (exact release freeze vs.
  reviewed-relationship staleness) and deliberately use different exclusion sets over the
  same machinery.
