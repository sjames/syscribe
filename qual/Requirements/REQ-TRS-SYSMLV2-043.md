---
id: REQ-TRS-SYSMLV2-043
type: Requirement
name: "Tool shall lift a SysMLv2 alias declared in a named package onto that Package element as an aliases: entry"
status: verified
reqDomain: software
verificationMethod: test
---

An `alias <name> for <target>;` member declared in the body of a named `package` (or `library
package`/`namespace`, `REQ-TRS-SYSMLV2-044`) shall become an `{name, for}` entry (plus `shortName`
when the alias declares one) in the `aliases:` list of the synthesized `Package` element for that
package, the same field and shape a hand-authored package uses (spec 3.7.2), so the existing
scoped resolver already honours it. An alias at the root of the submodel directory has no
synthesized package to carry it and stays counted by `W543`.

**Source:** `REQ-TRS-SYSMLV2-043` (product model), `ADR-SYS-SYSMLV2-001`.
