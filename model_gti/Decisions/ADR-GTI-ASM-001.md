---
type: ADR
id: ADR-GTI-ASM-001
name: "Assume hybrid means multi-string high-voltage PV with MPPT, and moot means MPPT"
status: proposed
date: "2026-10-11"
tags: [assumption, open-question]
---

## Context

The request says "hybrid solar mode, which means multi string high voltage solar strings with moot". There is no commonly used term "moot" in this context; MPPT (maximum power point tracking) is the evident intent.

## Decision

Read *hybrid solar mode* as multi-string, high-voltage PV input with **one MPPT per string group**. Battery storage and backup operation are out of scope.

## Consequences

If "hybrid" is meant to include battery storage, a new bidirectional DC stage, islanding/backup mode and different anti-islanding rules are needed; most of the safety analysis would change.
