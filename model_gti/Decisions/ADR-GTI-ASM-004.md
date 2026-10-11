---
type: ADR
id: ADR-GTI-ASM-004
name: "Carry regulatory limits as flagged grid-profile parameters"
status: proposed
date: "2026-10-11"
tags: [assumption, open-question]
---

## Context

Primary standards were not accessible; several limits are recalled or from secondary sources.

## Decision

Limits live in the GridProfile features with a verification status (verified / secondary / unverified). Requirements reference the active profile.

## Consequences

The compliance engineer must review every row tagged reg-unverified before certification work starts.
