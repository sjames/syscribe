---
id: REQ-TRS-SYSMLV2-099
type: Requirement
name: "A #T prefix on a usage applies metadata to that usage wherever the usage is declared"
status: verified
reqDomain: software
verificationMethod: test
---

A `#T` prefix on a usage (`#Safety part a : A;`) shall apply a `{type: T}` entry to that usage's `metadata:` list — resolved to the ingested `MetadataDef` innermost-scope-first, the same entry `REQ-TRS-SYSMLV2-086` gives a package-level `#T part def B;` — wherever the usage is declared: at a file's root, in a package body, or inside a definition or usage body, and in both shapes the 0.57 parser keeps the prefix in (an extension keyword on the usage's own occurrence prefix, or a bodiless `#T` member immediately preceding the usage in the enclosing body). A prefix member whose following member synthesizes no element of its own is dropped. The entry is exported as `@T;` in the usage's body, so an export re-ingests to the same entry. `docs/model-guide/sysmlv2-submodel.md` section 6 shall no longer claim that the parser drops a `#T` prefix inside a definition body.

**Source:** `REQ-TRS-SYSMLV2-099` (product model), `ADR-SYS-SYSMLV2-001`.
