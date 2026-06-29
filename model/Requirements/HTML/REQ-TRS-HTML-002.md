---
type: Requirement
id: REQ-TRS-HTML-002
name: "Each element has a page rendering its frontmatter, documentation, and metadata"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-HTML-000]
breakdownAdr: Decisions::HTMLExportADR
tags:
  - html-export
---

Each model element shall have its own HTML page presenting its identity and content.

## Page content

- The element's name, type, qualified name, stable id (if any), and status.
- A table of its declared frontmatter fields.
- Its documentation body rendered from Markdown to HTML.
- Its custom fields.
- A link to the element's hosted source when a source URL is configured (`[links]`).
