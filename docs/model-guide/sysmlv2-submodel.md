# Native SysMLv2 Submodels

`GUIDE · SYSMLV2-SUBMODEL`

A team or tool ecosystem may already hold real content in standards-track SysML v2 textual
notation. **Native SysMLv2 submodels** let a directory inside the model tree be authored in that
notation directly, parsed in-process, and merged into Syscribe's graph as first-class
elements — so a native `Requirement`/`TestCase`/`FeatureDef` can reference SysMLv2-authored
content, and vice versa, exactly as it would a hand-authored Markdown element (`ADR-SYS-SYSMLV2-001`).

Everything here is **opt-in**: a model with no `sysmlSubmodel: true` package behaves exactly as
before, and none of this runs.

This is **read-only ingestion**. The `.sysml`/`.kerml` subtree stays authoritative and is edited
by its own native tooling — Syscribe never writes into it.

Unlike [stdio-subprocess foreign-format plugins](stdio-plugins.md) (a separate, third-party-plugin
mechanism), this is a dedicated, always-on **native** subsystem: there is no `[plugins.<alias>]`
config, no alias, and no plugin process to spawn.
`sysml-v2-parser` (the crate doing the parsing) is a trusted, compile-time Rust dependency, not
arbitrary executable code — see `ADR-SYS-SYSMLV2-001`'s sub-decision 1 for why that distinction
matters enough to warrant a separate mechanism instead of a third `[plugins.<alias>]` engine
variant.

---

## 1. Marking a package — `sysmlSubmodel: true`

```yaml
---
type: Package
name: PropulsionSubsystem
sysmlSubmodel: true
---
```

Every `.sysml`/`.kerml` file anywhere in that directory's subtree — however nested — is parsed as
native SysML v2/KerML textual notation instead of Markdown+YAML frontmatter. The package's own
`_index.md` remains a normal native element (name, doc body, containment tree entry). Nested
subdirectories inside the marked subtree carry **no namespace meaning of their own** — a stray
`_index.md` found anywhere inside is excluded and reported as `W540`, not processed as a package.

Hand-authored `.md` element files may coexist alongside `.sysml`/`.kerml` content in the same
directory — both are parsed normally and contribute to the same package's namespace. This is not
forbidden; it's the expected shape when a team is migrating content gradually, or keeping some
elements (an `ADR`, a `TestPlan`) natively Markdown-authored right next to the SysML v2 source
they document.

## 2. What's ingested

Every element's qualified name is `<owning Syscribe package qname>::<SysML v2 fully-qualified
name>`, resolvable by every cross-reference kind — `derivedFrom:`, `satisfies:`, `verifies:`,
`Allocation` — exactly like a hand-authored element.

**Multi-file merge.** If two `.sysml`/`.kerml` files in the same subtree both declare pieces of
the same SysML v2 package, they merge into **one** namespace before qname assignment, rather than
colliding or duplicating:

```sysml
// Structure.sysml
package Propulsion {
    part def RotorAssembly;
}
```

```sysml
// Interfaces.sysml — same package, different file
package Propulsion {
    part def Drone {
        part rotorConfig : RotorAssembly;
    }
}
```

Both resolve under `PropulsionSubsystem::Propulsion::*`.

**Full-grammar parsing, fixed-set mapping.** The parser accepts the complete SysML v2/KerML
textual grammar — a file never fails to parse solely because it contains a construct outside the
mapped set. Only a fixed set of element kinds is synthesized into first-class,
cross-referenceable `RawElement`s:

`Package`, `Part(Def/Usage)`, `Attribute(Def/Usage)`, `Port(Def/Usage)`,
`Connection(Def/Usage)`, `Interface(Def/Usage)`, `Item(Def/Usage)`, `Requirement(Def/Usage)`,
`AllocationUsage`, `variation`/`variant` membership, — as of `REQ-TRS-SYSMLV2-018`/`-019` —
`State(Def/Usage)`/`Action(Def/Usage)` (§14, below), — as of `REQ-TRS-SYSMLV2-020`/`-021`/
`-022` — `View(Def/Usage)`, `ViewpointDef`, `ViewpointUsage`, `Rendering(Def/Usage)` (§15, below),
— as of `REQ-TRS-SYSMLV2-023` — `ConcernDef`/`Concern` (§16, below), — as of
`REQ-TRS-SYSMLV2-024` — `FlowDef`/`Flow` (§17, below), — as of `REQ-TRS-SYSMLV2-025` —
`EnumerationDef`/`Enumeration` (§18, below), — as of `REQ-TRS-SYSMLV2-026`/`-027`/`-028` —
`CaseDef`/`Case`, `AnalysisCaseDef`/`AnalysisCase`, `VerificationCaseDef`/`VerificationCase` (§19,
below), — as of `REQ-TRS-SYSMLV2-029` — `AllocationDef` (§20, below), and — as of
`REQ-TRS-SYSMLV2-033`..`-036` — `ConstraintDef`/`Constraint`, `CalculationDef`/`Calculation`,
`UseCaseDef`/`UseCase` and package-level `doc` (§22, below).

A construct outside that set — `occurrence`, `actor`, `dependency`, `metadata` usages, and similar — parses without
error but contributes **nothing** to the graph: no element, no `Finding`, invisible, the same way a
native Markdown model has no way to express content that isn't frontmatter or documentation body.
Parse-broad, map-narrow.

```sysml
package Propulsion {
    part def MappedPart;          // becomes SysML2::Propulsion::MappedPart
    occurrence def UnmappedOcc;   // parses fine, contributes nothing (W543)
}
```

## 3. Cross-references

Three directions are supported, all reusing existing, unmodified Syscribe machinery — no new
resolution logic, no new gating logic.

### `satisfy`/`verify` → a native `Requirement`

A SysMLv2 element's native `satisfy`/`verify` relationship can target a native Syscribe
`Requirement`, by its quoted `REQ-*` stable id (SysML v2's quoted-name syntax, needed because a
bare SysML v2 identifier cannot contain a hyphen) or by its Syscribe qualified name:

```sysml
part def RotorAssembly {
    satisfy 'REQ-DRONE-ENDUR-001';                      // quoted-id form
}

part def Drone {
    satisfy Requirements::'REQ-DRONE-THRUST-001';       // qualified-name form
}

requirement thrustCheck {
    verify 'REQ-DRONE-VERIFY-001';                      // verify keyword
}
```

The mapper carries the target string verbatim into the synthesized element's
`satisfies:`/`verifies:` field; resolution uses the existing id-or-qname resolver unchanged. An
unresolvable target is the same dangling-reference finding already raised for any other
unresolved `verifies:` (`E102`) or `satisfies:` (`E114`; `E512` in a `[repos]` model).

### A native `TestCase` → a SysMLv2 element

A native `TestCase`'s existing `verifies:` field resolves against the qname index of any ingested
SysMLv2 subtree, so a `TestCase` can verify a SysMLv2-authored element the same way it verifies a
native `Requirement`:

```yaml
---
type: TestCase
id: TC-DRONE-ROTOR-001
name: "Rotor assembly spins up to rated speed"
status: draft
testLevel: L3
verifies:
  - PropulsionSubsystem::Propulsion::RotorAssembly
---
```

This widening is scoped to elements that actually came from SysMLv2 ingestion — a hand-authored
native element of the same kind (a plain `PartDef`) is still rejected exactly as before.

### `@SyscribeFeature` → a `FeatureDef`

Variability/feature-model semantics have no equivalent construct in vanilla SysML v2, so a
`variation`/`variant` element uses SysML v2's own standards-compliant metadata-annotation
extension point to reach one:

```sysml
variation part def RotorConfigChoice {
    variant part quadConfig : RotorAssembly {
        @SyscribeFeature {
            featureId = 'FEAT-ROTOR-QUAD';
        }
    }
}
```

`featureId` lifts straight into the synthesized element's `appliesWhen:` — the exact field a
native element's `appliesWhen:` already populates — so `feature-check --deep`/`validate
--config`/`configure` reason about it identically to a native element, with **no solver
changes**. A `variation`/`variant` with no `@SyscribeFeature` annotation is ingested normally as a
purely structural element; it simply doesn't participate in the feature-model graph. An
unresolvable `featureId` is the same `E209` already raised for any other unresolved `appliesWhen:`
reference.

### `@SyscribeDomain`/`@SyscribeIntegrity`/`@SyscribeShortName`/`@SyscribeImplementedBy` → fixed fields

A fixed, named set of four `@Syscribe*` metadata annotations lift onto a `part def`/`part`
(including a `variant part` usage) exactly like `@SyscribeFeature` does, into the fields a
safety-relevant architecture needs most and that have no expressible form in real `.sysml` text
otherwise (`REQ-TRS-SYSMLV2-008`):

```sysml
part def CarSafetyServices {
    @SyscribeDomain {
        value = 'software';
    }
    @SyscribeIntegrity {
        asil = 'B';
    }
    @SyscribeShortName {
        value = 'car-safety-services';
    }
    @SyscribeImplementedBy {
        path = 'services/car-safety-services/';
    }
}
```

| Annotation | Field lifted | Existing validation reused |
|---|---|---|
| `@SyscribeDomain { value = '...'; }` | `domain:` | E303, E315, E313 |
| `@SyscribeIntegrity { asil = '...'; }` | `asilLevel:` | E010, E841–E843, W808 |
| `@SyscribeIntegrity { sil = ...; }` | `silLevel:` (bare integer, not quoted) | E009, W006, E841–E843, W808 |
| `@SyscribeIntegrity { pl = '...'; }` | `plLevel:` | — (format-checked only on `SafetyGoal` today, via `E837`; on a `part def`/`part` it's carried but unvalidated — same as a hand-authored one) |
| `@SyscribeShortName { value = '...'; }` | `shortName:` | — (display only) |
| `@SyscribeImplementedBy { path = '...'; }` | `implementedBy:` | W023 |

Every field lifted here already exists on the frontmatter schema and is already validated for a
hand-authored element (to whatever extent the existing validator actually checks that field on a
`PartDef`/`Part` — the "reused" column above is exact, not aspirational) — the mapper's entire job
is writing the same field a `.md` file would; **no validator changes** exist for this, exactly
like `@SyscribeFeature`. Note in particular that `W701` (integrity level should imply a
`verificationMethod`) is scoped to `type: Requirement` in the existing validator, so it never
fires here — not on a SysMLv2-lifted `PartDef`, and not on a hand-authored one either.
`@SyscribeIntegrity` may carry any of its three keys; more than one present on the same
annotation isn't specially rejected by the mapper — both fields are simply written, and the
pre-existing `asilLevel`/
`silLevel` mutual-exclusion warning (`W006`) fires on them exactly as it would for a hand-authored
element carrying both. A `part def`/`part` with none of these annotations is unaffected — no
regression versus today's behavior.

### Doc-comment `@Syscribe*:` directives on `interface def`/`port def`/`connection def`

The `@Name { field = value; }` annotation form above depends on a real `MetadataAnnotation` AST
node — and `InterfaceDefBodyElement`, `PortDefBodyElement`, and `ConnectionDefBodyElement` carry no
such variant at all in the vendored `sysml-v2-parser` grammar (confirmed by direct source
inspection, both the pinned and the latest release). `@SyscribeImplementedBy { path = '...'; }`
inside an `interface def { }` is a hard parse error, not silently dropped. For exactly these three
element kinds, the same fixed field set is instead reachable through a **structured directive line
inside the element's own `doc /* ... */` comment** (`REQ-TRS-SYSMLV2-014`):

```sysml
interface def IPowerInterface {
    doc /*
    Real documentation prose stays here.
    @SyscribeShortName: power-if
    @SyscribeImplementedBy: aidl/interfaces/car/power/IPowerInterface.aidl
    */
}
```

| Directive | Field(s) lifted |
|---|---|
| `@SyscribeShortName: <value>` | `shortName:` |
| `@SyscribeImplementedBy: <path>` | `implementedBy:` (drives `W023` exactly like the real annotation form) |
| `@SyscribeDomain: <value>` | `domain:` |
| `@SyscribeIntegrity: <key>=<value>[, <key>=<value>...]` (keys `asil`/`sil`/`pl`) | `asilLevel:`/`silLevel:`/`plLevel:` |

A recognized directive line is stripped out of the text that lands in the element's `doc:` field —
it's metadata, not documentation prose, exactly as a real annotation never appears in a `part
def`/`part`'s lifted `doc:` either. An unrecognized `@Something: ...` line is left in the doc text
untouched. A later directive for the same field overrides an earlier one, matching the real
annotation form's own last-wins behavior for repeated `@Syscribe*` annotations.

This is a **deliberately different spelling**, not an alternative parse of the same syntax — a
`.sysml` author writing metadata on these three element kinds uses a colon-suffixed comment line
specifically because the real `@Name{...}` form has nowhere to parse to here. Scoped to `interface
def`/`port def`/`connection def` only (not their usage counterparts) — see `examples/sysmlv2-submodel/`
and `ADR-SYS-SYSMLV2-001`'s addendum for the full design rationale, including why forking/vendoring
the parser to add real support was considered and rejected.

## 4. Validation

| Code | Condition |
|---|---|
| `W540` | A `_index.md` found anywhere inside a `sysmlSubmodel: true` package's subtree, other than that package's own anchor `_index.md` |
| `W541` | Either a `.sysml`/`.kerml` file failed to read (e.g. invalid UTF-8), or `sysml-v2-parser` failed to parse its contents |
| `W542` | A `connect` endpoint's genuinely two-segment chain fell back to a head-only edge because the tail isn't a locally-redeclared feature (§8's redeclaration lookahead didn't match) — identifies the dropped segment. Also raised when an `allocation` usage's `allocate` endpoint chain is truncated (§20) |
| `W543` | Advisory: a `.sysml`/`.kerml` file in a `sysmlSubmodel:` subtree contains parsed constructs with no Syscribe mapping (`metadata` usages, `occurrence`, `individual def`, `actor`, a root-level `alias`, an unresolved package-level `satisfy`, …); raised once per file with per-kind counts, and the constructs are not ingested (REQ-TRS-SYSMLV2-030). Example: `metadata def x2, alias x1`. It does not cover members nested inside a mapped definition's body (e.g. a requirement's `frame`/constraint body); gate it with `--deny W543` |
| `W544` | Advisory: an ingested usage's multiplicity has integer bounds with lower greater than upper, a negative bound, or a non-integer numeric literal bound (REQ-TRS-SYSMLV2-066); name or expression bounds are not evaluated |

All of them share a **dedicated code range**, distinct from the [stdio-subprocess plugin
family](stdio-plugins.md) (`E550`/`E551`/`W550`–`W553`) — this is native, always-on ingestion of a
trusted, compile-time dependency, not plugin execution, and conflating the two ranges would
misattribute the failure mode to anyone grepping a validation report.

A `W541` (either kind) downgrades only the affected file's contribution — fewer or no elements
from it — while every other file in the subtree, and the rest of the model, validates normally.
Never a hidden fallback to a previous run's output: a failed parse means fewer elements this run,
full stop.

## 5. Worked example

See [`examples/sysmlv2-submodel/`](https://github.com/sjames/syscribe/tree/main/examples/sysmlv2-submodel)
for a complete, runnable example: a small drone-propulsion model exercising every capability above
in one coherent scenario — multi-file merge, every mapped element kind, both `satisfy` forms plus
`verify`, a `TestCase` verifying a SysMLv2 element, a `variation`/`variant` pair gated by
`@SyscribeFeature` with two `Configuration`s making `--config`/`feature-check --deep` show real,
differing projection, and an unmapped-construct file demonstrating parse-broad/map-narrow.

```bash
syscribe -m examples/sysmlv2-submodel/model
syscribe -m examples/sysmlv2-submodel/model feature-check --deep
syscribe -m examples/sysmlv2-submodel/model validate --config CONF-QUAD-DRONE-001
```

Its own `README.md` documents every expected warning and a few real, general (not SysMLv2-specific)
gaps a realistic example surfaced along the way — worth reading if something in a real project's
own SysMLv2 submodel looks unexpectedly noisy.

## 6. What's not built yet

Explicitly out of scope, tracked as follow-on if a concrete need arises:

- **A writer/serializer** back into `.sysml`/`.kerml` text, or any two-way round-trip authoring.
  The SysMLv2 subtree stays authoritative; Syscribe only ever reads it.
- **Full SysML v2 static semantic validation** — type-checking, multiplicity legality,
  standard-library-aware inheritance. The AST-only parser used here resolves cross-boundary
  references through Syscribe's own resolver, not SysML v2 semantic legality; that stays a
  standards-compliant tool's (e.g. `spec42`) job, run separately.
- **`extend` of use cases** — SysML v2 has no `extend`; an `include X;` whose simple name resolves
  to an ingested use case is mapped to `includes:` (§25), including a qualified target
  (`include Q::Land;`) and the declaring form (`include use case v : Q::Land;`, `REQ-TRS-SYSMLV2-075`).

## 7. `doc /* ... */` comment lift

A `part def`/`part`/`interface def`/`interface` (usage)/`port def`/`port`/`connection def`/
`attribute def`/`attribute`/`item def`/`item` may declare one or more `doc /* ... */` members.
The text lifts into the synthesized element's `doc` body — the same field a hand-authored `.md`
file's body below its `---` closer populates (`REQ-TRS-SYSMLV2-009`):

```sysml
part def RotorAssembly {
    doc /* The primary rotor/motor/battery propulsion chain. */

    port fuelSupplyPort : FuelPort;
}
```

`W600`/`W601`-style empty-doc-body warnings apply unchanged: this `RotorAssembly` clears `W600`
exactly as a hand-authored `PartDef` with the same body text would. A `part def`/etc. with no
`doc` member is unaffected — `doc: ""`, `W600` still fires, no regression.

**Multiple `doc` blocks concatenate**, in source order, joined by a blank line — the grammar
permits several, and there's no reason to silently drop any of them:

```sysml
part def CarSafetyServices {
    doc /* First paragraph. */
    doc /* Second paragraph. */
}
```

lifts to `doc: "First paragraph.\n\nSecond paragraph."`. Each block's own text is trimmed of the
incidental whitespace directly adjacent to `/*`/`*/` (delimiter padding, not content) before
joining, and a block that trims to nothing (`doc /* */`) is dropped rather than leaving a stray
blank line; internal formatting within a single block is left untouched — otherwise verbatim, no
Markdown rendering or reflow.

`variant part`/`variant attribute`/`variant port`/`variant item` usages, a plain `item` usage, and
a plain `interface` usage all lift their own `doc` block the same way their def counterparts do:
`ItemUsage.body` is an `AttributeBody`, the same shared shape `AttributeDef`/`AttributeUsage`/
`ItemDef` already use, and `InterfaceUsage`'s own `body_elements` carries its own
`InterfaceUsageBodyElement::Doc` variant, distinct from (but handled the same way as)
`InterfaceDef`'s `InterfaceDefBodyElement::Doc`.

A named `connection name : Type connect a to b { doc /* ... */ }` usage's own trailing body lifts
the same way (`REQ-TRS-SYSMLV2-012`) — reusing `connection def { }`'s own doc-reading logic
unchanged, since `ConnectionUsageMember.body` is the identical `ConnectionDefBody` shape:

```sysml
connection carDisplayToCompositor : DisplayLink connect carDisplayService to compositor {
    doc /* Over Interfaces::Display::ICompositorControl. */
}
```

lifts onto the synthesized `Connection` element, not the owning part — distinct from, and
independent of, `REQ-TRS-SYSMLV2-010`'s endpoint lift onto the *owning part's* `connections:`
field. A connection usage with no trailing body is unaffected.

## 8. Connection-endpoint lift

A `part def`/`part`'s named `connection name : Type connect a to b (, c)*;` usage member lifts its
endpoints onto the **owning** `part def`/`part`'s `connections:` field — the same field a
hand-authored `.md` file's `connections:` populates — so `connectivity` (and unscoped `n2`; see
the limitation disclosed below) show real, resolvable wiring for a `sysmlSubmodel: true` subtree
(`REQ-TRS-SYSMLV2-010`). Covers a `connection` nested inside a `variant part` usage too, lifting
onto that usage's own `connections:` the same way an ordinary `part`/`part def` does.

```sysml
part def Holder {
    part a : Ecu;
    part b : Ecu;

    connection c : SomeConnDef connect a.p1 to b.p1;
}
```

lifts a `connections: [{typedBy: SomeConnDef, from: <qname>::a, to: <qname>::b}]` entry onto
`Holder` — **not** onto the nested `c` element, which is still synthesized unchanged
(`REQ-TRS-SYSMLV2-007`'s existing mapping). The n-ary `connect (a, b, c)` form lifts to the same
`ends: [{binds: ...}, ...]` shape a hand-authored n-ary entry already uses.

**Endpoints are qualified to `<owning qname>::<head>`, not carried verbatim.** A literal
`{from: "a.p1", ...}` — what the `.sysml` source text itself says — never resolves to a graph
edge in this codebase (confirmed by investigation before implementing, not assumed): the
connection-edge resolver only matches an exact full qname or a `features:`-declared head, neither
of which a SysMLv2-synthesized part ever has. Only the chain's first segment is kept — `a.p1`
under `Holder` becomes `Holder::a`, dropping `.p1` — matching this same resolver's own existing
precedent for `features:`-declared endpoints exactly (head resolved, everything past it
discarded). See `ADR-SYS-SYSMLV2-001`'s addendum for the full two-round investigation. (§10 below
widens this one step further — a trailing segment isn't *always* discarded any more, only when
nothing local resolves it.)

A named connection usage with no `connect` clause (`connection c : SomeConnDef;`) contributes no
entry, unaffected. The anonymous binary-connector form (no `connection name :` prefix) stays
unmapped — no identity to synthesize an entry against, consistent with the module's existing
precedent for other anonymous forms.

**Remaining disclosed limitation:** `n2`'s own edge-collection reads only the first two ends of
any n-ary connection (native or SysMLv2-lifted alike), so a three-way `connect (a, b, c)` shows
`a`↔`b` but not `a`↔`c` in `n2`; `connectivity` correctly builds the full star. A pre-existing
`n2.rs` characteristic this requirement doesn't touch. (An earlier, now-resolved limitation —
scoped `n2 <qname>` reporting no parts at all for any SysMLv2 subtree — is fixed by
`REQ-TRS-SYSMLV2-011`, §9 below.)

## 9. `n2`'s scoped axis includes SysMLv2-synthesized children

`n2 <qname>`'s subpart axis previously came exclusively from the scope element's own `features:`
list — the native-Markdown convention for declaring inline-typed subparts. A SysMLv2 element's
subparts are separate, qname-nested elements instead (§2's containment mapping), never
`features:` entries, so scoped `n2` on any SysMLv2 subtree reported `(no parts in scope)`
regardless of how much real `connection` wiring it contained (`REQ-TRS-SYSMLV2-011`):

```
$ ./target/debug/syscribe -m examples/sysmlv2-submodel/model n2 \
    PropulsionSubsystem::Propulsion::Drone
N² Interface Matrix — PropulsionSubsystem::Propulsion::Drone (depth 1)

               rotorConfig
rotorConfig    ■
```

`n2`'s axis-selection now additionally includes every direct-child `PartDef`/`Part` by qname
containment, alongside the existing `features:` source (the two are additive and de-duplicated).
`powerPort` still doesn't appear — `n2`'s axis stays `PartDef`/`Part`-only, unchanged; a `Port`
was never in scope for it. A `REQ-TRS-SYSMLV2-010`-lifted connection between two qname-contained
parts now populates the off-diagonal cell the same way a `features:`-declared one always did.
Unscoped `n2` (already `Part`/`PartDef`-inclusive regardless of origin) and a `features:`-only
hand-authored model are both unaffected — this is a strict widening, not a SysMLv2-only special
case (a hand-authored model that happens to nest a `PartDef`/`Part` as a real child file, rather
than an inline `features:` entry, gains the same axis inclusion).

## 10. Resolving a dotted `connect` endpoint to a redeclared nested feature

§8's head-only qualification is the reliable default, but it's needlessly lossy when a `.sysml`
author explicitly redeclares the referenced feature on the usage itself, rather than only
inheriting it from the type (`REQ-TRS-SYSMLV2-013`):

```sysml
part def Top {
    part a : A {
        interface fooProvider : IFoo;
    }
    part b : B {
        interface fooClient : IFoo;
    }

    connection link1 : Link connect a.fooProvider to b.fooClient;
}
```

lifts the full-precision edge `Top::a::fooProvider -> Top::b::fooClient` — not just
`Top::a -> Top::b` — because `a`'s own body genuinely redeclares `fooProvider`, and `b`'s own body
genuinely redeclares `fooClient`. This resolution is purely **local**: for a two-segment chain
(`head.tail`, no further `.`), the owning body is searched for a `part` usage named by the head,
and *that* usage's own already-parsed body is searched for a direct
`port`/`attribute`/`interface`/nested-`part` child named by the tail — no resolver, no global
element list, no inheritance reasoning of any kind. Whenever that lookahead doesn't find a match
(the overwhelmingly common case — an inherited-only feature, a chain of three or more segments, or
a head that isn't itself a `part` usage in the same body), the endpoint falls back to §8's
existing head-only qualification exactly as before — a strict widening, never a new failure mode.

## 11. `W542` — a truncated `connect` endpoint is no longer silent

§10's redeclaration lookahead only reaches a feature *explicitly redeclared* on the usage — the
far more common case (a feature *inherited* from the head's type, e.g. `part carDisplayService :
Services::CarDisplayService;` where `CarDisplayService` declares the interface, never redeclared
on the usage) still falls back to §8's head-only qualification, exactly as before. As of
`REQ-TRS-SYSMLV2-015`, that fallback is no longer silent: whenever a genuinely two-segment chain
(`head.tail`, no further `.`) can't be resolved via §10's lookahead, a `W542` finding identifies
the dropped segment:

```
$ ./target/debug/syscribe -m <model> validate
| W542 | .../Model.sysml | connect endpoint 'a.p1' has no locally-redeclared 'p1' feature on 'a'
                            -- truncated to the head-only edge 'Top::a' (a feature inherited from
                            'a's type, rather than redeclared on the usage, cannot be verified
                            without a full-model resolver; see REQ-TRS-SYSMLV2-013/-015) |
```

A chain that resolves via §10's lookahead, a bare (undotted) endpoint, and a three-or-more-segment
chain all raise no `W542` — the three-plus-segment case is §10's own separate, deliberate,
still-unwarned fallback (extending the lookahead to walk multiple levels was rejected as
unnecessary complexity), not something this requirement revisits. Full resolution through the
inherited type (rather than only warning) was considered and rejected: it needs the head's type's
full definition, which may live in a different file and isn't available as a synthesized element
yet at the single-file, ingest-time point this resolution runs at — the same reason §10's own
lookahead stayed purely local rather than reaching for a resolver.

## 12. Scoped resolution for `typedBy:` — `W600`'s documentation fallback across packages

The general validator's `W600` ("PartDef/Part has an empty documentation body") suppression — a
`Part` usage whose `typedBy:` target itself carries documentation doesn't also need its own —
originally only resolved a `typedBy:` reference by an exact, already-fully-qualified qname match.
A SysMLv2-authored `part x : Services::Documented;` written inside `package System { ... }`
produces the literal, *package-relative* text `"Services::Documented"` on `x`'s `typedBy:` — not
`Documented`'s real full qname (`SysML2::Services::Documented`) — since `ingest.rs` performs no
resolution of its own at parse time. The exact-match lookup only happened to succeed when a
`.sysml` file's content stayed in a single package; the moment SysMLv2 content spans more than one
package (the ordinary shape of a real, multi-file architecture submodel), the suppression stopped
firing where it should (`REQ-TRS-SYSMLV2-016`).

`Resolver::resolve_scoped_ref` now searches outward through the referencing element's own
enclosing-package scope chain — innermost first, down to the model root — before falling back to
the original exact/id/display-name lookup, so `Services::Documented` written inside `System`
resolves to `SysML2::Services::Documented` correctly. Originally scoped narrowly to `W600`'s
suppression check; `graph.rs`'s `TypedBy` edge and `W007`'s "never used as a supertype or type"
tracking were widened the same way next (`REQ-TRS-SYSMLV2-017`, below). The `mutate::guard`
dangling-`typedBy:` check (`EREF`) remains on the plain, unscoped lookup — see
`ADR-SYS-SYSMLV2-001`'s addenda for why each call site was widened (or deliberately deferred) on
its own.

## 13. Widening scoped resolution to `W007` and `graph.rs`'s `TypedBy` edge

A real, multi-file `.sysml` submodel — where splitting interfaces, services, and system
composition into separate packages is the normal, encouraged shape — hit the same root cause as
§12 in two more places: `W007` ("defined but never used as a supertype or type") flagged a `*Def`
as unused whenever its only reference was a cross-package, package-relative `typedBy:`/`supertype:`
(confirmed against a real CarOS/`sabaton-caros` conversion: 35 of 36 `W007` warnings there were
this exact false positive), and `graph.rs`'s `TypedBy` edge — which did a bare exact-qname
`idx.get` lookup, narrower even than plain `resolve_ref` — silently produced no edge at all for the
same reference, so `connectivity`/`n2`/`impact` never traversed it.

`REQ-TRS-SYSMLV2-017` routes both through `resolve_scoped_ref`, the same widening §12 already made
for `W600`. `exhibitsStates:` is deliberately left on the plain `resolve_ref` — it is never
synthesized by SysMLv2 ingestion, so it is always already fully qualified from the model root. The
`Supertype` graph edge and the `mutate::guard` dangling-`typedBy:` check (`EREF`, gating MCP
guarded-write commits) are not widened by this requirement — a write-path guard rail deserves its
own scrutiny, separate from a read-path validator warning or graph traversal.

## 14. State machines and actions — `REQ-TRS-SYSMLV2-018`/`-019`

`state def`/`state` and `action def`/`action` join the fixed mapped set, becoming real `StateDef`/
`State`/`ActionDef`/`Action` elements — see [State Machines](state-machines.md) for the full native
target schema this mapping produces. A top-level `state`/`action` usage (declared directly in a
package or part) becomes its own real, qname-addressable element; a `state`/action-body construct
found *nested inside* another `StateDef`/`ActionDef`'s own body becomes inline YAML data only
(`subStates:`/`subActions:`/`controlNodes:`), never a separate element — matching how a
hand-authored composite state machine or activity is already written.

```sysml
state def FlightStates {
    state disarmed {
        transition first disarmed accept StartCmd then armed;
    }
    state armed;
    then disarmed;
}
action def MissionExecution {
    action takeoff;
    action navigate;
    first takeoff then navigate;
}
```

synthesizes `subStates:`/`transitions:` and `subActions:`/`successionConnections:` in exactly the
shape a hand-authored `FlightStates.md`/`MissionExecution.md` would use — the existing `W070`–`W080`
completeness checks apply identically, with no validator changes at all.

**A real, non-negotiable ceiling**: `fork`/`join`/`decide`/`merge` block bodies are parsed by
`sysml-v2-parser` and then discarded by the parser itself — their `{...}` contents carry no data to
recover, at any pinned version. They become flat `controlNodes:` markers (`{name, kind}` only, no
internal content) — not a Syscribe scope choice, an upstream parser fact. Guard/condition text for
the long tail of `Expression` shapes this crate doesn't specially recognize (`Classification`/
`Select`/`Collect`/`Conditional`/…) falls back to a fixed placeholder rather than vanishing — a
Syscribe-owned, revisitable-later limitation, explicitly distinct from the fork/join ceiling above.
See `ADR-SYS-SYSMLV2-001`'s addendum for the full rationale.

## 15. Views, viewpoints, and renderings — `REQ-TRS-SYSMLV2-020`/`-021`/`-022`

`view def`/`view`, `viewpoint def`/`viewpoint`, and `rendering def`/`rendering` join the fixed
mapped set — see [`model/Viewpoints/SystemsEngineerViewpoint.md`](https://github.com/sjames/syscribe/blob/main/model/Viewpoints/SystemsEngineerViewpoint.md)
and [`model/Views/SystemArchitectureView.md`](https://github.com/sjames/syscribe/blob/main/model/Views/SystemArchitectureView.md) for the
native target schema this mapping produces. Every one of the six kinds, wherever declared, becomes
its own real, qname-addressable element — unlike state machines/activities, there's no "nested vs.
top-level" split, since none of these six carry a further, separate `RawElement` inside their own
body.

```sysml
package Views {
    viewpoint def SafetyViewpoint {
        stakeholder SafetyEngineer;
        purpose SafetyCoverage;
    }
    rendering def TableRendering;
    view def SystemView {
        render asTable : TableRendering;
    }
    view archView : SystemView {
        expose UAV::Airframe;
        expose UAV::Propulsion::*;
        satisfy SafetyViewpoint;
    }
}
```

synthesizes a `ViewpointDef` with `stakeholders:`/`concerns:`, a `RenderingDef`, a `ViewDef` with
`rendering:`, and a `View` with `expose:`/`viewpoint:`/`rendering:` — exactly the shape a
hand-authored `SystemArchitectureView.md` uses. `expose:` entries are always flat qname strings
(never a `{ref, isRecursive, filter}` map), matching real hand-authored usage; the existing `W500`/
`W502` cross-reference checks apply identically, with no validator changes at all.

**Structural asymmetries, not oversights**: a `view def`'s own body cannot syntactically carry
`expose`/`satisfy` at all — only a `view` usage can (see `ADR-SYS-SYSMLV2-001`'s addendum for why
this lines up with `W500`/`W502`'s existing scope rather than fighting it). There is no dedicated
`Viewpoint` usage element kind; a `viewpoint <name> defined by <Type>;` usage synthesizes a `View`,
matching the native schema's own framing of `View` as "usage of a ViewDef or ViewpointDef".
`ViewpointDef`'s `methods:`/`satisfiedBy:` fields are never populated by this mapping — deliberately,
per §12.1's OSLC upstream-link-direction rule, not because the information is unavailable. A
`view`/`viewpoint`/`rendering` declared directly inside a `part` usage body doesn't just stay
unmapped — it fails to parse outright, gracefully degrading to a `W541` finding (§4) rather than a
crash, since `PartUsageBodyElement` carries no grammar production for the whole family at all.

## 16. Concerns — `REQ-TRS-SYSMLV2-023`

`concern def`/`concern` joins the fixed mapped set — the direct follow-on to §15's Viewpoint work:
`ViewpointDef.concerns:`/`RequirementDef.concerns:` are native fields, but until this mapping
nothing existed for them to reference. `ElementType::ConcernDef`/`ElementType::Concern` already
existed in the native schema; this mapping is what finally makes them reachable.

```sysml
package Concerns {
    concern def BaseConcern;
    concern def MassConcern : BaseConcern {
        subject vehicle : UAV::UAVSystem;
        stakeholder ChiefEngineer;
    }
    concern massBudgetConcern : MassConcern;
}
```

synthesizes a `ConcernDef` (`MassConcern`, `supertype: BaseConcern`, `subject: UAV::UAVSystem`,
`stakeholders: [ChiefEngineer]`) and a `Concern` (`massBudgetConcern`, `typedBy: MassConcern`) —
the existing hand-authored `ConcernDef`/`Concern` schema, just never previously exercised anywhere
in `model/`.

**A structural quirk worth knowing**: the vendored parser has no separate `ConcernDef` AST struct —
one `ConcernUsage` node parses both `concern def X` and `concern x` forms, `is_definition`
discriminating them, and the *same* `: Y` clause means a supertype for the definition form but a
typedBy for the usage form (see `ADR-SYS-SYSMLV2-001`'s addendum for the full parser-level
rationale). A `concern`/`concern def` declared inside *any* `part`/`part def` body — not just a
`part` usage body, unlike View/Viewpoint/Rendering — fails to parse outright, degrading to `W541`
(§4). `requires:`/`assume:`/`parameters:` are not lifted by this mapping (no expression-rendering
work has been built for `RequireConstraint`'s nested content yet, for any element kind); and no new
validator check resolves `concerns:` entries against real `ConcernDef`s — both existing
hand-authored Viewpoint files write `concerns:` as free prose today, not qnames, so adding one now
would immediately fire on correct, already-committed content.

## 17. Flows — `REQ-TRS-SYSMLV2-024`

`flow def`/`flow` joins the fixed mapped set — cross-referencing
`model/Flows/PowerFlowDef.md`/`TelemetryFlowDef.md` as the target hand-authored shape.

```sysml
package Flows {
    flow def PowerFlow;
    part def Battery { port out; }
    part def Motor { port in; }
    part def Drone {
        part battery : Battery;
        part motor : Motor;
        flow battery.out to motor.in;
        message alertEvt : Fault from battery to motor;
    }
}
```

synthesizes a `FlowDef` (`PowerFlow`) and, on the owning `Drone` part, a `flowConnections:` list
with two entries — one from the anonymous `flow battery.out to motor.in;` (`kind: streaming`, no
`name:`) and one from the named `message alertEvt : Fault from battery to motor;` (`kind: message`,
`name: alertEvt`, `item: Fault`) — matching §8.6.2's `flowConnections:` sub-schema exactly. Because
the named one has a name, it *also* becomes its own standalone `Flow` element
(`Drone::alertEvt`, `itemType: Fault`) — the same dual element-plus-lift pattern §8's `connections:`
lift already established for named `connection` usages nested in a part.

**A parser-grammar quirk worth knowing, found only by testing against real parsed output**: a
`flow`/`message` endpoint like `battery.out` parses as `Expression::MemberAccess`, not
`Expression::FeatureChainRef` — a different shape than a plain `connect` endpoint's identical-looking
`a.p1` syntax uses (see `ADR-SYS-SYSMLV2-001`'s addendum). `ends:`/`itemType:` (§8.6.1's `FlowDef`
fields, matching `model/Flows/PowerFlowDef.md`'s own shape) are **not** derived from a `flow def`'s
own body — the vendored parser gives no unambiguous "this nested member is an end port" signal to
extract them from. `payload.multiplicity` (`of qty : Payload[1..3]`) is not lifted either (no
multiplicity-to-string renderer exists yet). A `flow` nested inside an `action def` body stays
invisible, unchanged — already excluded by §14's own scope, not something this section touches.

## 18. Enumerations — `REQ-TRS-SYSMLV2-025`

`enum def`/`enum` joins the fixed mapped set — cross-referencing
`model/Enumerations/ArmStatus.md`/`FlightMode.md` as the target hand-authored shape. The simplest
mapping in this series: no cross-element lift like Flow, no struct-folding like Concern.

```sysml
package Enums {
    enum def ArmStatus {
        enum disarmed;
        armed;
        fault;
    }
    part def Vehicle {
        enum armStatus : ArmStatus;
    }
}
```

synthesizes an `EnumerationDef` (`ArmStatus`, `values: [{name: disarmed}, {name: armed}, {name:
fault}]` — the `enum` keyword prefix is optional per the grammar, both forms shown above) and, nested
inside `Vehicle`, an `Enumeration` (`armStatus`, `typedBy: ArmStatus`) — the existing hand-authored
`EnumerationDef`/`Enumeration` schema, just never previously exercised by SysMLv2 ingestion.

**A first for this mapping series, worth knowing**: `enum def`'s body (`EnumerationBody`) carries no
`doc /* ... */` capability at all — every other mapped kind so far had at least *some* path to a
`Doc` variant in its body type, even Flow's indirect one. A `doc` comment written inside an `enum
def` has nowhere to land in this parser version; the synthesized `EnumerationDef`'s `doc` stays
empty, unconditionally. Each literal also carries only its `name` — an initializer
(`low = 0.25;`) is parsed and discarded by the vendored parser itself, so a `values:` entry is
always just `{name: ...}`, never the spec's richer optional `value:`/`valueKind:`/`unit:`/
`metadata:` sub-fields. `Enumeration` (the usage) has no documented schema of its own beyond
`typedBy:`/`doc` — `multiplicity`/the rare `end enum` prefix form aren't lifted.

## 19. Cases, analysis cases, verification cases — `REQ-TRS-SYSMLV2-026`/`-027`/`-028`

`case def`/`case`, `analysis def`/`analysis`, and `verification def`/`verification` join the fixed
mapped set — the "case family" — deliberately **excluding** `use case def`/`use case`, which stays
unmapped (see §2's example above).

```sysml
package Cases {
    part def System;
    part def Pilot;
    attribute def VerdictKind;
    verification def SafetyVerification {
        doc /* Confirms the safety case holds. */
        subject sys : System;
        actor pilot : Pilot;
        objective safetyObjective : VerdictKind;
        return verdict : VerdictKind;
    }
    analysis def PerformanceAnalysis {
        subject sys : System;
        return thrust : System;
    }
}
```

synthesizes a `VerificationCaseDef` (`subject: System`, `actors: [Pilot]`, `objectives:
[safetyObjective]`, `result: VerdictKind`, doc lifted) and an `AnalysisCaseDef` — the existing
hand-authored `CaseDef`/`AnalysisCaseDef`/`VerificationCaseDef` schema (§8.12), never previously
exercised by SysMLv2 ingestion and, unlike most kinds in this series, never exercised by a
hand-authored `model/` example either — this mapping follows the spec text and the AST directly.

**All six constructs (`Case`/`AnalysisCase`/`VerificationCase`, each Def and Usage) share exactly
one AST body type** — a real, structural confirmation of SysMLv2's own specialization hierarchy
(`AnalysisCase`/`VerificationCase` specialize `Case`), not a coincidental resemblance. **A real
reachability asymmetry worth knowing**: only `analysis def`/`analysis` can be declared directly
inside a `part` *usage* body — `case`/`verification` fail to parse there outright, degrading to
`W541` (§4), the same posture Concern's Package-only gap established. `verifies:`/
`verdictExpression:`/`verdictType:` (§8.12.3's `VerificationCaseDef`-specific fields) are **not**
lifted — the grammar this pinned parser version implements for case bodies carries no verify-statement
or verdict-semantics content at all, confirmed against the vendored crate's own compliance-matrix
caveat marking this family's body-depth coverage `partial`.

## 20. Allocation definitions — `REQ-TRS-SYSMLV2-029`

`allocation def` joins the fixed mapped set as a native **`AllocationDef`** — the existing
`ElementType` for SysMLv2's `allocation def` (the native schema already carried it; ingestion
simply never produced one). It is mapped both at package level and nested in a `part def` body
(a `part` *usage* body has no allocation variant at all in this parser version):

```sysml
package Deploy {
    allocation def SoftwareToHardware {
        doc /* Deploys a software component onto a hardware board. */
    }
    part def Rack {
        allocation def RackSlot;
    }
    allocation deployCtl : SoftwareToHardware;   // typedBy: SoftwareToHardware -> resolves
    allocation slotUse : Rack::RackSlot;          // typedBy: Rack::RackSlot -> resolves
    allocation broken : NoSuchAllocationDef;      // E111
}
```

synthesizes `Deploy::SoftwareToHardware` and `Deploy::Rack::RackSlot` as `AllocationDef`s
(`supertype:` from a `:>` clause, `doc` lifted by the same helper `flow def` uses — the two share
one thin `DefinitionBody` AST type). Because the definition now exists in-model, an ingested
`allocation` usage's `typedBy:` is checked like every other `typedBy:` (`E111`, §4 of the
[validation rules](../validation/rules.md)) — previously every ingested `Allocation` was exempted
from `E111`, which also hid genuinely dangling types (GH #142). A usage typed by a standard-library
name (`Allocations::Allocation`, bare `Allocation`) is a library reference and is never flagged.

### The `allocate` clause becomes an allocation edge (GH #144)

A named allocation usage's `allocate <source> to <target>` clause is lifted onto the synthesized
`Allocation` as `allocatedFrom: [<source>]` / `allocatedTo: [<target>]` — a §12.9 **form 2**
allocation — so an ingested allocation feeds the unified allocation-edge set exactly like a
Markdown `Allocation` element: `E314` (deployment allocation), `W034` (freedom from
interference), `W503` (redundancy), `matrix --allocations` and the derived `allocatedFrom` index.

```sysml
package Deploy {
    part def Controller { part ctl : CtlSw; }
    part def Hw { part mcu : Mcu; }
    part sys : Controller;
    part board : Hw;

    allocation deploy : SoftwareToHardware allocate Arch::SwPackage to Arch::Board;
    allocation chained allocate sys.ctl to board.mcu;
    allocation truncated allocate sys.nosuch to board;   // W542, edge sys -> board
    allocation dangling allocate NoSuchSource to NoSuchTarget;   // E502 / E503
}
```

Unlike `connect` lifting (§8/§10, a purely local AST lookahead), allocation endpoints are
resolved **after** the whole model is merged, because they routinely cross packages and name
features a usage inherits from its type:

| Endpoint | Resolution |
|---|---|
| `Arch::SwPackage` | Head resolved innermost scope first — `Deploy::Arch::SwPackage`, then each enclosing namespace, then the model root — so a native Markdown element resolves too |
| `sys.ctl` | Head `sys` → `Deploy::sys`; `ctl` is not declared on the usage, so it is looked up through `sys`'s `typedBy:`/`supertype:` chain → `Deploy::Controller::ctl` |
| `sys.nosuch` | No such feature anywhere on the chain: truncated to the deepest resolved prefix (`Deploy::sys`) with a `W542` naming the endpoint |
| `NoSuchSource` | Resolves nowhere: kept as written (`.` → `::`), reported by `E502` (`allocatedFrom`) / `E503` (`allocatedTo`) |
| `'REQ-X-001'` | A stable id is global and kept verbatim |

The optional `end ::>` end-name prefix (`allocate logical ::> a to physical ::> b`) is accepted and
ignored. The **anonymous** `allocate a to b;` statement has no identity to synthesize an
`Allocation` element against and stays unmapped — give it a name (`allocation n allocate a to b;`)
or author the edge natively (`allocatedTo:` on the source).

---

## 21. Inspecting a submodel — `syscribe sysml` / MCP `sysml_submodels` (`REQ-TRS-SYSMLV2-031`/`-032`)

```bash
syscribe -m model/ sysml          # human-readable report
syscribe -m model/ sysml --json   # machine-readable
```

For each `sysmlSubmodel: true` package the report shows the files parsed (and whether each
parsed — a failure is also a `W541`), the ingested element counts per kind, the unmapped-construct
counts per kind (the data behind `W543`) and the `W540`-`W543` findings. It is read-only and exits
zero, also when the model has no submodel (the JSON then has an empty `submodels` array).

```json
{"submodels": [{"package": "model::PropulsionSubsystem", "fileCount": 3, "filesParsed": 3,
  "elementTotal": 26, "elementsByKind": {"PartDef": 3, "Port": 3},
  "unmappedTotal": 0, "unmapped": {}, "findings": []}]}
```

The read-only MCP tool `sysml_submodels` (no arguments) returns exactly the same JSON from the
server's in-memory model. The data comes from `syscribe_model::sysmlv2::report`.


## 22. Constraints, calcs, use cases and package docs — `REQ-TRS-SYSMLV2-033`..`-036`

`constraint def`/`constraint`, `calc def`/`calc`, `use case def`/`use case` and a package-level
`doc` are ingested into the existing native types, and `W543` no longer counts them.

```sysml
package Analysis {
    doc /* Mass and economy analysis. */          // -> the Analysis package's doc text
    constraint def MassLimit {
        in actualMass : Real;
        in maxMass : Real;
        actualMass <= maxMass                       // -> expression: "actualMass <= maxMass"
    }
    constraint massCheck : MassLimit;               // -> Constraint, typedBy: MassLimit
    calc def FuelEconomy {
        in distance : Real;
        in fuel : Real;
        return economy : Real;                      // -> returnType: Real
        distance / fuel                             // -> body: "distance / fuel", bodyLanguage: kerml
    }
    use case def Drive { subject v : Vehicle; actor d : Driver; }
}
```

| SysMLv2 | Native | Lifted fields |
|---|---|---|
| `constraint def` / `constraint` | `ConstraintDef` / `Constraint` | `supertype:` or `typedBy:`, `parameters:`, `expression:` (opaque string), doc |
| `calc def` / `calc` | `CalculationDef` / `Calculation` | `typedBy:`, `parameters:` (incl. `direction: return`), `returnType:`, `body:` + `bodyLanguage: kerml`, doc |
| `use case def` / `use case` | `UseCaseDef` / `UseCase` | `supertype:`/`typedBy:`, `subject:`, `actors:`, `objectives:`, `result:`, `isAbstract:`, doc |
| `doc /* ... */` in a package | the `Package` element | doc text |

Expression text is kept as an opaque string and never evaluated. Where the parser allows them:
package level (constraint/calc def, use case def/usage, constraint usage); a `calc`/`use case`/
`constraint` usage in a `part def` body; a `constraint` usage in a `part` usage body. A non-`draft`
ingested `UseCaseDef` raises the existing advisory `W307` (no `refines:`). Not lifted: nested
constraint members, `assert`/negation, `include`/`extend` (see §24).


## 23. Exporting native elements as SysML v2 text — `export-sysml` (`ADR-SYS-SYSMLV2-002`, `REQ-TRS-SYSMLV2-037`..`-042`)

Ingestion is read-only; **export** is a separate, one-way writer for handing a Syscribe model to
SysML v2 tooling:

```bash
syscribe -m model/ export-sysml                       # whole model to stdout
syscribe -m model/ export-sysml UAV --out uav.sysml   # one package subtree to a file
syscribe -m model/ export-sysml --out out/            # one .sysml per top-level package
```

| Syscribe | SysML v2 |
|---|---|
| directory / `Package` | nested `package` (missing levels become implicit packages) |
| `PartDef` / `Part` | `part def` / `part` (`supertype` -> `:>`, `typedBy` -> `:`, `multiplicity` -> `[n]`, `isAbstract` -> `abstract`) |
| `PortDef`/`Port`, `AttributeDef`/`Attribute`, `ConnectionDef`/`Connection`, `InterfaceDef`/`Interface`, `ItemDef`/`Item` | the matching `... def` / usage |
| `RequirementDef`, native `Requirement` | `requirement def` (native ones named by their stable id, e.g. `'REQ-X-001'`; body as `doc /* */`; `verifies:` -> `verify <target>;`) |
| `ActionDef`/`Action`, `StateDef`/`State` bodies | the statements ingestion reads back (§26, REQ-TRS-SYSMLV2-056..058); anything it would not read back identically is a `//` comment |
| `ConstraintDef`/`CalculationDef` and usages | header, doc, `in`/`out`/`return` parameters and the `expression:`/`body:` text (REQ-TRS-SYSMLV2-052) |
| `satisfies:` on a part | `satisfy <target>;` in its body |
| `satisfies:` on any other element | a package-level `satisfy <target> by <element>;` (REQ-TRS-SYSMLV2-051) |
| usage `subsets:` / `redefines:` | `:> a, b` / `:>> a` after the typing and multiplicity (REQ-TRS-SYSMLV2-049) |
| inline `features:` / `connections:` on a part | `attribute`/`port` members (a numeric `value:` with `unit:` becomes `= 5 [kg]`), `connection ... connect a.x to b.y;` |
| anything else (`TestCase`, `ADR`, `PlanningItem`, `FeatureDef`, ...) | `// skipped: <qname> (<type>)` and a count in the summary |

The export is lossy and deterministic; re-importing it into a `sysmlSubmodel: true` package
reproduces the supported kinds and qnames (a native `Requirement` returns as a `RequirementDef`),
which is exactly what the parse-back tests check. The MCP tool `export_sysml {package?}` returns
the same text without writing anything. The writer is `syscribe_model::sysmlv2::export`.

## 24. Closing ingestion gaps — `REQ-TRS-SYSMLV2-043`..`-048`

More of what the parser exposes now lands in a native target; each stops counting toward `W543`.

| SysML v2 | Native | Notes |
|---|---|---|
| `alias m for X;` in a named package | `aliases: [{name: m, for: X}]` on that `Package` | the field the scoped resolver already reads (spec 3.7.2); `<s>` short names become `shortName`; a **root-level** alias has no package to carry it and stays counted |
| `library package` / `namespace` | `Package` | at the file root or nested; same-named declarations merge; the `standard` flag is dropped |
| `metadata def N :> S` | `MetadataDef` | `supertype:`, `isAbstract:`, doc; `metadata` *usages* stay unmapped |
| `satisfy R by X;` at package level | `X`'s `satisfies:` gains `R` | `X` is resolved innermost-scope-first after the whole subtree is merged; an unresolved subject, the bare `satisfy R;` shorthand, a negated or inline one stay counted as `satisfy` |
| `doc /* */` in `requirement def` / `requirement` | the element's doc text | joined and trimmed like every other doc lift |
| `part`/`attribute`/`port`/`item` usage `[m]`, `:>`, `:>>` | `multiplicity:` (`2`, `0..1`, `1..*`, `*`), `subsets:`, `redefines:` | resolved by the `E112`/`E113` structural checks like `typedBy:` |

```sysml
package P {
    part def Axle;
    part def Car {
        part wheels : Axle [2];
        part spare  : Axle [0..1] :> wheels;      // multiplicity: "0..1", subsets: [wheels]
    }
    alias Wheelset for Car;                        // P gains aliases: [{name: Wheelset, for: Car}]
    satisfy 'REQ-X-001' by Car;                    // Car gains satisfies: ['REQ-X-001']
}
```

**Deliberately not mapped** (still `W543`): `extend` of use cases (SysML v2 has none) and an
`include` whose name does not resolve (see §25); `metadata` usages, `occurrence`, `individual def`, package-level `actor`, `dependency`,
`filter`, textual representation and the KerML declaration forms have no native target. `item`
usages are limited by the parser to `item name [m] : T` (a trailing `T[m]` is not read), and a
package-level `item` usage is read without its typing. No new validation code was added: an
advisory for a usage typed by a non-definition was evaluated and skipped, since the native format
does not enforce that rule for hand-authored elements either. `syscribe sysml`/`sysml_submodels`
take their counts from the ingestion pass itself, so an unresolved package-level `satisfy` (or
`include`) appears in the report exactly as in `W543` (REQ-TRS-SYSMLV2-059).

## 25. `include`, attribute values/units, parser version — `REQ-TRS-SYSMLV2-053`..`-055`, `-059`

| SysML v2 | Native | Notes |
|---|---|---|
| `include Pay;` / `then include Pay;` in a `use case def`/`use case` body | `includes: [<qname of Pay>]` | `Pay` is resolved innermost-scope-first against the ingested `UseCaseDef`/`UseCase` elements; an unresolved name (or one naming a non-use-case, or the use case itself) is dropped and counted as `include` in `W543` |
| `attribute mass : Real = 12.5 [kg];` (in a `part def`/`part` body) | `Attribute` with `value: 12.5`, `unit: kg` | only *literal* values (number, string, boolean) map; any other expression leaves `value:` unset. `unit:` is a standard element field (the spec's inline-feature shorthand) |

`export-sysml` writes `unit:` back as `= 12.5 [kg]`. A package-level `attribute x : T = v;` is an attribute usage on the pinned parser, like any other.

`syscribe sysml` prints `Parser: sysml-v2-parser <version> (AST <n>)` and `--json`/MCP `sysml_submodels`
carry `parser: {name, version, astVersion}`. A test compares the reported version with the pin in
`crates/syscribe-model/Cargo.toml`, so it cannot go stale.

**Parser version.** `sysml-v2-parser` is pinned at 0.57.0 (`REQ-TRS-SYSMLV2-073`). Behaviour is the
same as on 0.54 apart from four documented, parser-driven differences, recorded in the
`ADR-SYS-SYSMLV2-001` migration addendum: a bare package-level `attribute`/`port`/`item` is read as
the usage it is (not a definition), constructs 0.54 rejected for the whole file (a `view` in a `part`
usage, `then fork`/`join`/`decide`) no longer raise `W541`, a placeholder guard that is not an
expression is exported as a comment with the reason "does not parse". The 0.57 parser also removes
three earlier limits (`REQ-TRS-SYSMLV2-074`..`-076`): a guarded succession `first a if g then b;` is
ingested into the native `guard:` of a `successionConnections:` entry and exported in that form; an
`include` target may be qualified or use the declaring form; and a bare package-level `attribute`
carries its `value:`/`unit:` (`attribute maxMass : Real = 12.5 [kg];`).

## 26. Exported behaviour bodies — `REQ-TRS-SYSMLV2-056`..`-058`

`export-sysml` writes `ActionDef`/`Action` and `StateDef`/`State` bodies in exactly the statement forms
ingestion (§§ on states/actions) reads, and only when it can prove the round trip: each top-level
entry is re-parsed through the real ingestion converters and emitted only if it comes back equal.

| Native | SysML v2 text |
|---|---|
| `subActions:` `PerformAction` | `action n [: T];` |
| `AcceptAction` / `SendAction` | `accept n [: payload];` / `send n [: payload];` |
| `AssignmentAction` | `assign target := value;` |
| `LoopAction` (`while`/`loop`/`for`) | `while c { … }` / `loop { … }` / `for v in seq { … }` |
| `IfAction` (`then`/`else`) | `if c { … } else { … }` |
| `TerminateAction` | `terminate [target];` |
| `controlNodes:` | `fork n;` `join n;` `decide n;` `merge n;` |
| `successionConnections:` | `first a then b;` |
| `entryAction:`/`doAction:`/`exitAction:` | `entry action n;` `do action n;` `exit action n;` |
| `subStates:` (+ `isInitial`/`isFinal`) | nested `state n [: T] { … }`, `then n;`, `final n;` |
| `transitions:` | `transition first s accept a [via p] if g do effect then t;` (nested: the `first s` clause is the stored `source`, omitted when absent) |

**What becomes a comment** (`// subAction not exported (<reason>): <what>`): an unknown `kind:`/`loopKind`;
extra fields ingestion cannot hold (anything but the documented set, including the `@SyscribeStep` fields
of §28); a guard/condition/effect that is
not valid SysML text (e.g. a `<conditional expression>` placeholder); a top-level transition without
`source:`; a non-string `entryAction:`. Nothing is approximated. A `first a then b;` whose endpoint names a
step that was itself commented out is commented too (`successionConnection not exported (endpoint 'b' was
not exported)`), so the exported body never references a step it does not contain.

## 27. Named control steps, standard-library tables, compound units — `REQ-TRS-SYSMLV2-060`..`-066`

**Named steps.** The pinned parser gives `if`/`while`/`loop`/`for`/`assign`/`terminate` no name, so
ingestion synthesizes `if_1`, `while_1`, …. A hand-chosen name is spelled with the action usage that owns
the one statement, and both directions read it:

```sysml
action navigateWaypoints { for waypoint in waypoints { action awaitArrival; } }
```

| SysML v2 | Native |
|---|---|
| `action <name> { <one if/while/loop/for/assign/terminate> }` — no typing, `:>`, `:>>`, multiplicity or `accept`/`send`, exactly one statement in the body | the statement's `subActions:` entry (`IfAction`, `LoopAction`, …) named `<name>`; the synthesized counter is not advanced |
| any other nested `action` usage | `PerformAction` (unchanged) |

`export-sysml` writes the bare statement when the entry's name is the synthesized positional one
(`if_1`, the first bare `if`), and the wrapper above for any other name. Running it over this repository's
`model/` leaves 10 entry comments (8 top-level, down from 10, plus 2 now visible inside the newly exported `navigateWaypoints` and `checkWeather`): they need fields ingestion has no slot for
(`via:`/`referent:`/`valueKind:`/`trigger:`, `loopKind: until`) or a form it reads back in another shape
(`accept: {payload: …}` mapping vs the string form; `and` vs `&&` spelling in a guard), plus 3 successions
commented because an endpoint is among them. All of those are closed by §28: the repository `model/`
now exports with zero such comments.

**Standard-library tables.** Nothing is imported; three small tables answer every reference:
`ScalarValues` (`Boolean`, `String`, `NumericalValue`, `Number`, `Complex`, `Real`, `Rational`,
`Integer`, `Natural`, `ScalarValue`) and `Base` are closed, so a misspelt member is `W043` while every
real one resolves; `ISQ::…Value` quantity names and `SI` unit names/symbols (`kg`, `m`, `s`, `N`, `W`,
`rpm`, `deg`, `Nm`, `ISQ::TorqueValue`, …) are recognised for `W044`'s quantity/unit dimension check; the
remaining standard-library package names are lenient. `export-sysml` emits all of them verbatim.

**Compound units.** A unit built from table units with `*`, `/` and `^` (`N*m`, `m/s`, `m^2`,
`kg*m/s^2`) has a derived dimension, so `W044` checks `unit: N*m` against `ISQ::TorqueValue`. The 0.54
lexer reads `[N*m]` as one token but `[N * m]` as an operator expression that it attaches to the attribute
as a multiplicity; ingestion recovers it as the unit (a bracket after a literal value is a unit), and
export writes `= 4 [N*m]` — an expression, not the quoted name `['N*m']`.

**`W544`.** `part sub : M [3..1];` (and `[-1..2]`, `[1.5..3]`) raises the advisory on the ingested usage;
`[n]`, `[0..n]` and `[0..*]` do not. The multiplicity text itself is stored unchanged.

## 28. Closing the remaining degradations — `REQ-TRS-SYSMLV2-067`..`-072`

**Canonical forms in the read-back check.** `export-sysml` writes an entry only if re-ingesting its text
gives the same entry. That comparison now runs both sides through the form ingestion itself produces,
instead of comparing raw YAML:

| Authored | Read back | Treated as equal |
|---|---|---|
| `accept: {payload: Cmd}` (no `via`) | `accept: Cmd` | yes — the plain string is canonical; a mapping is `{payload, via}` only when a `via:` is present (spec §8.8.3 already defines the string as shorthand for `{payload: …}`) |
| `guard: "a and b"` | `guard: "a && b"` | yes — guards, conditions, assigned values/targets and `for` sequences all pass through ingestion's one expression renderer |
| `value: 10` | `value: "10"` | yes |
| `guard: "<conditional expression>"` | does not parse | no — still a comment |

The authored spelling is what is written to the `.sysml` text; only the comparison is canonical.

**`@SyscribeStep`.** The pinned 0.54 parser has no syntax for `via` on an action `accept`/`send`, for
`referent`/`valueKind` on an `assign`, for an accept `trigger`, or for an `until` loop. Rather than loosen
the check, those native fields travel in a Syscribe metadata annotation inside the step's own body (the
same family as `@SyscribeDomain`/`@SyscribeImplementedBy`); the statement itself stays plain SysML:

```sysml
send commandDescent : Items::ControlCommand { @SyscribeStep { via = 'controlOut'; } }
action setThrottle {
    assign self := 0.6;
    @SyscribeStep { feature = 'throttlePercent'; valueKind = 'initial'; }
}
action waitForGround {
    loop { accept monitorDescent : Items::GPSFix { @SyscribeStep { via = 'gpsIn'; } } }
    @SyscribeStep { loopKind = 'until'; condition = 'self.altitudeM <= 0.1'; }
}
accept awaitArrival : Items::GPSFix { @SyscribeStep { triggerKind = 'change'; triggerCondition = 'd < 2.0'; } }
```

| Native field | Annotation attribute | Applies to |
|---|---|---|
| `via` | `via` | `AcceptAction`, `SendAction` |
| `trigger: {kind, condition}` | `triggerKind`, `triggerCondition` | `AcceptAction` |
| `referent` | `feature` (the 0.54 lexer drops an attribute spelled `referent`) | `AssignmentAction` |
| `valueKind` | `valueKind` | `AssignmentAction` |
| `loopKind: until` + `condition` | `loopKind = 'until'`, `condition` | an unconditioned `loop { … }` |

Ingestion reads the annotation back into the same entry fields; a statement that carries one is written
as the named-step wrapper (§27), which owns the annotation, and so never takes a synthesized name. Other
SysML v2 tools see valid SysML and ignore the metadata, so an exported `until` loop reads in them as an
unconditioned `loop` — the condition is in the annotation only. Values must be plain text; any other
extra field, a `via` that is not a string, or a `trigger` with keys other than `kind`/`condition` still
degrades to a comment.

**Reporting.** The summary line ends `behaviour entries degraded to comments: N` (stderr and the trailing
`// ---- export summary ----` block). A repository test (`crates/syscribe-model/tests/sysmlv2_export_ratchet.rs`)
exports every `.md`-native `ActionDef`/`Action`/`StateDef`/`State` in `model/`, `model_auto/`, `model_mg/`,
`model_sil/` and the `examples/*/model` roots and compares the total with a recorded budget, failing when
it grows or shrinks (lower the budget to lock in the gain). Over those roots the budget is 0 since
`REQ-TRS-SYSMLV2-074`: the four guarded `successionConnections:` entries of `model_sil` were the last
degradations, and the 0.57 parser reads `first a if g then b;`.
