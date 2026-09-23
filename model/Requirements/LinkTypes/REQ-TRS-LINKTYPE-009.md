---
type: Requirement
id: REQ-TRS-LINKTYPE-009
name: "Existing relationship and traversal commands shall include user-defined links"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-LINKTYPE-000]
breakdownAdr: Decisions::LinkTypesADR
tags:
  - link-types
---

`links` **shall** list custom outbound links under their type name and inbound links under
their `inverse` (or `<type> (inbound)` when none is declared). `refs` **shall** include inbound
custom links. `impact` **shall** traverse custom links (upstream along the link, downstream
against it) and accept custom type names in `--kinds`. `trace` **shall** list the custom links
touching the requirement. `show` **shall** display the element's `links:`.

**Acceptance criteria:** each command's output names the custom link and its counterpart for a
fixture element.
