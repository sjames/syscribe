---
id: REQ-TRS-CLI-006
type: Requirement
name: "Tool shall print a MagicGrid modeling prompt via --agent-instructions magicgrid"
status: draft
reqDomain: software
verificationMethod: test
---

The `--agent-instructions` flag **shall** accept an **optional topic argument** so an LLM can
be taught either general Syscribe modeling or MagicGrid modeling specifically:

- `syscribe --agent-instructions` (no topic) **shall** print the general model-authoring
  prompt (unchanged — the embedded `prompts/create-model.md`).
- `syscribe --agent-instructions magicgrid` **shall** print a **dedicated, self-contained
  MagicGrid modeling prompt** that teaches an LLM the MagicGrid method *and* how to author a
  MagicGrid model with this tool: the `mg_`-prefixed `custom_fields:` overlay
  (`mg_cell`/`mg_external`/`mg_soi`/`mg_moe*`/`mg_mop*`/`mg_layer`/`mg_variant`), the base
  `refines:`/`actors:`/`allocatedTo:` links, the `[profiles.magicgrid]` gate, the
  authoring workflow (stakeholder needs → use cases → system context → MoEs → system
  requirements → functional analysis → logical/physical architecture → allocations →
  configurations → trade study), the read-only commands (`magicgrid`, `magicgrid --audit`,
  `trade-study`, `matrix --allocations`), and the `MG###` findings and how to clear them.
- An **unrecognised** topic **shall** exit non-zero with an error naming the available topics.

The MagicGrid prompt **shall** be embedded in the binary (`include_str!`), print to stdout,
and require **no** model directory (handled before model resolution, like the base flag).

**Source:** user request — a CLI helper to teach an LLM MagicGrid modeling with Syscribe.
Extends the `--agent-instructions` mechanism of [[REQ-TRS-CLI-003]]; the content mirrors the
[MagicGrid guide](../../docs/model-guide/magicgrid.md) and the features of
[[REQ-TRS-MG-001]]–[[REQ-TRS-MG-014]] and [[REQ-TRS-ALLOC-001]].

**Acceptance criteria:**

- `--agent-instructions magicgrid` exits 0 and prints the MagicGrid prompt — it names the
  grid (`mg_cell`), the gate (`validate --profile magicgrid`), and the reports
  (`magicgrid --audit`, `trade-study`).
- `--agent-instructions` with no topic still prints the general authoring prompt and **not**
  the MagicGrid prompt's distinctive heading.
- `--agent-instructions <unknown>` exits non-zero and the message names the available topics
  (including `magicgrid`).
- Both forms work without a model directory present.
