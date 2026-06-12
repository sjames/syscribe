---
id: REQ-TRS-VAL-012
type: Requirement
name: Tool shall resolve sourceFile as model-relative, repo-relative, absolute, or remote URI
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** interpret a `sourceFile:` value according to its form, so a model can choose the location semantics per element:

| Form | Semantics |
|---|---|
| `path` (bare relative) | relative to the model root (default) |
| `model:<path>` | relative to the model root (explicit) |
| `repo:<path>` | relative to the repository root |
| `/abs/path` | absolute filesystem path |
| `file://…` | local path from a file URI |
| `scheme://…` (any other scheme) | remote location addressed by URI |

The repository root **shall** be determined from `repo_root` in `<model_root>/.syscribe.toml` if present (resolved against the model root when relative), otherwise by locating the nearest ancestor directory containing `.git`.

For **local** forms, the existence check (`W004`) and function-level check (`W009`) **shall** apply as normal. For **remote** URIs, the tool **shall** treat the file as external: it **shall not** emit `W004` (the path cannot be verified locally) and **shall** skip `W009` (the content cannot be read locally).

**Source:** Feature request — different filepath semantics (model-relative, absolute, remote URI); §11.12 (`W004`, `W009`).

**Acceptance criteria:** a `model:`/bare/absolute/`file://` `sourceFile` pointing at an existing file produces no `W004`; a missing local `sourceFile` produces `W004`; a `scheme://` remote `sourceFile` produces neither `W004` nor `W009`.
