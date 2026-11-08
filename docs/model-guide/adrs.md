# Architecture Decisions

`GUIDE · ADRS`

Architecture Decision Records (ADRs) document the rationale behind every significant modeling decision. In Syscribe they are first-class elements with a stable `ADR-*` identifier.

## ADR element schema

```yaml
---
type: ADR
id: ADR-SYS-SAFE-001
name: Decompose safety goal into FC fault detection and safe landing sub-requirements
status: accepted
---

## Context

The top-level safety requirement REQ-UAV-SAFE-000 is too broad to assign to a single
architecture element. It needs to be split into concrete, measurable sub-requirements.

## Decision

Decompose REQ-UAV-SAFE-000 into:
- REQ-UAV-FC-001 (flight controller fault detection ≤ 50 ms)
- REQ-UAV-SAFE-001 (autonomous safe landing on battery-critical event)

Both are assigned to software elements in the avionics bay.

## Consequences

The flight controller becomes the primary safety-relevant software element (SIL 3 / ASIL C).
A dedicated safe-landing action must be formally verified.
```

## Required fields

| Field | Required | Values |
|---|---|---|
| `id` | Yes | `ADR(-[A-Z0-9]{2,12})*-[0-9]{3,8}` |
| `name` | Yes | Short description of the decision (free prose) |
| `status` | Yes | `proposed` · `accepted` · `deprecated` · `superseded` |

## Lifecycle

```
proposed → accepted → deprecated
                   ↘ superseded (by a newer ADR)
```

An ADR is `proposed` while under review. It must be `accepted` before it can be cited in `breakdownAdr:`. Citing a `proposed` ADR from an `approved` requirement fires **W303**.

## Superseding an ADR

A newer ADR replaces an older one with `supersedes:` (an ADR id or qualified name, or a list):

```yaml
id: ADR-SYS-012
status: accepted
supersedes: ADR-SYS-004
```

| Code | Meaning |
|---|---|
| `E320` | `supersedes:` names an element that does not resolve or is not an `ADR` |
| `E321` | The ADR is on a `supersedes:` cycle (a self-reference included) |
| `W313` | An `accepted` ADR supersedes an ADR whose status is not `superseded` (set the old one to `superseded`) |
| `W314` | A non-draft element's `breakdownAdr:` resolves to a `superseded` ADR (cite its successor) |

## Amending an accepted ADR

An accepted decision is **amended, never silently edited**. When implementation shows it must change — the patch decided for message A must also cover message B — edit the decision and record the change in `amendments:`:

```yaml
id: ADR-SYS-012
status: accepted
amendments:
  - date: 2026-03-01
    by: alice
    reason: "The patch also applies to message B"
    kind: scope-change          # clarification | scope-change
    ref: PI-PATCH-001#Q2        # the owner's answer that approved it
```

Let the tool write the entry — it fills `date` with today and `by` from `--by`, and checks `kind` and `ref` first:

```bash
syscribe -m model/ set ADR-SYS-012 amendment.add kind=scope-change reason="The patch also applies to message B" ref=PI-PATCH-001#Q2 --by alice
```

| Key | Meaning |
|---|---|
| `date` | ISO date of the change (`YYYY-MM-DD`) |
| `by` | Who made the change |
| `reason` | Why the accepted text changed |
| `kind` | `clarification` — the meaning is unchanged; `scope-change` — what is decided changed |
| `ref` | Optional: a question `PI-X#Qn`, an element id (typically a `ReviewRecord`), or a commit |

A `scope-change` needs an **owner reference**: an answered question, a `ReviewRecord`, or a commit a person made (a `Syscribe-As: human` trailer, or no `Syscribe-Actor` trailer and an author outside `[agents] actors`). A `Requirement` at `approved`/`implemented`/`verified` takes `amendments:` the same way. `show` and `export-html` print the history as a table.

To make unrecorded edits visible, opt in:

```toml
[traceability]
require_amendments = true
```

`validate` then reads each accepted ADR's (and confirmed requirement's) git history: when its body or normative fields changed since the commit that set the status — uncommitted edits included — and no amendment is dated on or after the change, it raises **W324**. Without git it reports nothing.

| Code | Meaning |
|---|---|
| `E330` | `amendments:` malformed (not a list of mappings, a missing or unknown key, a bad date or kind) or a `ref` that resolves to nothing |
| `W324` | Opt-in: normative content changed since acceptance with no amendment dated on or after the change |
| `W325` | A `scope-change` without an owner reference |

An amendment entry alone never makes links to the ADR suspect (`amendments` is outside the suspect projection); the edit it records does (`W090`). To **replace** a decision instead of changing it, write a new ADR with `supersedes:` (above).

## Where ADRs are required

The validator (rule §12.2) requires a `breakdownAdr:` on every requirement that has `derivedFrom:`. This means every time you split a parent requirement into children, you must:

1. Create an ADR explaining why the split is correct
2. Set the ADR to `accepted`
3. Reference it from each child: `breakdownAdr: Decisions::MyADR`

## ID convention

ADR IDs encode the domain and a sequence number:

| Example ID | Scope |
|---|---|
| `ADR-SYS-001` | System-level decision |
| `ADR-SW-SCHED-001` | Software scheduler decision |
| `ADR-HW-PWR-001` | Hardware power subsystem decision |
| `ADR-SYS-SAFE-001` | System-level safety decision |

Keep segment lengths between 2 and 12 uppercase alphanumeric characters.
