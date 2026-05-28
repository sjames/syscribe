---
type: Package
name: HARA
---

Hazard Analysis and Risk Assessment (HARA) performed per ISO 26262-3:2018. The HARA
identifies hazardous events arising from the Engine ECU and assigns ASIL ratings using
the standard parameter method: Severity (S), Exposure (E), and Controllability (C).

## Hazardous events and safety goals

| Hazardous event | S × E × C | ASIL | Safety goal | FTTI |
|---|---|---|---|---|
| `HE-ENG-001` — throttle stuck open | S3 × E4 × C2 | D | `SG-ENG-001` | 100 ms |
| `HE-ENG-002` — engine stall at highway speed | S2 × E3 × C2 | B | `SG-ENG-002` | 500 ms |
| `HE-ENG-003` — engine over-speed on downshift | S2 × E2 × C3 | A | `SG-ENG-003` | 200 ms |

## FTTI definition

The Fault Tolerant Time Interval (FTTI) is the maximum time from the occurrence of a fault
to the onset of the hazardous event if no safety mechanism intervenes. Safety mechanisms must
detect and react within the FTTI. The safety monitor cycle time (5 ms) provides a ×20 margin
against the tightest FTTI (100 ms for ASIL D).

## Assumptions of use

The HARA assumes the vehicle is operated on public roads by a licensed driver, the Engine ECU
is installed per OEM specifications, and external power supply is within the 9–16 V range
specified in `System::EngineECU`. Operation outside these conditions is not within scope.
