---
type: ADR
id: ADR-XREF-001
name: "Breakdown decision for the XREF-006 fixture requirement"
status: accepted
---

## Context

The child requirement is derived from a parent; a requirement with derivedFrom
needs an accepted breakdownAdr (E310). This ADR satisfies that rule so the only
finding of interest is the unresolved-reference error plus its hint.

## Decision

Decompose the parent into the child requirement.

## Consequences

The fixture stays clean apart from the intended unresolved-reference finding.
