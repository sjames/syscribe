# Traceability

`GUIDE · TRACEABILITY`

Section §12 of the format spec defines seven enforced traceability rules. All are checked by the validator.

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
| Implementation | Architecture element | `implementedBy: [src/...]` |
| Test case verification | TestCase | `verifies: [REQ-*]` |
| Allocation (default) | Source element | `allocatedTo: [target]` — `allocatedFrom` is **derived** |

### Allocation — two forms (§12.9)

An allocation can be authored two ways, sharing one edge model and a derived
`allocatedFrom` reverse index (shown as **`## Allocated from`** on the target in `show`):

- **`allocatedTo:` on the source element** — the OSLC-canonical, lightweight default. The
  source holds `allocatedTo: <target>`; `allocatedFrom` is derived, never authored (same
  direction as `satisfies`/`verifies`/`refines`). Use this for simple allocations.
- **A standalone `Allocation` element** naming both `allocatedFrom` and `allocatedTo` — a
  *reified relationship artifact*, kept for **documented** allocations whose body carries
  rationale (e.g. the freedom-from-interference / deployment allocations of §12.6). Naming
  both endpoints is its purpose, not redundancy.

Both forms feed `MG041`/`MG081`, `matrix --allocations`, and the derived index identically. A
`features:` entry is an edge when it has both `allocatedFrom` and `allocatedTo`, with or
without a per-entry `type: Allocation`. Declaring the **same** edge in *both* forms is
redundant — warning **`W503`**. Guidance: `allocatedTo` by default; promote to an `Allocation`
element only when the allocation needs its own documentation.

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

## Rule 7 — Implementation trace (§12.8)

The optional `implementedBy:` field closes the downstream leg of the V-model, linking an architecture element to the source artifact(s) that realise it:

```
Requirement ─satisfies→ Architecture ─implementedBy→ Code ─verifies→ Test
```

```yaml
# Scheduler.md — architecture element pointing at its implementation
type: PartDef
domain: software
satisfies: [REQ-SCHED-001]
implementedBy:
  - src/scheduler/mod.rs
  - repo:src/scheduler/bitmap.rs
```

The field accepts a single string or a list. Path resolution is identical to a TestCase's `sourceFile`: model-/repo-relative, `model:`/`repo:` prefixes, absolute, and `file://` paths are checked on disk; remote URIs (`scheme://`) are accepted as external pointers and not verified locally.

- **W023** — a non-`draft` `Part`/`PartDef`/`Interface`/`InterfaceDef` has an `implementedBy:` path that does not exist on disk

The check is **opt-in** (only when `implementedBy:` is present) and **draft-suppressed** (skipped for `status: draft`, where the implementation may not exist yet). Gate it in CI with `validate --deny W023`.

Discoverability:

- `syscribe links <element>` lists the `implementedBy` paths as outbound relationships.
- `syscribe refs <path-or-dir>` reverse-maps a source path (or a directory prefix) back to the architecture element(s) that declare it.

## Integration test coverage for parent requirements (W305)

Leaf-level TestCases (L1/L2) verify individual derived requirements but do not cover the emergent, composed behaviour expressed by the parent. A parent requirement at `approved`, `implemented`, or `verified` status must have at least one active TestCase at `testLevel: L3` (system), `L4` (system integration), or `L5` (hardware-in-the-loop).

```yaml
# TC-UAV-PERF-001.md — system test covering the parent requirement
type: TestCase
id: TC-UAV-PERF-001
name: Full performance envelope system test
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

## Suspect links — has a reviewed link gone stale? (`ADR-SYS-SUSLINK-001`)

A trace link asserts a relationship that was valid *when a human reviewed it*. When the
target later changes, the assertion may silently become stale — a test may no longer cover
the requirement it verifies. **Suspect-link detection** flags this by storing a content
**baseline** of the target at review time and re-checking it on every `validate`.

The baseline is a `blake3` hash of the target's *normative projection* (its body plus
normative frontmatter — `status`, `reqDomain`, safety fields, …). Editorial fields (`name`,
`displayOrder`, `extRef`, diagram layout) are **excluded**, so cosmetic edits never raise a
false flag. It lives on the **source** element (the one holding the link, per Rule 1),
keyed by the target reference:

```yaml
# Verification/SafeLandingTest.md — a reviewed verification link
verifies:
  - REQ-UAV-SAFE-001
traceBaselines:
  REQ-UAV-SAFE-001: "blake3:1ddab032…"   # captured with `suspect accept`
```

The feature is **opt-in and additive**: a link with no baseline is never flagged. Baseline
a reviewed link, and thereafter any change to the target's projected content raises **W090**.

```bash
# Discover links (suspect vs. never-baselined); onboard an existing model in one pass
syscribe -m model/ suspect list
syscribe -m model/ suspect accept --all-unbaselined     # baseline every un-baselined link

# Capture a single reviewed link, then gate CI on staleness
syscribe -m model/ suspect accept TC-UAV-SAFE-001 REQ-UAV-SAFE-001
syscribe -m model/ validate --deny W090
```

Try it on the demo model: edit the body of `REQ-UAV-SAFE-001` and re-run `validate` — the
`TC-UAV-SAFE-001 → REQ-UAV-SAFE-001` link (baselined above) is now reported suspect (W090).
Review the change and clear it by re-running `suspect accept` (or `suspect accept --all` to
clear every suspect link at once). Propagation is **implicit, one hop per review**: only the
direct link goes suspect; a link *into* the source flips only once the source's own
projection actually changes. The same operations are exposed over MCP as `suspect_list` and
the guarded `suspect_accept`.
