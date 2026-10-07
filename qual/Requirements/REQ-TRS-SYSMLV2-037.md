---
id: REQ-TRS-SYSMLV2-037
type: Requirement
name: Tool shall provide an export-sysml command rendering the model or a package subtree as SysML v2 text
status: verified
reqDomain: software
verificationMethod: test
---

The tool **shall** provide `syscribe -m <root> export-sysml [<package-qname>] [--out <file|dir>]`, writing SysML v2 textual notation to stdout, a file or one file per top-level element, with the exported/skipped summary on stderr, a non-zero exit for an unknown package, and no change to the model.

**Source:** `REQ-TRS-SYSMLV2-037` (product model), `ADR-SYS-SYSMLV2-002`.
