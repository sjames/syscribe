---
type: ADR
id: ADR-SYS-SYSMLV2-001
name: "Native SysML v2/KerML submodel ingestion: in-process sysml-v2-parser, not a WASM plugin"
status: accepted
tags:
  - sysmlv2
  - interop
  - traceability
---

## Context

`ADR-SYS-PLUGIN-001` already anticipated SysMLv2 as the flagship example of a foreign modeling
methodology living inside a Syscribe model tree, and `examples/wasm-plugins/sysmlv2-toy/` is a toy
TypeScript/WASM parser built against that exact alias. That precedent proves the *shape* of the
problem — a subtree authored in another notation, parsed into origin-agnostic `RawElement`s,
merged into the graph so every cross-reference kind (`derivedFrom:`, `satisfies:`, `verifies:`,
`Allocation`) can target it exactly like a hand-authored element — but real, full-grammar SysML v2
support does not fit the *mechanism* built for that precedent.

Two Rust crates from the `elan8/spec42` project were evaluated:

- `spec42` itself: a monorepo (VS Code extension, LSP, CLI) with a full semantic engine —
  import/inheritance resolution, the bundled OMG standard library — but its internal crates
  (`sysml_model`, `sysml_diagnostics`, `language_service`) are not published individually, its
  CLI's JSON output is an undocumented, pre-1.0 (v0.49.0), fast-moving contract, and it bundles the
  OMG SysML standard library under LGPL v3.0 terms.
- `sysml-v2-parser`: a standalone, MIT-licensed, crates.io-published (55 versions) crate that
  `spec42` itself depends on. It parses SysML v2 and KerML textual notation to an AST
  (`parse(input) -> Result<RootNamespace, ParseError>`) with source spans, but does no cross-file
  import/inheritance resolution and has no standard library or serializer — AST-only, one file at
  a time.

The scope actually needed — read-only ingestion plus three specific cross-reference directions
(SysMLv2 → Syscribe `Requirement`, Syscribe `TestCase` → SysMLv2 element, SysMLv2 variation point →
Syscribe `FeatureDef`) — resolves entirely through Syscribe's own qname/id index, not through SysML
v2 semantic legality (type-checking, multiplicity, standard-library-aware inheritance). None of
that requires spec42's semantic engine.

## Decision

A package `_index.md` may declare `sysmlSubmodel: true`. Every `.sysml`/`.kerml` file anywhere in
that directory's subtree is parsed in-process via `sysml-v2-parser`, merged across files that
declare pieces of the same SysML v2 package, and injected into the graph as ordinary `RawElement`s
— qname `<owning Syscribe package qname>::<SysML v2 fully-qualified name>` — through a dedicated
module invoked directly from `walker::walk_model`, the same injection point the FMEA/TARA
row-explosion passes and the WASM-plugin merge already use.

Four sub-decisions, each with a rejected alternative:

1. **A dedicated native subsystem, not a third `[plugins.<alias>]` engine variant.** Routing this
   through the existing `foreignFormat:`/`[plugins.<alias>]` mechanism was considered — literally
   adding a `native = "sysmlv2"` alternative to `wasm = "..."` in `PluginEntry`. *Rejected:* the
   trust models are fundamentally different. `[plugins.*]`'s `timeout_ms`/`memory_max_bytes` and
   the whole sandboxing apparatus exist because a plugin is arbitrary third-party WASM code that
   can hang or misbehave; `sysml-v2-parser` is a trusted, non-executing, compile-time Rust
   dependency with neither failure mode. Forcing it through that abstraction would carry
   meaningless config fields and mislabel a core, always-available capability as a "plugin." The
   marker is a plain `sysmlSubmodel: true` boolean instead of `foreignFormat: <alias>` — there is
   exactly one built-in engine, so the alias indirection (which exists to let multiple named
   third-party engines coexist) buys nothing here.
2. **AST-only `sysml-v2-parser`, not spec42's semantic engine.** *Rejected:* shelling out to the
   `spec42` CLI, which would gain import/inheritance resolution and the standard library "for
   free" but adds an external-binary runtime dependency, rides an undocumented pre-1.0 JSON
   contract, and indirectly bundles LGPL-licensed standard-library data. The three cross-reference
   directions this feature exists to serve don't need that semantic engine — they resolve through
   Syscribe's own resolver, not SysML v2 static semantics.
3. **Full SysML v2 grammar parsing, narrow element-kind mapping.** The parser handles all ~640
   textual productions regardless of scope, so nothing about coverage constrains parsing itself.
   *Rejected:* mapping every construct into a first-class `RawElement` up front. Behavior bodies,
   `analysis`/`case`/`verification def`, `calc`/`constraint` are counted/named for browsing but not
   deeply modeled in this phase — an open-ended mapping surface with no natural stopping point,
   versus a fixed set matched to what the three link directions and reasonable structural browsing
   actually require (`REQ-TRS-SYSMLV2-007`).
4. **Cross-boundary links reuse SysML v2's own vocabulary, not a Syscribe-invented syntax.**
   `satisfy`/`verify` (targeting a `Requirement` by id — quoted, since `REQ-*` ids contain hyphens —
   or qname) and a `@SyscribeFeature { featureId = '...' }` metadata annotation (targeting a
   `FeatureDef`, since feature-model/SAT semantics have no SysML v2 equivalent) are both real,
   standards-parseable AST nodes. *Rejected:* a comment-based directive convention for either
   direction — fragile (no structural guarantee it parses, easy to typo silently), and it would
   make the `.sysml` source less portable to real SysML v2 tooling than using the language's own
   constructs.

## Rationale

- **Why not just extend the plugin abstraction, since it already exists and already named
  SysMLv2 as the example?** Naming it as an example didn't commit the mechanism to it. The
  sandboxing machinery is the expensive, purpose-built part of `ADR-SYS-PLUGIN-001`, and none of
  it applies to a compile-time Rust dependency — carrying it forward would be cargo-culting
  ceremony, not architecture.
- **Why is AST-only sufficient for a "validator"?** Because the validation this feature commits to
  is structural well-formedness and referential integrity (can every cross-boundary reference be
  resolved?), not SysML v2 static semantic legality (is this a well-typed model per the OMG
  spec?). The latter stays `spec42`'s job, run separately, exactly as PlantUML rendering today
  stays an optional external tool rather than something Syscribe reimplements.
- **Why a separate error/warning code range from the WASM-plugin family?** `E530`–`E532`/
  `W530`–`W534` are documented as plugin-execution codes. Reusing them here would misattribute a
  native-parser failure as a plugin failure to anyone grepping a validation report.

## Consequences

- A model with no `sysmlSubmodel: true` package is completely unaffected.
- A `.sysml`/`.kerml` parse failure degrades to zero elements from that file plus a warning,
  never aborting the rest of `validate` — the same graceful-degradation posture multi-repo's
  `RefState::Unknown` and the WASM-plugin path already established (`REQ-TRS-SYSMLV2-006`).
- `examples/wasm-plugins/sysmlv2-toy/` now collides in name with a real, differently-mechanized
  feature; it should be renamed (e.g. to a made-up toy format name) once this ships, so the
  repository doesn't have two things called "sysmlv2" built two different ways.
- **Explicitly out of scope**, tracked as follow-on if a concrete need arises: a writer/serializer
  back into `.sysml`/`.kerml` text, two-way round-trip authoring, and full SysML v2 static semantic
  validation (type-checking, multiplicity legality, standard-library-aware inheritance).

## Addendum: `@Syscribe*` fixed-field metadata annotations (`REQ-TRS-SYSMLV2-008`)

Sub-decision 3 above deliberately drew the mapped-element-kind boundary narrow; this addendum
draws an analogous boundary for a different axis — the *fields* a mapped `part def`/`part` can
carry, not which element kinds get mapped at all.

- **A fixed, named annotation per field group, not a generic `@Syscribe*` → `custom_fields:`
  passthrough.** Rejected for the same reason sub-decision 3 rejected open-ended element mapping:
  an unbounded surface with no natural stopping point, versus four names
  (`@SyscribeDomain`/`@SyscribeIntegrity`/`@SyscribeShortName`/`@SyscribeImplementedBy`) matched
  to exactly the fields real safety-relevant architecture authoring needs and nothing else. Growing
  this set later is expected to extend the fixed list, not replace the principle.
- **`@SyscribeIntegrity` bundles `asil`/`sil`/`pl` into one annotation name, not three.** These
  three fields are mutually-exclusive alternatives on a native element already (ISO 26262 vs. IEC
  61508 vs. ISO 13849-1 integrity scales) — one annotation with three optional keys mirrors that
  relationship directly, and lets the pre-existing `silLevel`/`asilLevel` mutual-exclusion warning
  (`W006`) do the "don't set more than one" enforcement with zero new validation code, rather than
  inventing a parallel check for the SysMLv2-originated path.
- **Lift-only: no new validation, no origin-aware branching.** Every field this addendum lifts —
  `domain:`, `asilLevel:`/`silLevel:`/`plLevel:`, `shortName:`, `implementedBy:` — already exists
  on `RawFrontmatter` and is already validated for a hand-authored element. The mapper's entire job
  is writing the same field a hand-authored `.md` file would; the validator needs no changes at
  all, exactly like `@SyscribeFeature` → `appliesWhen:` needed none of the feature-model/SAT engine.
