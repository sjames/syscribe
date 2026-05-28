# Traceability

`GUIDE · TRACEABILITY`

Section §12 of the format spec defines six enforced traceability rules. All are checked by the validator.

## Rule 1 — OSLC link direction (§12.1)

Links always point **upstream**. The derived/verifying/satisfying artifact holds the reference.

```
✓  FaultTolerantFCReq.md  →  derivedFrom: [REQ-UAV-SAFE-000]
✗  SafetyParentReq.md    →  derivedChildren: [REQ-UAV-FC-001]  # WRONG direction
```

Concretely:

| Relationship | Field on | Points to |
|---|---|---|
| Requirement derivation | Child requirement | `derivedFrom: [REQ-PARENT-*]` |
| Requirement satisfaction | Architecture element | `satisfies: [REQ-LEAF-*]` |
| Test case verification | TestCase | `verifies: [REQ-*]` |
| Function allocation | Allocation element | `allocatedFrom:` / `allocatedTo:` |

## Rule 2 — Decomposition requires an ADR (§12.2)

Every requirement with `derivedFrom:` must have `breakdownAdr:` set to an **accepted** ADR.

```yaml
derivedFrom:
  - REQ-UAV-SAFE-000
breakdownAdr: Decisions::SafetyDecompositionADR   # must be status: accepted
```

- **E310** — `derivedFrom:` present but `breakdownAdr:` absent
- **E311** — `breakdownAdr:` does not resolve, or does not resolve to an ADR element
- **W303** — `breakdownAdr:` resolves to a `proposed` ADR but the requirement is `approved` or higher

## Rule 3 — Leaf assignment (§12.3)

Requirements must be decomposed until each leaf can be assigned to a single architecture element. Leaf requirements at `approved` or `implemented` status with no `satisfies:` link fire **W300**.

A leaf requirement should have exactly one satisfying element. More than one fires **W301**.

## Rule 4 — No parent assignment (§12.4)

A requirement with `derivedChildren` (i.e. other requirements derive from it) must not appear in any `satisfies:` list. Only leaf requirements can be directly satisfied by an architecture element.

- **E312** — parent requirement found in a `satisfies:` list

## Rule 5 — Domain classification (§12.5)

Requirements carry `reqDomain: system | hardware | software`. Architecture elements carry `domain: system | hardware | software`. A leaf requirement at a non-`system` domain can only be satisfied by an element in the same domain.

```yaml
# FlightController.md
domain: software
satisfies:
  - REQ-UAV-FC-001    # reqDomain: software  ✓
  - REQ-UAV-NAV-001   # reqDomain: hardware  ✗ E313
```

- **E313** — `satisfies` domain mismatch
- **E302** — unknown `reqDomain` value
- **E303** — unknown `domain` value
- **W302** — leaf requirement at `implemented` / `verified` still has `reqDomain: system`

## Rule 6 — HW/SW independence (§12.6)

Elements with `domain: software` and `domain: hardware` must not share a direct `supertype:` or `typedBy:` link. Cross-domain integration uses `Allocation` elements.

- **E315** — cross-domain direct reference
- **E314** — element with `isDeploymentPackage: true` has no `Allocation` to a hardware element
- **W304** — `isDeploymentPackage: true` combined with `domain: hardware`

## Integration test coverage for parent requirements (W305)

Leaf-level TestCases (L1/L2) verify individual derived requirements but do not cover the emergent, composed behaviour expressed by the parent. A parent requirement at `approved`, `implemented`, or `verified` status must have at least one active TestCase at `testLevel: L3` (system), `L4` (system integration), or `L5` (hardware-in-the-loop).

```yaml
# TC-UAV-PERF-001.md — system test covering the parent requirement
type: TestCase
id: TC-UAV-PERF-001
title: Full performance envelope system test
status: active
testLevel: L3
verifies:
  - REQ-UAV-PERF-000   # the parent requirement
```

This is distinct from ASIL D tests: W702 specifically requires an L5 test for ASIL D requirements, while W305 applies to all parent requirements regardless of safety level.

- **W305** — parent requirement at `approved` or higher has no active L3/L4/L5 TestCase

## Traceability matrix

The validation report (section 4) prints a matrix of all leaf requirements against all active TestCases, with `✓` where `verifies:` covers the requirement.

```
| Requirement       | TC-UAV-FC-001 | TC-UAV-NAV-001 | Active TCs |
|---|---|---|---|
| REQ-UAV-FC-001    | ✓             |                | 1          |
| REQ-UAV-NAV-001   |               | ✓              | 1          |
```
