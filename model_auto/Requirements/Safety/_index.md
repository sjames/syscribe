---
type: Package
name: Safety
---

Safety requirements are derived from the Hazard Analysis and Risk Assessment (HARA) in
`Safety/HARA/`. The decomposition from the parent safety goal `REQ-ENG-SAFE-000` to leaf
requirements is documented in `Decisions/ADR-ENG-SAFE-001`.

## ASIL allocation

| ID | ASIL | Domain | Hazard addressed |
|---|---|---|---|
| `REQ-ENG-SAFE-001` | D | software | HE-ENG-001 (unintended acceleration) |
| `REQ-ENG-SAFE-002` | D | hardware | HE-ENG-001 (unintended acceleration) — HW channel |
| `REQ-ENG-SAFE-003` | B | software | HE-ENG-002 (engine stall at speed) |
| `REQ-ENG-SAFE-004` | A | software | HE-ENG-003 (engine over-speed) |
| `REQ-ENG-SAFE-005` | D | software | HE-ENG-001 — throttle close verification |

ASIL D is decomposed into a software channel (`REQ-ENG-SAFE-001`, `REQ-ENG-SAFE-005`) and a
hardware channel (`REQ-ENG-SAFE-002`) per ISO 26262-9 ASIL decomposition, establishing
independence between the AUTOSAR software stack and the external watchdog IC. Each channel
achieves ASIL D(D) / ASIL D(D) meaning no reduction in individual channel requirements.

## Verification strategy

All ASIL B–D requirements are verified at hardware-in-the-loop (HIL) test level (L5), per
ISO 26262-6 §9 which mandates hardware-integrated testing for ASIL C/D software. All test
cases carry `verificationMethod: test` and are in `Verification/`.
