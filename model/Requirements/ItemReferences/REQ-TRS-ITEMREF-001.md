---
type: Requirement
id: REQ-TRS-ITEMREF-001
name: "Requirement body items are numbered by position and may carry a stable anchor"
status: draft
reqDomain: software
reqClass: system
tags:
  - requirements
  - references
---

A Requirement body written as a numbered list shall expose its items as citable units (GH #453).

1. {#item-definition} **Items.** Every top-level ordered-list item of a Requirement body (a line starting at column 0 with `<digits>.` or `<digits>)`, outside fenced code) is an item. Its text runs to the next top-level item, a heading or fence at column 0, or a column-0 line after a blank line.
2. {#numbering} **Numbering.** Items are numbered by position, 1-based, across the whole body; the number the author typed is not used.
3. {#anchor-syntax} **Anchors.** An item may begin with `{#slug}`, slug `^[a-z][a-z0-9-]*$`, giving it a name that survives insertions and reordering.
4. {#display} **Display.** `show`, the web detail panel and `export-html` render the item without the `{#slug}` token and keep its number; `export-html` emits an HTML anchor `id="item-<slug>"`.
5. {#anchor-problems} **Anchor problems.** A slug defined twice in one requirement, or a malformed `{#…}` token, is `W092` on that requirement.
