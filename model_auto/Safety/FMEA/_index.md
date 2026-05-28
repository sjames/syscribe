---
type: Package
name: FMEA
---

Failure Mode and Effects Analysis (FMEA) performed per IEC 60812:2018 and ISO 26262-8:2018
§8. The FMEA complements the FTA by systematically enumerating failure modes at the
component and software function level, computing Risk Priority Numbers (RPN), and identifying
recommended actions for high-RPN items.

## FMEA sheets

| Sheet | Subsystem | Entries | Max RPN |
|---|---|---|---|
| `FMEA-ENG-001` | Throttle, fuel control, and safety monitor | FM-ENG-001 … FM-ENG-010 | 216 (PID windup) |
| `FMEA-ENG-002` | Sensor and hardware subsystem | FM-ENG-011 … FM-ENG-016 | 180 (TPS divergence) |

## RPN interpretation

RPN = Severity × Occurrence × Detection (each 1–10). Entries with RPN > 100 require a
`recommendedAction`. Actions are tracked to closure and re-rated after implementation.
High-RPN items with a safety impact are cross-referenced to the FTA cut sets to verify that
the recommended action reduces the corresponding cut-set probability below the ASIL target.

## Relationship to FTA

FMEA entries for sensor and actuator failure modes reference their corresponding FTA basic
events (`FTE-*`) in the `ref:` field. This bidirectional traceability ensures that every
failure mode enumerated in the FMEA is either represented in a fault tree or explicitly
excluded with justification.
