---
type: ADR
id: ADR-PR-SW-001
title: "Break the parent safety requirement into two independent leaf requirements"
status: accepted
date: "2026-06-08"
---

## Context

The parent requirement spans two distinct software concerns that are each
assignable to a single architecture element.

## Decision

Decompose the parent into two leaf requirements, each satisfied by a dedicated
software part and verified by a dedicated test case.

## Consequences

The parent is satisfied and verified transitively through its leaves; it is
never satisfied directly (E312 / §12.4).
