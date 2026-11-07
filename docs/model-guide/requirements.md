# Requirements & Test Cases

`GUIDE · REQUIREMENTS`

## Native Requirement

A native Requirement carries a **stable opaque identifier** (`REQ-*`) that never changes, even if the file is renamed or moved.

### Required fields

```yaml
---
type: Requirement
id: REQ-UAV-FC-001
name: FC fault detection ≤ 50 ms
status: approved
---

The flight controller **shall** detect any single sensor failure within 50 ms
and transition to the fault state.
```

| Field | Required | Values |
|---|---|---|
| `id` | Yes | `REQ(-[A-Z0-9]{2,12})*-[0-9]{3,8}` |
| `name` | Yes | Short human-readable label (free prose) |
| `status` | Yes | `draft` · `review` · `approved` · `implemented` · `verified` |

### Optional traceability fields

| Field | Description |
|---|---|
| `reqDomain` | `system` · `hardware` · `software` |
| `silLevel` | IEC 61508 SIL 1–4 |
| `asilLevel` | ISO 26262 ASIL A–D |
| `derivedFrom` | List of parent REQ-* IDs |
| `breakdownAdr` | Qualified name of the ADR justifying this derivation (required when `derivedFrom:` is set — E310, unless `[traceability] require_breakdown_adr = false`) |

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

### Item anchors and references

A requirement written as a numbered list exposes each **top-level** numbered item as a citable
unit. Items are numbered by position (1-based, across the whole body). Give an item a stable
**anchor** by writing `{#slug}` right after its number (slug: lowercase letters, digits and `-`,
starting with a letter):

```markdown
The key server shall:

1. {#grant-keys} **Server keys.** Hold the grant keys in an HSM.
2. {#rotate} **Rotation.** Rotate the keys every 12 months.
```

`show`, the web UI and `export-html` display the item without the token (`export-html` adds an
HTML anchor `id="item-grant-keys"`).

Cite an item or a PlanningItem question from any element body:

| Write | Means |
|---|---|
| `REQ-KEY-001#grant-keys` | the item anchored `{#grant-keys}` — **preferred**, survives insertions |
| `REQ-KEY-001#2` | the 2nd numbered item, by position |
| `REQ-KEY-001 item 2` | legacy form of `#2` |
| `PI-KEY-001#Q3` | question `Q3` of PlanningItem `PI-KEY-001` |

`validate` checks them (outside fenced code, only when the element part resolves):

- `W092` — the anchor, item number or question does not exist, or an anchor is defined twice /
  malformed in one requirement.
- `W093` — an element past draft (e.g. `review`, `approved`, `active`, `done`) cites a question
  that is still `open`.
- `W094` — an `answered` question is cited with "to confirm"-style wording in the same sentence.
  The phrases are configurable:

```toml
# .syscribe.toml
[references]
open_markers = ["to confirm", "to be confirmed", "TBC", "pending confirmation", "awaiting"]  # the defaults; [] disables W094
```

Query them with `check-ref REQ-KEY-001#grant-keys`, `refs REQ-KEY-001#grant-keys` (who cites
that item; `refs REQ-KEY-001` adds an *Item and question citations* section) and
`impact REQ-KEY-001#grant-keys`. `lint-docs` applies `W092`/`W094` to external Markdown.

**Item-level suspect links.** `suspect accept TC-KEY-004 REQ-KEY-001#grant-keys` baselines just
that item (a `traceBaselines:` key `REQ-KEY-001#grant-keys`), so `W090` fires only when *that*
item's text changes — not when another item is edited or inserted. Only anchored references can
be baselined. See [Suspect links](../design/suspect-links.md#item-level-baselines).

## Requirement hierarchy

Parent requirements have `derivedChildren`; leaf requirements have `derivedFrom`. The validator enforces:

- A parent requirement must not appear in any `satisfies:` list — only leaves are assigned (E312)
- Every `derivedFrom:` must cite an accepted `breakdownAdr:` (E310, E311)
- Leaves at `approved` or `implemented` status with no satisfying element fire W300
- **Parent requirements need an integration-level TestCase** — derived children's unit and property-based tests (L1/L2) are not sufficient to verify the emergent composed behaviour expressed by the parent. A parent requirement at `approved`, `implemented`, or `verified` status must have at least one active TestCase at `testLevel: L3`, `L4`, or `L5` (W305).

```
REQ-UAV-PERF-000  (parent — needs L3/L4/L5 TestCase)
  ├── REQ-UAV-COMM-001  (leaf — reqDomain: software, needs L1–L5 TestCase)
  ├── REQ-UAV-ENDUR-001 (leaf — reqDomain: hardware, needs L1–L5 TestCase)
  └── REQ-UAV-NAV-001   (leaf — reqDomain: hardware, needs L1–L5 TestCase)
```

## Native TestCase

````markdown
---
type: TestCase
id: TC-UAV-FC-001
name: FC fault injection — sensor dropout under 50 ms
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
````

### Required fields

| Field | Required | Values |
|---|---|---|
| `id` | Yes | `TC(-[A-Z0-9]{2,12})*-[0-9]{3,8}` |
| `name` | Yes | Short human-readable label (free prose) |
| `status` | Yes | `draft` · `review` · `approved` · `active` · `retired` |
| `testLevel` | Yes | `L1`–`L5` |
| `verifies` | Yes | At least one REQ-* ID |

### Test levels

| Level | Description |
|---|---|
| L1 | Unit test, hosted |
| L2 | Property-based / fuzz test, hosted |
| L3 | Formal proof |
| L4 | Integration test on an emulator (QEMU) |
| L5 | Hardware-in-the-loop |

L3–L5 count as integration-level for a parent requirement (`W305`); L1 and L2 do not.

### Gherkin body

The document body must contain at least one ` ```gherkin ` fenced block (E011). The first block must have a `Feature:` line (E015). `Scenario Outline:` blocks must have an `Examples:` table (E014).

### Coverage

A **leaf** Requirement (one that no other requirement names in its `derivedFrom:`) at `approved` or `implemented` status with no active TestCase (`status: active`) fires **W002**. A parent requirement is covered through its leaf descendants instead, and needs an active integration-level TestCase (`testLevel` L3–L5) of its own (**W305**). Coverage is tracked in the traceability matrix section of the validation report.
