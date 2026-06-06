---
id: REQ-TRS-MOVE-002
type: Requirement
name: Move Updates All Qualified-Name References
title: Tool shall update every qualified-name reference to a moved element
status: draft
reqDomain: software
verificationMethod: test
---

When an element or package is moved (REQ-TRS-MOVE-001), the tool **shall** update every qualified-name reference to the moved element across all other elements in the model, so that no reference is left dangling. Specifically:

- a reference whose value equals the old qualified name `source` **shall** be rewritten to `dest`;
- a reference to a descendant or sub-feature (value beginning with `source::`) **shall** be rewritten to the corresponding `dest::` value (this covers package moves and nested endpoints such as connection `from:`/`to:` and feature `typedBy:`);
- references **shall** be updated wherever they appear in frontmatter, including nested structures (`connections`, `features`, `ports`, `ends`, etc.), not only top-level fields;
- multi-segment qualified-name references in the Markdown body (e.g. a backtick-wrapped `` `source` `` cited in an ADR or requirement) **shall** also be updated;
- qualified-name references inside SVG diagrams — both inline (embedded in a `.md` body) and companion (separate `.svg` files), e.g. `sysml:ref=`, `data-qname=`, and `href=` attributes — **shall** also be updated;
- the rewrite **shall not** alter unrelated values, free-text frontmatter fields (`name`, `title`, `shortName`), ordinary prose words, or qualified names that merely share a prefix segment but are not the moved element (e.g. `source` must not match `sourceOther`).

**Source:** Feature request — "all other elements that reference this must also be updated".

**Acceptance criteria:** after a move, validating the model resolves all references (no E102/E103-class unresolved-reference findings introduced by the move); references via the old qualified name now resolve to `dest`; a similarly-named sibling sharing a prefix is left untouched.
