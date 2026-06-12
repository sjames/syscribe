---
id: REQ-TRS-SCRIPT-002
type: Requirement
name: Tool shall run extension scripts in a sandboxed, resource-limited, deterministic engine
status: draft
reqDomain: software
verificationMethod: test
---

Because extension scripts are user-authored and may be untrusted, the tool **shall** execute
them in a **sandboxed** Rhai engine with **resource limits**, exposing **only** the
read-only model API ([[REQ-TRS-SCRIPT-003]]) plus the registration and `finding`/output
functions — nothing else.

### Sandbox

- The engine **shall not** expose filesystem, network, clock, randomness, environment, or
  process access, and **shall** disable `eval`. A script therefore cannot read or write
  files, reach the network, or observe wall-clock/random state. The **sole** permitted side
  effect is **text output to stdout/stderr** (`print`/`eprint`, [[REQ-TRS-SCRIPT-003]]), which
  conveys no read capability and is deterministic.
- The model API **shall** be **read-only** (getters only); a script **shall not** be able to
  mutate the model or any file.
- The module resolver **shall** be confined to the scripts directory ([[REQ-TRS-SCRIPT-001]]).

### Resource limits

- The engine **shall** enforce limits — a maximum **operation/instruction budget**, maximum
  **call/expression depth**, and maximum **string / array / map sizes** — so a runaway or
  infinite script is **aborted with an error** (non-zero, bounded time), never an unbounded
  hang.

### Determinism

- Given the same model and the same scripts, a script's output **shall** be **reproducible**
  (no nondeterministic host functions are exposed), so script results are stable in CI.

### Error handling

- A script **parse error, runtime error, or limit breach** **shall** be reported with the
  offending script's name and a message, and **shall not** crash the tool or corrupt other
  scripts' results.

**Source:** user request (sandboxed, reusable, deterministic extension scripts). Engine:
Rhai (pure-Rust, sandbox-by-default) — to be recorded as an engine-choice ADR alongside the
`batsat` precedent.

**Acceptance criteria:**

- A script attempting filesystem/network/`eval` access fails (the capability is absent),
  not silently succeeds.
- A script with an unbounded loop is aborted by the operation limit with a clear error and a
  non-zero exit, not a hang.
- A syntactically invalid script is reported with its name and a parse error; other scripts
  still load and run.
- Running the same command twice over the same model produces identical output.
