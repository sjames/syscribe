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
