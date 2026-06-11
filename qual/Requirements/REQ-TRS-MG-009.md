---
id: REQ-TRS-MG-009
type: Requirement
title: MagicGrid gate shall validate a System-of-Interest boundary marker (mg_soi)
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** support a positive **System-of-Interest** marker for the MagicGrid
System Context (cell B3) and **shall** validate it under the MagicGrid gate
(`[profiles.magicgrid]`, [[REQ-TRS-OUT-012]]). Today `mg_external: true` ([[REQ-TRS-MG-002]])
marks what lies *outside* the boundary, but nothing marks the SoI itself — so the boundary
that B3 is fundamentally about is implicit and the tool cannot answer "what is the system of
interest" (the gap surfaced while modelling `model_mg/`, where the SoI was an ordinary
`PartDef` indistinguishable from its internals).

The SoI is expressed as `custom_fields: { mg_soi: true }` on the system-of-interest
`Part`/`PartDef` (no new element type). The field is inert in the base format; the checks
below emit `MG06#` findings only when the gate is active.

### Gate checks (active only under `--profile magicgrid`)

- **`MG060` — wrong host.** `mg_soi: true` on an element that is not a `Part`/`PartDef`.
- **`MG061` — ambiguous boundary.** More than one element in the model is marked
  `mg_soi: true` — a MagicGrid model has a single system of interest.
- **`MG062` — contradictory marking.** An element marked **both** `mg_soi: true` and
  `mg_external: true` (the SoI cannot also be external to itself).

Zero SoI markers is **not** an error (a model need not adopt the marker). When exactly one
SoI is present, the `magicgrid` report ([[REQ-TRS-MG-003]]) **shall** identify it (e.g. note
the SoI alongside the B3 Structure cell), so the System-Context boundary is legible.

**Source:** MagicGrid System Context boundary (B3) — gap identified building the `model_mg/`
EV-charging-station model. Complements the external-actor marking of [[REQ-TRS-MG-002]]
(`mg_external`).

**Acceptance criteria:**

- A single `PartDef` with `mg_soi: true` validates clean under the gate and is identified as
  the system of interest in the `magicgrid` report.
- `mg_soi: true` on a `Requirement` (or other non-part) raises `MG060`.
- Two elements marked `mg_soi: true` raise `MG061`; an element marked both `mg_soi: true`
  and `mg_external: true` raises `MG062`.
- `mg_soi` produces no finding when the magicgrid profile is not active.
