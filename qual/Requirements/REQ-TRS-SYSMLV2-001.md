---
id: REQ-TRS-SYSMLV2-001
type: Requirement
name: Tool shall scope a sysmlSubmodel package's subtree out of native Markdown parsing
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** accept an optional `sysmlSubmodel: true` field on a package `_index.md`. When
present, every file anywhere in that directory's subtree — however nested — is excluded from
native `.md`/YAML-frontmatter parsing, **except**:

1. the package's own `_index.md`, which remains a normal native element (name, doc body,
   containment tree entry);
2. hand-authored `.md` element files coexisting alongside `.sysml`/`.kerml` content in the same
   directory, which are parsed normally and contribute to the same package's namespace.

A stray `_index.md` found anywhere inside the marked subtree **shall** be excluded and reported
as a warning ("ignored — inside a sysmlSubmodel subtree"), not processed as a package — nested
subdirectories inside a `sysmlSubmodel` subtree carry no namespace meaning of their own.

A model declaring `sysmlSubmodel: true` on zero packages **shall** validate identically to a
model built before this feature existed (no new errors or warnings, no elements added or
removed).

**Source:** `ADR-SYS-SYSMLV2-001`, `REQ-TRS-SYSMLV2-001` (product model).

**Acceptance criteria:** a `sysmlSubmodel: true` package's own `_index.md` still validates as a
normal Package element; a nested `_index.md` inside that subtree is excluded and produces exactly
one warning naming it; a hand-authored `.md` sibling in the same directory still parses and
resolves under the package's qualified name; a model with no `sysmlSubmodel: true` package
anywhere produces byte-identical `validate` output to the same model with the field never
introduced.
