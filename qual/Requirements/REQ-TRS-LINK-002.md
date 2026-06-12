---
id: REQ-TRS-LINK-002
type: Requirement
title: Tool shall wrap each element's SVG shape in a hyperlink to its hosted URL
status: draft
reqDomain: software
verificationMethod: test
---

When a `[links]` source URL is configured ([[REQ-TRS-LINK-001]]), every **SVG** diagram the
renderer produces **shall** make each file-backed element's shape **clickable** by wrapping its
shape group in an SVG hyperlink to that element's resolved URL.

### Behaviour

- Each element shape group is wrapped as
  `<a xlink:href="<url>" href="<url>" target="_blank" rel="noopener">…shape…</a>` (both the
  SVG 1.1 `xlink:href` and the SVG 2 `href` for broad renderer support), opening in a new tab.
- A shape whose element resolves to **no** URL (no config, or a non-file element) is left
  **unwrapped** — never an empty/`#` link.
- The `<url>` is XML-attribute-escaped.
- This applies to **all** renderer SVG outputs — block-definition, internal-block, sequence,
  state, use-case diagrams, and the MagicGrid grid / allocation-matrix / trade-study SVGs.
- Inert when `[links]` is not configured (the SVG is byte-for-byte as today).

**Source:** clickable element links in exported SVG diagrams. Consumes [[REQ-TRS-LINK-001]].
The live-server SVG affordance is [[REQ-TRS-LINK-005]].

**Acceptance criteria:**

- With `[links]` configured, an exported SVG (e.g. `magicgrid --svg`, or a rendered BDD) wraps
  each element's shape in `<a … href="<the element's URL>" target="_blank">`.
- An element with no resolved URL has its shape rendered with **no** surrounding `<a>`.
- With no `[links]` table, the SVG contains no `<a>` element wrappers.
