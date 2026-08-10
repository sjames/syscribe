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

Unlike WASM foreign-format plugins (a separate, sandboxed third-party-plugin mechanism —
see its own model-guide page once that feature lands on this checkout), this is a dedicated,
always-on **native** subsystem: there is no `[plugins.<alias>]` config, no alias, and no sandbox.
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
`AllocationUsage`, and `variation`/`variant` membership.

A construct outside that set — a behavior body, `analysis`/`case`/`verification def`,
`calc`/`constraint`, and similar — parses without error but contributes **nothing** to the graph:
no element, no `Finding`, invisible, the same way a native Markdown model has no way to express
content that isn't frontmatter or documentation body. Parse-broad, map-narrow.

```sysml
package Propulsion {
    part def MappedPart;          // becomes SysML2::Propulsion::MappedPart
    state def UnmappedState;      // parses fine, contributes nothing
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
unresolved `verifies:` (`E102`) — note that this codebase does **not** currently raise an
equivalent local finding for an unresolved `satisfies:` outside multi-repo mode (`E512`); that's a
pre-existing, general characteristic unrelated to this feature.

### A native `TestCase` → a SysMLv2 element

A native `TestCase`'s existing `verifies:` field resolves against the qname index of any ingested
SysMLv2 subtree, so a `TestCase` can verify a SysMLv2-authored element the same way it verifies a
native `Requirement`:

```yaml
---
type: TestCase
id: TC-DRONE-ROTOR-001
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

## 4. Validation

| Code | Condition |
|---|---|
| `W540` | A `_index.md` found anywhere inside a `sysmlSubmodel: true` package's subtree, other than that package's own anchor `_index.md` |
| `W541` | Either a `.sysml`/`.kerml` file failed to read (e.g. invalid UTF-8), or `sysml-v2-parser` failed to parse its contents |

Both are a **dedicated code range**, distinct from the WASM-plugin family (`E530`–`E532`/
`W530`–`W534`) — this is native, always-on ingestion of a trusted, compile-time dependency, not
plugin execution, and conflating the two ranges would misattribute the failure mode to anyone
grepping a validation report.

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
- **`doc /* ... */` comment lift on `Package`/`Requirement`** — §7 below covers every other
  mapped element kind, but a nested `package Inner { doc /* ... */ ... }` or a `requirement`/
  `requirement def`'s own doc block is not lifted; a deliberate, matching-issue-scope descope
  (`REQ-TRS-SYSMLV2-009`'s Scope section), not an oversight.

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
