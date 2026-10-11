---
type: ADR
id: ADR-GTI-ASM-003
name: "Assume DC arc-fault detection and rapid shutdown are mandatory in US only"
status: proposed
date: "2026-10-11"
tags: [assumption, open-question]
---

## Context

NEC 690.11 (arc-fault) and 690.12 (rapid shutdown) apply to US installations; the Indian sources consulted do not mandate them.

## Decision

Both are features: required when Region is US, optional in India (a customer may still select them).

## Consequences

If Indian customers or DISCOMs require them, the Region.IN constraints change to requires.
