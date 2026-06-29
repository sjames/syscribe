---
type: Requirement
id: REQ-TRS-HTML-006
name: "Site provides offline client-side search over elements"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-HTML-000]
breakdownAdr: Decisions::HTMLExportADR
tags:
  - html-export
---

The site shall provide a client-side search over the model's elements that works offline.

## Behaviour

- The export shall emit a search index of the elements (qualified name, id, type, name, page
  URL) and a bundled script that filters it in the browser.
- Selecting a search result shall navigate to that element's page.
- Search shall require no network access.
