---
id: REQ-TST-001
type: Requirement
name: Verified requirement with empty verifiedBy
status: verified
reqDomain: system
verificationMethod: test
---

This element **shall** satisfy the test condition. Requirement has `status: verified` but no TestCase references it — `verifiedBy` index will be empty, should produce W003.
