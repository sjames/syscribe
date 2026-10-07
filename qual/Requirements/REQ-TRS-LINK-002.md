---
id: REQ-TRS-LINK-002
type: Requirement
name: Tool shall wrap each element's SVG shape in a hyperlink to its hosted URL
status: verified
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

**Verification note:** the original verification (`TC-TRS-LINK-002`, via `diagram req`) was retired
with the CLI diagram toolkit (`ADR-SYS-VIS-001`). It is revived against the static SVG writer:
`TC-TRS-LINK-002` now exercises `syscribe diagram export --format svg` on a fully pinned diagram
(`REQ-TRS-VIS-009`/`REQ-TRS-VIS-010`), asserting the anchor wrapper with and without `[links]`,
and additionally checks the `click` directives of `--format mermaid` (the Mermaid counterpart,
`REQ-TRS-LINK-003`'s convention applied to generated text).

**Source:** clickable element links in exported SVG diagrams. Consumes [[REQ-TRS-LINK-001]].
The live-server SVG affordance is [[REQ-TRS-LINK-005]].

**Acceptance criteria:**

- With `[links]` configured, an exported SVG (`diagram export --format svg` of a fully pinned
  diagram, or `magicgrid --svg`) wraps each element's shape in
  `<a … href="<the element's URL>" target="_blank">`.
- An element with no resolved URL has its shape rendered with **no** surrounding `<a>`.
- With no `[links]` table, the SVG contains no `<a>` element wrappers.
