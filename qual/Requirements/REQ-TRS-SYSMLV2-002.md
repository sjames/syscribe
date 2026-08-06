---
id: REQ-TRS-SYSMLV2-002
type: Requirement
name: Tool shall natively parse .sysml/.kerml content and merge it into the graph as qname-mapped elements
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** parse every `.sysml`/`.kerml` file inside a `sysmlSubmodel: true` subtree
in-process (no external process, no sandbox), merge SysML v2 packages declared across multiple
files in the subtree into **one** namespace before qualified-name assignment, and inject the
result into the element graph as ordinary, origin-agnostic elements. Each synthesized element's
qualified name **shall** be `<owning Syscribe package qname>::<SysML v2 fully-qualified name>`,
resolvable by every cross-reference kind exactly like a hand-authored element.

A `.sysml`/`.kerml` parse failure **shall** downgrade only that one file's contribution (fewer or
no elements from it, plus a warning) — never abort the rest of the model's validation.

**Source:** `REQ-TRS-SYSMLV2-002` (product model).

**Acceptance criteria:** two files each declaring a piece of the same SysML v2 package produce
one merged namespace, not two colliding/duplicated ones; a nested SysML v2 package produces a
`::`-qualified name matching its full nesting depth; a file with a syntax error produces a
warning naming that file and contributes zero elements, while every other file in the same
subtree — and the rest of the model — still validates normally.
