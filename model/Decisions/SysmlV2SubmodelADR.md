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
  on `RawFrontmatter`. The mapper's entire job is writing the same field a hand-authored `.md` file
  would; the validator needs no changes at all, exactly like `@SyscribeFeature` → `appliesWhen:`
  needed none of the feature-model/SAT engine. This is a claim about the mapper, not a claim that
  every lifted field is fully validated on a `PartDef`/`Part` today — a review caught an
  overstatement in an earlier draft, which listed `W701` (`Requirement`-scoped) and `E837`
  (`SafetyGoal`-scoped) as "reused" against `asilLevel:`/`plLevel:` on a `PartDef`, where neither
  actually fires. Both gaps are pre-existing and identical for a hand-authored `PartDef` — this
  addendum doesn't introduce or worsen them — but the requirement's validation-reuse table
  (`REQ-TRS-SYSMLV2-008`) is corrected to say so exactly, rather than aspirationally.

## Addendum: connection-endpoint qualification (`REQ-TRS-SYSMLV2-010`)

`REQ-TRS-SYSMLV2-010`'s originating issue proposed lifting `connect a.x to b.y;` endpoints
verbatim into a `connections:` entry — `{from: "a.x", to: "b.y"}`, textually identical to what a
hand-author might type. Investigation before implementing (and a second round after the first
fix's own empirical check surfaced a further gap) found this doesn't work, and explains why the
implementation instead writes a qualified, head-only qname.

- **Round 1 — empirically confirmed a literal chain never resolves at all.** A hand-authored
  `PartDef` with `connections: [{from: "a.p1", to: "b.p1"}]` and no `features:` entry produces
  **zero** `n2`/`connectivity` edges. `graph.rs::build_graph`'s `resolve_endpoint` (its own code
  comment: "NOTE (deferred, issue #26 MVP): edges carry `kind` only") resolves a chain exactly two
  ways: an exact match against *this element's own* `features:` list (`{name: <head>, typedBy:
  <Type>}` — head only, matched to the *type*, everything past the first `.` discarded), or the
  whole chain treated as an exact qname/id/display-name. A SysMLv2-synthesized `part def`/`part`
  never populates `features:` (its subparts are separate synthesized child elements,
  `REQ-TRS-SYSMLV2-002`'s existing mapping), and a bare `"a.p1"` string is never anyone's exact
  qname (real qnames join with `::`). Literal transcription would satisfy the letter of "lift the
  endpoints" while producing a `connections:` field that looks complete but is functionally inert
  — exactly the kind of gap this session's review passes on `#92`/`#94` kept catching, caught here
  proactively during design instead of after.
- **Round 1's first fix — full-chain qualification — was itself still wrong, caught by the same
  discipline before it shipped.** The first attempt rewrote the *whole* chain, `.`→`::`, prefixed
  with the owning qname (`a.p1` → `Holder::a::p1`), reasoning that a subpart usage's qname is
  always `<owning qname>::<its name>`. True for the head (`Holder::a` always exists), but `p1` is
  overwhelmingly a port *inherited* from `a`'s type (`Ecu`) rather than redeclared on the usage —
  and this module does no inheritance resolution (sub-decision 2 above), so `Holder::a::p1` isn't
  actually a synthesized element in the common case either. A second empirical check (fixture +
  `n2`, deliberately re-run after the "fix" rather than trusted on reasoning alone) confirmed this
  produced the same zero-edges outcome as round 1, just via a different, more-qualified-looking
  string.
- **Final fix: qualify to the head segment only** (`<owning qname>::<head>`, e.g. `a.p1` →
  `Holder::a`) — matching this same connection graph's own existing precedent for
  `features:`-declared endpoints exactly (head-only resolution, rest of the chain discarded), just
  reached via the resolver's *other* existing path (exact-qname match against a real synthesized
  instance) instead of a `features:` declaration SysMLv2 content never has reason to carry.
  Confirmed empirically — fixture + both `n2` **and** `connectivity --format json` — to produce
  real, visible wiring for the realistic, common case. `n2` turned out to have two of its own
  unrelated, pre-existing limitations, found while cross-checking rather than trusting one tool's
  output alone (both in `crates/syscribe/src/n2.rs`, neither touched by this requirement, neither
  this requirement's job to fix): its `collect_edges` reads only the first two `ends:` entries of
  any n-ary connection — native or SysMLv2-lifted alike — so a three-way `connect (a, b, c)` never
  shows the `a`↔`c` edge in `n2`, only `a`↔`b` (root-caused by reading; an earlier draft of this
  addendum mischaracterized this as a display-only "one edge per matrix cell" quirk before the
  actual `collect_edges` code was read closely enough to find the real cause); and its **scoped**
  `n2 <qname>` builds its axis exclusively from `features:` (`subpart_axis`), which a
  SysMLv2-synthesized part never populates, so `n2` scoped to any SysMLv2 subtree reports no parts
  at all regardless of this requirement — only unscoped `n2` (whole-model axis, built differently)
  and `connectivity` benefit from this lift. `connectivity` correctly builds a full star over
  every end and was the tool actually used to confirm the n-ary case end-to-end.
- **Rejected: also synthesizing matching `features:` entries**, to route through the *same*
  resolution path a hand-authored model would use. Rejected because it would (a) duplicate data
  already present in a different, already-correct form (each subpart's own synthesized child
  element and its `typedBy:`), inviting the two representations to drift, and (b) only resolve at
  *type* granularity (collapsing every instance of the same `PartDef` onto one shared node,
  `resolve_endpoint`'s own behavior for the `features:` path) — strictly less precise than
  qualifying directly to the *instance* qname, which resolves distinct subpart usages to distinct
  nodes even when they share a type.
- **Consequence, disclosed rather than silently accepted:** port-level (or deeper)
  specificity is lost — two different ports on the same pair of subparts collapse onto the same
  instance-level edge. This is the identical granularity loss `resolve_endpoint`'s own
  `features:` path already has for a hand-authored model (deliberate MVP scope, sub-decision
  above, not a new gap this requirement introduces); reaching finer-than-instance precision would
  require this module to also resolve inheritance to know which ports a usage actually has, which
  `ADR-SYS-SYSMLV2-001` sub-decision 2 already, deliberately, doesn't attempt.
- **`variant part` coverage was silently missing from an earlier draft, caught by review.** The
  lift was first wired only into `convert_part_def`/`convert_part_usage`, not into
  `convert_variant_usage`'s `Part` branch — a genuine, undisclosed gap (unlike the deliberate
  `Package`/`Requirement` descope `REQ-TRS-SYSMLV2-009` states explicitly), since a `variant part`
  usage shares the identical `PartUsageBody` shape `REQ-TRS-SYSMLV2-008`/`-009` already extended
  their own lifts to. Fixed by wiring `part_usage_connection_entries` into that branch too, the
  same way `with_syscribe_meta`/`with_doc` already were.
