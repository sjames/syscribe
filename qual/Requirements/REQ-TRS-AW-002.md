---
id: REQ-TRS-AW-002
type: Requirement
name: Tool shall display an element's own and effective appliesWhen when applies-when is run with no flags
status: draft
reqDomain: software
verificationMethod: test
---

When the `applies-when` command ([[REQ-TRS-AW-001]]) is invoked on an element with
**neither** `--set` **nor** `--clear`, it **shall** act as a **read-only** query and report
the element's gate. This makes the command symmetric (read / write) and answers
"under what condition does this element apply?" without naming a specific configuration
(unlike `why-active`, which evaluates the gate against one `Configuration`).

### Display

- `syscribe -m <root> applies-when <element>` (no `--set`/`--clear`) **shall** print:
  - the element's **own** `appliesWhen:` expression, or that it has none; and
  - the element's **effective** condition — its own gate, or, when it declares none, the
    gate **inherited** from its nearest ancestor package (transitive package
    conditioning, [[REQ-TRS-VAR-006]]), naming the package it was inherited from.
- An element with no own gate and no ancestor-package gate **shall** be reported as
  applying **unconditionally** ("always applies").
- A `FEAT-*` id operand **shall** render identically to its qualified-name form
  ([[REQ-TRS-ID-006]]); the displayed expression is shown as authored.
- The command **shall** be strictly **read-only**: it modifies no file.
- `<element>` resolves by qualified name or stable id; an unresolved target **shall** exit
  non-zero. A `--json` form **shall** emit the same information as a machine-readable object
  (`element`, `own`, `effective`, `inheritedFrom`).

**Source:** user request (a bare `applies-when` should show when the element applies).
Read complement of [[REQ-TRS-AW-001]]; distinct from `show` (which prints only the *own*
field) and `why-active` (which evaluates the gate for a specific `Configuration`).

**Acceptance criteria:**

- `applies-when <gated-element>` prints the element's own `appliesWhen` expression and an
  effective condition equal to it.
- `applies-when <element-gated-only-by-an-ancestor-package>` reports no own gate and an
  effective condition equal to the inherited package gate, naming that package.
- `applies-when <ungated-element>` reports that it always applies.
- The command writes nothing (the file is byte-identical afterwards).
- An unresolved element exits non-zero; `--json` emits the structured form.
