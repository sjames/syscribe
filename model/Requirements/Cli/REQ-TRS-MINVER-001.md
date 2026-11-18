---
type: Requirement
id: REQ-TRS-MINVER-001
name: "A model declares the minimum syscribe version it needs, and an older syscribe refuses to load it"
status: approved
reqDomain: software
reqClass: system
tags:
  - cli
  - config
---

`[syscribe] minVersion = "<version>"` (or `">=<version>"`) in a model's `.syscribe.toml` shall make every command that reads the model, `syscribe-server`, `syscribe mcp` and `syscribe lsp` refuse it when the running syscribe is older: exit `1`, naming the declared and the running version, before the model is read (GH #496). A value that is neither a version nor `>=<version>` is an error naming the file. Commands that need no model are unaffected. `syscribe policy` shall report the declared minimum (`minVersion`), `capabilities -m <root>` shall report it (`modelMinVersion`), and `validate` shall warn `W329` when the model uses a feature newer than the declared minimum, never when none is declared.
