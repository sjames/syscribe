---
type: ADR
id: ADR-RTH-001
name: "Return-to-Home capability breakdown into logging and anti-false-trigger requirements"
status: accepted
tags:
  - rth
---

## Context

The top-level Return-to-Home (RTH) requirement (`REQ-RTH-001`) bundles two
concerns that need independent verification: what the system must record when
RTH fires, and what it must *not* do (spuriously trigger on noisy sensor
data).

## Decision

Break `REQ-RTH-001` into two derived requirements: `REQ-RTH-002` (event
logging) and `REQ-RTH-003` (anti-false-trigger). Each is independently
verifiable and independently assignable to a piece of engineering work.

## Consequences

`REQ-RTH-002`/`REQ-RTH-003` each set `derivedFrom: [REQ-RTH-001]` and
`breakdownAdr: Decisions::ADR-RTH-001`, per the standard requirement-breakdown
rule (`CLAUDE.md` §12.2 / `E310`).
