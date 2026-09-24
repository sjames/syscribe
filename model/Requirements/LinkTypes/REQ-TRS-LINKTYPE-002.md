---
type: Requirement
id: REQ-TRS-LINKTYPE-002
name: "Elements shall author user-defined links under a links: map whose keys are declared link types and whose targets resolve like satisfies:"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-LINKTYPE-000]
breakdownAdr: Decisions::LinkTypesADR
tags:
  - link-types
---

Any element **shall** accept an optional `links:` frontmatter field: a mapping from a declared
link-type name to either a single reference or a list of references. References **shall**
resolve by stable id or qualified name exactly as `satisfies:` targets do. The element holding
the entry is the link's source (upstream direction, §12.1).

- A key that is not a declared (valid) link type **shall** raise error `E630`; the message
  **shall** list the declared link types (or state that none are declared and how to declare
  one).
- A `links:` value that is not a mapping, or a key whose value is not a string or a list of
  strings, **shall** raise error `E631`.
- A reference that does not resolve **shall** raise error `E632`.
- `links:` **shall not** raise `W047` (unknown field).

**Acceptance criteria:** a valid link validates clean; an undeclared key raises `E630` naming
the declared types; a malformed shape raises `E631`; a dangling target raises `E632`; a model
that uses neither `[linkTypes]` nor `links:` is unaffected.
