---
id: REQ-TRS-PUML-054
name: "plantuml command injects img tag into markdown body when absent"
type: Requirement
status: draft
reqClass: system
reqDomain: software
derivedFrom: REQ-TRS-PUML-010
breakdownAdr: Decisions::PlantUMLADR
tags: [diagram, plantuml]
---

When `syscribe plantuml` generates or regenerates a companion `.puml` file for a
Diagram element, it also inspects the element's markdown body. If the body contains
neither a Markdown image link (`![`) nor an HTML `<img` tag, the command appends
the following line to the end of the markdown file:

```markdown
![[diagram name]](./<stem>.svg)
```

The path is relative to the `.md` file's directory, matching the location where
`plantuml render` deposits the SVG output. This ensures the rendered diagram is
visible when the `.md` file is viewed on GitHub or any Markdown renderer without
requiring raw HTML.

If the body already contains a `![` or `<img` reference, the file is not modified.

Validation warning **W413** is updated to fire when the body contains neither form.
