---
type: TestCase
id: TC-TRS-TMPL-001
name: "template Requirement honours [ids.prefixes] and --prefix"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/template_prefix.rs
verifies:
  - REQ-TRS-TMPL-001
tags:
  - cli
---

```gherkin
Feature: template Requirement id prefix

  Scenario: default prefix comes from the model config
    Given a model whose .syscribe.toml sets [ids.prefixes] Requirement = ["STK", "SYS"]
    When template Requirement is run
    Then the id line starts with STK-

  Scenario: explicit prefix
    When template Requirement --prefix SYS is run
    Then the id line starts with SYS-

  Scenario: unknown prefix is rejected
    When template Requirement --prefix NOPE is run
    Then the command fails and lists REQ, STK and SYS

  Scenario: no configuration keeps the built-in prefix
    Given a model without [ids.prefixes]
    Then the id line starts with REQ-
```

Review follow-ups also covered by the same test file: malformed configured prefixes are never offered, `--prefix` without a value or a misspelled flag is an error, `--prefix` may precede the type, and `--prefix` on a non-Requirement type is rejected.
