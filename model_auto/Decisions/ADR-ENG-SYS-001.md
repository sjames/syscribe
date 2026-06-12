---
type: ADR
id: ADR-ENG-SYS-001
name: Decompose system requirements into performance, safety, and security sub-trees
status: accepted
---

## Context

REQ-ENG-SYS-000 expresses the top-level engine management obligation. For independent
development, verification, and ASIL decomposition, the requirement must be broken
down by concern area.

## Decision

Decompose REQ-ENG-SYS-000 into three independent sub-requirement trees:

1. **Performance** (REQ-ENG-PERF-000) — functional timing and efficiency obligations
2. **Safety** (REQ-ENG-SAFE-000) — ISO 26262 ASIL obligations
3. **Security** (REQ-ENG-SEC-001) — ISO/SAE 21434 cybersecurity obligations

Security is kept as a single leaf requirement because the current TARA yields only
one cybersecurity goal requiring a software mitigation.

## Consequences

- Separate ADRs govern further decomposition within each sub-tree.
- Safety and performance sub-trees are independently verified.
- ASIL D independence criterion (ISO 26262-9 §5.4) is met: performance and safety
  software functions (ThrottleControl, SafetyMonitor) are allocated to separate
  AUTOSAR SWCs with no shared memory regions.
