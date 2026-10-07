---
id: REQ-TRS-SYSMLV2-051
type: Requirement
name: export-sysml shall emit satisfies on non-part elements as satisfy statements and verifies on requirements as verify statements
status: verified
reqDomain: software
verificationMethod: test
---

An exported element that is not a `Part`/`PartDef` and has `satisfies:` shall export one real
package-level `satisfy <requirement> by <element>;` statement in its enclosing package instead of
the `// satisfies:` comment; a `RequirementDef`/`Requirement` with `verifies:` shall export
`verify <target>;` members in its body, the only context the parser (and so ingestion) accepts.
Anywhere SysML has no `verify` form, the link stays a comment. Re-ingesting the output yields the
same `satisfies:`/`verifies:` links.

**Source:** `REQ-TRS-SYSMLV2-051` (product model), `ADR-SYS-SYSMLV2-002`.
