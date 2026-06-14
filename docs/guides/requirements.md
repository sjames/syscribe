# Part II — Requirements

`GUIDE · PART II — REQUIREMENTS`


### 2.1 Authoring a requirement

Every requirement is a `.md` file. The frontmatter carries the structured attributes; the body
carries the prose `shall` statement and rationale.

```yaml
---
type: Requirement
id: REQ-SCHED-001
name: "Scheduler shall guarantee priority ordering"
status: approved
reqDomain: software
asilLevel: D          # or silLevel: 4 for IEC 61508
verificationMethod: test
derivedFrom:
  - Safety::SG-KERNEL-001-SchedulingIntegrity   # parent safety goal
breakdownAdr: Architecture::Decisions::ADR-SCHED-001  # required when derivedFrom is set
tags: [safety, scheduler]
---

The scheduler **shall** select the highest-priority ready thread for execution at every
scheduling point, with no priority inversion in the absence of a deliberate inheritance
mechanism.

<!-- Rationale: ThreadX-like semantics; strictly deterministic for SIL4/ASIL-D. -->
```

**Mandatory fields**: `type`, `id`, `name`, `status`. The body must contain at least one
`shall` — W001 fires if absent.

**ID pattern**: `REQ(-[A-Z0-9]{2,12})+-[0-9]{3,8}`. Use `next-id` to avoid clashes:

```bash
syscribe -m model next-id REQ-SCHED
# → REQ-SCHED-002
```

### 2.2 Requirement hierarchy and derivation

Requirements derive from parents with `derivedFrom:`. Every leaf (derived) requirement **must**
cite an `breakdownAdr:` that records why the derivation was made — this is the traceability
rationale auditors look for.

```
SafetyGoal (SG-*)
  └─ derivedFrom → Requirement (REQ-* asilLevel: D)
       └─ derivedFrom → Requirement (REQ-* asilLevel: D, more specific)
```

**Critical rule** (E312): You cannot derive from a *parent* package `Requirement` that is
itself derived from a safety goal if you intend to also derive directly from the goal. Pick one
parent; derive linearly. The validator catches cycles and ambiguous chains.

**Trace a requirement end-to-end**:

```bash
syscribe -m model trace REQ-SCHED-001
```

This shows: parent goals → this requirement → architecture elements that satisfy it →
test cases that verify it, with verification results if ingested.

### 2.3 Status lifecycle

| Status | Meaning | Use |
|---|---|---|
| `draft` | Being written; may change | Work in progress; not in coverage count for approved configs |
| `review` | Under formal review | Optional intermediate state |
| `approved` | Baselined; changes need change notice | The normal production state |
| `implemented` | Code/design done; awaiting test | Optional; useful for gap tracking |
| `verified` | Test evidence recorded | Set when `who-verifies` is fully covered |

Track how many requirements are in each state:

```bash
syscribe -m model audit
# Reports: approved 80 / draft 28 / ... per package
```

### 2.4 Traceability links: what goes where

| Link | Direction | Field | Validated? |
|---|---|---|---|
| Requirement → parent requirement or goal | up | `derivedFrom:` | Yes (E/W) |
| Requirement → architecture element | down | `satisfiedBy:` on the PartDef, OR `satisfies:` on the PartDef | Yes |
| Requirement → safety goal | up | `derivedFromSafetyGoal:` | Yes |
| Requirement → cybersecurity goal | up | `derivedFromSecurityGoal:` | Yes |
| Test case → requirement | up | `verifies:` on the TestCase | Yes |

**Coverage gap**: W015 fires when a requirement is active in a configuration but no test case
runs in that configuration. The matrix shows gaps at a glance:

```bash
syscribe -m model matrix --gaps-only
```

### 2.5 Integrity levels

Use exactly one of:
- `asilLevel: A | B | C | D` — ISO 26262
- `silLevel: 1 | 2 | 3 | 4` — IEC 61508
- `plLevel: a | b | c | d | e` — ISO 13849-1

Never mix them on the same requirement (W006). The parent element's level propagates
automatically into derived requirements; the validator flags a derived requirement that claims
a higher level than its parent (E841).

### 2.6 Multi-variant products (product line engineering)

If your product ships in multiple configurations (e.g., entry-level vs. full-featured, or
multiple hardware platforms), use `appliesWhen:` to gate which requirements apply in which
variant:

```yaml
appliesWhen: Features::MPU_ENABLED
```

The `matrix` command then shows coverage per configuration. Requirements inactive in a
configuration show `—` (N/A) and do not contribute to its coverage percentage.

---
