---
id: REQ-TRS-ID-008
type: Requirement
name: Tool shall accept a Configuration id with no numeric suffix, as for FEAT- and BL- ids
status: draft
reqDomain: software
verificationMethod: test
---

A `Configuration` names a product variant, and a descriptive id such as `CONF-GTI-US-M3` reads better than one forced to end in a number. `FEAT-*` ([[REQ-TRS-ID-006]]) and `BL-*` ids already need no numeric suffix.

The tool **shall**:

- accept a `Configuration` id matching `^CONF(-[A-Z0-9]{2,12})+$`, so `CONF-GTI-US-M3` and `CONF-GTI-IN-M2` raise neither `E200` nor `W042`;
- keep accepting the numeric form (`CONF-GTI-US-001`), which matches the same grammar; as for `FEAT-*`/`BL-*`, a single segment after the prefix is enough, so `CONF-001` is also valid;
- keep rejecting with `E200` a bare `CONF`, a lower-case segment, and a segment shorter than 2 or longer than 12 characters;
- compile an additional `[ids.prefixes]` prefix for `Configuration` ([[REQ-TRS-ID-007]]) with the same no-suffix grammar;
- keep the `E023` digit cap for an id whose last segment is numeric.

**Source:** GH #327, found while building the grid-tie inverter model (`model_gti/`).

**Acceptance criteria:**

- `CONF-GTI-US-M3` and `CONF-GTI-IN-M2` validate with no `E200` or `W042`, and `validate --config CONF-GTI-US-M3` selects that variant.
- `CONF-GTI-US-001` still validates.
- `CONF`, `CONF-gti-US`, `CONF-G` and a 13-character segment are `E200`.
