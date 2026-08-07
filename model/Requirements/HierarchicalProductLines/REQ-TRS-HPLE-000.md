---
type: Requirement
id: REQ-TRS-HPLE-000
name: "A product-line Configuration can be consolidated from already-configured lower-tier product-line models"
status: draft
reqDomain: software
reqClass: stakeholder
tags:
  - variability
  - multi-repo
---

Syscribe shall let a `Configuration` be built by consolidating one already-configured `Configuration`
from each of one or more independently-developed, independently-versioned lower-tier product-line
models (a prime integrator assembling subcontractor/supplier product lines — a Multiple Software
Product Lines / staged-configuration structure), so a multi-party, multi-tier product line can be
composed without any lower tier needing to be aware of, or authored with foreknowledge of, whoever
consolidates it.

## Rationale

This is a studied problem (Multiple Software Product Lines; Czarnecki et al.'s staged
configuration, extended to software supply chains in a DaimlerChrysler/TU Berlin research
collaboration), and it is not solved by combining this codebase's two existing, adjacent
mechanisms alone: single-model product-line engineering (`FeatureDef`/`Configuration`) has no
notion of a lower tier at all, and multi-repo composition (§14) brings a peer repo's elements into
the local namespace but has no concept of "the peer's `Configuration` this consolidation actually
resolves to."

## Scope

- In scope: the schema and validation for consolidating already-resolved lower-tier
  `Configuration`s, including transitive parameter-binding deferral across an arbitrarily deep
  chain of tiers.
- Out of scope (this requirement and its children): cross-tier `appliesWhen:`/feature-model
  constraint expressions referencing an individual descendant `FeatureDef` directly (a consolidated
  `Configuration` is treated as a fixed, already-resolved unit, not a set of individually-gateable
  features); any dedicated multi-tier browsing/reporting UI beyond what already falls out of
  existing `show`/`export` once elements are mounted.
- Which specific field names and mechanisms implement this (`subConfigurations:`, the extension of
  `parameterBindings:`) are architectural decisions captured in `ADR-SYS-HPLE-001`, not part of this
  requirement.
