---
id: REQ-TRS-LINKTYPE-010
type: Requirement
name: "The MCP server shall expose read-only link_types and follow tools"
status: draft
reqDomain: software
verificationMethod: test
---

The MCP server **shall** expose a read-only `link_types` tool returning the same data as
`link-types --json`, and a read-only `follow` tool taking `element`, `link`, optional
`reverse`, `transitive` and `depth`, returning the same data as `follow --format json`. The
generic element create/update tools **shall** accept a `links:` field.

**Acceptance criteria:** both tools are listed by `tools/list` and return the expected data for
a fixture model.

**Source:** `REQ-TRS-LINKTYPE-010` (product model), `ADR-SYS-LINKTYPE-001`.
