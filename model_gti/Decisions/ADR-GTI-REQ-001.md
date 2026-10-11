---
type: ADR
id: ADR-GTI-REQ-001
name: "Requirement breakdown: stakeholder needs, regional system requirements, then HW/SW leaves"
status: accepted
date: "2026-10-11"
tags: [requirements, breakdown]
---

## Context

One product serves two regions with different grid codes. Duplicating the requirement tree per region would drift apart.

## Decision

Stakeholder needs (`REQ-GTI-STK-*`) are broken down into system requirements (`REQ-GTI-SYS-*`) that state behaviour against "the active grid profile", with `appliesWhen:` on the few requirements that exist in one region only. System requirements are then allocated to hardware (`REQ-GTI-HW-*`) and software (`REQ-GTI-SW-*`) leaves, each satisfied by exactly the architecture element that realises it.

## Consequences

Region numbers are confined to the grid-profile parameters and the regulatory matrix; a limit change touches one place. Leaves are verified per configuration through TestPlans.
