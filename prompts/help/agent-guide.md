# agent-guide — a project-specific instruction block for agents, generated from the model

## SYNOPSIS
    syscribe -m <root> agent-guide [--format markdown|json] [--sections ids,types,gates,rules]
        [--write <file> | --check <file>]

## DESCRIPTION
Prints the guidance a script or LLM agent needs to edit *this* model safely, built from the
loaded model and `.syscribe.toml` so it cannot go stale the way a hand-written instruction file does:

- **ids** — the id prefix of each type in use (plus any extra `[ids.prefixes]`) and the `next-id` command for it;
- **types** — per lifecycle type: the status vocabulary, the required fields, and the commands that act on it;
- **gates** — the exact `validate` commands to run (one per `[profiles.<name>]`), and the `[gates]` they enforce, each of which must exit 0 before work is called done, and the `[planning]` evidence gates that are switched on (`require_testcase_evidence` → `W319`, `require_commit_evidence` → `W321`, `check_commit_paths` → `W320`) and `[traceability] require_amendments` (→ `W324`), with the command that satisfies each;
- **rules** — safe mutation: `next-id`/`template` for new elements, `set`/`move`/`claim`/`release` instead of hand edits, `--dry-run` previews, citing a requirement item by its anchor (`REQ-X#slug`) rather than its number, writing a design value as a value reference `{{Elem.attr}}` rather than copying the number, amending (`set <id> amendment.add`) rather than silently editing an accepted ADR or approved Requirement, and that a question with `answerBy: human` is a person's to answer;
- **policy** — the project's `[agents]` rules in plain words (e.g. `human_only_answers`: only a person answers questions), when it has any.

The output is deterministic (no timestamps, no tool version), so it only changes when the
model or `.syscribe.toml` does. The Markdown block is wrapped in
`<!-- syscribe:agent-guide -->` … `<!-- /syscribe:agent-guide -->`.

**`--write <file>`** replaces only that region of `<file>` (creating the file, or appending the
region when it has none); everything outside the markers is never touched. Damaged or repeated
markers are refused. **`--check <file>`** exits 1 when the region is missing or out of date, for CI.
The MCP server's `initialize` instructions are generated from the same data
(`syscribe mcp --print-instructions`).

## OPTIONS
    --format markdown|json   Markdown block (default) or the structured guide (schemaVersion 1.0).
    --sections <list>        Comma-separated subset of ids, types, gates, rules (default: all).
    --write <file>           Update the marked region of <file> in place.
    --check <file>           Fail (exit 1) if <file>'s region differs from what would be generated.

## EXIT CODES
    0  printed / written / up to date
    1  usage error, unknown section, unreadable or damaged file, or a stale region under --check

## EXAMPLES
    syscribe -m model/ agent-guide
    syscribe -m model/ agent-guide --write CLAUDE.md
    syscribe -m model/ agent-guide --check CLAUDE.md          # in CI
    syscribe -m model/ agent-guide --format json --sections types

## SEE ALSO
    mcp, help, template, next-id, validate
