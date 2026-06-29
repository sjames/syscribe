---
type: Requirement
id: REQ-TRS-HTML-001
name: "export-html writes a multi-file static site to an output directory"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-HTML-000]
breakdownAdr: Decisions::HTMLExportADR
tags:
  - html-export
---

`syscribe -m <root> export-html [--out <dir>]` shall generate a multi-file static HTML site for
the whole model and write it to the output directory (default `html/`), creating the directory
if needed.

## Behaviour

- The site shall include a root `index.html` overview page and one page per model element.
- The command shall report the output location and the number of pages written, and exit `0` on
  success.
- Re-running the command shall regenerate the site (overwriting prior output).
