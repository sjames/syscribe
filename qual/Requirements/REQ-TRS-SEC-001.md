---
id: REQ-TRS-SEC-001
type: Requirement
name: Tool shall cross-link damage/threat scenarios to hazards/safety goals and provide a co-analysis view
status: draft
reqDomain: software
verificationMethod: test
---

Syscribe is unusual in holding **both** a functional-safety layer
(`HazardousEvent`, `SafetyGoal` — ISO 26262 / IEC 61508) and an automotive
cybersecurity layer (`DamageScenario`, `ThreatScenario`, `CybersecurityGoal`,
`SecurityControl` — ISO/SAE 21434) in one model. Historically the two analyses
were **disjoint graphs**: nothing linked a cyber damage/threat scenario to the
hazard or safety goal it can endanger, and nothing checked for security-induced
safety impact. ISO/SAE 21434 requires damage scenarios to consider a **safety**
impact category, and ISO 26262 increasingly requires considering **malicious**
causes of hazards. A dual FS+CS assessor of a connected vehicle asks first:
*"which cyber threats can violate a safety goal, and where is that analysed?"*

This requirement adds the cross-link field, two validation checks, and a
read-only co-analysis view that answers that question.

## `hazardRef` field

The tool **shall** accept an optional **`hazardRef`** field on `DamageScenario`
and `ThreatScenario` frontmatter.

- The value **shall** be either a **single string** or a **list of strings**,
  following the common string-or-list pattern (as `extRef`/`implementedBy`).
- Each value **shall** resolve via the standard cross-reference resolver
  (`id` or qualified name) to a **`HazardousEvent`** or **`SafetyGoal`** — the
  hazard/safety goal the scenario can endanger.
- The field **shall** be **optional**. A scenario with no `hazardRef` represents
  no declared safety linkage.

```yaml
# DamageScenario or ThreatScenario
hazardRef: SG-ENG-001
# or
hazardRef:
  - HE-ENG-001
  - SG-ENG-002
```

## `E844` — unresolved or wrong-type hazardRef

The tool **shall** define error code **`E844`**, emitted when a `hazardRef`
value on a `DamageScenario` or `ThreatScenario`:

- **does not resolve** to any model element, **or**
- resolves to an element that is **not** a `HazardousEvent` or `SafetyGoal`.

One finding is emitted per offending `hazardRef` value.

## `W030` — safety-tagged DamageScenario with no hazardRef

The tool **shall** define warning code **`W030`**, emitted by `validate` when a
`DamageScenario` whose `impactCategories` includes `safety` has **no**
`hazardRef`. This is the cross-domain gap an FS+CS assessor flags first.

- **Opt-in** — fires only for safety-tagged damage scenarios; a damage scenario
  without `safety` in `impactCategories` never triggers it.
- **Non-fatal but gateable** — `W030` is a warning (exit code unchanged), and
  is selectable via `--deny W030` so a project may make the gap fail the build.

## `co-analysis` command

The tool **shall** provide a read-only command
`syscribe -m <root> co-analysis [--json]` that builds the cross-domain chain

```
ThreatScenario --damageScenarios--> DamageScenario --hazardRef--> HazardousEvent/SafetyGoal
```

(plus a `ThreatScenario`'s own direct `hazardRef`, if set) and outputs:

- For each `SafetyGoal`/`HazardousEvent` that is a `hazardRef` target: the
  safety-relevant `DamageScenario`s linked to it and, transitively, the
  `ThreatScenario`s that lead to them — i.e. *which cyber threats can violate
  this safety goal/hazard.*
- A section listing safety-tagged `DamageScenario`s with **no** `hazardRef`
  (the W030 gaps).

Output modes: **text** is a readable grouped report; **`--json`** is a
structured document `{ goals: [{ id, type, damageScenarios:[...], threats:[...] }],
unlinkedSafetyDamage: [...] }`. With no relevant model content the command emits
a notice and exits `0`. The view traverses frontmatter and the resolver
directly; it does **not** build the model graph.

## Deferred (future work)

Issue #28 check (b) — *"a SafetyGoal whose realising architecture has an attack
surface (a reachable `ThreatScenario`/`VulnerabilityReport`) with no security
consideration"* — is **deferred**. It requires goal→architecture→vulnerability
reachability analysis and is out of scope for this requirement; it is tracked as
future work.

**Source:** combined functional-safety (ISO 26262) + automotive-cybersecurity
(ISO/SAE 21434) audit; GH issue #28. Complements the safety-only checks
(E800-E837, W800-W807).

**Acceptance criteria:** `hazardRef` parses as a single string and as a list on
both `DamageScenario` and `ThreatScenario`; a `hazardRef` that does not resolve,
or that resolves to a non-HazardousEvent/non-SafetyGoal element, produces
`E844`; a safety-tagged `DamageScenario` with no `hazardRef` produces exactly one
`W030`, and linking it clears the finding; a non-safety damage scenario never
triggers `W030`; `validate --deny W030` exits non-zero in the presence of a
`W030`; `co-analysis` (text) names each `SafetyGoal`/`HazardousEvent` with the
`ThreatScenario`s that can violate it and the unlinked safety-tagged damage
scenarios; `co-analysis --json` is valid JSON carrying `goals` and
`unlinkedSafetyDamage`; an empty model exits `0`.
