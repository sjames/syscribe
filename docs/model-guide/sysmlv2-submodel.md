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
- **`doc` comment extraction** — a SysMLv2-synthesized element's `doc:` body is currently always
  empty (`W600` fires for every one of them); the mapper does not yet lift `doc /* ... */`
  comments out of the parsed AST.
