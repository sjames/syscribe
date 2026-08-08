---
type: Requirement
id: REQ-DOMAIN-001
name: "Domain-mismatch satisfy target for TC-TRS-SYSMLV2-008"
status: approved
reqDomain: hardware
---

`satisfy` target for `MismatchedDomainPart` — deliberately `reqDomain: hardware` while the
satisfying SysMLv2 element's lifted `domain:` is `software`, to exercise the existing `E313`
domain-compatibility check against a `@SyscribeDomain`-lifted value.
