# Demo Models

`EXAMPLE · REFERENCE MODELS`

The repository ships with three fully-worked reference models covering different engineering domains and safety standards. Each model is self-contained, validates with zero errors, and exercises the complete Syscribe element palette.

| Model | Domain | Standards | Location |
|---|---|---|---|
| [Engine ECU](engine-ecu.md) | Automotive powertrain | ISO 26262 (ASIL D), ISO/SAE 21434, AUTOSAR SecOC | `model_auto/` |
| [SIL 4 Interlocking](interlocking.md) | Railway signalling | IEC 61508, EN 50128/50129, EN 50159 Cat 2, ISO/SAE 21434 | `model_sil/` |
| [UAV Autonomous Flight](uav.md) | Unmanned aerial system | General SysMLv2 element palette | `model/` |

## Quickstart

Run the validator against any model with the `-m` flag:

```bash
syscribe -m model_auto/ validate
syscribe -m model_sil/ validate
syscribe -m model/ validate
```

Or point the server at a model to browse it in a web UI:

```bash
syscribe-server -m model_auto/
syscribe-server -m model_sil/
```

## Choosing a starting point

The **Engine ECU** and **SIL 4 Interlocking** models are the primary reference models. They demonstrate functional safety analysis (HARA, FTA, FMEA), cybersecurity analysis (TARA, VulnerabilityReport), full requirements traceability from system goals to leaf requirements to test cases, and architecture decision records with ADR-driven breakdown.

The **UAV** model is a broad showcase of the Syscribe structural element palette — part hierarchies, ports, connections, behaviors, flows, constraints, and calculations — but does not go deep on safety standards.
