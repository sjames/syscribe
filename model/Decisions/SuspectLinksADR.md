---
type: ADR
id: ADR-SYS-SUSLINK-001
name: "Suspect-link detection via opt-in BLAKE3 content baselines"
status: accepted
tags:
  - traceability
  - suspect-links
---

## Context

Trace links (`verifies`, `derivedFrom`, `satisfies`, …) assert a relationship that was
valid *at the moment a human reviewed it*. When the link's target later changes, the
assertion may no longer hold — a test may no longer cover the requirement it verifies; a
child requirement's rationale may be invalidated by an edit to its parent. Syscribe has
no way to flag this today.

A `git diff` can say "the target file changed," but that is a poor signal: it fires on
typos, field reordering, and doc-only edits, producing *suspect storms* that train
reviewers to ignore the flag. The multi-repo ref-drift checks (`RefState`, W511/W512)
operate at whole-repo, git-commit granularity and cannot see an individual element's
content changing while the repo stays on the same commit, nor intra-repo links.

## Decision

Introduce **suspect-link detection** driven by a stored **content baseline hash** of each
link's target, captured at review time and compared at validation:

- **Storage.** The *source* element (which holds the link, per §12.1) carries an optional
  `traceBaselines:` map from target identifier → algorithm-prefixed hash. No `acceptedAt`
  / `acceptedBy` provenance fields: the hash *is* the state; who re-baselined it and when
  is recovered from VCS history, avoiding duplication and diff churn.
- **What is hashed.** A canonical *projection* of the target — its markdown body plus the
  normative frontmatter fields — **not** the whole file. Editorial/presentation fields
  (`displayOrder`, `extRef`, `name`, layout, comments) and `traceBaselines` itself are
  excluded. v1 uses a single default projection for all element types; per-type surfaces
  may be refined later.
- **Digest.** BLAKE3, stored algorithm-prefixed (`blake3:<hex>`) so the digest — and, if
  needed, the projection version — can be migrated without ambiguity.
- **Opt-in.** Only links that already carry a baseline are checked. A link with no
  baseline is **not** flagged during validation (no warning); baselining is opt-in per
  link via `suspect accept`. The dedicated `suspect list` command can still surface
  unbaselined links on demand, so coverage gaps remain discoverable.
- **Scope.** All trace-link kinds are covered: `verifies`, `derivedFrom`, `satisfies`,
  `satisfiedBy`, `refines`, `implementedBy`, `supertype`, `subsets`, `redefines`,
  `breakdownAdr`, and the domain trace links.
- **Propagation.** Implicit, one hop per review. Clearing a suspect link by re-baselining
  and editing the source changes the source's own projection, so links that target it
  flip suspect on the next validation. No eager transitive flooding of the trace graph.

## Consequences

- Suspect detection is content-level and version-control-agnostic, complementing (not
  duplicating) `RefState` ref-drift; the baseline travels inside the model files across
  repos.
- The feature is purely additive: existing models emit no new warnings until a link is
  baselined.
- Changing the default projection later is a breaking re-baseline event (stored hashes no
  longer match); the algorithm/version prefix makes such a migration explicit.
- "Who owns clearing a suspect link" is a review-process concern (git + PR review), not a
  field. For safety audits the VCS is the audit trail; provenance-in-frontmatter can be
  added later if a certification context demands it.
