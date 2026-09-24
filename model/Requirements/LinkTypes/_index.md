---
type: Package
name: LinkTypes
---

Requirements for user-defined link types (`[linkTypes]` in `.syscribe.toml`, `links:` in
frontmatter). All derive from `REQ-TRS-LINKTYPE-000` and are governed by `ADR-SYS-LINKTYPE-001`
(`Decisions::LinkTypesADR`). The scope covers declaring link types and their constraints,
authoring and validating `links:`, extending a built-in trace link with per-type rule
relaxation, the `follow` and `link-types` commands, integration with the existing relationship
commands, MCP and suspect links, and making the project's link vocabulary discoverable by an
LLM authoring agent.

The member requirements are listed by `syscribe show Requirements::LinkTypes` (generated from
this directory — not maintained here).
