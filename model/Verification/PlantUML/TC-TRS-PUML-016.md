---
id: TC-TRS-PUML-016
name: "style_file config emits !include with absolute path"
type: TestCase
status: draft
testLevel: L2
verifies: [REQ-TRS-PUML-042]
tags: [diagram, plantuml, config]
---

```gherkin
Feature: [plantuml] style_file config key

  Scenario: !include with absolute path is emitted when style_file is configured
    Given a .syscribe.toml containing:
      """
      [plantuml]
      style_file = "style/syscribe.puml"
      """
    And the file style/syscribe.puml exists at <model-root>/style/syscribe.puml
    And a Diagram element with diagramKind: IBD
    When syscribe -m <root> plantuml <qname> --output - is run
    Then the first non-blank line of stdout is "!include <absolute-path-to-style-file>"
    And stdout does not contain "skinparam"

  Scenario: W415 is emitted when the style_file does not exist
    Given a .syscribe.toml containing:
      """
      [plantuml]
      style_file = "style/missing.puml"
      """
    And no file exists at <model-root>/style/missing.puml
    When syscribe -m <root> validate is run
    Then the output contains W415
    And the message mentions the missing path
```
