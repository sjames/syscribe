---
id: REQ-TRS-SCRIPT-001
type: Requirement
title: Tool shall load Rhai extension scripts from a configured scripts directory outside the model
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** support user-authored **extension scripts** written in **Rhai**, stored
in a **scripts directory outside the model** and executed via the `scripts` command family.
Scripts are **tooling**, not model content: they are never parsed as model elements, never
appear in the model graph, and are **not** evaluated by the built-in `validate` pass — which
keeps the tool-qualification boundary crisp (a script can never masquerade as a built-in
rule).

### Location and configuration

- The scripts directory path **shall** be configured by a **`[scripts] path`** key in
  `<model_root>/.syscribe.toml`, resolved relative to the config/model root. The default
  when the key is absent **shall** be **`.syscribe/scripts/`**.
- The tool **shall** discover every `*.rhai` file under the scripts directory (recursively).
  An **absent** scripts directory is **not an error** — the model simply has no extensions.

### Reusable library modules

- The scripts directory **shall** be the Rhai **module-import root**: any `*.rhai` file is
  importable from another script by its path-without-extension —
  `import "lib/helpers" as h;` — so shared functions ("library" files) can be **registered
  once and reused** across scripts.
- The module resolver **shall** be confined to the scripts directory; an import escaping it
  (`..`, absolute path) **shall** fail rather than read outside the directory
  ([[REQ-TRS-SCRIPT-002]]).

**Source:** user feature request — extension scripts stored in a configured scripts folder,
reused and executed via `syscribe`. Sandbox/determinism in [[REQ-TRS-SCRIPT-002]], the model
API in [[REQ-TRS-SCRIPT-003]], the two registration shapes in [[REQ-TRS-SCRIPT-004]],
invocation in [[REQ-TRS-SCRIPT-005]]/[[REQ-TRS-SCRIPT-006]].

**Acceptance criteria:**

- With `[scripts] path` unset, `*.rhai` files under `.syscribe/scripts/` are discovered;
  with `[scripts] path = "<dir>"` set, files under `<dir>` are discovered instead.
- A model with **no** scripts directory runs every command normally (no error).
- A script can `import` another `.rhai` file from the scripts directory and call its
  functions; an import that resolves outside the directory fails.
- Scripts are not surfaced as model elements (e.g. `list`/`show` do not see them) and do not
  affect built-in `validate` output.
