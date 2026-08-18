# Single-File Feature Model Example

A worked example of `REQ-TRS-FM-005`: a **full-fledged** feature model authored as
one `type: FeatureModel` sheet, exercising every capability the single-file form
supports — deliberately built as the vehicle for a "can a real feature model be
modelled this way" review, so it also doubles as regression coverage.

## What it exercises

`model/Features/_index.md` (one file) declares 14 `FeatureDef`s via a flat,
dot-named `featureTree:`, plus `crossTreeConstraints:` and a sheet-level
`parameterConstraints:`:

| Capability | Where |
|---|---|
| 3-level dotted nesting | `Platform` → `Platform.CortexM` → `Platform.CortexM.Fpu` |
| `mandatory: true` + `groupKind: alternative` (mandatory XOR group) | `Platform`, `DataLink` |
| `mandatory: true` + `groupKind: or` (mandatory, ≥1-of-N group) | `Sensors` |
| Typed `parameters:` — `range`, `default`, `isRequired`, `bindingTime`, `buildVar`, `enumValues`, `isFixed` | `Wdt.timeoutMs`, `Wdt.mode` |
| `buildExports:` | `Platform.CortexM` |
| `parentFeature:` override (logical parent ≠ dotted-path parent) | `OrphanRelocated` (relocated under `Wdt`) |
| `contributesTo:` (two-level feature models, informational) | `DataLink` |
| Inline `requires:`/`excludes:` — dotted, absolute-qname, **and** stable-id forms | `crossTreeConstraints:` entries |
| A per-file `FeatureDef` referenced from the sheet by stable id — the two authoring forms interoperating in one model | `Features/Legacy/SafeMode.md` |
| `parameterConstraints:` declared directly on the `FeatureModel` sheet (not a `Package`) | `PC-WDT-TIMEOUT` |
| `appliesWhen:`-gated architecture, `Requirement`, and `TestCase` | `Architecture/LidarDriver.md`, `Requirements/REQ-EX-WDT-001.md` |
| Two `Configuration`s exercising both branches of every constraint, `parameterBindings:`, and `buildOverrides:` | `Configurations/` |

## Running it

```bash
cargo build --workspace   # once, if you haven't already

M=examples/single-file-feature-model/model

./target/debug/syscribe -m $M validate                 # 0 errors, 3 warnings (see below)
./target/debug/syscribe -m $M feature-check --deep      # 0 findings; void model: false
./target/debug/syscribe -m $M matrix                    # Requirement x Configuration coverage
./target/debug/syscribe -m $M matrix --features          # Feature x Configuration product map
./target/debug/syscribe -m $M features                   # the whole tree, correctly nested
./target/debug/syscribe -m $M feature Features::Wdt      # one feature's card
./target/debug/syscribe -m $M configure CONF-EX-CORTEXM-001
./target/debug/syscribe -m $M why-active Architecture::LidarDriver --config CONF-EX-CORTEXM-001
./target/debug/syscribe -m $M why-active Architecture::LidarDriver --config CONF-EX-RISCV-001
./target/debug/syscribe -m $M build-config --config CONF-EX-CORTEXM-001 --format env
./target/debug/syscribe -m $M diff --config CONF-EX-CORTEXM-001 --config CONF-EX-RISCV-001
./target/debug/syscribe -m $M validate --all-configs
```

Current output: **0 errors, 3 warnings** on `validate` — all three are deliberate and
expected, since this example's sole purpose is exercising the PLE/feature-model
mechanics, not a full V-model traceability chain: `W005`/`W300` (the one demo
`Requirement` has no parent and no satisfying architecture element) and `W007`
(the one demo `PartDef` is never instantiated as a `Part`). `feature-check --deep`
reports the model sound: not void, no dead or false-optional features, no invalid
configurations.

## The two `Configuration`s

| | `CONF-EX-CORTEXM-001` | `CONF-EX-RISCV-001` |
|---|---|---|
| Platform | CortexM + FPU | RiscV |
| Sensors | IMU + Lidar | GPS |
| Wdt | on, window mode, `timeoutMs: 2000` | off |
| `OrphanRelocated` | on | off |
| DataLink | LoRa | WiFi |
| `Legacy::SafeMode` | on (forced by `Wdt requires FEAT-LEGACY-SAFEMODE`) | off |

`CONF-EX-RISCV-001` deliberately keeps `Wdt`/`WindowMode`/`Lidar` off — selecting
any of them would trip a `crossTreeConstraints:` entry (`Lidar requires
Platform.CortexM`, `RiscV excludes Wdt::WindowMode`) against RiscV/WiFi, which is
exactly what `validate --all-configs` and `feature-check` check on every push.

## Mixing the two authoring forms

`Features::Legacy::SafeMode` is a deliberately *ordinary*, per-file `FeatureDef`
(`Features/Legacy/SafeMode.md`) — not part of the sheet's `featureTree:` — to prove
the old and new authoring styles coexist freely: it is referenced from the sheet's
`crossTreeConstraints:` by its stable id (`FEAT-LEGACY-SAFEMODE`), resolves cleanly,
and participates in `feature-check`/`feature-check --deep` exactly like any other
feature, regardless of which file wrote it.

See `docs/model-guide/variability.md` and §9.6a of `spec/markdown-sysml-format.md`
for the full reference.
