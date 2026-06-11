---
type: ADR
id: ADR-SYS-PLE-001
title: "Adopt a 150% product-line model with a feature model and per-product configurations"
status: accepted
tags:
  - product-line
  - variability
  - decomposition
---

## Context

The UAV is sold in several configurations that share most of their architecture
but differ in propulsion, payload, data link, and flight-controller redundancy.
Maintaining a separate model per product would duplicate the common core and let
the variants drift apart.

## Decision

Model the whole family as one **150% model**. A feature model under `Features/`
declares the variation points (Propulsion, Payload, DataLink as XOR groups; an
optional DualFlightController), and each marketable product is a `Configuration`
(`CONF-UAV-*`). Variant-specific elements carry `appliesWhen:` conditions over
feature qualified names; per-product correctness is certified with
`validate --config` and gated across the family with `validate --all-configs`.

The variant capability goal **REQ-UAV-VAR-000** is decomposed into one
variant-specific leaf requirement per differentiating capability:

- REQ-UAV-MAP-001 → `Mapping` payload (LiDAR point density)
- REQ-UAV-CARGO-001 → `Delivery` payload (cargo release timing)
- REQ-UAV-REDUN-001 → `DualFlightController` (failover latency)

## Consequences

The common core is written once and reused. Each leaf requirement is active only
in the products whose features select it, so coverage and traceability are
evaluated per product rather than against the superset.
