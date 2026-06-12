---
id: REQ-TRS-CLI-008
type: Requirement
name: Tool shall route top-level commands through a clap registry that rejects unknown commands
status: draft
reqDomain: software
verificationMethod: test
---

The tool's top-level command line **shall** be parsed by a `clap`-based router that holds
a **single declarative registry** of every subcommand (derived from the one embedded
help-page list, so the registry cannot drift from the help pages).

The router **shall**:

- **reject an unknown command** (e.g. `syscribe bogus`) with a clear error to stderr and
  a **non-zero** exit code, **without** first requiring a model directory — command
  validation happens before model resolution, so an unknown command is reported the same
  from any working directory;
- **accept every registered command**, dispatching it to its existing handler with its
  arguments unchanged (per-command flag parsing is unaffected — a command's own flags,
  including `--`-prefixed ones, pass through to that command);
- **preserve** the existing behaviours unchanged:
  - man-page help via `--help`, `-h`, and `help <command>` ([[REQ-TRS-CLI-005]]) — these
    are handled before the router and still print the `SYNOPSIS` man-page and exit 0;
  - `--version` / `-V` / `version` ([[REQ-TRS-CLI-007]]);
  - `--agent-instructions [topic]` ([[REQ-TRS-CLI-006]]);
  - the `spec [<section>]` browser;
  - model-root resolution priority — `--model`/`-m` > `SYSCRIBE_MODEL` > walk-up to
    `.syscribe.toml` > the `model/` default ([[REQ-TRS-CLI-004]]);
- treat a bare invocation with a model but **no** subcommand (and the explicit `report`
  command) as the default full validation report.

The registry **shall** stay consistent with the help pages: every command that has a
man-page is a registered command, and every registered command has a man-page.
