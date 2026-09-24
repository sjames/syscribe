---
type: Requirement
id: REQ-TRS-LINKTYPE-010
name: "The MCP server shall expose read-only link_types and follow tools"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-LINKTYPE-000]
breakdownAdr: Decisions::LinkTypesADR
tags:
  - link-types
---

The MCP server **shall** expose a read-only `link_types` tool returning the same data as
`link-types --json`, and a read-only `follow` tool taking `element`, `link`, optional
`reverse`, `transitive` and `depth`, returning the same data as `follow --format json`. The
generic element create/update tools **shall** accept a `links:` field.

**Acceptance criteria:** both tools are listed by `tools/list` and return the expected data for
a fixture model.
