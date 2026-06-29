---
type: Requirement
id: REQ-TRS-HTML-000
name: "The model can be exported as a standalone offline HTML site"
status: draft
reqDomain: software
reqClass: stakeholder
tags:
  - html-export
---

Syscribe shall be able to export the entire model as a self-contained, offline, browsable HTML
website, so the model can be published, shared, and reviewed without the toolchain or a running
server.

## Rationale

The model is otherwise reachable only through the live web server (which needs a running process)
or as raw Markdown. Teams need a portable artifact — to attach to a review package, publish to a
static host, or read offline — that faithfully renders the model's structure, documentation,
diagrams, and traceability.
