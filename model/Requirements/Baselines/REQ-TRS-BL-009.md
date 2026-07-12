---
type: Requirement
id: REQ-TRS-BL-009
name: "Baseline type and commands are documented for humans and LLMs"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-BL-000]
breakdownAdr: Decisions::BaselineADR
tags:
  - baseline
  - documentation
---

The `Baseline` element type and the `baseline` command family shall be documented consistently
with every other first-class element type and command.

- The `baseline` command shall have a detailed help page surfaced by `syscribe help baseline`
  and `syscribe baseline --help`, covering `create` / `verify` / `diff` / `list` / `show`.
- The `Baseline` type — its purpose, schema, `frozenScope`, seal, and lifecycle — shall be
  described in the model-authoring guidance: the LLM generation prompt
  (`prompts/create-model.md`, surfaced via `--agent-instructions`) and the model guide.
- The `BL-*` id scheme shall be documented in the project's ID Scheme reference alongside the
  other stable-id types.

This keeps the baseline feature discoverable and correctly authorable by both human modelers
and LLM agents.
