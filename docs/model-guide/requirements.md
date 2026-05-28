# Requirements & Test Cases

`GUIDE · REQUIREMENTS`

## Native Requirement

A native Requirement carries a **stable opaque identifier** (`REQ-*`) that never changes, even if the file is renamed or moved.

### Required fields

```yaml
---
type: Requirement
id: REQ-UAV-FC-001
title: FC fault detection ≤ 50 ms
status: approved
---

The flight controller **shall** detect any single sensor failure within 50 ms
and transition to the fault state.
```

| Field | Required | Values |
|---|---|---|
| `id` | Yes | `REQ(-[A-Z0-9]{2,12})+-[0-9]{3}` |
| `title` | Yes | Short human title |
| `status` | Yes | `draft` · `review` · `approved` · `implemented` · `verified` |

### Optional traceability fields

| Field | Description |
|---|---|
| `reqDomain` | `system` · `hardware` · `software` |
| `silLevel` | IEC 61508 SIL 1–4 |
| `asilLevel` | ISO 26262 ASIL A–D |
| `derivedFrom` | List of parent REQ-* IDs |
| `breakdownAdr` | Qualified name of the ADR justifying this derivation (required when `derivedFrom:` is set — E310) |

### Normative text

The document body is the normative text. The first section (before any `##` heading) must:

- Be non-empty (E012)
- Contain the word `shall` (W001 warning if absent)

```markdown
The system **shall** maintain uplink with the ground station at a range
of not less than 5 km line-of-sight under nominal atmospheric conditions.

## Rationale

Derived from the regulatory link-loss safe-landing trigger distance.
```

## Requirement hierarchy

Parent requirements have `derivedChildren`; leaf requirements have `derivedFrom`. The validator enforces:

- A parent requirement must not appear in any `satisfies:` list — only leaves are assigned (E312)
- Every `derivedFrom:` must cite an accepted `breakdownAdr:` (E310, E311)
- Leaves at `approved` or `implemented` status with no satisfying element fire W300
- **Parent requirements need an integration-level TestCase** — derived children's unit/integration tests (L1/L2) are not sufficient to verify the emergent composed behaviour expressed by the parent. A parent requirement at `approved`, `implemented`, or `verified` status must have at least one active TestCase at `testLevel: L3`, `L4`, or `L5` (W305).

```
REQ-UAV-PERF-000  (parent — needs L3/L4/L5 TestCase)
  ├── REQ-UAV-COMM-001  (leaf — reqDomain: software, needs L1–L5 TestCase)
  ├── REQ-UAV-ENDUR-001 (leaf — reqDomain: hardware, needs L1–L5 TestCase)
  └── REQ-UAV-NAV-001   (leaf — reqDomain: hardware, needs L1–L5 TestCase)
```

## Native TestCase

```yaml
---
type: TestCase
id: TC-UAV-FC-001
title: FC fault injection — sensor dropout under 50 ms
status: active
testLevel: L5
verifies:
  - REQ-UAV-FC-001
---

```gherkin
Feature: Flight controller fault detection

  Scenario: Single IMU sensor dropout
    Given the flight controller is in flying state
    When the primary IMU stops reporting for 60 ms
    Then the fault state is entered within 50 ms of dropout onset

  Scenario: GPS fix loss
    Given the flight controller is navigating by GPS
    When GPS fix is lost for 100 ms
    Then the flight controller transitions to dead-reckoning mode
```
```

### Required fields

| Field | Required | Values |
|---|---|---|
| `id` | Yes | `TC(-[A-Z0-9]{2,12})+-[0-9]{3}` |
| `title` | Yes | Short human title |
| `status` | Yes | `draft` · `review` · `approved` · `active` · `retired` |
| `testLevel` | Yes | `L1`–`L5` |
| `verifies` | Yes | At least one REQ-* ID |

### Test levels

| Level | Description |
|---|---|
| L1 | Unit / module test |
| L2 | Integration / analysis / review |
| L3 | Formal / property-based |
| L4 | Simulation / QEMU |
| L5 | Hardware-in-the-loop / physical |

### Gherkin body

The document body must contain at least one ` ```gherkin ` fenced block (E011). The first block must have a `Feature:` line (E015). `Scenario Outline:` blocks must have an `Examples:` table (E014).

### Coverage

A Requirement at `approved` or `implemented` status with no active TestCase (`status: active`) fires **W002**. Coverage is tracked in the traceability matrix section of the validation report.
