---
id: REQ-TRS-CLI-003
type: Requirement
name: Tool shall print the LLM generation prompt with --agent-instructions
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** print the LLM model generation prompt to `stdout` and exit with code `0` when invoked with the `--agent-instructions` flag, without loading or validating any model directory.

**Source:** CLAUDE.md §Common Commands

**Acceptance criteria:** `syscribe --agent-instructions` prints a non-empty prompt to stdout and exits `0`. It does not require `-m` to be specified.
