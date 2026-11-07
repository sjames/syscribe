---
type: TestCase
id: TC-TRS-AGENTPOL-004
name: "Agents cannot answer questions reserved to a person"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/agent_policy.rs
verifies:
  - REQ-TRS-AGENTPOL-004
tags:
  - cli
  - agents
---

The MCP scenario lives in `crates/syscribe/tests/mcp_policy.rs` (`no_write_tool_answers_a_question_reserved_to_a_person`).

```gherkin
Feature: Questions reserved to a person

  Scenario: the policy reserves every question
    Given `[agents] human_only_answers = true`
    When an agent runs `set <PI> question.answer Q1 text=…`, with or without --dry-run
    Then it exits 1 naming `human_only_answers` and nothing is written
    And an agent may still add a question, and a person (no --by, or --as-human) may answer

  Scenario: answerBy on the question wins over the default
    Then `answerBy: human` reserves a question without any [agents] table
    And `answerBy: any` opens a question to agents under the policy
    And an unrecognised answerBy is E724 and counts as human

  Scenario: no MCP write tool answers a reserved question
    Then `question`, `update_element` and `apply_changes` are refused with `human_only_answers`, dry run or not, and the file is unchanged

  Scenario: validate flags an agent-written answer
    Then W323 is raised when the commit that wrote the answer is an agent's or answeredBy matches [agents] actors
    And nothing is raised for a person's answer, an uncommitted answer, or a question with answerBy any

  Scenario: questions --json exposes who may answer and who did
    Then each question has answerBy and, when git records it, answerProvenance with by, agent, at and commit
```
