---
type: ADR
id: ADR-DEC-001
name: Split the scheduler requirement into bitmap and WCET sub-requirements
status: accepted
date: "2026-05-20"
deciders:
  - Stakeholders::SystemsEngineer
  - Jane Doe
tags:
  - scheduler
---

## Context

The requirement conflates two independently verifiable properties.

## Decision

Decompose it into a bitmap requirement and a WCET requirement.
