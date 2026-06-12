---
id: REQ-TRS-MG-014
type: Requirement
name: MagicGrid gate shall flag grid completeness / coverage gaps (MG080-MG083)
status: draft
reqDomain: software
verificationMethod: test
---

The MagicGrid gate (`[profiles.magicgrid]`, [[REQ-TRS-OUT-012]]) **shall** add **completeness /
coverage** checks — the *gap-analysis* half of MagicGrid validation that complements the
existing well-formedness checks — verifying that each link of the MagicGrid trace chain
(needs → use cases → system context → MoEs → system requirements → architecture) is actually
present. These are **advisory warnings** (`Severity::Warning`, draft-suppressed where
applicable, promotable via a profile and gateable with `--deny`), surfaced by
`magicgrid --audit` ([[REQ-TRS-MG-013]]). They emit only when the gate is active.

### Gate checks (active only under `--profile magicgrid`)

- **`MG080` — orphan stakeholder need.** A non-`draft` `Requirement` in cell `B1`
  (`custom_fields: { mg_cell: B1 }`) that is **neither refined by any use case/behavioral
  element** (`refinedBy` empty) **nor derived into any system requirement**
  (`derivedChildren` empty) — a dangling top-of-trace need.
- **`MG081` — unallocated functional-analysis element.** An `ActionDef`/`Action`/`StateDef`/
  `State` in cell `W2` (`custom_fields: { mg_cell: W2 }`) that is the `allocatedFrom` of no
  `Allocation` edge whose target is a `Part`/`PartDef` marked `mg_layer: logical` — a
  function not allocated to any logical (W3) subsystem.
- **`MG082` — missing System of Interest.** The model declares a System Context — at least
  one `mg_external: true` element — but **no** element is marked `mg_soi: true`. Emitted once
  (model-level): a B3 system context with no system-of-interest boundary.
- **`MG083` — MoE without a MoP.** An `mg_moe` element with an empty `mopRefinedBy` index —
  a Measure of Effectiveness that no Measurement of Performance refines (the measure is never
  realised white-box).

**Source:** MagicGrid model completeness / coverage / gap-analysis validation (researched from
the MagicGrid V&V approach and the Cameo validation suite — model completeness & correctness,
gap analysis, coverage, traceability). Builds on the reverse indices and overlay markers of
[[REQ-TRS-MG-001]], [[REQ-TRS-MG-002]], [[REQ-TRS-MG-005]], [[REQ-TRS-MG-008]]; surfaced by the
audit of [[REQ-TRS-MG-013]].

**Acceptance criteria:**

- A non-`draft` B1 need with no refining use case and no derived system requirement raises
  `MG080`; adding either a refining use case or a `derivedFrom` child clears it; a `draft`
  B1 need never raises it.
- A W2 `ActionDef` allocated to no logical part raises `MG081`; adding an `Allocation` from it
  to an `mg_layer: logical` part clears it.
- A model with an `mg_external` actor but no `mg_soi` raises `MG082`; marking one part
  `mg_soi: true` clears it; a model with no `mg_external` element raises no `MG082`.
- An `mg_moe` element that no `mg_mop` refines raises `MG083`; adding a MoP whose
  `mg_mop_refines` names it clears it.
- All four are inert (no finding) when the magicgrid profile is not active, and none change
  the `validate` exit code on their own (warnings).
