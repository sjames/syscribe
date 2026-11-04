---
type: Requirement
id: REQ-TRS-WRITEGUARD-001
name: "Writes can be guarded against stale reads, report themselves as JSON, and the exit codes are fixed"
status: draft
reqDomain: software
reqClass: system
tags:
  - cli
  - git
---

Writing commands shall refuse to overwrite a concurrent change, report their result in a stable envelope, and keep a documented exit-code contract (GH #444, `ADR-SYS-PROVENANCE-001`).

## Behavior

- `--expect-hash <hash>` (CLI: `set`, `claim`, `release`, `move`, `applies-when`, `scaffold-gherkin --fix`; MCP: `update_element`, `move_element`, `delete_element`, `question`) refuses with exit 1 and writes nothing when the target file's content hash differs. The hash is BLAKE3 of the whole file, shown as `contentHash` by `show` and the MCP `get_element` detail.
- With `--json`, `set`, `claim`, `release` and `move` print one envelope `{schemaVersion, ok, dryRun, command, wrote, wouldWrite, commit, messages, diff}` instead of text; commands that are not writers are unchanged.
- Exit codes: `0` success; `1` a usage error, an unknown element, or a refused write; `2` a tripped gate. `connectivity`, `follow` and `impact` also exit `2` on a usage error and are documented exceptions. There is no exit `3`.
- `mcp --read-only` hides exactly the tools not annotated read-only, and none that are.
