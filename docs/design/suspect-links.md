# Design: Suspect Links + Content Hashing

**Status:** Implemented (v1) — decisions ratified in `ADR-SYS-SUSLINK-001`
**Date:** 2026-07-12

> **Implementation note.** v1 landed as `crates/syscribe-model/src/suspect.rs`
> (projection + BLAKE3 hash + `scan`), the `traceBaselines` field on
> `RawFrontmatter`, warning `W090` in the validator, and the `suspect
> list`/`accept` CLI (`crates/syscribe/src/suspect.rs`). Requirements
> `REQ-TRS-SUS-LINKS-000..007`, tests `TC-TRS-SUS-LINKS-001..007/100/101`
> (`crates/syscribe/tests/suspect_links.rs`). The four open decisions in §8 are
> resolved below.

A mechanism to detect when a trace link's *target* has changed since the link
was last reviewed, and to surface those links as **suspect** so they can be
re-reviewed. The change signal is a **content hash** of the target captured at
review time, stored next to the link.

This is the classic DOORS/RM "suspect link" concept, adapted to Syscribe: the
hash makes it **version-control-agnostic and portable across repos**, which
matters for multi-repo composition (§14).

---

## 1. Motivation

Trace links (`verifies`, `derivedFrom`, `satisfies`, …) assert a relationship
that was valid *at the moment a human reviewed it*. When the target later
changes, the assertion may no longer hold — a test may no longer cover the
requirement it verifies; a child requirement's rationale may be invalidated by
an edit to its parent. Today nothing flags this.

`git diff` can say "the target file changed," but that is a poor signal: it
fires on typos, field reordering, and doc-only edits, producing *suspect
storms* that train reviewers to ignore the flag. We want a **low-false-positive,
semantically meaningful** signal.

### Relationship to multi-repo ref-drift (E510–E515)

`RefState` (`crates/syscribe-model/src/config.rs:118`) already detects when a
peer repo's `HEAD` drifts from its pinned `ref:`. That is **whole-repo,
commit-granularity**. It cannot see:

- an individual referenced element's content changing while the repo stays on
  the same commit, or
- intra-repo suspect links (same repo, no ref involved).

A per-element content hash operates at **element granularity within a single
repo**, so it is **complementary, not redundant** — it fills the gap below
`RefState`'s resolution.

---

## 2. Core mechanism

A **suspect link** is a trace link whose target changed after the link was last
reviewed.

1. When a reviewer accepts a link A→B, capture a **baseline hash** of B's
   *normative projection* (see §3) and store it on A, next to the link.
2. At validation, recompute B's projection hash and compare to the stored
   baseline.
3. Mismatch ⇒ the link is **suspect** ⇒ emit a warning.
4. A reviewer clears it by re-baselining (capture the current hash).

Three decisions define the feature: **(1) what we hash, (2) where the baseline
lives, (3) how suspicion propagates.**

---

## 3. What we hash — normative projection (Axis 1)

**Decision: hash a per-type "trace-significant" projection, not the whole file.**

| Option | Behavior | Verdict |
|---|---|---|
| A. Whole file (frontmatter + body) | Any edit flips every incoming link | ✗ suspect storms; reproduces `git diff` with no added signal |
| **B. Normative projection** | Hash only fields downstream consumers depend on, per type | ✓ **chosen** — meaningful, low false-positive |
| C. Explicit `@normative` region | Author marks the span | ✗ manual, easy to forget |

### Why B is also the *easiest* to implement here

Elements retain only the parsed `RawFrontmatter` struct + markdown body — the
raw YAML text is **not** stored (`crates/syscribe-model/src/frontmatter.rs:6`).
But there is already a precedent for canonical serialization:

- frontmatter → canonical JSON with null-stripping (`crates/syscribe/src/export.rs:44`)
- `custom_fields` is a `BTreeMap` specifically for deterministic sorted output
  (`element.rs:759`)

So the projection is simply: **select a subset of struct fields → serialize to
canonical JSON (sorted keys) → hash the bytes.** This *solves canonicalization
for free* — serde produces deterministic bytes when we control field selection
and use sorted maps. Whole-file hashing would be *more* work (re-read the file,
normalize line endings/whitespace) and worse signal.

### Projection contents (starting point — refine per type)

The projection should contain the fields a downstream consumer actually depends
on. Default surface, then per-type overrides:

- **Requirement**: normative body text + `status` + `reqDomain` + safety fields
  (SIL/ASIL). *Not* editorial doc prose, `displayOrder`, `extRef`, layout.
- **TestCase**: scenario/Gherkin body + `testLevel` + `testFunctions`.
- **Part / PartDef**: ports/interface surface, not doc prose.

> **Open decision (2):** define an explicit per-type significant-surface map up
> front, or ship a sensible default projection (body + `status` + normative
> fields) and specialize per type as false positives appear. Recommend:
> default first, specialize on evidence.

### Hash algorithm

**Recommend `blake3`** — faster than SHA-256, and truncated hex reads cleaner
in frontmatter. `sha2` is the alternative if FIPS familiarity matters. No
cryptographic hashing exists in the tree today (only a non-stable
`DefaultHasher` for cache filenames in `remote.rs`), so this adds the first real
digest dependency to `crates/syscribe-model/Cargo.toml`.

> **Open decision (3):** `blake3` vs `sha2`.

Store as `blake3:<hex>` (algorithm-prefixed) so the digest can be migrated later
without ambiguity.

---

## 4. Where the baseline lives (Axis 2)

**Decision: on the source element, colocated with the link.**

Links point **upstream** and the source holds the reference (§12.1), so the
source also holds the baseline. Keyed by target, one hash each:

```yaml
# TC-SCHED-BITMAP-001.md — a TestCase
verifies: [REQ-SCHED-BITMAP-001]
traceBaselines:
  REQ-SCHED-BITMAP-001: "blake3:9f2a…"
```

### No `acceptedAt` / `acceptedBy`

The **hash is the state**. Who re-baselined it and when is answered by
`git blame` on the line that changed the hash — the model is git-controlled, so
storing provenance in frontmatter duplicates git history and adds diff churn for
no detection value. This matches how the rest of the project leans on git rather
than re-recording provenance.

Consequence to accept: "who owns clearing this suspect link" is a review-process
concern (git + PR review), not a field. For most safety audits the VCS *is* the
audit trail. If a certification context ever requires provenance that outlives
git — or a hash captured in another repo under multi-repo composition — add it
*then*, not now.

New field on `RawFrontmatter` (`crates/syscribe-model/src/element.rs:291`):
`trace_baselines: Option<BTreeMap<String, String>>` (`traceBaselines`), keyed by
target id/qname → algorithm-prefixed hash. `BTreeMap` for deterministic output.

---

## 5. Propagation (Axis 3)

When B changes, the direct link A→B goes suspect. Does A's dependent C→A also go
suspect?

**Decision: implicit one-hop propagation; no eager graph flooding.**

- Reviewer inspects the B change, clears A→B, and — if the change actually
  matters — edits A. That edit changes A's own normative projection, so C→A
  flips suspect on the next validation. **Suspicion rides the same hash
  mechanism one hop at a time, gated by a real human edit.** No "suspect
  storms," no transitive closure to maintain.
- Optionally add a `--show-affected` *advisory* view that computes the
  transitive reachable set ("potentially affected") without flipping them
  suspect.

Rejected: eager transitive closure (floods the trace DAG immediately; usually
noise — a downstream change need not affect grandparents).

### Where it lives in code

The named reverse indices (`verified_by`, `derived_children`, …) are built inline
on `ValidationResult` (`crates/syscribe-model/src/validator.rs:54-79`, built
around `validator.rs:3561-3640`). The suspect pass is a natural **post-pass**
consuming `ValidationResult` + `Resolver`. Because propagation is implicit, the
pass only needs to: for each link, resolve the target, hash its projection,
compare to the stored baseline.

---

## 6. Lifecycle & CLI

New `suspect` subcommand. The CLI is a hand-rolled `match` (not derive-clap):
add `("suspect", include_str!("../../../prompts/help/suspect.md"))` to `HELP`
(`crates/syscribe/src/help.rs:8`) and a `"suspect" => { … }` arm near
`crates/syscribe/src/main.rs:693`.

| Verb | Behavior |
|---|---|
| `suspect list` | Report all suspect links (source, target, kind) |
| `suspect accept <src> <tgt>` | Re-baseline: capture the target's current hash into `traceBaselines` |
| `suspect accept --all` | Re-baseline every currently-suspect link (bulk review) |

### Validation code

Emit a new **W-code** (e.g. `W0xx`) from the suspect pass:
`findings.push(warning("W0xx", file, msg))`
(`crates/syscribe-model/src/validator.rs:6995`). W-severity ⇒ draft-suppressible
and non-fatal by default, gateable in CI via `--deny W0xx`
(`crates/syscribe/src/query.rs:1877`). Add the prose doc entry under
`prompts/help/` / the code reference.

A missing baseline for an existing link is a *separate* condition — either a
distinct info/warn code ("link has no baseline; run `suspect accept`") or
silently treated as suspect. Recommend: distinct code so "never baselined" is
visibly different from "baseline stale."

---

## 7. Integration points (summary)

| Concern | Location |
|---|---|
| Link fields (raw strings) | `RawFrontmatter`, `crates/syscribe-model/src/element.rs:291`+ |
| New `traceBaselines` field | same struct |
| Resolve link target | `Resolver::resolve_ref`, `resolver.rs:519` |
| Canonical serialization precedent | `export.rs:44` (JSON + null-strip), `BTreeMap` at `element.rs:759` |
| Reverse indices (propagation home) | `ValidationResult`, `validator.rs:54-79`, built `validator.rs:3561-3640` |
| Finding / severity | `Finding`/`Severity`, `validator.rs:26-40`; helpers `6991-6999` |
| `--deny` gate | `parse_gate_options` `main.rs:182`, `GateOptions` `query.rs:1877` |
| CLI dispatch | `HELP` `help.rs:8`, match arm `main.rs:693`+ |
| New dep | `blake3` (or `sha2`) → `crates/syscribe-model/Cargo.toml` |
| Complements (not duplicates) | `RefState` commit-drift, `config.rs:118` |

---

## 8. Decisions (resolved)

1. **Scope** — **all trace-link kinds** (REQ-TRS-SUS-LINKS-003): `verifies`,
   `derivedFrom`, `satisfies`, `satisfiedBy`, `refines`, `implementedBy`,
   `supertype`, `subsets`, `redefines`, `breakdownAdr`, `typedBy`, and the domain
   links (`hazardRef`, `mitigatedBy`, `supports`, `evidence`, `confirms`). One
   `traceBaselines` map on the source covers every kind, keyed by target.
2. **Projection** — **default projection** (REQ-TRS-SUS-LINKS-002): body +
   normative frontmatter, excluding `name`/`displayOrder`/`extRef`/`title`/layout
   /`traceBaselines`. Per-type surfaces deferred to a later phase.
3. **Hash** — **BLAKE3**, stored `blake3:<hex>`.
4. **Missing-baseline** — **not treated as suspect and silent during
   validation** (opt-in/additive, REQ-TRS-SUS-LINKS-004); unbaselined links are
   surfaced on demand by `suspect list` rather than via a distinct warning code,
   keeping `validate` output free of coverage-gap noise.

## 9. Rough phasing

1. Add `traceBaselines` field + `blake3` dep + projection function (default
   surface) + `suspect list`/`accept` + W-code. Scope to `verifies` /
   `derivedFrom` / `satisfies`.
2. Per-type projection specialization as false positives surface.
3. `--show-affected` advisory transitive view.
4. Extend to remaining link kinds; multi-repo baseline provenance if a
   certification context demands it.
