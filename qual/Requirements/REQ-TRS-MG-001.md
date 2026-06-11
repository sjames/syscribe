---
id: REQ-TRS-MG-001
type: Requirement
title: UseCaseDef shall carry an optional refines link to requirements; a magicgrid profile gates missing refinement
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** support an optional `refines:` traceability link on a `UseCaseDef`
(and its `UseCase` usages), expressing the SysML `«refine»` relationship by which a use
case gives concrete behavioural meaning to a requirement. The link is **methodology-
neutral** in the base format — plain SysML does not mandate it, so its absence is never an
error — while teams following the **MagicGrid** methodology can promote a missing link to
a CI gate, because MagicGrid is needs-first (a use case `«refine»`s a stakeholder need
that already exists; problem-domain black-box row B2 → B1).

### The `refines:` field

- `refines:` is an **optional list of strings** accepted on `UseCaseDef` and `UseCase`.
  Absent or empty means "declares no refinement" — legal in the base format.
- Each operand is a cross-reference to a **requirement** — a native `Requirement` or a
  `RequirementDef` — resolved by its **qualified name or its stable `REQ-*` id**, using
  the same id-based resolution as `verifies:` and `derivedFrom:` (§11.10).
- **Direction (§12.1, [[REQ-TRS-TRACE-001]]).** The link points **upstream**: the use case
  (the concrete, downstream element) holds `refines:` and names the requirement (the
  abstract, upstream artifact). This matches the ownership of `satisfies:`, `verifies:`,
  and `derivedFrom:` — the downstream element always holds the reference.
- **Reverse index (§11.11).** The tool **shall** compute the inverse `refinedBy` index on
  each referenced requirement, alongside the existing `verifiedBy` / `derivedChildren`
  indices, so the refinement is navigable from the requirement side.

### Resolution and target validation (`E316`)

- The tool **shall** emit error **`E316`** when a `refines:` operand does **not** resolve,
  or resolves to an element that is **not** a `Requirement`/`RequirementDef`. The message
  names the offending operand and the owning use case. (A use case may legitimately refine
  more than one requirement; each operand is checked independently.)

### Missing-refinement warning (`W307`)

- The tool **shall** emit warning **`W307`** when a `UseCaseDef` at a non-`draft` status
  carries no `refines:` link (absent or empty). `W307` is **advisory and draft-suppressed**
  — a `draft` use case never triggers it — and **gateable** with `--deny W307`, exactly
  like the other "should-trace" warnings (`W300`, `W302`, `W023`).

### MagicGrid profile gate ([[REQ-TRS-OUT-012]])

- The named-severity-profile mechanism **shall** be able to promote `W307` to a gate
  failure. The project ships (or a team declares) a `[profiles.magicgrid]` profile in
  `<model_root>/.syscribe.toml` such that `syscribe validate --profile magicgrid` exits
  non-zero (exit 2) when any non-`draft` `UseCaseDef` lacks a `refines:` link, enforcing
  MagicGrid's needs-first ordering. The profile is the home for future MagicGrid gates.

**Source:** MagicGrid methodology support (needs-first ordering; use case `«refine»`s a
stakeholder need, B2 → B1). Builds on the OSLC upstream-direction rule of
[[REQ-TRS-TRACE-001]] (§12.1), the id-based cross-reference resolution and reverse-index
machinery shared with `verifies:`/`derivedFrom:` (§11.10, §11.11), and the named severity
profiles of [[REQ-TRS-OUT-012]].

**Acceptance criteria:**

- A `UseCaseDef` with `refines: [REQ-UAV-STK-001]` parses, resolves the target by id, and
  the named requirement reports the use case under its computed `refinedBy` index; the same
  works when the operand is the requirement's qualified name.
- `refines:` naming an unresolved operand, or an operand that resolves to a non-requirement
  (e.g. a `PartDef`), produces `E316` against the use case.
- A non-`draft` `UseCaseDef` with no `refines:` link produces `W307`; the same use case at
  `status: draft` produces no finding; plain `validate` exit code is unaffected by `W307`
  alone.
- With a `[profiles.magicgrid]` profile present, `validate --profile magicgrid` exits
  non-zero when any non-`draft` use case lacks `refines:`, and exits zero once every such
  use case declares a resolvable refinement.
