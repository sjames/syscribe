---
id: TC-AMP-001
type: TestCase
name: "Lockstep monitor reports a divergence"
status: approved
testLevel: L3
verifies: [REQ-AMP-001]
---
Exercises the AMP lockstep monitor. It verifies REQ-AMP-001, which is **inactive**
in the single-core configuration. Under `audit --config CONF-SINGLE-001` this
TestCase must NOT be counted as a dangling/error finding just because its
verified requirement was projected out of the variant (GH #36): the reference
still resolves against the full model.

```gherkin
Feature: AMP lockstep monitoring
Scenario: Lockstep divergence is reported
  Given the two cores are running in lockstep
  When core B diverges from core A
  Then the monitor shall raise a lockstep fault within 1 ms
```
