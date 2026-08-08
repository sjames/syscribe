---
id: REQ-TRS-SYSMLV2-008
type: Requirement
name: A fixed set of @Syscribe* metadata annotations shall lift domain, integrity level, shortName, and implementedBy onto a SysMLv2 part def/part
status: draft
reqDomain: software
verificationMethod: test
---

A SysMLv2 `part def`/`part` (including a `variant part` usage) **shall** be able to declare
`@SyscribeDomain { value = '...'; }`, `@SyscribeIntegrity { asil = '...'; }`/`{ sil = ...; }`/`{ pl
= '...'; }`, `@SyscribeShortName { value = '...'; }`, and `@SyscribeImplementedBy { path = '...';
}` metadata annotations. The tool **shall** lift each into the synthesized element's
`domain:`/`asilLevel:`/`silLevel:`/`plLevel:`/`shortName:`/`implementedBy:` field respectively —
the same fields a hand-authored element uses — so every existing validation rule on those fields
(domain-compatibility, integrity-level format/propagation, the `silLevel`/`asilLevel`
mutual-exclusion warning, the `implementedBy` disk-check) applies unchanged, with **no new
validation code**.

A `part def`/`part` carrying **none** of these annotations **shall** be ingested exactly as it is
today — no regression.

**Source:** `REQ-TRS-SYSMLV2-008` (product model).

**Acceptance criteria:** a `part def`/`part` with `@SyscribeDomain { value = 'software'; }` and
`@SyscribeIntegrity { asil = 'B'; }` shows `domain: software`/`asilLevel: B` on the synthesized
element and validates identically to a hand-authored equivalent; `@SyscribeIntegrity { asil = 'D';
sil = 2; }` on the same annotation raises the existing `W006` mutual-exclusion warning; a `part
def`/`part` with none of these annotations carries no lifted fields and raises no new finding.
