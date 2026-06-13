---
id: REQ-TRS-SEC-008
type: Requirement
name: "TestCase shall support securityTestMethod field for ISO/SAE 21434 §13 test methods"
status: draft
reqDomain: software
verificationMethod: test
---

The `TestCase` element **shall** support an optional field **`securityTestMethod`** (YAML key, string scalar) that designates the security-specific test method used, per ISO/SAE 21434 §13.3.

Valid values: `fuzz`, `penetration_test`, `security_regression`, `vulnerability_scan`, `threat_modeling`.

An invalid value **shall** trigger warning **`W809`** ("TestCase.securityTestMethod '{}' is not a recognised security test method").

**Acceptance criteria:**

- `securityTestMethod: fuzz` on a TestCase validates without warning.
- `securityTestMethod: penetration_test` validates without warning.
- `securityTestMethod: unknown_method` triggers W809.
- Absent `securityTestMethod` never triggers W809 (the field is optional).
- `list TestCase --json` includes `securityTestMethod` (string or null) on each item.
