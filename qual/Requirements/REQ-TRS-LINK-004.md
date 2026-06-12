---
id: REQ-TRS-LINK-004
type: Requirement
name: Tool shall render element references in Markdown report and export as hosted links
status: draft
reqDomain: software
verificationMethod: test
---

When a `[links]` source URL is configured ([[REQ-TRS-LINK-001]]), the tool **shall** render
element references in its **Markdown** outputs as Markdown links to the element's resolved URL,
so a published report is navigable back into the model.

### Behaviour

- In the **validation report** (the default Markdown report), wherever an element is named with
  a link to its file path, the reference **shall** be rendered as `[<label>](<url>)` using the
  resolved hosted URL when one exists; otherwise the existing plain text / local path is kept.
- The `export` command's Markdown surfaces that name elements likewise link to the resolved URL.
- The `<label>` is the element's display name/id; the URL is used verbatim (already encoded by
  [[REQ-TRS-LINK-001]]).
- Inert when `[links]` is not configured (output unchanged).

**Source:** clickable element references in generated Markdown (validation report, `export`).
Consumes [[REQ-TRS-LINK-001]].

**Acceptance criteria:**

- With `[links]` configured, an element named in the validation report is rendered as
  `[<name>](<hosted url>)`.
- An element with no resolved URL is rendered as before (plain text / local path).
- With no `[links]` table, the report/export contain no hosted links.
