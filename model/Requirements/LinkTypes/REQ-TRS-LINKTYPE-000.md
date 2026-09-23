---
type: Requirement
id: REQ-TRS-LINKTYPE-000
name: "Projects can define their own link types, choose which built-in traceability rules apply to them, and traverse them"
status: draft
reqDomain: software
reqClass: stakeholder
tags:
  - link-types
---

Syscribe shall let a project define its own relationship (link) types beyond the built-in
trace links, declare the constraints that govern each one, selectively relax built-in rules for
link types that specialize a built-in link, and follow those links from the command line and
the MCP server — with the vocabulary discoverable by an LLM authoring agent from the tool itself.

## Rationale

Real projects need relationships the format does not name (`mitigates`, `conflictsWith`,
`partiallySatisfies`, …). Forcing them into built-in fields inherits rules that do not fit;
pushing them into `custom_fields` loses resolution, validation and traversal.
