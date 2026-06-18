# Modeling Guide

`GUIDE · OVERVIEW`

This guide walks through building a Syscribe model from scratch — from a directory skeleton to a fully traced, validated system description.

## Prerequisites

- Rust toolchain (stable, 1.75+)
- `cargo` available on `$PATH`
- A text editor with YAML syntax highlighting

## Directory skeleton

```
model/
  _index.md          # root namespace (type: Namespace or Package)
  Requirements/
    _index.md        # Requirements package
  Verification/
    _index.md
  Decisions/
    _index.md
  MySystem/
    _index.md
```

Each `_index.md` declares the package:

```yaml
---
type: Package
name: Requirements
---

System requirements for the XYZ project.
```

## Running the validator

```bash
cargo run --package syscribe -- model/
```

This prints a full Markdown report to stdout with all errors, warnings, and a traceability matrix. Pipe it to a file for review:

```bash
cargo run --package syscribe -- model/ > reports/validation.md
```

## Running the web browser

```bash
cargo run --package syscribe-server -- model/
# Listening on http://0.0.0.0:3000
```

The server watches the model directory for changes and reloads automatically.

## Typical workflow

1. **Identify stakeholder goals** — write parent `Requirement` elements with `REQ-SYS-000` style IDs
2. **Decompose requirements** — create child requirements with `derivedFrom:` and `breakdownAdr:`
3. **Write an ADR** for each decomposition decision — the validator enforces this (E310)
4. **Build the architecture** — create `PartDef`, `PortDef`, `InterfaceDef` elements
5. **Assign requirements** — add `satisfies:` and `domain:` to architecture elements
6. **Link implementation** — add `implementedBy:` on `Part`/`PartDef`/`Interface`/`InterfaceDef` to point at the source artifact that realises each element (§12.8); the validator flags missing paths with `W023`
7. **Write test cases** — create `TestCase` elements with `verifies:` and Gherkin bodies
8. **Draw diagrams** — create `Diagram` elements; add `layout:` for SVG rendering
9. **Validate** — run the validator; fix all errors; review warnings

### Going further: a product line

If the system ships in more than one configuration, model the whole family as one **150% model** instead of copying it per product:

10. **Declare a feature model** — add `FeatureDef` elements under `Features/` (XOR/`or`/optional groups, `requires:`/`excludes:`, typed `parameters:`)
11. **Condition the variants** — add `appliesWhen:` to the elements (parts, requirements, test cases) that belong only to some products
12. **Name the products** — add `Configuration` elements (`CONF-*`) selecting one variant per group
13. **Analyse and certify** — `feature-check --deep` proves the feature model is sound; `validate --config <C>` certifies one product; `validate --all-configs` gates the whole family in CI

See the [Variability & Product Lines](variability.md) guide for the full treatment, and the worked example below.

## Worked example — the UAV product line

The bundled `model/` is a UAV product line you can run every command against:

```bash
syscribe -m model/ feature-check --deep                      # feature model is sound
syscribe -m model/ matrix                                    # Requirement × Configuration grid
syscribe -m model/ validate --all-configs                    # certify every product
syscribe -m model/ diff --config CONF-UAV-SURVEY-001 \
                        --config CONF-UAV-DELIVERY-001        # what differs between products
syscribe -m model/ links UAV::Avionics::FlightController     # see its implementedBy code link
syscribe -m model/ refs firmware/flight_control/             # which elements that code realises
```

Its shape:

- **Feature model** (`model/Features/`) — three mandatory XOR groups (`Propulsion` = Quad ⊕ Hex, `Payload` = Survey ⊕ Mapping ⊕ Delivery, `DataLink` = LoRa ⊕ Cellular ⊕ Satcom), an optional `DualFlightController`, cross-tree constraints (`Delivery`/`DualFlightController` `requires` `Hex`), and a typed parameter (`Delivery.payloadCapacityKg`).
- **Configurations** (`model/Configurations/`) — three products: `CONF-UAV-SURVEY-001`, `CONF-UAV-MAPPING-001`, `CONF-UAV-DELIVERY-001`.
- **Conditioned elements** — variant parts (`PropulsionChoices`, `PayloadChoices`, `DataLinkChoices`, `BackupFlightController`) and variant requirements/tests (`REQ-UAV-MAP-001`, `REQ-UAV-CARGO-001`, `REQ-UAV-REDUN-001`) carry `appliesWhen:`, all derived under `ADR-SYS-PLE-001`.
- **Implementation trace** — architecture elements point at `repo:firmware/...` via `implementedBy:`.

## Further reading

- [Requirements & Test Cases](requirements.md) — stable IDs, lifecycle, Gherkin
- [Traceability](traceability.md) — the seven §12 rules, including `implementedBy:`/`W023`
- [Architecture Decisions](adrs.md) — ADR lifecycle and breakdown rules
- [State Machines](state-machines.md) — `StateDef` transitions, hierarchy/parallel regions, the `W070`–`W079` completeness checks
- [Variability & Product Lines](variability.md) — feature models, `appliesWhen`, `matrix`, `feature-check`, the `--config` lens
- [Multi-Repository Composition](multi-repo.md) — `[repos]`, `repoImports:`, cross-repo resolution, the `E510`–`E515`/`W510`–`W512` reproducibility gates, git-submodule integration
- [MagicGrid](magicgrid.md) — the B/W/S × 1-4 overlay (`mg_` custom fields), MoEs/MoPs, logical/physical layers, the `magicgrid` / `trade-study` reports
