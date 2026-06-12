---
id: REQ-TRS-MG-005
type: Requirement
name: MagicGrid gate shall validate logical/physical layering and logical-to-physical allocation
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** let a model distinguish MagicGrid's **logical** subsystems (white-box
W3) from **physical** components (solution S3) via a `custom_fields: { mg_layer: <v> }`
overlay on a `Part`/`PartDef`, and **shall** validate the layering and the logical→physical
realisation under the MagicGrid gate (`[profiles.magicgrid]`, [[REQ-TRS-OUT-012]]). The
field is inert in the base format; the checks below emit `MG04#` findings only when the gate
is active. No new element type is introduced — the existing single `PartDef` notion is
annotated, and realisation reuses the existing `Allocation` element.

### Gate checks (active only under `--profile magicgrid`)

- **`MG040` — bad layer.** `mg_layer` is present and not `logical` or `physical`.
- **`MG041` — unrealised logical element.** A `Part`/`PartDef` with `mg_layer: logical` and
  no `Allocation` to a `physical` element — a logical subsystem must be realised by at
  least one physical component (analogous to the deployment-package rule `E314`, §12.6).
- **`MG042` — cross-layer coupling.** A `logical` and a `physical` element that share a
  `supertype:`/`typedBy:` link directly — logical and physical layers must be related only
  through an explicit `Allocation`, not by specialization/typing (mirrors the HW/SW
  independence rule `E315`, §12.6).

**Source:** MagicGrid logical (W3) vs physical (S3) solution layering and the
logical→physical allocation between them. Overlay design: layer on `mg_layer`, validation
gated via [[REQ-TRS-OUT-012]], realisation via the existing `Allocation` element. The
resulting allocations are surfaced by the matrix of [[REQ-TRS-MG-006]].

**Acceptance criteria:**

- With the gate inactive, `mg_layer` on a part raises no finding.
- Under `validate --profile magicgrid`, `mg_layer: subsystem` raises `MG040`; a
  `mg_layer: logical` part with no `Allocation` to a physical part raises `MG041`.
- A `logical` part whose `supertype:` is a `physical` part raises `MG042`; routing the same
  relationship through an `Allocation` instead clears it.
