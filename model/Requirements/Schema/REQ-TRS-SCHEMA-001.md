---
type: Requirement
id: REQ-TRS-SCHEMA-001
name: "Unrecognized frontmatter fields raise a warning"
status: draft
reqDomain: software
reqClass: stakeholder
tags:
  - schema
  - validation
---

Syscribe shall recognise a fixed schema of frontmatter fields, as defined by the format
specification. Any **top-level** frontmatter key on an element that is **not** a
recognised schema field — and is not the sanctioned `custom_fields:` mechanism — shall
raise warning **`W047`**. The warning shall identify both the offending key and the
element's file, and shall direct the author to move author-defined or ad-hoc data under
`custom_fields:` (§3.15).

The warning shall be **advisory** (warning severity, never a hard error) so that
pre-existing models continue to validate, and it shall be **CI-gateable** via
`--deny W047`. A recognised schema field shall **never** trigger `W047`, and any key
placed under `custom_fields:` shall **never** trigger it. An element with no
unrecognised keys shall produce no `W047` findings.

## Rationale

The parser accepts a frontmatter document and binds each recognised key to a typed
field; any remaining key is captured by a catch-all and **silently discarded** from
validation. This silent drop is a correctness hazard: a typo such as `reqDomian:` (for
`reqDomain:`) or `verifis:` (for `verifies:`) is accepted without complaint, so the
author believes a constraint, a domain, or a trace link is set when in fact the data was
thrown away. Nothing in the report reveals the mistake.

Turning an unrecognised key into a visible `W047` converts silent data loss into an
actionable diagnostic that points at the exact key and file. It also establishes a
single, sanctioned channel for genuinely custom metadata — `custom_fields:` — which,
unlike the anonymous catch-all, is preserved, queryable (`custom.<key>`), and
shape-checked (`W041`). Authors who want custom data are steered to the mechanism built
for it; authors who made a mistake are told immediately.
