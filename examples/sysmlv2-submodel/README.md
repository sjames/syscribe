# SysMLv2 Submodel Example

A small, standalone drone-propulsion model demonstrating every capability of
native SysML v2/KerML submodel ingestion (`ADR-SYS-SYSMLV2-001`,
`REQ-TRS-SYSMLV2-000` through `-012`) in one coherent scenario. It is a
separate model root from this repository's own `model/` — running validation
here never affects that model's baseline.

## Running it

```bash
cargo build --workspace   # once, if you haven't already
./target/debug/syscribe -m examples/sysmlv2-submodel/model
./target/debug/syscribe -m examples/sysmlv2-submodel/model feature-check --deep
./target/debug/syscribe -m examples/sysmlv2-submodel/model validate --config CONF-QUAD-DRONE-001
./target/debug/syscribe -m examples/sysmlv2-submodel/model why-active PropulsionSubsystem::Propulsion::RotorConfigChoice::quadConfig --config CONF-HEX-DRONE-001
```

Current output: **0 errors, 9 warnings** on the base `validate` report (all
expected/documented below); `feature-check --deep` is **0 errors, 1 warning**
(also documented below), reports both `Configuration`s as valid models of the
feature model, and `void model: false`.

## Scenario

A drone's propulsion subsystem is authored natively in SysML v2 text
(`PropulsionSubsystem/*.sysml`) and cross-references hand-authored native
Syscribe `Requirement`s, `TestCase`s, and a `FeatureDef`/`Configuration` pair
sitting alongside it in the same model.

## File layout

```
model/
  _index.md                          Root package
  Requirements/
    REQ-DRONE-ENDUR-001.md           satisfy target (quoted-id form)
    REQ-DRONE-THRUST-001.md          satisfy target (qname form)
    REQ-DRONE-VERIFY-001.md          verify target (SysMLv2 element's own `verify`)
  Tests/
    TC-DRONE-ENDUR-001.md            native TestCase -> native Requirement (ordinary)
    TC-DRONE-THRUST-001.md           native TestCase -> native Requirement (ordinary)
    TC-DRONE-ROTOR-001.md            native TestCase -> SysMLv2 element (REQ-TRS-SYSMLV2-004)
  Features/
    RotorConfig/
      _index.md                      FeatureDef FEAT-ROTOR-CONFIG (mandatory XOR group)
      Quad.md                        FeatureDef FEAT-ROTOR-QUAD (child)
      Hex.md                         FeatureDef FEAT-ROTOR-HEX (child)
  Configurations/
    CONF-QUAD-DRONE-001.md           selects Quad
    CONF-HEX-DRONE-001.md            selects Hex
  PropulsionSubsystem/
    _index.md                        type: Package, sysmlSubmodel: true — the only native
                                      element in this directory
    Structure.sysml                  file 1 of the `Propulsion` SysML v2 package
    Interfaces.sysml                 file 2 of the same `Propulsion` package (multi-file merge)
    Behavior.sysml                   file 3: unmapped constructs (coverage-boundary demo)
```

## What each `.sysml` file demonstrates

**`Structure.sysml`** — `package Propulsion { ... }`, part 1 of the multi-file
merge (`REQ-TRS-SYSMLV2-002`):

- `item def Fuel;` — **ItemDef**
- `port def PowerPort;` / `port def FuelPort;` — **PortDef**
- `attribute def ThrustRating;` — **AttributeDef**
- `part def RotorAssembly { doc /* ... */ port fuelSupplyPort : FuelPort;
  item fuelItem : Fuel; attribute thrustReading : ThrustRating;
  @SyscribeDomain { value = 'hardware'; } @SyscribeIntegrity { asil = 'B'; }
  @SyscribeShortName { value = 'rotor-assembly'; } satisfy
  'REQ-DRONE-ENDUR-001'; }` — **PartDef** containing a `doc /* ... */`
  member lifting straight into the synthesized element's `doc` body
  (`REQ-TRS-SYSMLV2-009`; this is the one `PartDef`/`Part` in this model
  that clears `W600`), a **Port usage**, an **Item usage**, an **Attribute
  usage** (all nested-in-a-part-body forms), three `@Syscribe*` fixed-field
  metadata annotations lifting `domain: hardware`/`asilLevel:
  B`/`shortName: rotor-assembly` onto the synthesized element
  (`REQ-TRS-SYSMLV2-008`), and a `satisfy` targeting a native `Requirement`
  by its quoted `REQ-*` id (`REQ-TRS-SYSMLV2-003`, id form)
- `variation part def RotorConfigChoice { variant part quadConfig :
  RotorAssembly { @SyscribeFeature { featureId = 'FEAT-ROTOR-QUAD'; } }
  variant part hexConfig : RotorAssembly { @SyscribeFeature { featureId =
  'FEAT-ROTOR-HEX'; } } }` — the **variation/variant** pair, each carrying a
  `@SyscribeFeature` metadata annotation targeting a real `FeatureDef`
  (`REQ-TRS-SYSMLV2-005`)

**`Interfaces.sysml`** — same `Propulsion` package, part 2 of the merge —
contributes different members than `Structure.sysml` (no name collisions):

- `connection def PowerLink;` — **ConnectionDef**
- `interface def PowerInterface { port supplyPort : PowerPort; }` —
  **InterfaceDef** with a nested **Port usage**
- `requirement def RotorThrustReqDef;` — **RequirementDef**
- `part def Drone { port powerPort : PowerPort; interface powerIface :
  PowerInterface; connection powerLink : PowerLink connect powerPort to
  rotorConfig { doc /* ... */ } part rotorConfig : RotorConfigChoice;
  allocation motorAlloc : RotorAssembly; requirement thrustCheck :
  RotorThrustReqDef { verify 'REQ-DRONE-VERIFY-001'; } satisfy
  Requirements::'REQ-DRONE-THRUST-001'; }` — a **PartDef** containing an
  **Interface usage**, a **Connection usage** whose `connect powerPort to
  rotorConfig;` clause lifts onto `Drone`'s own `connections:` field as a
  real, resolvable `connectivity` edge (`REQ-TRS-SYSMLV2-010`), and whose own
  trailing `{ doc /* ... */ }` body lifts onto the synthesized `powerLink`
  element itself (`REQ-TRS-SYSMLV2-012`) — two independent lifts from the
  same usage, onto two different elements — a **Part usage** (typed by the
  variation point above), an **AllocationUsage**, a **Requirement usage**
  whose own `verify` targets a native `Requirement` by qname
  (`REQ-TRS-SYSMLV2-003`, `verify` keyword), and a `satisfy` targeting a
  different native `Requirement` by
  its Syscribe qualified name (`REQ-TRS-SYSMLV2-003`, qname form)
- `part droneInstance : Drone;` — a package-level **Part usage** of `Drone`
  (keeps `Drone` genuinely referenced as a type, matching how everything else
  in this example is used somewhere)

Between the two files, every one of `REQ-TRS-SYSMLV2-007`'s fixed mapped
kinds appears at least once: `Package`, `Part(Def/Usage)`,
`Attribute(Def/Usage)`, `Port(Def/Usage)`, `Connection(Def/Usage)`,
`Interface(Def/Usage)`, `Item(Def/Usage)`, `Requirement(Def/Usage)`,
`AllocationUsage`, and `variation`/`variant`.

**`Behavior.sysml`** — coverage-boundary demonstration
(`REQ-TRS-SYSMLV2-007`): `state def RotorHealthState;` and `action def
MonitorRotorHealth;` are real, legally-parsed SysML v2 constructs that
coexist in the same `Propulsion` package as the mapped structural content
above, but behavior modeling is outside the fixed mapped-element set. Run
`syscribe -m examples/sysmlv2-submodel/model export` and confirm there is
**no** `RotorHealthState` or `MonitorRotorHealth` anywhere in the output —
they parse without error and contribute nothing to the graph. Parse-broad,
map-narrow.

## Cross-reference summary

| Direction | Source | Target |
|---|---|---|
| `satisfy` (quoted id) | `RotorAssembly` (SysMLv2) | `REQ-DRONE-ENDUR-001` (native) |
| `satisfy` (qname) | `Drone` (SysMLv2) | `Requirements::REQ-DRONE-THRUST-001` (native) |
| `verify` | `Drone::thrustCheck` (SysMLv2) | `REQ-DRONE-VERIFY-001` (native) |
| `TestCase.verifies:` | `TC-DRONE-ENDUR-001` (native) | `REQ-DRONE-ENDUR-001` (native, ordinary) |
| `TestCase.verifies:` | `TC-DRONE-THRUST-001` (native) | `REQ-DRONE-THRUST-001` (native, ordinary) |
| `TestCase.verifies:` | `TC-DRONE-ROTOR-001` (native) | `RotorAssembly` (SysMLv2, `REQ-TRS-SYSMLV2-004`) |
| `@SyscribeFeature` | `quadConfig` variant (SysMLv2) | `FEAT-ROTOR-QUAD` (native `FeatureDef`) |
| `@SyscribeFeature` | `hexConfig` variant (SysMLv2) | `FEAT-ROTOR-HEX` (native `FeatureDef`) |

## Fixed `@Syscribe*` field annotations (`REQ-TRS-SYSMLV2-008`)

`RotorAssembly` also carries `@SyscribeDomain`, `@SyscribeIntegrity`, and `@SyscribeShortName` —
lifting straight onto the synthesized element's `domain:`/`asilLevel:`/`shortName:` fields exactly
as if hand-authored, no different from the `@SyscribeFeature` → `appliesWhen:` lift above:

```
$ ./target/debug/syscribe -m examples/sysmlv2-submodel/model export --ndjson | \
    grep '"name":"RotorAssembly"'
{"frontmatter":{"asilLevel":"B","domain":"hardware","shortName":"rotor-assembly", ...
```

`domain: hardware` matches `REQ-DRONE-ENDUR-001`'s own `reqDomain: hardware`, so no `E313`
domain-mismatch fires; `asilLevel: B` has no `derivedFromSafetyGoal`/`derivedFrom` chain to
propagate through, so `E841`/`E842` don't apply here either — both are exercised for real (firing,
not just staying silent) in `qual/fixtures/TC-TRS-SYSMLV2-008/`, which is the deliberately
adversarial half of this feature's coverage; this worked example instead shows the everyday,
validates-clean case. `@SyscribeImplementedBy` is demonstrated only in the qual fixture, not here,
since a path that doesn't resolve on disk would add a `W023` to this example's otherwise-clean
warning list for no explanatory benefit.

## `doc /* ... */` comment lift (`REQ-TRS-SYSMLV2-009`)

`RotorAssembly` also carries a `doc /* ... */` member — lifting straight into the synthesized
element's `doc` body, the same field a hand-authored `.md` file's body below its `---` closer
populates:

```
$ ./target/debug/syscribe -m examples/sysmlv2-submodel/model show \
    PropulsionSubsystem::Propulsion::RotorAssembly
...
## Documentation

The primary rotor/motor/battery propulsion chain — the physical assembly whose endurance
REQ-DRONE-ENDUR-001 constrains.
```

This is why `RotorAssembly` doesn't appear among the `W600` elements below — a `doc` member clears
`W600` exactly as a hand-authored element's non-empty body would. It also has a second-order
effect: `quadConfig`/`hexConfig` (both `typedBy: RotorAssembly`) don't appear among the `W600`
elements either, once `REQ-TRS-VAL-017` (see "Expected / documented warnings" below) started
suppressing `W600` on a `Part` usage whose *type* is documented, even when the usage itself carries
no `doc` of its own.

## Connection-endpoint lift (`REQ-TRS-SYSMLV2-010`)

`Drone`'s `connection powerLink : PowerLink connect powerPort to rotorConfig { doc /* ... */ }`
lifts onto `Drone`'s own `connections:` field — not onto the nested `powerLink` element — as a
real, resolvable graph edge between two of `Drone`'s own direct children:

```
$ ./target/debug/syscribe -m examples/sysmlv2-submodel/model connectivity \
    PropulsionSubsystem::Propulsion::Drone::powerPort --format json
{
  "edges": [
    {
      "from": "PropulsionSubsystem::Propulsion::Drone::powerPort",
      "kind": "connection",
      "to": "PropulsionSubsystem::Propulsion::Drone::rotorConfig"
    }
  ],
  ...
}
```

Both `powerPort` and `rotorConfig` are bare (unchained) names here — no `.` segment to drop — so
this demonstrates the common case cleanly; see `qual/fixtures/TC-TRS-SYSMLV2-010/` for the dotted
(`a.p1`) and n-ary (`connect (a, b, c)`) forms, and `ADR-SYS-SYSMLV2-001`'s addendum for why a
dotted chain's trailing segment is deliberately dropped rather than resolved. This particular edge
is visible via `connectivity` but not `n2` — `n2`'s axis stays `PartDef`/`Part`-only, by its own
pre-existing design unrelated to this feature, so `powerPort` (a `Port`) never appears there
regardless, scoped or unscoped.

## Connection-usage doc lift (`REQ-TRS-SYSMLV2-012`)

That same `powerLink` usage's trailing `{ doc /* ... */ }` body lifts independently, onto the
synthesized `powerLink` element itself this time — not onto `Drone`'s `connections:`:

```
$ ./target/debug/syscribe -m examples/sysmlv2-submodel/model show \
    PropulsionSubsystem::Propulsion::Drone::powerLink
...
## Documentation

Primary power feed from the airframe bus to the active rotor configuration.
```

Two independent lifts read the same `connection powerLink : PowerLink connect powerPort to
rotorConfig { doc /* ... */ }` statement: `REQ-TRS-SYSMLV2-010` reads `connect_from`/`connect_to`
onto the *owning part* (`Drone`); `REQ-TRS-SYSMLV2-012` reads the trailing `{ }` body onto
`powerLink` *itself*. Neither depends on the other — a connection usage can carry either, both, or
neither.

## `n2`'s scoped axis (`REQ-TRS-SYSMLV2-011`)

`n2 PropulsionSubsystem::Propulsion::Drone` used to report `(no parts in scope)` — its axis came
exclusively from `features:`, which no SysMLv2-synthesized part populates. It now also includes
direct-child containment, so `Drone`'s one `Part`-typed direct child shows up:

```
$ ./target/debug/syscribe -m examples/sysmlv2-submodel/model n2 \
    PropulsionSubsystem::Propulsion::Drone
N² Interface Matrix — PropulsionSubsystem::Propulsion::Drone (depth 1)

               rotorConfig
rotorConfig    ■
```

Only `rotorConfig` appears — `powerPort` still doesn't, for the reason above (`n2`'s axis is
`Part`/`PartDef`-only, unrelated to this fix), and there's no *other* `Part`-typed sibling for
`rotorConfig` to show a wired cell against here. See `qual/fixtures/TC-TRS-SYSMLV2-011/` for a
scoped `n2` run with two wired `Part`-typed siblings, where the off-diagonal cell does populate.

## Feature model / configuration

`Features::RotorConfig` is a mandatory XOR group with two children, `Quad`
and `Hex`. `CONF-QUAD-DRONE-001` selects `Quad`; `CONF-HEX-DRONE-001` selects
`Hex`. Both variant parts' `@SyscribeFeature` annotations lift straight into
`appliesWhen:`, so projection genuinely differs per configuration:

```
$ ./target/debug/syscribe -m examples/sysmlv2-submodel/model why-active \
    PropulsionSubsystem::Propulsion::RotorConfigChoice::quadConfig --config CONF-QUAD-DRONE-001
Verdict: active

$ ./target/debug/syscribe -m examples/sysmlv2-submodel/model why-active \
    PropulsionSubsystem::Propulsion::RotorConfigChoice::quadConfig --config CONF-HEX-DRONE-001
Verdict: inactive
```

`feature-check --deep` reports `void model: false`, `core features:
Features::RotorConfig` (it's mandatory), no dead/false-optional features, and
both configurations as valid models — exactly what a two-way mandatory XOR
group driven partly from the SysMLv2 side should look like.

## Expected / documented warnings

Every warning below is understood and either inherent to this feature as
currently scoped, or an ordinary artifact of a deliberately small demo model
— none is a defect in this example. (`REQ-TRS-VAL-017` is a general validator
refinement, not SysMLv2-specific, but this composition-heavy example is
exactly the kind of model it was motivated by — see the `W600` entry below.)

- **`W600` × 4 ("PartDef/Part has an empty documentation body")** — the
  remaining four `PartDef`/`Part` elements in `PropulsionSubsystem/*.sysml`
  carry no `doc /* ... */` member *and* have no documented `typedBy:` target
  to fall back on, so they get an empty `doc:` body exactly like a
  hand-authored element with no body text would (`REQ-TRS-SYSMLV2-009` lifts
  `doc` comments where they're written; it doesn't invent documentation for
  elements that have none). `RotorAssembly` demonstrates the lift itself —
  see the "`doc /* ... */` comment lift" section below — and no longer trips
  this warning, which is why the count dropped from 7 to 6 once that landed;
  it dropped again to 4 once `REQ-TRS-VAL-017` started suppressing `W600` on
  a `Part` usage whose `typedBy:` target is itself documented — `quadConfig`
  and `hexConfig` (both `typedBy: RotorAssembly`) are exactly that case.
- **`W005` × 3 ("no derivedFrom and no derivedChildren — possible orphan")**
  — ordinary consequence of this being a small, flat demo with no requirement
  breakdown hierarchy; unrelated to SysMLv2.
- **`W015`/`W022` ("requirement ...::thrustCheck is active ... but covered in
  none")** — the SysMLv2 `thrustCheck` requirement usage is a `type:
  Requirement` element with no `status:` (the mapper never sets one), so the
  native "is this requirement covered per configuration" checks treat it as
  an ordinary non-draft requirement needing V&V closure, even though it's
  really just a carrier for the `verify` statement demonstrated above. This
  is a genuine, worth-knowing edge of the origin-agnostic design (there is no
  way, by design, to tell these checks "this one is different") — see the
  Surprises section below.

## Surprises a real example surfaced that unit tests didn't

- **The `@SyscribeFeature`/feature-model wiring only works if the
  `FeatureDef` hierarchy is nested correctly.** An earlier draft of this
  example put the mandatory XOR-group `FeatureDef` directly on
  `Features/_index.md` (qname `Features`) and had `Configuration`s select
  `Features::RotorConfig: true` — a qname that didn't exist. That silently
  produced `E225` ("configuration is not a valid model of the feature model:
  root feature 'Features' is mandatory") for *both* configurations. The fix
  was mirroring this repository's own `model/Features/Propulsion/` layout
  exactly: the mandatory group lives at `Features/RotorConfig/_index.md`
  (qname `Features::RotorConfig`), with `Quad.md`/`Hex.md` beside it as
  children — `Features/` itself carries no `_index.md` at all, exactly like
  the main model. Nothing about this is SysMLv2-specific; it would have bitten
  a hand-authored `FeatureDef` hierarchy identically. Pure unit tests never
  caught it because they always built single, already-correct `FeatureDef`
  fixtures directly.
- **The qname-form `satisfy` target is invisible in the validate report's
  §7 "Elements with `satisfies`" table**, even though it resolves correctly
  and genuinely suppresses `W300`. `crates/syscribe/src/main.rs`'s report
  builder filters that column through `is_req_id(s)`, which only recognizes
  bare `REQ-*` ids — a qname like `Requirements::REQ-DRONE-THRUST-001` (a
  fully legitimate, documented `REQ-TRS-SYSMLV2-003` form) gets silently
  dropped from display, showing "—" for `Drone` even though it has a real
  satisfies link. `syscribe why PropulsionSubsystem::Propulsion::Drone`
  shows it correctly. This is a pre-existing report-rendering limitation
  (the column was never written expecting anything but a bare id), exposed
  — not caused — by exercising the qname form of `satisfy` for real. No
  Rust code changed to build this example; noting it here for a future fix.
- **`syscribe types`/`list` mislabel every `Attribute` (usage) element as
  `"Other"`.** `crates/syscribe/src/query.rs`'s `type_label` function has an
  explicit match arm for `ElementType::AttributeDef` but none for
  `ElementType::Attribute`, so it falls through to the catch-all. This
  example's `thrustReading` attribute usage — a completely ordinary,
  correctly-typed element (`syscribe export`/`show`/`why` all report it
  correctly as `Attribute`) — shows up as `Other` in `syscribe types`'s
  count table. Pre-existing, general, and not SysMLv2-specific (a
  hand-authored native `Attribute` usage hits the identical gap); flagging
  here since a real example is what surfaced it.
- **Bare (no `def` keyword) declarations disambiguate to different
  Def/Usage forms depending on nesting**, confirmed again while writing this
  example: `attribute`/`port`/`item` default to the *Def* form at package
  level but the *Usage* form when nested inside another construct's body
  (e.g. inside a `part def`'s body), while `connection`/`requirement`/
  `allocation` default to *Usage* in both positions. This was already known
  from earlier unit-test work, but a full, multi-file, realistic model made
  it very easy to accidentally get the "wrong" (but still legally-parsed)
  kind by not paying attention to nesting — worth calling out for anyone
  hand-authoring `.sysml` content for this feature.
