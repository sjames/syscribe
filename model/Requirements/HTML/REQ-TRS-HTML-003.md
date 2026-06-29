---
type: Requirement
id: REQ-TRS-HTML-003
name: "Site provides namespace navigation and resolved cross-reference links"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-HTML-000]
breakdownAdr: Decisions::HTMLExportADR
tags:
  - html-export
---

The site shall be navigable by namespace and shall turn model cross-references into working
hyperlinks between pages.

## Behaviour

- Every page shall present a navigation tree of the model's namespace hierarchy, each entry
  linking to its element page.
- Reference fields (e.g. `supertype`, `subsets`, `typedBy`, `verifies`, `derivedFrom`,
  `satisfies`, allocations) and computed reverse indices (e.g. `verifiedBy`, `derivedChildren`)
  shall be rendered as hyperlinks to the referenced element pages.
- A reference that does not resolve shall be shown as plain text (not a broken link).
