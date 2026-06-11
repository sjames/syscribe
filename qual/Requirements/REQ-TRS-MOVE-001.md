---
id: REQ-TRS-MOVE-001
type: Requirement
title: Tool shall move an element or package to a new qualified name
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** provide a `move <source> <dest>` command that relocates a model element to a new qualified name, where:

- `source` is an existing element identified by its qualified name or its stable id;
- `dest` is the new fully-qualified name (`::`-separated);
- the underlying `.md` file is moved to the path derived from `dest`, creating intermediate package directories as needed;
- a **package** (a namespace directory, with or without an `_index.md`) is moved with its entire subtree, so every descendant's qualified name changes from `source::*` to `dest::*`.

The command **shall** refuse to proceed when: `source` does not resolve; `dest` already exists; `dest` equals `source`; or `dest` is nested inside `source` (a package cannot be moved into its own subtree). A `--dry-run` option **shall** report the planned file move and reference updates without modifying any file.

**Source:** Feature request — move an element/package with reference integrity.

**Acceptance criteria:** `move A::B::C A::D::C` relocates the file to the path for `A::D::C`; moving a package relocates its whole subtree; moving onto an existing target, onto itself, or into its own subtree is rejected with a non-zero exit and no change.
