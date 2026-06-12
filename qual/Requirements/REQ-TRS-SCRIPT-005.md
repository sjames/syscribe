---
id: REQ-TRS-SCRIPT-005
type: Requirement
name: Tool shall provide scripts list and scripts run to discover and execute commands
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** provide a **`scripts`** command family to discover and run registered
extensions ([[REQ-TRS-SCRIPT-004]]).

### `scripts list`

- `syscribe -m <root> scripts list [--json]` **shall** enumerate **every** registered command
  and check, showing each one's **name**, **kind** (`command` | `check`), **description**, and
  **source file**. `--json` emits the same as a machine-readable array.
- With no scripts directory or no registrations, it **shall** report that none are defined
  (exit 0), not error.

### `scripts run`

- `syscribe -m <root> scripts run <command> [--json]` **shall** invoke the named **command**'s
  function with the read-only `model` ([[REQ-TRS-SCRIPT-003]]) and **print its returned
  string** output (`--json` if the command returns structured data). A command **may** also
  stream to **stdout** via `print` and to **stderr** via `eprint` while running
  ([[REQ-TRS-SCRIPT-003]]); the returned value is its final result, stderr is for diagnostics.
- An **unknown** command name **shall** exit non-zero with a message (and `scripts run` with
  no name **shall** print usage). A **check** name is not runnable via `scripts run` (it runs
  under `scripts validate`, [[REQ-TRS-SCRIPT-006]]) — naming one **shall** say so.
- A script error during the run **shall** be reported with the script/command name and exit
  non-zero ([[REQ-TRS-SCRIPT-002]]).

**Source:** user decision — invoke via `syscribe scripts run/list`.

**Acceptance criteria:**

- `scripts list` shows a registered command and a registered check with their kind and source
  file; `--json` carries the same fields.
- `scripts run <command>` prints the command's returned report; `--json` emits its JSON.
- An unknown command exits non-zero; attempting `scripts run <check-name>` reports that it is
  a check, not a command.
- A model with no scripts: `scripts list` reports none (exit 0).
