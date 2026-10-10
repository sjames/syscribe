---
type: Requirement
id: REQ-TRS-REQIFIMP-001
name: "import-reqif creates Requirement elements from a ReqIF document and keeps the OEM identifier"
status: draft
reqDomain: software
reqClass: system
tags:
  - cli
  - interchange
---

`syscribe import-reqif <file>` shall create native `Requirement` elements from the SPEC-OBJECTs of a ReqIF document, so customer requirements need not be created by script (GH #241, v1: ReqIF only; CSV/Excel mapping and suspect flagging follow).

## Behavior

- `import-reqif <file.reqif> [--into <package-qname>] [--id-prefix <PFX>] [--class <reqClass>] [--domain <reqDomain>] [--update] [--dry-run]`.
- Each SPEC-OBJECT whose type is not `Package`/`TestCase` becomes a `Requirement` (`status: draft`, default `reqClass: stakeholder`, `reqDomain: system`) in `--into` (default `Requirements`), with the name from the `NAME`/`ReqIF.Name`/`ReqIF.ChapterName`/`Title` attribute (else the object's `LONG-NAME`) and the body from the `DESC`/`ReqIF.Text`/`Text`/`Description` attribute reduced to plain paragraphs.
- Ids are `<PFX>-NNN` (default prefix `REQ-IMP`), allocated after any existing id with that prefix. The OEM identifier (`ReqIF.ForeignID`, `ID` or `SYSCRIBE_ID` attribute, else the object IDENTIFIER) is kept in `extRef: ["reqif:<id>"]`.
- Re-importing is idempotent: an object whose `reqif:<id>` already exists is reported as existing and left alone; with `--update` its name and body are rewritten when they differ (every other field kept).
- `--dry-run` lists what would be created or updated and writes nothing. A document with no SPEC-OBJECTs, or malformed XML, exits 1 without writing.
