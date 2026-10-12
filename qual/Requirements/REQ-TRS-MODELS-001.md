---
id: REQ-TRS-MODELS-001
type: Requirement
name: Tool shall check an executable part's models link against the architecture element it models
status: draft
reqDomain: software
verificationMethod: test
---

An executable part may declare `models:`, the architecture element(s) it is a behavioural model of, with optional `portMap:`, `abstracts:` and `fidelity:` (spec §12.11). The executable model and the architecture differ in granularity and fidelity, so the link is many-to-many and the checks run on the union of the parts that model an element.

The tool **shall**:

- report a `models:` target that does not resolve or is not a `PartDef`/`Part` with a `domain:` as `E990`;
- report an executable port that corresponds (by name or `portMap:`) to no port of the elements it models, a `portMap:` key that is not a port, or a counterpart of incompatible direction as `E991`; a `portMap:` value may be a list or `null`;
- report a port of a modelled element that no modelling part covers and none names in `abstracts:` as `W990`;
- report a connection of an executable composite with no architecture counterpart, or an architecture connection between two modelled elements with no executable counterpart, as `W991`, skipping connections across the hardware/software boundary (§12.6);
- report a `TestCase` that verifies a requirement whose architecture satisfiers no part of its `subject:` models, or whose modelling parts all name the requirement in `abstracts:`, as `W992`;
- show the link in `links`, `refs`, `follow`, `impact`, `trace` and `why`, and include it in suspect-link detection;
- report none of these codes for a model in which no part declares `models:`.

**Source:** GH #330, found while building the grid-tie inverter model (`model_gti/`): a protection TestCase listed `REQ-GTI-SYS-037` and `REQ-GTI-HW-002` although the executable chain models neither.

**Acceptance criteria:**

- A model whose executable parts model the architecture consistently validates with none of `E990`, `E991`, `W990`–`W992`.
- Each defect class gives its code.
