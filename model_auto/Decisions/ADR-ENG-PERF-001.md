---
type: ADR
id: ADR-ENG-PERF-001
title: Decompose performance requirement into throttle response and fuel efficiency
status: accepted
---

## Context

REQ-ENG-PERF-000 covers all performance obligations for engine management. Two
distinct and measurable sub-goals require separate allocation targets:
a real-time control latency and a steady-state efficiency target.

## Decision

Decompose REQ-ENG-PERF-000 into:

- **REQ-ENG-PERF-001** — Throttle response latency ≤ 50 ms, allocated to
  `System::Software::ThrottleControl`
- **REQ-ENG-PERF-002** — Closed-loop fuel efficiency within 2 % of lambda
  target, allocated to `System::Software::FuelControl`

## Consequences

Each sub-requirement maps to exactly one software component, enabling direct
single-element traceability and independent unit testing.
