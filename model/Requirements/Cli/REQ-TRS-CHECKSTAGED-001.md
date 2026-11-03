---
type: Requirement
id: REQ-TRS-CHECKSTAGED-001
name: "check-staged validates only the staged model files, suitable for a pre-commit hook"
status: draft
reqDomain: software
reqClass: system
tags:
  - cli
  - git
  - validation
---

`syscribe check-staged` shall run the checks of `validate` restricted to what a commit touches, quickly enough to run on every commit (GH #445).

## Behavior

- **Touched elements** are those in the staged model files plus the elements that directly reference them. Staging `.syscribe.toml` or another model-wide config file touches every element.
- The checks run on the unprojected model and, when the model has a feature model, under each `Configuration` a touched element is active in (`--skip-configs` omits the latter). Only findings on touched elements are reported.
- The staged content is judged: the working tree when it matches the index, else an export of the index, so unstaged edits can neither hide nor cause a finding.
- A staged deletion or rename also reports every finding the commit introduces compared with `HEAD`, since it can leave a dependant dangling.
- With no staged model file the command decides from `git` alone and exits `0` without loading the model.
- Exit `1` on an error finding or a usage problem (not a git repository), `2` on a warning gated by `--deny`/`--max-warnings`/`--warnings-as-errors`. `--fix-hints` adds each code's explanation and the commands that fix it; `--json` emits `schemaVersion: "1.0"`.
- A staged `TestCase` whose `testFunctions` have no Gherkin blocks fails the check with a finding that names `scaffold-gherkin <TC> --fix`.
- `--install-hook` writes a `pre-commit` hook (honouring `core.hooksPath`) that runs the command from the repository root; it refuses to overwrite a hook it did not write, and the hook lets a commit through, with a message, when `syscribe` is not on `PATH`.
