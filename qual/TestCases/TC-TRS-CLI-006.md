---
id: TC-TRS-CLI-006
type: TestCase
testLevel: L2
status: draft
title: "Verify --agent-instructions topic: magicgrid prints the MagicGrid prompt; no topic prints the general prompt; an unknown topic exits non-zero; works with no model directory."
verifies:
  - REQ-TRS-CLI-006
---

```gherkin
Feature: --agent-instructions accepts an optional topic
  Scenario: magicgrid topic prints the MagicGrid modeling prompt
    When syscribe --agent-instructions magicgrid is run
    Then it exits zero and prints the MagicGrid prompt naming mg_cell and magicgrid --audit and trade-study

  Scenario: no topic prints the general modeling prompt
    When syscribe --agent-instructions is run with no topic
    Then it exits zero and does not print the MagicGrid prompt heading

  Scenario: an unknown topic exits non-zero naming the topics
    When syscribe --agent-instructions with an unknown topic is run
    Then it exits non-zero and the message names magicgrid

  Scenario: it works without a model directory
    When syscribe --agent-instructions magicgrid is run with no -m and no model present
    Then it still prints the MagicGrid prompt
```
