---
id: REQ-TRS-OUT-020
type: Requirement
name: Tool shall export Requirements as a ReqIF 1.2 document (export-reqif command)
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** provide a read-only, export-only **`export-reqif`** command (§21, GH #73)
that maps native `Requirement` elements (and their containing packages) to a ReqIF 1.2 XML
document for interchange with DOORS Next / Jama / Polarion / PTC.

**`syscribe export-reqif [--output <file>] [--scope <qname>] [--config <CONF>] [--include-tests] [--zip]`**

- Each `Requirement` **shall** be emitted as a **`SPEC-OBJECT`** of type `REQ_TYPE` with
  attributes `SYSCRIBE_ID` (`LONG-NAME`), `NAME`, `QUALIFIED_NAME`, a `STATUS` enumeration,
  `SIL_LEVEL` (integer), `ASIL_LEVEL`/`PL_LEVEL`/`VERIFICATION_METHOD` (strings), a `DOMAIN`
  enumeration, and an XHTML `DESC` converted best-effort from the Markdown body (Gherkin
  blocks omitted).
- The containing **packages** (directory tree) **shall** be preserved as a nested
  `SPEC-HIERARCHY` within a `SPECIFICATION` (package folders as folder `SPEC-OBJECT`s).
- Each `derivedFrom:` entry **shall** produce a `SPEC-RELATION` of type `DERIVED_FROM`
  (`SOURCE` = child, `TARGET` = parent — OSLC direction). With `--include-tests`, TestCases
  are emitted as `TEST_CASE` `SPEC-OBJECT`s with `VERIFIED_BY` relations.
- The document **shall** be well-formed ReqIF 1.2 XML. `--output` writes `<file>.reqif`
  (or `<file>.reqifz` with `--zip`); `--scope`/`--config` restrict/project the export.

**Source:** §21 (ReqIF Export), GH #73. Export-only; no new element types or rules.

**Acceptance criteria:**

- The output is well-formed ReqIF 1.2 XML (`xmllint --noout`); every in-scope `Requirement`
  appears as a `SPEC-OBJECT`.
- The package hierarchy is preserved as nested `SPEC-HIERARCHY` entries.
- `derivedFrom:` links appear as `SPEC-RELATION`s of type `DERIVED_FROM`.
- `--include-tests` adds `TEST_CASE` objects and `VERIFIED_BY` relations; `--zip` writes a
  readable `.reqifz`.
