---
id: REQ-TRS-IMPL-002
type: Requirement
name: Tool shall surface implementedBy in links, refs, spec fields, and the docs
status: draft
reqDomain: software
verificationMethod: test
---

So that the architecture-to-code linkage introduced by [[REQ-TRS-IMPL-001]] is usable for navigation and review, the `implementedBy` relationship **shall** be discoverable in both directions and documented.

The tool **shall**:

- surface `implementedBy` as an **outbound** relationship in `links <element>` — running `links` on a `Part`/`PartDef` that declares `implementedBy` **shall** list each linked code path;
- surface the relationship in the **reverse** direction via `refs <module-or-element>` — querying a module path **shall** report the architecture element(s) that declare it under `implementedBy`, so the owner of a module is discoverable from the module itself;
- list `implementedBy` in `syscribe spec fields` as a recognised frontmatter field on `Part`/`PartDef`; and
- document the field in the docs alongside the other source-location fields.

**Source:** GH issue #13 (architecture↔code drift); discoverability companion to [[REQ-TRS-IMPL-001]].

**Acceptance criteria:** `links` on a `PartDef` that declares `implementedBy` shows the linked path(s); `refs` on a module path reports the architecture element that owns it; `spec fields` lists `implementedBy`.
