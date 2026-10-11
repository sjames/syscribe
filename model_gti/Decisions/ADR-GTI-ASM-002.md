---
type: ADR
id: ADR-GTI-ASM-002
name: "Assume a 10 kW three-phase platform with 1000 V DC strings"
status: proposed
date: "2026-10-11"
tags: [assumption, open-question]
---

## Context

Power class, phase configuration and DC voltage class were not specified.

## Decision

10 kVA rated, three-phase four-wire (US 208 V/60 Hz, India 415 V/50 Hz), maximum DC input 1000 V, 2 or 3 MPPT trackers with two string inputs each.

## Consequences

A 1500 V class, single-phase variants or a larger power class change the MCU headroom (ADR-GTI-MCU-001) and the protection design.
