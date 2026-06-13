---
id: TC-TRS-SEC-004
type: TestCase
name: "AOU.appliesTo accepts CybersecurityGoal and enforces E859 for wrong types"
status: active
testLevel: L1
verifies: REQ-TRS-SEC-004
---

Verify that `AssumptionOfUse.appliesTo` accepts `CybersecurityGoal` targets without error, and that referencing a non-allowed element type triggers E859.
