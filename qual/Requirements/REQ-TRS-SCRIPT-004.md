---
id: REQ-TRS-SCRIPT-004
type: Requirement
title: Tool shall let a script register two shapes — a top-level command and a validation check
status: draft
reqDomain: software
verificationMethod: test
---

An extension script **shall** declare what it offers by **registering** at load time. There
are **two shapes**, both surfaced under the `scripts` command family:

### Registration API

- **`register_command(name, description, fn)`** — registers a **top-level command**
  ([[REQ-TRS-SCRIPT-005]]). `fn(model)` is a Rhai function pointer the host invokes; its
  returned string (if any) is the command's output.
- **`register_check(name, description, fn)`** — registers a **validation hook**
  ([[REQ-TRS-SCRIPT-006]]). `fn(model)` emits findings via `finding(...)`
  ([[REQ-TRS-SCRIPT-003]]).

### Rules

- `name` is a stable **slug** handle (used on the command line and as the finding namespace).
  A **duplicate** command/check `name` across the loaded scripts **shall** be reported as an
  error (deterministic load failure), not silently shadowed.
- A single file **may** register **multiple** commands and/or checks. A file that registers
  **none** is a pure **library** (importable only, [[REQ-TRS-SCRIPT-001]]).
- Registration runs in the same sandbox as execution ([[REQ-TRS-SCRIPT-002]]); a registration
  that errors is reported with the script name and does not abort the other scripts.

**Source:** user decisions — a script can register a top-level CLI command **and** can hook
into validation; two shapes.

**Acceptance criteria:**

- A script calling `register_command("coverage", "…", coverage)` makes `coverage` runnable
  via `scripts run coverage`; one calling `register_check("naming", "…", naming)` makes
  `naming` run under `scripts validate`.
- A file registering both a command and a check exposes both.
- A `.rhai` file that registers nothing is importable as a module but is not itself runnable.
- Two scripts registering the same command name produce a clear duplicate-name error.
