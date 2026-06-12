---
id: REQ-TRS-LINK-003
type: Requirement
title: Tool shall emit Mermaid click directives linking nodes to hosted URLs
status: draft
reqDomain: software
verificationMethod: test
---

When a `[links]` source URL is configured ([[REQ-TRS-LINK-001]]), generated **Mermaid** diagrams
**shall** make each file-backed node clickable by appending a Mermaid `click` directive that
opens the node's resolved URL in a new tab.

### Behaviour

- For each node whose element resolves to a URL, append a directive of the form
  `click <nodeId> "<url>" _blank` after the node/edge definitions.
- A node whose element resolves to **no** URL emits **no** `click` directive.
- The `<url>` is escaped for the Mermaid string context (double quotes).
- Inert when `[links]` is not configured (the Mermaid source is unchanged).

**Source:** clickable element links in Mermaid diagrams (e.g. `RequirementTraceMermaid`).
Consumes [[REQ-TRS-LINK-001]].

**Acceptance criteria:**

- With `[links]` configured, a generated Mermaid diagram contains a `click <node> "<url>" _blank`
  line for each node whose element resolves to a URL.
- A node with no resolved URL has no `click` line.
- With no `[links]` table, the Mermaid output contains no `click` directives.
