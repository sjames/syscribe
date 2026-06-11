---
id: REQ-TRS-CLI-004
type: Requirement
title: Tool shall resolve the model root by flag, env, walk-up to .syscribe.toml, then default
status: draft
reqDomain: software
verificationMethod: test
---

The model root is otherwise implicit — it must be supplied on every invocation ([[REQ-TRS-CLI-001]]). To let a user run commands from anywhere inside a model tree (the `git`/`cargo` ergonomics), the tool **shall** resolve the model root in the following precedence, using the **first** that yields a path:

1. the `-m` / `--model` flag;
2. the `SYSCRIBE_MODEL` environment variable;
3. **auto-discovery** — walk upward from the current working directory and select the **nearest ancestor directory that contains a `.syscribe.toml` file**; that directory is the model root;
4. the literal `model` (relative to the current working directory) — the pre-existing default.

This is strictly **additive and backward-compatible**: steps 1, 2, and 4 are unchanged, so existing invocations (`-m model/`, `SYSCRIBE_MODEL`, or a bare `syscribe` next to a `model/` directory) behave exactly as before. Step 3 only ever applies when no flag/env was given **and** a `.syscribe.toml` exists in some ancestor; a tree with no marker falls straight through to step 4.

## Marker semantics

- The marker is the existing **`.syscribe.toml`** config file (§ tooling config). No new marker file or folder is introduced; an **empty** `.syscribe.toml` is a valid "this directory is the model root" marker for a model that needs no other configuration.
- The marker is a **tooling locator only**. The discovered directory is the model root exactly as if it had been passed to `-m`; auto-discovery **shall not** affect qualified-name resolution, the implicit root namespace, or any model semantics.
- Walk-up selects the **nearest** ancestor marker, so a nested model is resolved to its own root; an explicit `-m` always overrides discovery.
- Subcommands that do not load a model (`--agent-instructions`, `spec …`) are unaffected — they never trigger discovery.

**Source:** developer-ergonomics gap — the model root was implicit and `-m` effectively required from outside the model's parent directory. Reuses the existing `.syscribe.toml` (config / `[matchers]` / `[remote]`) rather than adding a new marker.

**Acceptance criteria:** from a subdirectory of a model whose root holds `.syscribe.toml`, a command given **no** `-m` and **no** `SYSCRIBE_MODEL` resolves to that root and operates on its elements; an explicit `-m` from any directory overrides discovery; from a directory with no `.syscribe.toml` in any ancestor and no `model/` present, the tool falls back to the `model` default (no spurious discovery) and reports the missing path.
