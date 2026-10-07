---
id: REQ-TRS-SYSMLV2-036
type: Requirement
name: Tool shall lift a package-level SysMLv2 doc comment onto the synthesized Package element's doc text
status: verified
reqDomain: software
verificationMethod: test
---

A `doc /* ... */` declared directly in a `package` body shall become the doc text of the
synthesized `Package` element for that package (several blocks, or blocks from several files that
merge into one package, are joined by a blank line; each block's own delimiter padding is trimmed
as for every other doc lift, `REQ-TRS-SYSMLV2-009`). A nested package gets only its own docs.
Package-level `doc` no longer counts toward `W543`.

`doc` on a `requirement def`/`requirement` body remains a separate follow-up.
