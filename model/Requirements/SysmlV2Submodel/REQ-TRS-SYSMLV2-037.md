---
type: Requirement
id: REQ-TRS-SYSMLV2-037
name: "A syscribe export-sysml command renders the model, or one package subtree, as SysML v2 textual notation to stdout, a file or a directory"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-000]
breakdownAdr: Decisions::SysmlV2ExportADR
tags:
  - sysmlv2
  - cli
  - export
---

`syscribe -m <root> export-sysml [<package-qname>] [--out <file|dir>]` shall render the loaded
model as SysML v2 textual notation without modifying the model or any `.sysml` input. With no
`<package-qname>` the whole model is exported; with one, only that element's subtree, emitted at the
top level under its own name. Without `--out` the text goes to stdout; `--out <file>` writes one
file; `--out <dir>` (an existing directory or a path ending in `/`) writes one `<TopLevelName>.sysml`
file per top-level element. An unresolvable `<package-qname>` shall be a usage error with a non-zero
exit status. A one-line export summary (exported and skipped counts) goes to stderr so stdout stays
pure SysML.

The data gathering and rendering shall live in `syscribe-model` (`sysmlv2::export`) so the MCP tool
(`REQ-TRS-SYSMLV2-042`) reuses it.
