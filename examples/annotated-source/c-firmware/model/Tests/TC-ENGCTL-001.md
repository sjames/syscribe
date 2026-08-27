---
type: TestCase
id: TC-ENGCTL-001
name: "Target RPM is held after being set"
testLevel: L1
status: draft
verifies:
  - Firmware::EngineController
---

Verifies `Firmware::EngineController` — a `Part` synthesized by scanning the
marker in `../Firmware/engine_ctrl.c`, not a native `Requirement`. Legal only
because it was actually synthesized by an annotation scan
(`ADR-SYS-ANNOTATE-001`'s widening of `E104`'s `verifies:` target-legality
check) — an ordinary hand-authored `Part` outside an `annotationFormat:`
package would still be rejected.

```gherkin
Feature: Engine controller target RPM
  Scenario: target RPM is held
    Given the engine controller is initialized
    When a target RPM is set
    Then reading the target RPM back returns the same value
```
