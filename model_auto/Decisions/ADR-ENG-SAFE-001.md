---
type: ADR
id: ADR-ENG-SAFE-001
name: ASIL D decomposition for engine safety requirement into SW + HW sub-requirements
status: accepted
---

## Context

REQ-ENG-SAFE-000 derives from SafetyGoals SG-ENG-001 (unintended acceleration,
ASIL D) and SG-ENG-002 (critical engine stall, ASIL B). ISO 26262-9 §5.4 permits
ASIL D decomposition into two independent ASIL B channels when freedom-from-
interference is demonstrated.

## Decision

Decompose REQ-ENG-SAFE-000 into three leaf requirements:

- **REQ-ENG-SAFE-001** (ASIL D → software) — Safety monitor software detects all
  safety faults within 100 ms and asserts fail-safe output. Allocated to
  `System::Software::SafetyMonitor`.
- **REQ-ENG-SAFE-002** (ASIL D → hardware) — External watchdog timer resets the
  MCU within 30 ms of software failure. Allocated to `System::Hardware::WatchdogTimer`.
- **REQ-ENG-SAFE-003** (ASIL B → software) — Engine stall monitor detects CPS loss
  and initiates controlled deceleration. Allocated to
  `System::Software::EngineStallMonitor`.

Software (SafetyMonitor) and hardware (WatchdogTimer) channels are independent:
no shared failure mode exists between them.

## Consequences

Each channel requires independent development, code review, and hardware-in-the-loop
testing (L5 testLevel). Combined failure probability achieves the ASIL D target.
