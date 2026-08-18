---
id: REQ-TRS-FM-006
type: Requirement
name: Tool shall auto-derive a FeatureDef's FEAT-* id from its name on a featureTree entry that omits id
status: draft
reqDomain: software
verificationMethod: test
---

A `featureTree:` entry (REQ-TRS-FM-005) still requires a hand-typed `id:` on every one of what can be a large flat list of entries, even though the mandatory `FEAT-*` id (`E201`) exists mainly for the id-form cross-reference shorthand (`appliesWhen:`, `Configuration.features:`, `crossTreeConstraints:`) — most of the time it is entirely mechanical, derivable from the entry's own dotted `name:`. This requirement removes that repetition for the single-file form specifically, where it is felt most (a directory-per-feature layout naturally has only one `id:` per file).

The tool **shall**:

- treat `id:` as **optional** on a `featureTree:` entry (absent, `null`, or the empty string all count as "not given"); an explicit, non-empty `id:` always wins and is used verbatim, exactly as today — derivation only happens when none is given;
- when an entry gives no `id:`, derive one from its own dotted `name:` path (the same path already used to build the entry's qname, §9.6a): split on `.`, uppercase each segment, strip any character that is not `[A-Z0-9]` (so a basic-name underscore, e.g. `Cortex_M`, is dropped, not preserved), and join the segments with `-`, prefixed `FEAT-` — e.g. `name: Wdt` → `id: FEAT-WDT`; `name: Platform.CortexM` → `id: FEAT-PLATFORM-CORTEXM`;
- assign the derived id to the synthesized `FeatureDef` exactly as if the author had typed it, so every existing id-related rule applies unchanged and without a new error code: a derived id that doesn't match the `FEAT-*` pattern (e.g. a name segment that strips to fewer than 2 or more than 12 `[A-Z0-9]` characters) is still `E006`, and a derived id that collides with another element's id anywhere in the model (hand-authored or itself derived) is still `E101` — both exactly as they already fire for a hand-authored id;
- leave a plain per-file `FeatureDef` (not part of a `featureTree:`) unaffected — its `id:` remains mandatory, and a missing one is still `E201`, unchanged.

**Out of scope:** no new validation code. No attempt to auto-repair a grammar-invalid derived id (padding/truncation heuristics) — the author renames the feature or supplies an explicit `id:` instead. No change to `crossTreeConstraints:`'s own id-form reference resolution (REQ-TRS-FM-005) — a `crossTreeConstraints:` entry referencing a feature by a *derived* id works exactly like referencing one by a hand-authored id, since by the time cross-references resolve, the two are indistinguishable.

**Source:** §9.6 (FeatureDef stable id), §9.6a (single-file `featureTree:`, REQ-TRS-FM-005).

**Acceptance criteria:**

- A `featureTree:` entry `{ name: Wdt }` (no `id:`) synthesizes `Features::Wdt` with `id: FEAT-WDT`.
- A `featureTree:` entry `{ name: Platform.CortexM }` (no `id:`) synthesizes `Features::Platform::CortexM` with `id: FEAT-PLATFORM-CORTEXM`.
- A `featureTree:` entry that sets an explicit `id:` keeps exactly that id; derivation never overrides it.
- A `featureTree:` entry whose derived id fails the `FEAT-*` pattern (e.g. `name: X`, a single character) produces `E006` naming the derived id — not a new code, and not a silent drop.
- Two `featureTree:` entries (in one sheet or across sheets in the same model) that derive to the same id produce `E101`, the same as two hand-authored `FeatureDef`s sharing an id.
- A plain per-file `FeatureDef` with no `id:` still produces `E201`, unchanged.
