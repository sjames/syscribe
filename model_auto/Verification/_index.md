---
type: Package
name: Verification
---

Test cases verifying Engine ECU requirements, organised by domain. All test cases carry a
`testLevel:` (L1–L5), a `status:` (active/retired), and `verifies:` links to the
requirements they cover.

## Test levels

| Level | Description |
|---|---|
| L2 | Analysis / review |
| L3 | Integration (software-in-the-loop) |
| L4 | System integration |
| L5 | Hardware-in-the-loop (HIL) |

The test cases are listed by `syscribe show Verification` (generated from this directory); each
carries its own `testLevel:`, and `syscribe verification-depth` shows the per-requirement mix.

## HIL test environment

L5 tests run on a HIL bench with a production-representative ECU, engine plant model, and
real sensor/actuator interfaces. Fault injection is performed via a fault insertion unit (FIU)
that can break or short individual signal lines under software control. The HIL environment
is required by ISO 26262-6 §9 for ASIL C/D software verification.

## Coverage targets

- All ASIL D requirements (REQ-ENG-SAFE-001, -002, -005): verified at L5 HIL.
- ASIL B requirement (REQ-ENG-SAFE-003): verified at L4 system integration.
- ASIL A requirement (REQ-ENG-SAFE-004): verified at L5 HIL (rev limiter independent of TPS).
- Performance requirements: verified at L3 software-in-the-loop simulation.
- Security requirements: verified at L2 (analysis) or L3 (integration).
