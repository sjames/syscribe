---
type: Package
name: FTA
---

Fault Tree Analysis (FTA) performed per IEC 61025:2006 and ISO 26262-9:2018 Annex B.
Each fault tree is rooted at a `SafetyGoal` from `Safety/HARA/` and decomposes the
top-level hazard into combinations of basic events using Boolean logic gates.

## Fault trees

| Tree | Top event | ASIL | Approach |
|---|---|---|---|
| `FT-ENG-001` | `SG-ENG-001` (unintended acceleration) | D | Binary decision diagram, cut-set enumeration |
| `FT-ENG-002` | `SG-ENG-002` (engine stall at speed) | B | Probabilistic, β-factor common-cause analysis |

## Nesting convention

Gates (`FTG-*`) and basic events (`FTE-*`) are placed in a subdirectory named after their
parent fault tree, so qualified names form the prefix `Safety::FTA::FT-ENG-001::FTG-ENG-001`.
This satisfies the Syscribe nesting rule (W900) and keeps each fault tree self-contained.

## Failure rates

Basic event failure rates are derived from IEC 62380 generic data for automotive electronics
and field return data from comparable powertrain ECU programmes. Rates are expressed in
failures per hour (λ) and are used to compute cut-set probabilities at the top event.
