---
id: REQ-TRS-SET-002
type: Requirement
name: Tool shall provide schema-aware achieves.add and evidence.add list-append operations
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** provide `set <qname|id> achieves.add <req-id>` and
`set <qname|id> evidence.add ref=<id>|path=<path> [rationale=<text>]`, appending one entry to
the target's `achieves:`/`evidence:` list without disturbing the existing entries' order.

`achieves.add`'s target **shall** be validated before anything is written: it must resolve to a
native `Requirement` (mirroring `E714`/`E715`) or the command refuses with no file written.
Adding an id already present **shall** be a no-op (reported, not duplicated).

`evidence.add` **shall** accept exactly one of `ref=<id>` (which must resolve to some model
element, permissive by kind like `blockedBy:`, mirroring `E716`'s bar) or `path=<path>` (which
must exist on disk under the model root, or be an `http(s)://` URI accepted as external,
mirroring `E717`) — passing both, or neither, is refused. An optional `rationale=<text>` is
carried onto the new entry.

A `--dry-run` option **shall** print a unified diff of the would-be change without writing
anything.

**Source:** GitHub issue #112.

**Acceptance criteria:**
- `achieves.add` on a dangling or non-Requirement target is refused; no file written.
- `achieves.add` on a valid Requirement appends it after the existing entries, in order.
- `evidence.add ref=<id>` on a dangling ref is refused; no file written.
- `evidence.add path=<path>` on a nonexistent local path is refused; an existing local path or
  an `http(s)://` URI is accepted.
- `--dry-run` prints the diff and writes nothing for either operation.
