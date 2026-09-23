---
type: Package
name: LinkTypes
---

Requirements for user-defined link types (`[linkTypes]` in `.syscribe.toml`, `links:` in
frontmatter). All derive from `REQ-TRS-LINKTYPE-000` and are governed by
`ADR-SYS-LINKTYPE-001` (`Decisions::LinkTypesADR`): declaration (`-001`), authoring (`-002`),
source/target types (`-003`), cardinality (`-004`), acyclicity (`-005`), `extends`/`relax`
(`-006`), `follow` (`-007`), `link-types` (`-008`), integration with existing commands
(`-009`), MCP (`-010`), suspect links (`-011`) and LLM discoverability (`-012`).
