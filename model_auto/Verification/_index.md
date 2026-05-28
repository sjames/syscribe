---
type: Package
name: Verification
---

Test cases verifying Engine ECU requirements, organised by domain. All test cases carry a
`testLevel:` (L1–L5), a `status:` (active/retired), and `verifies:` links to the
requirements they cover.

## Test level distribution

| Level | Description | Test cases |
|---|---|---|
| L2 | Analysis / review | TC-ENG-SEC-001 |
| L3 | Integration (software-in-the-loop) | TC-ENG-PERF-001–003, TC-ENG-SEC-002–004, TC-ENG-SYS-001 |
| L4 | System integration | TC-ENG-SAFE-004 |
| L5 | Hardware-in-the-loop (HIL) | TC-ENG-SAFE-001–003, TC-ENG-SAFE-005–006 |

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
