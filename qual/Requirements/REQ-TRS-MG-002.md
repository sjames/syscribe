---
id: REQ-TRS-MG-002
type: Requirement
title: MagicGrid gate shall validate use-case actors, external boundary, and mandatory actors
status: draft
reqDomain: software
verificationMethod: test
---

Under the **MagicGrid gate** (`[profiles.magicgrid]`, [[REQ-TRS-OUT-012]]) the tool **shall**
validate the use-case `actors:` of a model as MagicGrid System-Context (B3) participants.
This requirement is an **overlay**: the base `actors:` field (accepted on `UseCaseDef`,
`UseCase`, and use-case-style `RequirementDef`/`Requirement`, §8.12.4) is **unchanged and
remains inert in the base format** — a model that does not opt into MagicGrid sees none of
these findings. MagicGrid-specific data rides on `custom_fields:` with the **`mg_`** prefix
(here, the external-boundary marker), and the checks fire only when the gate is active,
emitting findings in the dedicated **`MG###`** code namespace (kept out of the base code
registry to reinforce the opt-in overlay).

No new element type is introduced: an actor is a `Part`/`PartDef` (SysMLv2 models the
actor as a part usage), and "external to the system-of-interest" is expressed as
`custom_fields: { mg_external: true }` on that part.

### Gate checks (active only under `--profile magicgrid`)

Each `actors:` entry **shall** be resolved by qualified name or stable id (§11.10), and:

- **`MG010` — unresolved actor.** An `actors:` entry that resolves to no model element.
- **`MG011` — actor is not a part.** An entry resolving to an element that is not a
  `Part`/`PartDef` (names the resolved type).
- **`MG012` — actor not external.** A referenced actor part that is **not** marked
  `custom_fields: { mg_external: true }` — under MagicGrid an actor sits outside the SoI
  boundary, so a non-external actor is a B3 modelling error.
- **`MG013` — use case has no actor.** A non-`draft` `UseCaseDef` that declares an empty or
  absent `actors:` list — every black-box use case must name at least one actor.

Each entry is checked independently. The gate also **shall** compute/surface the inverse
**`actorIn`** index (the use cases each actor part participates in) for the grid report.

**Source:** MagicGrid System Context (B3). **Supersedes the base-format `E108`/`E109`
approach in the prior draft of this requirement**, per the agreed overlay design (all
MagicGrid validation gated, data on `mg_` custom fields, base format untouched). Reuses
id-based resolution (§11.10); gated via [[REQ-TRS-OUT-012]]; sibling of [[REQ-TRS-MG-001]].

**Acceptance criteria:**

- With the magicgrid gate inactive, a `UseCaseDef` with a dangling or non-part `actors:`
  entry produces **no** finding (base format unaffected).
- Under `validate --profile magicgrid`, an `actors:` entry naming no element raises
  `MG010`; one resolving to a non-part raises `MG011` (with the resolved type); the other
  entries in the same list are still checked.
- Under the gate, a referenced actor part lacking `mg_external: true` raises `MG012`, and a
  non-`draft` `UseCaseDef` with no `actors:` raises `MG013`.
- An actor part referenced by two use cases reports both under its computed `actorIn` index.
