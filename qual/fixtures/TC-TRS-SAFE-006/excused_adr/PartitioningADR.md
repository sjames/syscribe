---
type: ADR
id: ADR-FFI-PART-001
title: MPU-based spatial and temporal partitioning for mixed-criticality on HostECU
status: accepted
---

## Context

SafetyCore (ASIL D) and Infotainment (QM) co-reside on HostECU.

## Decision

Enforce freedom from interference via MPU spatial partitioning and AUTOSAR OS timing
protection between the ASIL D and QM partitions (ISO 26262-6 §7.4.9 / ISO 26262-9 §7).
