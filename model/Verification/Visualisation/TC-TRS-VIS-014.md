---
id: TC-TRS-VIS-014
type: TestCase
testLevel: L3
status: active
name: "Verify the visualisation stack carries tests at every layer: parser, generators, writers, embedded layout, server routes and client, with committed golden snapshots and an npm test script that runs every Node test."
verifies:
  - REQ-TRS-VIS-014
sourceFile: repo:qual/tests/tc/TC-TRS-VIS-014.sh
tags:
  - visualisation
  - testing
---

Structural check run by the qualification runner (`bash qual/tests/run_qual.sh TC-TRS-VIS-014`)
over the repository: the test files the requirement names exist and contain tests, the golden
snapshot directories are populated, and `package.json`'s `test` script runs every Node test. The
tests themselves run under `cargo test --workspace` and `npm test` in CI (the workspace test step
of the qualification workflow).

```gherkin
Feature: tests at every visualisation layer (TC-TRS-VIS-014)

  Scenario: every layer has a non-empty test file
    Given the repository
    Then the manifest parser, generator, writer, layout, vendor, server-route and client test files exist and contain tests

  Scenario: golden snapshots exist
    Then the derived-IR, writer and ELK snapshot directories are populated and the Node-produced ELK output is committed

  Scenario: npm test runs every Node test
    Then the frontend test script names the layout, connect-rules and server-sizes tests
```
