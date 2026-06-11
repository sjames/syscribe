---
id: REQ-TRS-CLI-005
type: Requirement
title: Tool shall provide detailed man-page help for every command via help <cmd> and <cmd> --help
status: draft
reqDomain: software
verificationMethod: test
---

A one-line listing is insufficient for a large command surface. The tool **shall** provide **detailed, example-rich help for every dispatchable command**, reachable two ways:

- `syscribe help <command>` — prints the command's man page;
- `syscribe <command> --help` (or `-h`) — prints the same page.

Each man page **shall** follow a consistent structure: a `SYNOPSIS`, a description, the options/arguments, worked `EXAMPLES`, exit codes (for commands that gate), and a `SEE ALSO` list. The pages **shall** be embedded in the binary (no model directory required to read them), mirroring the `syscribe spec` mechanism.

- `syscribe help` (no argument) **shall** print an **index** of every command with a one-line summary.
- `syscribe help <unknown>` **shall** print the index to stderr and exit non-zero.
- Every command accepted by the dispatcher **shall** have a help page (no command without one).

**Source:** developer-ergonomics — the CLI grew to ~40 commands and needed per-command detail. Reuses the embedded-markdown pattern of `syscribe spec` (`crates/syscribe/src/spec.rs`).

**Acceptance criteria:** for every dispatchable command, `syscribe help <command>` and `syscribe <command> --help` each print a non-empty man page (containing `SYNOPSIS`) and exit 0, with no model directory present; `syscribe help` prints an index naming multiple commands; `syscribe help <unknown>` exits non-zero.
