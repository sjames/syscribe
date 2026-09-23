---
id: REQ-TRS-LINKTYPE-012
type: Requirement
name: "An LLM authoring agent shall be able to discover a project's link types from the tool itself"
status: draft
reqDomain: software
verificationMethod: test
---

Because link types are project-specific, the tool itself **shall** make them discoverable to an
LLM agent:

- The general authoring prompt (`--agent-instructions`) **shall** document `links:`, instruct
  the agent to discover the project's link types with `link-types` (or the MCP `link_types`
  tool) before authoring links, and forbid inventing undeclared types.
- `syscribe -m <root> --agent-instructions` **shall** append a "Project link types" section
  listing each declared type (name, description, direction/inverse, source → target types,
  cardinality, extends/relax) when the model declares any.
- The MCP server's `initialize` instructions **shall** point the client at the `link_types` tool.
- `E630` **shall** name the declared link types (see `REQ-TRS-LINKTYPE-002`).

**Acceptance criteria:** the plain prompt mentions `links:` and `link-types`; with a model that
declares types the prompt ends with a section naming them; with a model that declares none, no
such section is appended.

**Source:** `REQ-TRS-LINKTYPE-012` (product model), `ADR-SYS-LINKTYPE-001`.
