---
type: ADR
id: ADR-EVCS-001
title: "Centralised power cabinet with distributed dispensers"
status: accepted
---

## Context

Stakeholder needs for the charging station (fast turnaround, reliability, safety,
affordability) must be broken down into implementable system requirements. The
key architectural choice is how to distribute power conversion across stalls.

## Decision

Adopt a **centralised power cabinet** (shared, scalable AC/DC conversion) feeding
several lightweight **dispensers** (the user-facing stalls) over a DC bus. Each
stakeholder need is broken down into system requirements that allocate a budget
(power, availability, reaction time, cost) to this architecture.

## Consequences

Power conversion can be pooled and dynamically allocated to active stalls,
improving utilisation and cost; dispensers stay simple, improving availability.
The DC bus and contactors become safety-critical and drive the isolation
requirement.
