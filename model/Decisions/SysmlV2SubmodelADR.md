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
  SysMLv2-synthesized part never populates, so `n2` scoped to any SysMLv2 subtree reported no
  parts at all regardless of this requirement as originally shipped — only unscoped `n2`
  (whole-model axis, built differently) and `connectivity` benefited from this lift at first.
  `connectivity` correctly builds a full star over every end and was the tool actually used to
  confirm the n-ary case end-to-end. (The scoped-`n2` gap was closed shortly after by
  `REQ-TRS-SYSMLV2-011`'s own addendum below, which widens `subpart_axis` itself; the n-ary
  `collect_edges` limitation remains open.)
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

## Addendum: `n2`'s scoped axis widened to include SysMLv2 children (`REQ-TRS-SYSMLV2-011`)

`REQ-TRS-SYSMLV2-010`'s addendum above already disclosed that `n2 <qname>`'s axis (`n2.rs`'s
`subpart_axis`) is `features:`-only, so scoped `n2` never sees a SysMLv2 subtree. This addendum
closes that gap, choosing the fix mechanism deliberately.

- **Widen `subpart_axis` to add qname-containment as a second, additive source — not a new
  `sysmlSubmodel:`-aware code path.** `subpart_axis` gains one more per-element check: is a
  candidate's qname a direct child (`<scope>::<name>`, no further `::`) of the current frontier
  element? This has nothing SysMLv2-specific in it — it's the same containment relationship
  `graph.rs`'s `Contains` edge already establishes for every element in the model, regardless of
  origin. **Rejected:** gating the new check behind `sysmlSubmodel: true`, which would have made
  the fix a special case rather than a general one, and would have missed the (admittedly rarer,
  but real) case of a hand-authored model that nests a `PartDef`/`Part` as a real child file
  instead of an inline `features:` entry — that shape gets the exact same, pre-existing "invisible
  to scoped `n2`" gap today, for the identical reason, and deserves the identical fix.
- **Rejected: synthesizing `features:` entries during ingestion instead of touching `n2.rs`
  directly**, mirroring `REQ-TRS-SYSMLV2-010`'s own earlier rejection of the same idea for
  `graph.rs`'s connection-edge resolver. Same two reasons apply again: duplicated data inviting
  drift, and `features:`-path resolution in `n2.rs` (like `graph.rs`) only resolves to the
  *type*, not the instance — `n2`'s axis would still be less precise (type-collapsed) than
  containment-based inclusion gives it directly.
- **`is_part`'s existing `PartDef`/`Part`-only filter is unchanged, deliberately.** A `Port`
  endpoint (e.g. this repository's own worked-example `powerLink connect powerPort to
  rotorConfig;`) still never appears on `n2`'s axis — widening `is_part` itself, or teaching `n2`
  to resolve a `Port` endpoint up to its owning `Part`, is a different, unscoped concern this
  requirement doesn't attempt.

## Addendum: local, lookahead-only resolution of a two-segment `connect` endpoint (`REQ-TRS-SYSMLV2-013`)

`REQ-TRS-SYSMLV2-010`'s head-only qualification is correct but lossy when a `.sysml` author
explicitly redeclares the referenced feature on the usage itself, rather than only inheriting it
from the type — `connect a.fooProvider to b.fooClient;` where `a`'s own body genuinely declares
`interface fooProvider : IFoo;` has a real, already-parsed answer sitting right there in the AST
that head-only qualification was discarding unconditionally.

- **Resolution stays purely local — no resolver, no global element list, no inheritance
  reasoning.** `find_sibling(head)` searches only the *same enclosing body* the `connection` usage
  itself lives in, for a `part` usage named by the head; if found, that usage's own already-parsed
  body (not a separately-synthesized `RawElement` — this all happens before any child conversion
  for that usage has occurred) is searched for a direct child named by the tail. This is
  deliberately narrower than "resolve like any other cross-reference" (the originating issue's own
  phrasing) — a real resolver-based approach would need the complete, cross-file element graph,
  which doesn't exist yet at the point `ingest_subtree` is converting one file's own AST. Rejected
  for the same reason a global lookup was rejected everywhere else in this module: this feature
  doesn't need it, and reaching for it would be a much larger, cross-cutting change (deferring
  connection-entry construction to a second pass after the whole model is known) for a case that's
  fully answerable from the AST already in hand.
- **Only a genuinely two-segment chain is eligible; three-plus segments always fall back.**
  Extending the lookahead to walk multiple levels deep (`a.b.c`, matching a nested `part` usage's
  own nested child) was considered and rejected as unnecessary complexity for a case the
  originating issue's own acceptance criteria didn't ask for — `REQ-TRS-SYSMLV2-013`'s own worked
  example is two segments (`a.fooProvider`), and every real motivating case found in practice was
  as well. A future requirement can widen the recursion depth if a concrete need for it surfaces;
  the one-level lookahead doesn't need restructuring to get there, just a loop instead of a single
  step.
- **The head must be a `part` usage specifically — not a `port`/`attribute` sibling, not a `part
  def`.** A connect endpoint's head is, definitionally, the thing that structurally *has* the
  nested feature the tail names; a `port`/`attribute` sibling has no body of its own to search
  (its shape doesn't carry nested named children the way a `part` usage's does), and a `part def`
  isn't the local usage instance a `connect` clause actually wires. Both fall back to head-only,
  same as any other non-match.
- **No `item` usage arm in the tail-matching search — confirmed, not assumed, to be correct.**
  `PartUsageBodyElement` (a `part` *usage*'s own body-element enum) carries no `ItemUsage` variant
  at all in this grammar version, unlike `PartDefBodyElement`'s — so a `part` usage genuinely
  cannot declare a nested `item` usage in the first place, verified against the parser's own enum
  definition rather than inferred from the absence of test coverage.

## Addendum: `W542` truncation warning for a genuinely two-segment, non-redeclared `connect` endpoint (`REQ-TRS-SYSMLV2-015`)

Issue #104: `REQ-TRS-SYSMLV2-013`'s local-redeclaration lookahead only reaches a feature explicitly
redeclared on the usage — the far more common case, a feature *inherited* from the head's type and
never redeclared, still silently truncates to the head-only edge, with nothing in `validate`
output distinguishing "resolved" from "silently dropped." Confirmed against a real, multi-file
CarOS submodel: every ordinary composed `part` reference hit this path, none of them redeclaring.

- **Full resolution through the inherited type was rejected as out of scope, for the same reason
  `REQ-TRS-SYSMLV2-013` itself stayed AST-local.** Resolving `a.fooProvider` when `fooProvider` is
  declared on `a`'s *type* (`A`), not on `a`'s own usage body, needs that type's full definition —
  which may live in a different file, processed before or after this one, and isn't available as a
  `RawElement` yet at the single-file, ingest-time point `qualify_connection_end` runs at. Deferring
  connection-entry construction to a second pass after the whole model is known (so a real resolver
  could run) was considered and rejected as a much larger, cross-cutting restructuring for a
  narrower problem than it would solve — the same reasoning `REQ-TRS-SYSMLV2-013`'s own addendum
  above already applied to rejecting a global resolver for this exact call site.
- **A warning instead: `W542` fires exactly when `qualify_connection_end` falls back to head-only
  for a genuinely two-segment chain** — the identical condition its own lookahead logic already
  isolates, so detecting "was this truncated" costs nothing beyond returning the fact alongside the
  qname it already computes. No new AST traversal, no new resolver, no new global state.
- **Attached to the owning part element via `RawElement.derive_findings`, computed *before* that
  element exists as a `RawElement`.** `part_def_connection_entries`/`part_usage_connection_entries`
  run inside `convert_part_def`/`convert_part_usage`/`convert_variant_usage`, before `push_synth`
  creates the owning element — so truncation messages are threaded back as plain `Vec<String>`
  alongside the `connections:` YAML value, and attached to `out.last_mut()` (the element `push_synth`
  just pushed) one statement later, via a small shared `push_connection_truncation_findings` helper
  used at all three call sites. This mirrors `W540`'s existing pattern of a post-hoc
  `derive_findings.push` onto an already-synthesized element, not a new plumbing mechanism.
- **Three-plus-segment chains stay silent, unchanged.** `REQ-TRS-SYSMLV2-013`'s addendum above
  already documents, as a deliberate decision, that a chain beyond two segments always falls back
  to head-only with no attempt at resolution — that decision isn't revisited here; `W542` fires
  only for the narrower two-segment case this issue actually reported.

## Addendum: doc-comment `@Syscribe*` directives for `interface def`/`port def`/`connection def` (`REQ-TRS-SYSMLV2-014`)

Issue #100 asked for `REQ-TRS-SYSMLV2-008`'s real `@Name { field = value; }` metadata annotations
to widen from `part def`/`part` to `interface def`/`port def`/`connection def`. Investigated
before implementing, per this module's own established discipline (the same two-round pattern
`REQ-TRS-SYSMLV2-013`'s addendum above used): confirmed by direct inspection of the vendored
`sysml-v2-parser` source, in both the pinned 0.53.0 and the latest 0.54.0, that
`InterfaceDefBodyElement`, `PortDefBodyElement`, and `ConnectionDefBodyElement` carry **no
`MetadataAnnotation` variant at all** — not a missing `ingest.rs` dispatch arm the way an unmapped
element kind is, but a genuine absence in the grammar production for these three body kinds.
`@SyscribeImplementedBy { path = '...'; }` inside an `interface def { }` is a hard parse error
(`W541`), confirmed empirically, exactly matching the issue's own reported evidence.

- **Real fix requires an upstream parser change, which this repository doesn't own.**
  `sysml-v2-parser` is a plain crates.io version dependency (`sub-decision 2` above), not a
  vendored/forked local copy. Widening these three enums to accept `@Name { ... }` is a grammar
  change in a crate this repository doesn't control the source of (`elan8/sysml-v2-parser`).
  Forking/vendoring the parser to make that change locally was considered and rejected — it would
  reverse sub-decision 2's deliberate architectural choice (a trusted, non-executing, compile-time
  dependency over owning a semantic engine) for the sake of one field-lift issue, a much larger and
  higher-risk change than the problem warrants.
- **Doc-comment-embedded directive lines as a deliberate, different, substitute syntax.**
  `REQ-TRS-SYSMLV2-014` recognizes `@SyscribeDomain: ...`/`@SyscribeIntegrity:
  ...`/`@SyscribeShortName: ...`/`@SyscribeImplementedBy: ...` lines inside the `doc /* ... */`
  comment these three element kinds already support (`REQ-TRS-SYSMLV2-009`), stripping a
  recognized line out of the lifted `doc:` text and writing the corresponding frontmatter field
  instead — landing on exactly the same fields `REQ-TRS-SYSMLV2-008` lifts, through a text-scan of
  already-lifted doc content rather than a second `MetadataAnnotation`-walking function. This is
  explicitly **not** presented as the same syntax as the real `@Name{...}` annotation form — a
  `.sysml` author writing `interface def`/`port def`/`connection def` metadata uses a visibly
  different spelling (a colon-suffixed line inside a comment, not a structural annotation before a
  member) specifically because the real form has nowhere to parse to for these three kinds. Should
  upstream ever add the missing `MetadataAnnotation` coverage (issue #100's suggested path 1), this
  directive mechanism is not retired — it becomes a second, always-available spelling, the same way
  a stable `id` and a qualified name are both valid cross-reference targets today.
- **Reuses the lifted doc string, not a second AST walk.** The directive scanner runs over the
  text `doc_lift` (`REQ-TRS-SYSMLV2-009`) already produced for that element, rather than
  re-inspecting `DocComment` AST nodes directly — one text-processing step, not a duplicate
  traversal of the body-element list already walked once for the doc lift itself.
- **Last directive wins per field**, matching `REQ-TRS-SYSMLV2-008`'s existing behavior for
  multiple real `@Syscribe*` annotations of the same name on one element — no new "which one
  applies" rule invented for the doc-comment form.

## Addendum: scoped `typedBy:` resolution for `W600` (`REQ-TRS-SYSMLV2-016`)

Issue #105 reported that `REQ-TRS-VAL-017`'s `W600` suppression (a `Part` usage whose `typedBy:`
target itself carries documentation) only fired for a same-package reference — a cross-package
`part x : Services::Documented;`, `Documented` declared in a *different* `package` than `x`, still
raised `W600` even though the reference resolves correctly everywhere a human reads it (`links`,
`show`) and the target genuinely has documentation.

- **Root cause confirmed by direct inspection, not assumed from the issue's own diagnosis.** The
  issue's own "Proposed approach" assumed a working, scope-aware resolver already exists elsewhere
  in the codebase (citing `contains`/`featureTyped`/`connection` edges and cross-file package
  merging as evidence) and asked for the `W600` check to reuse it. That resolver doesn't actually
  exist: `graph.rs`'s `TypedBy` edge does an exact-index lookup with no fallback at all, and
  `Resolver::resolve_ref` (what `REQ-TRS-VAL-017` actually used) does exact qname / stable-ID /
  display-name matching only — no scope-relative resolution. `links`' apparent success at
  resolving `Services::Documented` (the CLI's `(matched: ...)` line) turned out to be a **fuzzy
  string-similarity fallback** in `query.rs::resolve`, a display-only heuristic that happened to
  score the real target highest for this particular input, not a real namespace resolver.
  Confirmed by reproducing the exact issue scenario and testing `connectivity`/`check-ref`/the
  `TypedBy` graph edge directly: none of them resolve the cross-package reference either. This
  changed the fix from "route through an existing mechanism" to "build the missing mechanism."
- **`Resolver::resolve_scoped_ref` implements real, minimal scoped-namespace lookup**: given the
  referencing element's own qname, it tries the reference prefixed by each enclosing package in
  turn — innermost first, walking outward one `::`-segment at a time (the same prefix-walk
  `graph.rs`'s `Contains` edge already establishes as this codebase's convention for "enclosing
  scope") — down to the model root, then falls back to the existing `resolve_ref` unchanged. This
  mirrors real SysML v2 namespace-lookup semantics (search the local scope, then each enclosing
  scope outward) closely enough for `typedBy:`/`supertype:` purposes without building a full
  semantic-scoping engine.
- **Wired into `W600`'s suppression check only, not into `graph.rs`'s `TypedBy` edge, the
  dangling-`typedBy:` check, or `W007`'s "never used as a supertype or type" tracking** — all four
  share the identical root cause (a package-relative SysMLv2-authored reference not resolving via
  plain `resolve_ref`), confirmed empirically in the same repro this addendum used. Widening every
  one of them was considered and rejected for this issue specifically: `#105`'s filed defect and
  acceptance criteria are `W600`-scoped, each of the other three call sites has its own blast
  radius (a `connectivity`/`n2` edge silently appearing where none did before is a more
  consequential behavior change than a warning going quiet), and `resolve_scoped_ref` is written as
  a general, reusable `Resolver` method precisely so a future issue can widen any of those three
  without re-deriving the resolution logic. Scope creep beyond one filed, well-bounded defect was
  deliberately avoided, the same discipline `REQ-TRS-SYSMLV2-013`'s addendum above applied to its
  own local-lookahead widening.

## Addendum: widening scoped resolution to `W007` and `graph.rs`'s `TypedBy` edge (`REQ-TRS-SYSMLV2-017`)

Issue #107 followed directly on the previous addendum's own deferred scope: it confirmed, against
the same live CarOS (`sabaton-caros`) architecture submodel, that 35 of 36 `W007` ("defined but
never used as a supertype or type") warnings there were false positives, all sharing the identical
root cause `REQ-TRS-SYSMLV2-016` had already diagnosed and fixed for `W600` — a `.sysml`-authored
`typedBy:`/`supertype:` value is frequently package-relative text (e.g. `Services::Documented`
written from inside a different package), which the plain `Resolver::resolve_ref` only resolves
when it happens to already equal the target's full model-root qname. Only the top-of-hierarchy
`CarOS` element itself was a genuine unused type.

- **`W007` widened.** Both the top-level `supertype:`/`typedBy:` scan and the nested `typedBy:`
  scan (`collect_typed_by_refs`, walking `features:`/`connections:`/`flowConnections:`/
  `bindingConnections:`/`successionConnections:`/`performs:`/nested `ports:`) now resolve through
  `resolve_scoped_ref(elements, <referencing element's own qname>, r)` instead of the plain
  `resolve_ref`. `exhibitsStates:` is deliberately left alone — it is never synthesized by SysMLv2
  ingestion (no `.sysml` construct maps to it), so it is always already fully qualified from the
  model root by this format's own hand-authored convention, and widening it would be a no-op at
  best and untested surface at worst.
- **`graph.rs`'s `TypedBy` edge widened too**, judged safe to include in the same change unlike the
  previous addendum's "each has its own blast radius" caution: on inspection, the edge's *previous*
  behavior was not merely unscoped but strictly narrower than `resolve_ref` — a bare `idx.get(s)`
  exact-qname lookup, with no id/display-name fallback at all. So the previous addendum's worry
  ("a `connectivity`/`n2` edge silently appearing where none did before is a more consequential
  behavior change than a warning going quiet") was re-examined and found to cut the other way here:
  every edge this widening adds is one that was semantically real all along (SysML v2's own
  namespace-scoping rules already make the reference resolve) and simply never appeared due to the
  bug, confirmed directly — `connectivity` on the issue's own repro showed a bare, edgeless node
  before this fix and the correct `[typedBy]` edge after. No case was found, in the existing test
  suite or by construction, where this widening could turn an *incorrect* non-edge into an
  incorrect edge: `resolve_scoped_ref` only ever produces a match that real SysML v2 namespace
  lookup would also produce.
- **Left open, same posture as before:** the `Supertype` edge in `graph.rs` (only `TypedBy` was
  named in scope by both the issue and this requirement) and `mutate::guard`'s dangling-`typedBy:`
  check (`EREF`, gating MCP guarded-write commits) — a write-path guard rail widening deserves its
  own scrutiny (a previously-refused commit becoming silently accepted is a different risk profile
  than a validator warning going quiet or a read-only graph traversal gaining an edge), not folded
  into a read-path fix. Tracked as a future issue if a concrete need arises.
- Not independently re-run against the real, external CarOS/`sabaton-caros` submodel from inside
  this repo (it lives outside this tree) — the fix is instead verified against the issue's own
  minimal two-package repro, reproduced faithfully as `REQ-TRS-SYSMLV2-017`'s qual test case
  (`TC-TRS-SYSMLV2-017`) and as a `syscribe-model` integration test driving the real SysMLv2 ingest
  pipeline end-to-end (`sysmlv2_typed_by_scoped.rs`), both confirming the exact false-positive
  pattern the issue described is gone while a genuinely unused `*Def` still fires.

## Addendum: `state def`/`action def` mapping — partial reversal of sub-decision 3 (`REQ-TRS-SYSMLV2-018`/`-019`)

Sub-decision 3 above drew the mapped-element-kind boundary narrow and named "behavior bodies" as
its own canonical example of what stays outside it. This addendum moves `state`/`action`
specifically out of that deferred set — not a reversal of the parse-broad/map-narrow *principle*
(`REQ-TRS-SYSMLV2-007` still holds it, and still explicitly defers `calc`/`constraint`/`case`/
`analysis`/`verification def`), but a boundary move for two kinds only, made possible by two things
that weren't true when sub-decision 3 was written: `StateDef`/`ActionDef` now have real, tested
`satisfies:`/`verifies:` traceability participation for hand-authored elements (this session's
`satisfies:`-shape audit, itself mirroring `refines:`'s `E316`/`REQ-TRS-MG-010` precedent), and the
target schema already exists, exercised by real hand-authored elements in this repo's own
`model/Behavior/` — this addendum maps onto that existing schema, it does not invent one.

- **A real, non-negotiable ceiling, not a Syscribe scope choice.** `fork`/`join`/`decide`/`merge`
  block bodies are parsed by the pinned `sysml-v2-parser` (`= "0.54.0"`) and then discarded by the
  parser itself: `FirstMergeBody`, the AST type backing all four, is `Semicolon | Brace` where
  `Brace` carries **no data at all** — confirmed directly against the vendored crate's source, not
  inferred. No mapping design on Syscribe's side, at this pinned version, can recover what a
  `fork { ... }`/`join { ... }`/`decide { ... }`/`merge { ... }` block actually contains. These four
  become flat `controlNodes:` markers (`{name, kind}` only) — the same shape the hand-authored
  convention (`MissionExecution.md`'s `ForkNode`/`JoinNode`) already uses for exactly this reason:
  a control node with no recoverable internal structure. *Rejected:* waiting for or forking the
  parser to retain fork/join/decide/merge bodies — out of proportion for a capability this
  requirement doesn't need (the point is state-machine/activity traceability participation, not
  full activity-diagram fidelity), and upstream parser behavior is not this project's to fix.
- **`if`/`while`/`loop`/`for` recurse for real**, unlike the four above — `IfStmt.then_body`/
  `.else_body`, `WhileStmt.body`, `LoopStmt.body`, `ForLoop.body` are all typed `ActionDefBody`
  (confirmed against the AST directly) and genuinely retain their nested content, so
  `subActions:`'s `IfAction`/`LoopAction` entries carry a real, recursively-built `then:`/`else:`/
  `body:` — matching `TakeoffAction.md`'s `LoopAction`/`MissionExecution.md`'s `IfAction` worked
  examples exactly.
- **Guard/condition rendering (`render_expression`) is Syscribe-owned and revisitable, explicitly
  distinct from the ceiling above.** Transitions/conditions/assign operands are general
  `Expression`s (23 variants); the common shapes (literals, references, binary/unary operators via
  the parser's own `as_str()`) render exactly, but the long tail (`Classification`/`Select`/
  `Collect`/`Conditional`/`MetaCast`/`TypeCheck`/`CollectionOp`/`MetadataAccess`/`Extent`) falls back
  to a fixed, kind-naming placeholder rather than vanishing — a guard must never disappear, since
  `W072`'s non-determinism check depends only on the field being present. *Rejected:* span-slicing
  the original source text for a fully faithful rendering — would require threading file content
  through every one of the ~20 existing conversion functions across the whole dispatch chain (none
  of which carry it today, only the file *path*), for a benefit (verbatim guard text for
  uncommon expression shapes) that doesn't change what any existing validator check actually does
  with the field. Revisitable later without touching the ceiling above, since it's a Syscribe
  rendering choice, not a parser fact.
- **`entry`/`do`/`exit`'s own nested body content is not mapped**, only the referenced action's
  name. `EntryAction`/`DoAction`/`ExitAction.body` is typed `StateDefBody` — the *state*-body
  grammar, not an action-body grammar, confirmed a parser-leniency artifact rather than deliberate
  per-action-body semantics — and the native schema has no field to hold nested content there
  regardless (`entryAction:`/`doAction:`/`exitAction:` are `string | {name, typedBy}` only). A
  deliberate, bounded gap, not an oversight.
- **`isParallel:` is not representable in this parser version at all.** Neither `StateDef` nor
  `StateUsage` carries a parallel/orthogonal-region flag (confirmed against both struct
  definitions) — left unset, not defaulted to `false` (which would be a false claim, not an absence
  of information).
- **Naming synthesis for unnamed control-flow constructs** (`if_1`, `while_1`, `perform_1` for an
  anonymous `perform action { ... }`, …) is a Syscribe-owned convention, not a parser fact — the
  grammar itself gives `IfStmt`/`WhileStmt`/`LoopStmt`/`ForLoop` no name field at all, yet
  `successionConnections:` needs a stable identifier to reference. Deterministic and stable across
  re-ingestion of unchanged source (a per-enclosing-body, ordered counter), but not guaranteed
  stable across an *edit* that reorders sibling constructs of the same kind — acceptable for a
  read-only ingestion feature with no round-trip authoring, tracked here rather than silently
  assumed.

## Addendum: `view def`/`viewpoint def`/`rendering def` mapping — further reversal of sub-decision 3 (`REQ-TRS-SYSMLV2-020`/`-021`/`-022`)

Continuing the boundary-move pattern the previous addendum established for `state`/`action`, this
addendum moves the six view-family kinds (`ViewDef`, `ViewUsage`, `ViewpointDef`, `ViewpointUsage`,
`RenderingDef`, `RenderingUsage`) out of sub-decision 3's deferred set. Same posture: the
parse-broad/map-narrow *principle* is untouched, only the mapped-set membership grows, and the
target schema already exists — exercised by real hand-authored elements
(`model/Viewpoints/SystemsEngineerViewpoint.md`, `model/Views/SystemArchitectureView.md`) and two
existing validator checks (`W500`, `W502`), both scoped to `ElementType::View` only.

- **`ViewDef` cannot syntactically carry `satisfies`/`expose` in this grammar, and that's not a
  gap.** `ViewDefBodyElement` (confirmed against the vendored crate's own `src/ast/view.rs`) carries
  `Doc`/`MetadataAnnotation`/`Filter`/`ViewRendering` only — no `Expose`/`Satisfy` variant at all.
  Those two only exist on `ViewBodyElement`, a `view` *usage*'s own body. This lines up exactly with
  `W500`/`W502` already being `View`-only, never `ViewDef`-only, so there is no tension to paper
  over: the native schema's own validated shape already anticipated this asymmetry.
- **`ExposeMember`/`SatisfyViewMember` body content is parsed and then discarded by the vendored
  parser itself — a real, non-negotiable ceiling, the same class of fact as the previous addendum's
  `FirstMergeBody` ceiling.** `ExposeMember.body`/`SatisfyViewMember.body` are both typed
  `ConnectBody`, a bodyless `Semicolon | Brace` enum with no payload at all (confirmed directly
  against the source). The BNF's optional `expose <target> [ <expr> ]` filter suffix is likewise
  parsed and its span thrown away (`src/parser/view.rs::expose_member`'s own comment: "skip content
  to reach body") — `ExposeMember` carries no `filter` field to receive it. Neither can be recovered
  by any mapping design on Syscribe's side at this pinned parser version.
- **`expose:` is always emitted as a flat plain string, never the richer `{ref, isRecursive,
  filter}` map form — a deliberate choice, not a missed opportunity, and one that sidesteps a
  pre-existing, unrelated bug rather than replicating it.** `ExposeMember.target` already includes
  any `::*`/`::**` suffix textually (confirmed against the parser's own diagnostic tests — `vehicle`,
  `vehicle::*`, `vehicle::**`, `vehicle::*::**`, dotted feature-chains all show up verbatim in
  `target`), so a flat string loses nothing `is_import_all`/`is_recursive` would otherwise add. This
  also happens to matches both real hand-authored `expose:` lists in `model/` (always flat qname
  strings) and sidesteps a real, pre-existing inconsistency worth naming explicitly: `validator.rs`'s
  `W502` map-form branch reads a `ref` key, while `spec/markdown-sysml-format.md` §8.14.3 documents
  `target` as the schema key. Fixing that mismatch is out of scope for this requirement — it predates
  this feature and affects hand-authored map-form `expose:` entries too, not just SysMLv2-sourced
  ones — but emitting flat strings here means SysMLv2-synthesized `expose:` entries are never
  affected by it either way.
- **No dedicated `Viewpoint` usage `ElementType` exists, so `ViewpointUsage` maps onto `View`.** The
  native schema's own documentation already frames `View` as "usage of a ViewDef or ViewpointDef" —
  this requirement takes that framing literally rather than inventing a new element kind for a
  usage-level distinction the schema itself doesn't draw.
- **`ViewpointDef`/`ViewpointUsage` reuse `RequirementDefBody` verbatim — confirmed structural
  identity, not a coincidental resemblance.** Both `ViewpointDef.body` and `ViewpointUsage.body` are
  literally typed `RequirementDefBody` in the AST, and the parser's own `viewpoint_def`/
  `viewpoint_usage` functions call straight into `requirement_def_body(...)`. `stakeholders:`/
  `concerns:` come from the same `Stakeholder`/`Purpose` variants a plain `requirement def` already
  exposes.
- **`methods:`/`satisfiedBy:` on `ViewpointDef` are deliberately never computed — not an oversight,
  and not merely "no AST source."** Even if a whole-model inversion pass were built (scanning every
  `view`'s own `satisfy <viewpoint>;` clause and writing the result back onto the `ViewpointDef` it
  names), doing so would point the link the wrong way per §12.1's OSLC upstream-link-direction rule:
  the `View` should hold the reference to the `ViewpointDef` it satisfies, not the reverse. Leaving
  these two fields unset is the traceability-correct choice, not just the cheaper one.
- **`PartUsageBodyElement` has zero coverage for the whole family — and unlike the analogous
  `ActionDef` gap in the previous addendum, this one is a genuine parser-level rejection, not a
  silent per-kind skip.** A `view`/`viewpoint`/`rendering` declared directly inside a `part` usage
  body fails to parse outright (confirmed empirically: the enclosing `.sysml` file's parse fails with
  a real diagnostic, gracefully degrading to a `W541` finding per `REQ-TRS-SYSMLV2-006`, not a crash
  and not a synthesized element) — stronger than "this dispatch site doesn't have an arm for it,"
  because there is no arm *to* have; the grammar itself doesn't accept the construct in that
  position.
- **The narrow nested-`view` shape inside a `rendering`/`render` usage body is deliberately not
  recursed into.** `RenderingUsageBodyElement::ViewUsage` exists specifically for the
  `view :>> columnView[N] { render ...; }` redefinition pattern (confirmed against real SysML v2
  standard-library fixtures — `Views Example.sysml`, `11a-View-Viewpoint.sysml`) — narrow, not
  representative of ordinary modeling, and there is no native "nested view" field on `RenderingDef`/
  `Rendering` to hold it regardless. *Rejected:* modeling it as a special-cased inline field —
  disproportionate for a single, standard-library-specific idiom.

## Addendum: `concern def`/`concern` mapping — further reversal of sub-decision 3 (`REQ-TRS-SYSMLV2-023`)

Direct follow-on to the previous addendum, moving `concern def`/`concern` out of sub-decision 3's
deferred set. Native `ElementType::ConcernDef`/`ElementType::Concern` already existed
(`element.rs:110,133`) but were reachable from neither hand-authored content (no `type: ConcernDef`
file exists anywhere in `model/`) nor SysMLv2 ingestion before this addendum — this closes the gap
`REQ-TRS-SYSMLV2-021`'s Viewpoint work opened but didn't need to close itself.

- **No separate `ConcernDef` struct exists in the vendored parser at all — a real, structural
  difference from View/Viewpoint/Rendering, not a smaller version of the same pattern.** A single
  `ConcernUsage` AST node (confirmed directly against `sysml-v2-parser-0.54.0/src/ast/requirement.rs:305-320`)
  parses both `concern def X` and `concern x` textual forms; `is_definition: bool` is the sole
  discriminator, and the struct's own doc comment states outright that the BNF's `ConcernDefinition`
  production "is not modeled as a distinct struct." Unlike the Viewpoint precedent (where Syscribe's
  native schema was the *narrower* side — no dedicated `Viewpoint` usage `ElementType` existed, so
  `ViewpointUsage` had to fold onto `View`), here Syscribe's native schema is the *richer* side: both
  `ConcernDef` and `Concern` already existed, so one conversion function branching on `is_definition`
  is the whole mapping — no folding, no invented element kind.
- **`ConcernUsage.type_name` carries a double meaning the AST itself doesn't disambiguate — confirmed
  by reading the parser function itself, not inferred from the struct shape.** `concern_usage`
  (`.../src/parser/requirement.rs:901-919`) calls the exact same `feature_usage_header` regardless of
  `is_definition`, populating one shared `type_name: Option<String>` field from whatever follows the
  `:`. For `concern def X : Y` this is semantically a supertype; for a bare `concern x : Y` usage
  it's semantically a typedBy — the same textual `:` syntax means two different relationships
  depending on `is_definition`, and the AST gives the mapping code no help telling them apart beyond
  that one boolean. `docs/PARSER_TECHNICAL_DEBT.md:64` (in the vendored crate) corroborates
  independently: concern usage "routes through the shared `usage.rs` header-parsing... alongside
  part/port/attribute/... usages" — i.e. `concern def` is parsed with the *usage* grammar, not a
  dedicated definition grammar, which is *why* the field is shared in the first place.
- **`ConcernUsage` is reachable only from `PackageBodyElement` — a strictly narrower surface than
  View/Viewpoint/Rendering, not merely "also missing from PartUsageBodyElement."** Confirmed absent
  from *both* `PartDefBodyElement` and `PartUsageBodyElement` (`grep -n "Concern"
  src/ast/structure.rs` → zero hits, in both directions) — the View/Viewpoint/Rendering family, by
  contrast, at least reached `PartDefBodyElement`. A `concern`/`concern def` nested inside *any*
  `part`/`part def` body fails to parse outright at this pinned parser version, degrading via the
  existing `W541` path, same posture as the narrower part-usage-only gap from the previous addendum
  — just one enum broader here.
- **`requires:`/`assume:`/`parameters:` are deliberately out of scope, not an oversight born of
  running out of time.** A `RequireConstraint`'s actual constraint content
  (`require`/`assume constraint <name> { ... }`) lives nested inside its own body's
  `ConstraintDefBodyElement::Expression` — real, buildable, but requiring the same class of
  expression-rendering work `render_expression` does for State/Action guards, and genuinely
  unattempted anywhere in `ingest.rs` today, including for native `Requirement`/`RequirementDef`
  (which share this exact same gap — `convert_requirement_def`/`convert_requirement_usage` don't
  lift `subject:`/`requires:`/`assume:` either, confirmed by reading them directly). `parameters:` is
  spec-documented (§8.11.5) but has no matching `RawFrontmatter` field at all today, for any element
  kind — inventing one for this increment alone would be scope creep unconnected to what a
  `concern`'s own AST can actually supply right now.
- **No new validator check ties `ViewpointDef.concerns:`/`RequirementDef.concerns:` to real
  `ConcernDef` elements — a deliberate deferral, not a missed opportunity.** The obvious next
  question after making `ConcernDef` real is whether a `W500`-style resolution check should follow.
  It doesn't, in this addendum, because both existing hand-authored Viewpoint files
  (`model/Viewpoints/{SystemsEngineerViewpoint,SafetyEngineerViewpoint}.md`) write `concerns:` as
  free descriptive prose today ("System-level architecture and decomposition", "Failure modes and
  effects", ...), not qnames — a resolution check would immediately fire on real, correct,
  already-committed content with no migration path offered in the same breath. Whether `concerns:`
  *should* move to qname references at all is a separate, real design question this addendum
  deliberately leaves open rather than forcing a hasty answer as a side effect of an unrelated
  mapping feature.

## Addendum: `flow def`/`flow` mapping and the `flowConnections:` lift (`REQ-TRS-SYSMLV2-024`)

Moves `flow def`/`flow` out of sub-decision 3's deferred set. Unlike Concern, `FlowDef`/`FlowUsage`
were already reachable from all three dispatch enums this module cares about — no parser-level
ceiling blocked the base mapping. The interesting content of this addendum is the
`flowConnections:` lift, and one real, empirically-discovered AST fact this session's "verify
against source, never assume" discipline caught mid-implementation.

- **`FlowDef`/`FlowUsage` are two distinct AST structs, closer in shape to the View/Viewpoint/
  Rendering precedent than to Concern's single-struct design.** `FlowDef { identification,
  specializes, body: DefinitionBody, membership }` and `FlowUsage { kind: FlowUsageKind, name:
  Option<String>, type_name: Option<String>, payload: Option<Node<PayloadFeature>>, from:
  Option<Node<Expression>>, to: Option<Node<Expression>>, body: DefinitionBody, membership }`
  (`sysml-v2-parser-0.54.0/src/ast/behavior.rs:349-389`) — two conversion functions,
  `convert_flow_def`/`convert_flow_usage`.
- **`ends:`/`itemType:` (§8.6.1, the shape `model/Flows/PowerFlowDef.md` uses) are not derivable
  from a `flow def`'s body — a real ceiling, not an oversight.** `FlowDef.body`/`FlowUsage.body`
  share a deliberately thin `DefinitionBody` (`DefinitionBodyElement`: `Error`/`Doc`/
  `OccurrenceMember`/`Other` only). Real fixtures show a `flow def` body *can* nest `attribute`/
  `part`/`flow` members (via the generic `OccurrenceMember(OccurrenceBodyElement)` variant), but
  nothing marks any of them as "this is an end port" the way a nested `StateUsage` was unambiguous
  for State. *Rejected:* guessing that the first nested `part`/`port` member is an end — no
  spec-grounded basis for that guess, and wrong more often than right for a body that can equally
  hold ordinary structural content.
- **A doc comment inside a `flow def`/`flow` usage body is not a direct `DefinitionBodyElement::Doc`
  — confirmed by parsing real source and inspecting the AST directly, not assumed from the enum
  shape (the implementation's first attempt got this wrong and a test caught it immediately).** It
  lands wrapped as `OccurrenceMember(OccurrenceBodyElement::Doc)` instead — `flow_body_doc` checks
  both shapes; every other `OccurrenceBodyElement` variant stays unwalked per the point above.
- **The `flowConnections:` lift mirrors `REQ-TRS-SYSMLV2-010`'s `connections:` lift almost exactly
  — the same dual pattern, verified against the live `connection_usage_entry`/
  `part_def_connection_entries`/`convert_connection_usage` code before writing a line of the flow
  equivalent.** Every `FlowUsage` found directly in a `part def`/`part` usage body — named or
  anonymous — is scanned into the owning part's `flowConnections:`; a *named* one additionally
  becomes its own standalone `Flow` element. `item_type:` (never `typedBy:`) is populated from
  `payload.type_name` (the `of` clause) or `type_name` (the bare `:` shorthand) — both item-shaped
  per real parser fixtures showing the two forms as parallel, interchangeable ways to say *what
  flows*, matching Syscribe's own spec framing of `itemType` as "shorthand: qualified name of the
  ItemDef carried by this flow." There is no AST field distinct from that item-type source that
  would represent "typed by an actual FlowDef", so `typedBy:` is never populated on a flow's own
  element or its `flowConnections:` entry.
- **A real, empirically-discovered AST fact that changed a shared helper, found only by testing
  against real parsed output rather than trusting the plan's assumption.** The plan assumed
  `connection_end_display` (the endpoint-to-string helper `REQ-TRS-SYSMLV2-010` already built) could
  be reused unchanged for flow endpoints. The first test run proved otherwise: `FlowUsage.from`/`.to`
  are typed as a general `Expression` (the value-expression grammar's postfix `.` chaining), *not*
  the dedicated `path_expression` production `connect` endpoints use — so `a.x` parses as nested
  `Expression::MemberAccess(FeatureRef("a"), "x")`, never `Expression::FeatureChainRef`.
  `connection_end_display` gained a new, recursive `MemberAccess` arm to handle this. Confirmed, by
  running the full existing connection-lift test suite afterward, not to change `connect` endpoint
  behavior at all — real `connect` endpoints never produce `MemberAccess` in the first place, so the
  new arm is purely additive for this module's existing callers.
- **`payload.multiplicity` (the `of name : Type[mult]` cardinality) is out of scope** — no
  multiplicity-to-string renderer exists anywhere in this module yet, the same class of descope as
  Concern's `requires:`/`assume:`.
- A `FlowUsage` nested inside an `ActionDef`/`ActionUsage` body stays excluded by
  `REQ-TRS-SYSMLV2-019`'s own, separate, pre-existing action-body walker — this addendum doesn't
  touch it. `InterfaceDefBodyElement`/`OccurrenceBodyElement`/`RequirementDefBodyElement` also carry
  a `FlowUsage` variant per the AST, but none of those bodies are recursively walked for nested
  elements anywhere in this module — unaffected, matching every other mapped kind's identical scope.
