---
id: REQ-TRS-SCRIPT-006
type: Requirement
name: Tool shall run validation-hook scripts via scripts validate, with namespaced findings and a CI gate
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** provide **`scripts validate`** to run the registered **validation-hook**
scripts ([[REQ-TRS-SCRIPT-004]]) and surface their findings — the "validation participation"
of extensions, kept **separate** from the built-in (qualified) `validate`.

### Behaviour

- `syscribe -m <root> scripts validate [gate flags]` **shall** invoke **every** registered
  `check` with the read-only `model` ([[REQ-TRS-SCRIPT-003]]), collect the `finding(...)`
  calls, and print them in a findings report.
- Each finding **shall** be **namespaced and marked as script-origin** — rendered as
  **`<check>/<code>`** (e.g. `naming/NOASIL`) with the **source script** shown — so it can
  **never** be confused with or collide with a built-in `E###`/`W###`/`MG###` code. A
  finding's `severity` is `error | warning | info`.
- The exit code **shall** follow the standard contract: `0` clean, `1` on any
  `error`-severity finding, `2` when a gate trips; it **shall** reuse the existing gate flags
  **`--deny <codes>`**, **`--max-warnings <n>`**, **`--warnings-as-errors`**.

### Separation from built-in validate

- `scripts validate` **shall not** alter the output or exit code of the built-in `validate`
  command, and built-in `validate` **shall not** run extension checks. The qualified
  validation pass and the user-extension pass are independent (qualification boundary).

**Source:** user decision — scripts hook into validation, invoked through the `scripts`
command (`scripts validate`), distinct from the built-in qualified `validate`.

**Acceptance criteria:**

- `scripts validate` runs the registered checks and prints their findings as `<check>/<code>`
  with the source script; an `error`-severity finding exits `1`, a clean run exits `0`.
- `--deny <check/code>` / `--max-warnings`/`--warnings-as-errors` gate the exit code as for
  built-in `validate`.
- Built-in `syscribe validate` over the same model is byte-for-byte unaffected by the presence
  of `check` scripts.
- A check that errors at runtime is reported (named) and exits non-zero, without masking other
  checks' findings.
