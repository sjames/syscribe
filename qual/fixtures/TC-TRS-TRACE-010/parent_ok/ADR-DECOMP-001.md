---
id: ADR-DECOMP-001
type: ADR
name: "MPU domain decomposition"
status: accepted
---
## Context
Decompose the memory-protection domain requirement into MPU leaves.
## Decision
Enforce via the MPU region table.
## Consequences
Leaves carry the SIL-4 obligation.
