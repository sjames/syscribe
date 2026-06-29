---
type: Requirement
id: REQ-TRS-HTML-007
name: "Styling is overridable via --css and output is fully offline"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-HTML-000]
breakdownAdr: Decisions::HTMLExportADR
tags:
  - html-export
---

The export's styling shall be user-overridable, and the generated site shall be fully offline.

## Behaviour

- Every page shall link a single `style.css`. When `--css <file>` is given, that file's contents
  shall be written as the site's `style.css`, fully replacing the bundled default stylesheet;
  without it, a bundled default stylesheet shall be written.
- The generated output shall contain **no** network (`http://`/`https://`) references — all
  scripts, styles, and assets are written into the output directory — so the site renders with no
  internet connection.
