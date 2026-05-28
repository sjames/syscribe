# Syscribe Traceability Rules (§12)

Seven mandatory traceability rules. A conformant tool enforces these via the validation
codes listed below.

---

## R-001 — OSLC link direction (§12.1)

**Rule:** The derived/verifying/satisfying artifact always holds the link field pointing
upstream. No reverse links are stored in model files.

| Link field | Direction | Who holds it |
|---|---|---|
| `derivedFrom:` | child → parent | child `Requirement` |
| `verifies:` | test → requirement | `TestCase` |
| `satisfies:` | arch element → requirement | `Part`/`PartDef` |
| `allocatedFrom:` | arch element → logical/security artifact | architecture element |
| `allocatedTo:` | allocation → target | `Allocation` element |
| `breakdownAdr:` | requirement → ADR | child `Requirement` |

Reverse indices (`verifiedBy`, `derivedChildren`, `satisfiedBy`) are **computed at load
time and never written to disk**.

---

## R-002 — Requirement breakdown needs an ADR (§12.2)

**Rule:** Every `Requirement` with one or more `derivedFrom:` entries must set
`breakdownAdr:` to the ID/QName of an `accepted` ADR documenting the breakdown rationale.

| Violation | Code |
|---|---|
| `breakdownAdr:` absent | `E310` |
| `breakdownAdr:` resolves to non-ADR or unresolvable | `E311` |
| Referenced ADR is still `proposed` | `W303` |

**Procedure:**
1. Author ADR (`type: ADR`, `status: proposed`).
2. Review and set `status: accepted`.
3. Create child requirements with `derivedFrom:` and `breakdownAdr:`.

---

## R-003 — Leaf-level assignment (§12.3)

A requirement is a **leaf** when no other requirement has `derivedFrom:` pointing to it.

**Rule:** A leaf `Requirement` at `status: approved` or higher must be assigned to
exactly one architecture element (one `Part`/`PartDef` has `satisfies:` pointing to it).

| Violation | Code |
|---|---|
| Zero satisfying elements | `W300` |
| More than one satisfying element | `W301` |

Assignment should point to the **deepest known** architecture element responsible for
fulfilling the requirement.

---

## R-004 — Parent requirements cannot be assigned (§12.4)

**Rule:** A requirement that has `derivedChildren` (has been broken down) must **not**
appear in any element's `satisfies:` list.

| Violation | Code |
|---|---|
| Parent requirement in `satisfies:` | `E312` |

---

## R-005 — Domain classification (§12.5)

| `reqDomain` | Meaning |
|---|---|
| `system` | Domain-agnostic; used for top-level requirements before allocation |
| `hardware` | Governs a hardware element |
| `software` | Governs a software element |

**Rule R-005a:** A leaf requirement must be satisfied only by an architecture element
whose `domain:` matches its `reqDomain:`, unless either is `system`. (Error `E313`.)

**Rule R-005b:** Leaf requirement at `implemented`/`verified` that still has
`reqDomain: system` should be refined. (Warning `W302`.)

---

## R-006 — Hardware/software independence (§12.6)

The hardware and software architectures are independent hierarchies that interact only
through `Allocation` elements.

**Rule R-006a:** `domain: software` elements must not have `supertype:`/`typedBy:`
referencing `domain: hardware`, or vice versa. (`E315`)

**Rule R-006b:** `isDeploymentPackage: true` parts must have at least one `Allocation`
to a `hardware` element. (`E314`)

**Correct cross-domain pattern:**

```yaml
# SW element
type: PartDef
name: SchedulerModule
domain: software
isDeploymentPackage: true

# Allocation — the only permitted cross-domain link
type: Allocation
name: schedulerToFC
allocatedFrom: Software::SchedulerModule
allocatedTo: Hardware::FlightComputer
```

---

## R-007 — Integrity level propagation (§12.7)

**Rule:** Once any element in the traceability chain carries `asilLevel:`, `silLevel:`,
or `plLevel:`, all downstream elements reached via `derivedFromSafetyGoal:`,
`derivedFrom:`, or `satisfies:` must also carry the same field.

A downstream element may carry a **lower** level (ASIL/SIL decomposition), but only
when `breakdownAdr:` is set to document the decomposition rationale.

| Link | Missing field | Lower level without ADR |
|---|---|---|
| `derivedFromSafetyGoal:` → `SafetyGoal` | `E841` | `W808` |
| `derivedFrom:` → parent `Requirement` | `E842` | `W808` |
| `satisfies:` → `Requirement` | `E843` | `W808` |

Level ranking:
- `asilLevel`: A < B < C < D
- `silLevel`: 1 < 2 < 3 < 4
- `plLevel`: a–e (not compared cross-element with ASIL/SIL)

**Example — ASIL D decomposed to ASIL B:**

```yaml
type: SafetyGoal
id: SG-BRAKE-001
asilLevel: D
hazardousEvents: [HE-BRAKE-001]
---
type: Requirement
id: REQ-BRAKE-HYD-001
asilLevel: B
derivedFromSafetyGoal: SG-BRAKE-001
breakdownAdr: ADR-BRAKE-DECOMP-001
```
