---
id: REQ-TRS-PKG-002
type: Requirement
name: "lint-docs shall flag an _index.md body that hand-enumerates its package's members"
status: draft
reqDomain: software
verificationMethod: test
---

When `lint-docs` scans an `_index.md` file that belongs to a package of the loaded model, it
**shall** report `W103` if the body (outside the frontmatter) mentions three or more distinct
stable ids that resolve to that package's own direct members. The finding **shall** name the count
and point the author to the generated listing (`show <package>`). `W103` is advisory: it **shall
not** by itself make `lint-docs` exit non-zero. Mentions of fewer than three direct members, of
ids outside the package, or in files that are not a package's `_index.md` **shall not** raise it.

**Acceptance criteria:** an `_index.md` enumerating three members raises `W103` and `lint-docs`
still exits zero; one referencing two members, or ids from another package, raises nothing.

**Source:** `REQ-TRS-PKG-002` (product model), `ADR-SYS-PKG-001`, GH #120.
