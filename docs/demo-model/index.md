# Demo Models

`EXAMPLE · REFERENCE MODELS`

The repository ships with four fully-worked reference models covering different engineering domains, safety standards and modeling methods. Each model is self-contained and validates with zero errors; CI gates on `validate` for every one of them. Remaining warnings are deliberate, advisory findings that show what the checks report — run `validate` for the current list.

| Model | Domain | Standards | Location |
|---|---|---|---|
| [Engine ECU](engine-ecu.md) | Automotive powertrain | ISO 26262 (ASIL D), ISO/SAE 21434, AUTOSAR SecOC | `model_auto/` |
| [SIL 4 Interlocking](interlocking.md) | Railway signalling | IEC 61508, EN 50128/50129, EN 50159 Cat 2, ISO/SAE 21434 | `model_sil/` |
| [UAV Autonomous Flight](uav.md) | Unmanned aerial system | General SysMLv2 element palette | `model/` |
| [EV DC Fast-Charging Station](../model-guide/magicgrid.md) | Electric-vehicle charging infrastructure | MagicGrid (problem/solution domain × four pillars) | `model_mg/` |

## Quickstart

Run the validator against any model with the `-m` flag:

```bash
syscribe -m model_auto/ validate
syscribe -m model_sil/ validate
syscribe -m model/ validate
syscribe -m model_mg/ validate --profile magicgrid
```

Or point the server at a model to browse it in a web UI:

```bash
syscribe-server -m model_auto/
syscribe-server -m model_sil/
syscribe-server -m model_mg/
```

## Choosing a starting point

The **Engine ECU** and **SIL 4 Interlocking** models are the primary reference models. They demonstrate functional safety analysis (HARA, FTA, FMEA), cybersecurity analysis (TARA, VulnerabilityReport), full requirements traceability from system goals to leaf requirements to test cases, and architecture decision records with ADR-driven breakdown.

The **UAV** model is a broad showcase of the Syscribe structural element palette — part hierarchies, ports, connections, behaviors, flows, constraints, and calculations — but does not go deep on safety standards. Alongside the UAV, `model/` also hosts Syscribe's own product requirements (`REQ-TRS-*`), ADRs and `PlanningItem` work tracking, so it doubles as a real, large model to query.

The **EV DC Fast-Charging Station** model (`model_mg/`) shows the MagicGrid method as a pure `custom_fields:` overlay: black-box/white-box problem domain and solution domain across the requirements, behaviour, structure and parameters pillars, checked by the opt-in `magicgrid` profile. The [MagicGrid guide](../model-guide/magicgrid.md) walks through it command by command.
