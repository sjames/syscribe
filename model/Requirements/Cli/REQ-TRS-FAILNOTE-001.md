---
type: Requirement
id: REQ-TRS-FAILNOTE-001
name: "trace and safety-case show why a verifying test failed"
status: draft
reqDomain: software
reqClass: system
tags:
  - cli
  - evidence
---

`trace` and `safety-case` shall show the retained failure evidence of a failing test (GH #258, retained messages).

## Behavior

- `trace <req>`: below the "Verified by" table, a "Failing tests" list gives, per failing verifying TestCase, each failing `testFunctions` function with the retained JUnit `message` and `time` (`<function> — <message> (<time>s)`). Nothing is printed when no verifier fails or no message was retained.
- `safety-case`: after the completeness block, a "Failure details" list gives the same per failing TestCase in the tree; `--format json` adds `failureDetails[{testCase, function, message, time}]` (empty array when none).
- A retained run selected with `--results-as-of` keeps no messages, so the lists are empty there.
