# Hierarchical Product-Line Example: Vehicle ← Battery Pack ← Battery Cell

A realistic multi-repo example demonstrating hierarchical product-line composition
(`ADR-SYS-HPLE-001`, `REQ-TRS-HPLE-000..005`): a `Configuration` consolidated from
already-configured lower-tier product-line models, at three independently-developed,
independently-versioned tiers — mirroring the ADR's own illustrative scenario (an OEM vehicle line
built from a battery-pack line, itself built from a cell-chemistry line).

Unlike `examples/planning-item/` and `examples/sysmlv2-submodel/` (each a single model root), this
example is **three separate model roots**, one per tier, each a genuinely independent repo from the
tool's point of view — connected only by `[repos]` + `subConfigurations:`, never by physically
merging them:

```
examples/hple-multitier/
  battery-cell/model/    # leaf tier — no subConfigurations
  battery-pack/model/    # consolidates battery-cell
  vehicle/model/         # consolidates battery-pack (which itself consolidates battery-cell)
```

## Running it

```bash
cargo build --workspace   # once, if you haven't already

# Each tier validates cleanly entirely on its own — no [repos] table needed to do this
# (battery-cell has none at all; battery-pack's and vehicle's [repos] tables are read only
# when subConfigurations: or repoImports: actually reach across them).
./target/debug/syscribe -m examples/hple-multitier/battery-cell/model
./target/debug/syscribe -m examples/hple-multitier/battery-pack/model
./target/debug/syscribe -m examples/hple-multitier/vehicle/model

# feature-check --deep on the vehicle tier: void/dead/core/invalid-configuration analysis
./target/debug/syscribe -m examples/hple-multitier/vehicle/model feature-check --deep

# Opt-in W513 gate (REQ-TRS-HPLE-004) — exit 2, since CONF-VEHICLE-PARTIAL-001
# deliberately still leaves one parameter open (see "The two vehicle Configurations" below)
./target/debug/syscribe -m examples/hple-multitier/vehicle/model validate --deny W513
```

Current output: `battery-cell` is **0 errors, 2 warnings**; `battery-pack` is **0 errors, 3
warnings**; `vehicle` is **0 errors, 2 warnings** — all expected and documented below.
`feature-check --deep` on the vehicle tier reports `void model: false`, no dead/false-optional
features, no invalid configurations.

## Scenario

Three tiers, each a `FeatureDef`/`Configuration` pair, each developed as if by a separate team with
no knowledge of who — if anyone — eventually consolidates it (`REQ-TRS-HPLE-005`):

| Tier | Feature | Parameter | Shape |
|---|---|---|---|
| `battery-cell` | `Cell` | `nominalVoltageV` | `default: 3.2` — never needs external injection at all |
| `battery-cell` | `Cell` | `cycleLifeRating` | open (`isRequired`, no default) — closed by `battery-pack`, **one hop** |
| `battery-cell` | `Cell` | `manufacturingSiteCode` | open — closed by `vehicle`, **two hops**, straight past `battery-pack` |
| `battery-pack` | `Pack` | `packCapacityKwh` | open — closed by `vehicle`, one hop |

This spans every case `REQ-TRS-HPLE-000..005` describes in one coherent scenario:

- **`subConfigurations:` at the top two tiers** — `vehicle`'s `Configuration` names
  `battery-pack`'s; `battery-pack`'s names `battery-cell`'s. `battery-cell`, the leaf, declares
  none (`REQ-TRS-HPLE-001`).
- **`parameterBindings:` closing something at the immediate consolidating tier** —
  `battery-pack` closes `Cell.cycleLifeRating` itself, one hop down, right where it's declared
  (`REQ-TRS-HPLE-002`).
- **`parameterBindings:` reaching transitively, at depth** — `vehicle` closes
  `Cell.manufacturingSiteCode` directly, *two* hops down, past `battery-pack` entirely, using the
  parameter's ordinary, already-mounted qname — no new addressing syntax
  (`REQ-TRS-HPLE-002`'s "at any depth" claim, concretely).
- **A parameter that never needs anyone's decision** — `Cell.nominalVoltageV` carries a `default:`
  and is untouched by every `parameterBindings:` in this example
  (`REQ-TRS-HPLE-004`'s scope: a defaulted parameter is self-sufficient by construction, never part
  of the open-parameter closure).
- **Zero upward awareness** — no `FeatureDef` or `Configuration` in `battery-cell` or
  `battery-pack` names, or is capable of naming, whoever consolidates it. Confirmed architecturally
  and by dedicated regression test in `PI-HPLE-ISOLATION-001`
  (`crates/syscribe-model/tests/hple_isolation.rs`), not repeated here.

## The two vehicle `Configuration`s

- **`CONF-VEHICLE-STD-001`** — the complete, happy-path consolidation. Both remaining open
  parameters (`Pack.packCapacityKwh`, `Cell.manufacturingSiteCode`) are bound; combined with
  `battery-pack`'s own closing of `cycleLifeRating` and `nominalVoltageV`'s `default:`, the entire
  three-tier subtree is fully closed. Validating the `vehicle` model reports **zero** `W513`
  findings attributable to this `Configuration`.
- **`CONF-VEHICLE-PARTIAL-001`** — a vehicle program mid-decision: pack capacity is settled,
  sourcing site is not. Demonstrates `REQ-TRS-HPLE-004` concretely: validating the `vehicle` model
  reports exactly one `W513`, naming `Features::Cell.manufacturingSiteCode` still open — a warning,
  not an error, silent to the exit code unless a CI run explicitly opts in with `--deny W513` (which
  only the repo actually positioned as the point of final assembly should ever do — see
  `ADR-SYS-HPLE-001`'s rationale for why this is graded by validation *context*, not a fixed
  severity).

## Findings reference (what each warning is, and why it's expected)

| Tier | Code | Meaning |
|---|---|---|
| `battery-cell` | `W017` ×2 | `cycleLifeRating`/`manufacturingSiteCode` unbound *at this leaf tier's own, single-model validation* — exactly what a leaf with genuinely open parameters looks like on its own, before anyone consolidates it. |
| `battery-pack` | `W510` | `[repos.battery_cell]` has no `ref:` pinned. This demo composes three plain subdirectories of one checkout, not three independent git repos, so ref-pinning (§14's `repos sync`/`repos status`) isn't meaningfully demonstrable here — orthogonal to HPLE itself. |
| `battery-pack` | `W017` | `Pack.packCapacityKwh` unbound at `battery-pack`'s own local validation — this tier's own isolated view of an intentionally open parameter. |
| `battery-pack` | `W513` | `Cell.manufacturingSiteCode` still open *across the consolidated subtree* — `battery-pack` genuinely doesn't close it (that's `vehicle`'s job); this is the same open parameter as the `W017` above, restated as `REQ-TRS-HPLE-004`'s transitive-closure view. |
| `vehicle` | `W510` | Same as `battery-pack`'s, one level up (`[repos.battery_pack]`). |
| `vehicle` | `W513` | From `CONF-VEHICLE-PARTIAL-001` only — see above. `CONF-VEHICLE-STD-001` contributes none. |

None of these are gated by default; `--deny <code>` is how whichever repo is the actual point of
final assembly promotes any of them to a CI failure, per `ADR-SYS-HPLE-001`'s established, reused
`W510`/`W511`/`W512`/`W023`/`W090` posture.
