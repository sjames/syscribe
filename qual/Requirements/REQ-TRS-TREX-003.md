---
id: REQ-TRS-TREX-003
type: Requirement
name: The export accepts a sort order — directory (default), ascending or descending by qualified name — applied to every list
status: verified
reqDomain: software
verificationMethod: test
---

`--sort directory|asc|desc` **shall** order the top-level `requirements` list and every
nested reference list: `directory` (the default) is the walker's file order (the order
`export` and `ls` use); `asc` and `desc` are lexicographic by full qualified name. The chosen
order **shall** be recorded in the document's `sort` field. An unknown value **shall** be a
usage error naming the three valid values, exit 1. For a given model, option set and sort the
output **shall** be byte-identical across runs.

**Source:** `REQ-TRS-TREX-003` (product model).

**Acceptance criteria:** on the `REQ-TRS-TREX-001` model, whose parent requirement sits one
directory deeper than its children (so walker order and qualified-name order differ): (a) the
default and `--sort directory` list the requirements in the exact order `export` emits them,
and nested lists follow it; (b) `--sort asc` puts `Requirements::Core::REQ-TX-000` first and
`--sort desc` reverses the requirement list, the parent's `derivedChildren` and a child's
`verifiedBy`, with the same entries and summary; (c) `--sort random` exits 1 naming
`directory, asc, desc` with nothing on stdout; (d) two runs with the same options produce
identical bytes.
