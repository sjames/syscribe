---
id: REQ-TRS-TREX-004
type: Requirement
name: The export is available as the CLI command trace-export and the read-only MCP tool trace_export with the same options and document
status: verified
reqDomain: software
verificationMethod: test
---

A CLI command `syscribe -m <root> trace-export [--config <C>] [--sort <order>] [--out
<file>]` **shall** write the document to stdout (or to `--out`, creating parent directories,
with nothing on stdout), exit 0, and treat an unknown option or a value-taking option with no
value as a usage error (named on stderr, nothing on stdout, exit 1) before the model is loaded,
consistent with the other commands' option checks. A read-only MCP tool `trace_export` with
optional parameters `config` and `sort` **shall** return the same document as its result and
report an invalid value as a tool error. Both **shall** be documented: `prompts/help/trace-
export.md` (registered in the help index) and the entry in `prompts/help/mcp.md`.

**Source:** `REQ-TRS-TREX-004` (product model).

**Acceptance criteria:** (a) `--out <dir>/nested/deeper/trace.json` writes the file, creates
the directories and leaves stdout empty, and the file holds the bytes stdout would carry; (b)
`trace-export --bogus` against a nonexistent model root exits 1 naming the option and the
valid ones; `--sort` with no value exits 1; (c) `syscribe help trace-export` and `trace-export
--help` print the man page; (d) the MCP `trace_export {}` result equals the CLI document for
the same model, honours `sort` and `config`, and `{sort: "random"}` / `{config:
"CONF-NOPE-001"}` are tool errors.
