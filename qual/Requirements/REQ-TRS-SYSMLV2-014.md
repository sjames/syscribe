---
id: REQ-TRS-SYSMLV2-014
type: Requirement
name: A doc-comment directive syntax lifts the fixed @Syscribe* field set onto interface def/port def/connection def
status: draft
reqDomain: software
verificationMethod: test
---

An `interface def`, `port def`, or `connection def` **shall** lift `shortName:`/`implementedBy:`/
`domain:`/`asilLevel:`/`silLevel:`/`plLevel:` from a structured `@Syscribe*: <value>` directive
line inside its `doc /* ... */` comment, since these three body grammars carry no
`MetadataAnnotation` slot for the real `@Name { field = value; }` form. A recognized directive
line **shall** be stripped from the element's lifted `doc:` text; an unrecognized `@...:` line is
left untouched. `implementedBy:` lifted this way **shall** drive `W023` exactly like the real
annotation form does on a `part def`/`part`.

**Source:** `REQ-TRS-SYSMLV2-014` (product model), `ADR-SYS-SYSMLV2-001` addendum.
