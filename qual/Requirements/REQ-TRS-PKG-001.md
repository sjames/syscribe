---
id: REQ-TRS-PKG-001
type: Requirement
name: "show, the web UI detail panel and export-html shall list a package's direct members generated from the directory"
status: draft
reqDomain: software
verificationMethod: test
---

For an element of `type: Package` (or any element that owns child elements in the directory
tree), `show` **shall** print a `## Members (N)` section listing each direct child — its stable id
(or qualified name when it has none), type, name and status — generated from the loaded model, in
a deterministic order. The web UI element detail panel and the `export-html` element page for a
package **shall** show the same member list, each entry linking to that member. A package with no
members **shall** show an explicit empty state. `show --no-related` **shall not** suppress it.

**Acceptance criteria:** `show` on a package lists exactly its direct children (not grandchildren)
with id, type, name and status, including a child the `_index.md` prose does not mention; the
export-html package page lists the same members with links.

**Source:** `REQ-TRS-PKG-001` (product model), `ADR-SYS-PKG-001`, GH #120.
