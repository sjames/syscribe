---
type: Package
name: Safety
---

This package contains all functional safety analysis artefacts for the Engine ECU, organised
by analysis method. The analysis follows ISO 26262:2018 (road vehicles) and, where applicable,
IEC 61508:2010 (functional safety for E/E/PE safety-related systems).

## Analysis layers

| Sub-package | Standard | Contents |
|---|---|---|
| `HARA/` | ISO 26262-3 | Hazardous events, risk parameters, safety goals |
| `FTA/` | ISO 26262-9 / IEC 61025 | Fault trees linked to safety goals |
| `FMEA/` | IEC 60812 / ISO 26262-8 | Failure mode tables for subsystems |

## Traceability flow

```
HazardousEvent (HARA)
  └── SafetyGoal (HARA)
        └── Requirement (Requirements/Safety/)    ← derivedFromSafetyGoal:
              └── PartDef/Part (System/)           ← satisfies:
                    └── FaultTree (FTA/)           ← topEvent:
                          └── FMEASheet (FMEA/)    ← entries[].ref:
```

All safety goals carry an `asilLevel:` and reference at least one `HazardousEvent` via
`hazardousEvents:`. Integrity levels propagate to derived requirements and satisfying
architecture elements per R-007; any ASIL decomposition is documented in an ADR.
