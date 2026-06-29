---
type: Requirement
id: REQ-TRS-HTML-005
name: "Site includes validation, coverage, and traceability report pages"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-HTML-000]
breakdownAdr: Decisions::HTMLExportADR
tags:
  - html-export
---

The site shall include report pages summarising the model's quality and traceability.

## Reports

- A **validation** report listing the validation findings (code, severity, file, message),
  with each file linked to its element page.
- A **coverage** report of requirement verification (verified, unverified leaves, and parents
  missing an integration test), reusing the shared coverage computation.
- A **traceability** report relating requirements to their derivation and verifying test cases.
