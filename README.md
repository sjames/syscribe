# Syscribe

> Structured project knowledge your AI agent can safely edit.

Syscribe keeps structured project state — requirements, architecture, tests, decisions, work items — as plain Markdown files with YAML frontmatter in your git repo, and gives LLM agents an [MCP](https://modelcontextprotocol.io) server to read and edit it. Writes default to a dry run that reports what the change would break, and a commit that would leave a dangling reference or violate a declared link type is refused.

**[Documentation →](https://sjames.github.io/syscribe)**

## See it work

![Terminal recording: an agent's write is refused, fixed, and committed](demo/guarded-write.gif)

An agent proposes a requirement that traces to something that doesn't exist. The dry run shows the damage, the commit gate refuses it, and the fixed version goes in. This is real output from a live `syscribe mcp` server on the bundled ISO 26262 demo model (`python3 demo/mcp-guarded-write.py` replays it):

```text
agent> create_element  Requirements::Safety::FaultLogging  derivedFrom: [REQ-ENG-SAFE-099]  dry_run: true
  ✗ new error   EREF  `derivedFrom` reference 'REQ-ENG-SAFE-099' does not resolve to any model element  [blocks commit]
  ✗ new error   E103  unresolved derivedFrom reference 'REQ-ENG-SAFE-099'
  written: false

agent> create_element  Requirements::Safety::FaultLogging  derivedFrom: [REQ-ENG-SAFE-099]  dry_run: false
  ✗ new error   EREF  `derivedFrom` reference 'REQ-ENG-SAFE-099' does not resolve to any model element  [blocks commit]
  ✗ new error   E103  unresolved derivedFrom reference 'REQ-ENG-SAFE-099'
  ⛔ refused: commit would introduce an unresolved reference
  written: false

agent> create_element  Requirements::Safety::FaultLogging  derivedFrom: [REQ-ENG-SAFE-000]  dry_run: true
  ✓ no new errors or warnings
  written: false

agent> create_element  Requirements::Safety::FaultLogging  derivedFrom: [REQ-ENG-SAFE-000]  dry_run: false
  ✓ no new errors or warnings
  written: true
```

The result is an ordinary, diffable file in your repo — review it, revert it, or open a PR like any other change.

## Quickstart

```bash
curl -fsSL https://raw.githubusercontent.com/sjames/syscribe/main/install.sh | sh   # -> ~/.local/bin/syscribe
git clone https://github.com/sjames/syscribe                                        # for the demo models

# Claude Code — add --read-only to `mcp` for analysis-only (write tools hidden and refused)
claude mcp add syscribe -- "$HOME/.local/bin/syscribe" -m "$PWD/syscribe/model_auto" mcp
```

Other MCP clients (Claude Desktop, …): [connect with a JSON block](#agent-native--the-mcp-server).

**Ways to use it:**
- **MCP server:** let an LLM agent read, analyze and safely edit the model with `syscribe -m model/ mcp` ([setup](#agent-native--the-mcp-server)).
- **CLI:** validate, query, trace and report on a model (`syscribe -m model/ …`).
- **GitHub Action:** gate pull requests on model validity with `uses: sjames/syscribe@v0` ([see below](#installation)).
- **LSP server:** diagnostics, navigation, completion and rename in your editor (`syscribe lsp`).
- **Web browser:** browse the model and its diagrams with `syscribe-server`.

## Build a project by talking to it

The MCP server isn't only for small edits. With it connected, one agent conversation can carry a project from an idea to tested, traced code — and the design, decisions, requirements, tests and plan stay in your repo as reviewable files instead of scrolling away in a chat. [`examples/chat-to-code/`](examples/chat-to-code/) is a real session (the agent's output as produced; only a file path in the test-results sidecar was made relative) that built a small thread-safe rate limiter in Python. What the human typed, and what landed in git:

| You said | The agent did |
|---|---|
| *"Set the model up for this project. Don't invent design decisions yet."* | Scaffolded the model: packages for requirements, design, tests and plan |
| *"Token bucket, sliding window or fixed window? Trade-offs and a recommendation — don't change the model yet."* | Compared them for your constraints, recommended one, changed nothing |
| *"Token bucket it is. Write the design down."* | `ADR-RL-001` (with the rejected alternatives), and a three-part architecture, flagging which parts were its own proposal |
| *"Write the requirements. Lean, and traced to the ADR."* | 8 requirements — stakeholder → testable leaves, each breakdown citing an ADR, each leaf linked to the part that satisfies it |
| *"Write the test cases, and decide the API you need."* | 6 test cases with Gherkin scenarios, a test plan, and the public API recorded as an ADR |
| *"Plan the work."* | An epic that `achieves` the requirements, six tasks with `blockedBy` ordering |
| *"Implement the plan. Claim each item, run pytest, mark done only when tests pass."* | Wrote ~100 lines of code and 17 tests, claimed each item, closed it with evidence |

```text
PI-RL-001  Deliver the rate-limiting library     [done]  achieves REQ-RL-001…008
├─ PI-RL-002  Scaffold the project               [done]
├─ PI-RL-003  Write the pytest suite             [done]
├─ PI-RL-004  Clock, decision, try_acquire       [done]
├─ PI-RL-005  Blocking acquire                   [done]
├─ PI-RL-006  Make the limiter thread-safe       [done]
└─ PI-RL-007  Run the suite and close out        [done]
                                                          8/8 requirements verified · 0 validation errors
```

It is also honest about the rough edges. Midway the agent stopped rather than fake a test run, because `pytest` wasn't installed and it lacked permission to install it. It caught its own thread-safety test being too weak to detect a missing lock and strengthened it. Its architecture and API choices were proposals for you to review, and it promoted requirements to `verified` on its own, skipping a human sign-off. The example's README lists every message the human sent, and the resulting model validates clean: `syscribe -m examples/chat-to-code/model validate`.

## Built for safety-critical work

The model behind those guardrails is a subset of [SysMLv2](https://www.omg.org/sysml/sysmlv2/) with 200+ validation rules drawn from safety-critical practice: enforced requirement → architecture → code → test traceability, ASIL/SIL integrity-level consistency, hazard and threat analysis, and content-hashed release baselines. The repo ships four demo models — an ISO 26262 ASIL D engine ECU, an IEC 61508 SIL 4 railway interlocking, a UAV flight system and an EV charging station ([Demo Models](#demo-models)). You don't need to know SysMLv2 to use any of it, and nothing about the write guardrails is specific to that domain.

---

## The Format

Systems-modeling tools have traditionally been built around proprietary binary formats or complex XML schemas — powerful but opaque, hard for humans to read raw and nearly impossible for LLMs to generate or reason about reliably. Syscribe takes the opposite bet: the project state is plain text, so people, git and agents can all work on it directly.

Syscribe maps SysMLv2 semantics onto plain Markdown files with YAML frontmatter. Every model element is a `.md` file. The directory structure encodes the namespace hierarchy. YAML frontmatter declares the element type and its structural relationships. The Markdown body is the documentation.

```
model/
  System/
    Software/
      SafetyMonitor.md      # part def, ASIL D
      ThrottleControl.md
  Requirements/
    Safety/
      REQ-ENG-SAFE-001.md   # native Requirement with stable ID
  Verification/
    TC-ENG-SAFE-002.md      # TestCase with Gherkin scenarios
  Decisions/
    ADR-ENG-SAFE-001.md     # Architecture Decision Record
```

```yaml
---
type: Requirement
id: REQ-ENG-SAFE-001
name: "Safety monitor shall detect all safety faults within 100 ms"
status: approved
reqDomain: software
asilLevel: D
derivedFrom:
  - REQ-ENG-SAFE-000
breakdownAdr: ADR-ENG-SAFE-001
derivedFromSafetyGoal: SG-ENG-001
---

The safety monitor shall perform a complete supervision cycle within 100 ms...
```

## Why This Format Is Interesting

**For humans** — every element is a readable, diffable, grep-able text file. Edit in any editor, navigate with standard file tools, review in any git diff viewer.

**For LLMs** — Markdown and YAML are what large language models handle best. An LLM can generate a complete model from a natural-language description, add requirements, or trace a safety goal to test cases — all by reading and writing plain files. The built-in prompt (`syscribe --agent-instructions`) gives an LLM everything it needs to produce valid models.

**For teams** — the model lives in a git repository alongside the code. Pull requests, blame, branches, and merges all work as expected.

**For external tools** — because every element is a separate file, it has a stable URL in any git host. A GitHub permalink to `model_auto/Requirements/Safety/REQ-ENG-SAFE-001.md` points to that exact requirement at that exact commit forever. JIRA tickets, Confluence pages, code review comments, and CI reports can all link directly to a specific requirement, test case, or architecture decision — at the branch tip, at a release tag, or pinned to a specific commit hash.

## Agent-Native — the MCP Server

Syscribe is a [Model Context Protocol](https://modelcontextprotocol.io) server, so an LLM agent works with the model as a first-class client — reading, analyzing, and *safely writing* it — not just generating files from a prompt.

**Writes are guarded.** Every `create_element` / `update_element` / `move_element` / `delete_element` / `apply_changes` call defaults to `dry_run: true`, returns the **validation delta** the change would cause, and refuses to commit anything that would break referential integrity — so an agent can propose a change, inspect its effect, and only then commit it. The delta lists every newly introduced or resolved warning and error. Each error carries a `gating` flag: only dangling references (`EREF`) and user-defined link-type violations (`E630`–`E636`) refuse the commit, so incomplete drafts stay creatable. Other errors (for example `E310`, a derived requirement with no `breakdownAdr`) are reported with `gating: false`; agents (and CI) should fix them or run a full `validate`. Sealing a release stays a deliberate CLI/CI action.

```bash
syscribe -m model_auto/ mcp                # stdio MCP server (`syscribe help mcp` lists the tools)
syscribe -m model_auto/ mcp --read-only    # analysis only; write tools hidden & refused
```

**Connect it to an MCP client.** Install the `syscribe` binary (see [Installation](#installation)), then register it with the model directory it should serve. The commands below assume the [install script](#installation)'s default location, `~/.local/bin/syscribe` (use `~/.cargo/bin/syscribe` if you installed with `cargo install`). Use absolute paths. Add `--read-only` after `mcp` whenever the agent should only look, not touch.

For Claude Code:

```bash
claude mcp add syscribe -- "$HOME/.local/bin/syscribe" -m /abs/path/to/model mcp
# or, analysis only:
claude mcp add syscribe -- "$HOME/.local/bin/syscribe" -m /abs/path/to/model mcp --read-only
```

For clients configured with an `mcpServers` JSON block (such as Claude Desktop, in `claude_desktop_config.json`):

```json
{
  "mcpServers": {
    "syscribe": {
      "command": "/home/you/.local/bin/syscribe",
      "args": ["-m", "/abs/path/to/model", "mcp"]
    }
  }
}
```

JSON does not expand `~` or `$HOME`, so write your home directory out in full (`/home/you/…` on Linux, `/Users/you/…` on macOS). Use `"args": ["-m", "/abs/path/to/model", "mcp", "--read-only"]` for an analysis-only server. Claude Desktop only reads its config at launch, so fully quit and reopen it after editing. The server speaks MCP over stdio, needs no network access, and serves exactly the model directory passed with `-m`. It watches that directory and reloads automatically when files change outside MCP (an editor, the CLI, `git checkout`); pass `--no-watch` to disable.

Read tools cover retrieval, fuzzy search, the containment/graph, `trace` / `impact`, validation, coverage, and the suspect/baseline surfaces.

## Traceability That Survives Change

Static validation answers *"is the model internally consistent right now?"* Syscribe also answers the two questions that matter across the life of a safety program.

**Has a reviewed link gone stale?** A trace link (`verifies`, `derivedFrom`, `satisfies`, …) asserts a relationship that was valid *when a human reviewed it*. `suspect accept` captures a BLAKE3 hash of the target's normative content; if the target later changes, the link surfaces as `W090` (*suspect*) so it can be re-reviewed. Editorial edits are excluded, so the signal is low-noise and version-control-agnostic — a precise flag that one specific reviewed relationship needs re-confirmation.

```bash
syscribe -m model_auto/ suspect list                          # suspect + un-baselined links
syscribe -m model_auto/ suspect accept TC-ENG-SAFE-002 REQ-ENG-SAFE-001
syscribe -m model_auto/ validate --deny W090                  # gate CI on stale links
```

**What exactly was released, and can you prove it hasn't changed?** A `Baseline` (`BL-*`) freezes a scope of the model into a git-anchored, content-hashed release an assessor can point to directly. The scope is the whole model, a package subtree, a projected product-line variant (`config=`), or the trace closure of one safety goal (`closureFrom=`). A **released** baseline is frozen: any change to its sealed content is a hard error (`E520`); `verify` re-proves the content hash *and* the git tag↔commit; `diff` shows exactly what changed between two releases.

```bash
syscribe -m model_auto/ baseline create --tag REL-2026-07 --approver "J. Roe"
syscribe -m model_auto/ baseline verify --all                 # CI gate: content + git proof
syscribe -m model_auto/ baseline diff BL-2026-06 BL-2026-07   # what changed between releases
```

Together these turn a git-controlled model into an **audit trail**: every relationship is either confirmed-current or flagged for review, and every release is a provable, comparable snapshot.

## Installation

**Install script (Linux, macOS).** Downloads the latest release binary for your platform (the static musl build on Linux), verifies it against the SHA-256 published with the release, checks that it runs, and puts it in `~/.local/bin`. A checksum mismatch aborts the install:

```bash
curl -fsSL https://raw.githubusercontent.com/sjames/syscribe/main/install.sh | sh
# or choose the install directory:
curl -fsSL https://raw.githubusercontent.com/sjames/syscribe/main/install.sh | sh -s -- --dir /usr/local/bin
```

**Prebuilt binaries.** Each [GitHub release](https://github.com/sjames/syscribe/releases) ships a standalone `syscribe` CLI binary, named `syscribe-<target>` (`.exe` on Windows):

| Platform | Asset |
|---|---|
| Linux x86_64 | `syscribe-x86_64-unknown-linux-gnu` |
| Linux aarch64 | `syscribe-aarch64-unknown-linux-gnu` |
| Linux x86_64, static (musl) | `syscribe-x86_64-unknown-linux-musl` |
| Linux aarch64, static (musl) | `syscribe-aarch64-unknown-linux-musl` |
| macOS Intel | `syscribe-x86_64-apple-darwin` |
| macOS Apple silicon | `syscribe-aarch64-apple-darwin` |
| Windows x86_64 | `syscribe-x86_64-pc-windows-msvc.exe` |

Releases also publish `<asset>.sha256` next to each binary (check with `sha256sum -c syscribe-<target>.sha256`, or `shasum -a 256 -c` on macOS). The checksum comes from the same place as the binary, so it guards against corrupt or truncated downloads, not against a compromised release.

The musl builds (from v0.41.0) are fully static: they run on Alpine, `scratch`/distroless containers and hosts with an old glibc.

```bash
curl -fsSL -o syscribe \
  https://github.com/sjames/syscribe/releases/latest/download/syscribe-x86_64-unknown-linux-gnu
chmod +x syscribe
```

**From source.** Needs a recent stable Rust toolchain ([rustup.rs](https://rustup.rs)). `syscribe` is not published on crates.io, so install straight from the repository:

```bash
cargo install --git https://github.com/sjames/syscribe --locked syscribe     # CLI + MCP server -> ~/.cargo/bin/syscribe
cargo install --git https://github.com/sjames/syscribe --locked syscribe-server   # web browser (not a release asset)
```

Or from a checkout, which is also how you get the demo models:

```bash
git clone https://github.com/sjames/syscribe && cd syscribe
cargo install --path crates/syscribe --locked          # add crates/syscribe-server for the web UI
cargo build --workspace                                # dev build: target/debug/syscribe and target/debug/syscribe-server
```

A release build takes a few minutes. Make sure `~/.cargo/bin` is on your `PATH`.

**In GitHub Actions.** The repository is also a reusable action that downloads the release binary for the runner and validates a model:

```yaml
- uses: actions/checkout@v4
- uses: sjames/syscribe@v0
  with:
    model-path: model/          # default
    version: latest             # or a release tag, e.g. v0.40.1
    args: --deny W090           # extra arguments to `validate`
    fail-on-warnings: false
    upload-report: true         # upload the report as a workflow artifact
```

It exposes `errors` and `warnings` counts as step outputs. See the [CI/CD guide](https://sjames.github.io/syscribe/guides/cicd/) for more.

## Running

### Validate a model

The model root is set with `-m` / `--model`, or via the `SYSCRIBE_MODEL` environment variable:

```bash
# Full validation report
syscribe -m model_auto/

# Findings only
syscribe -m model_auto/ validate

# Machine-readable JSON output
syscribe -m model_auto/ validate --json

# Scoped to a single file
syscribe -m model_auto/ validate --file model_auto/System/Software/SafetyMonitor.md

# CI gating — exit 0 (clean) / 1 (errors) / 2 (gated warnings)
syscribe -m model_auto/ validate --deny W004,W009    # named warnings become fatal
syscribe -m model_auto/ validate --max-warnings 0    # any warning fails the build
syscribe -m model_auto/ validate --warnings-as-errors

# Fetch & verify remote sourceFiles via the .syscribe.toml [remote] hook (opt-in)
syscribe -m model_auto/ validate --fetch-remote
```

`sourceFile:` values resolve by form — bare/`model:` (model root), `repo:` (repo root), absolute, `file://`, or a remote `scheme://` URI. Remote URIs are accepted unverified by default; with a `[remote] download` hook in `.syscribe.toml` and `--fetch-remote`, they are downloaded (cached under `.syscribe/cache/`) and checked like local files. The hook only runs with the explicit flag.

### Query the model

```bash
# Trace a requirement end-to-end
syscribe -m model_auto/ trace REQ-ENG-SAFE-001

# What does this component satisfy?
syscribe -m model_auto/ why System::Software::SafetyMonitor

# Who tests this requirement?
syscribe -m model_auto/ who-verifies REQ-ENG-SAFE-001

# All relationships on an element (impact analysis)
syscribe -m model_auto/ links System::Software::SafetyMonitor

# Next available stable ID
syscribe -m model_auto/ next-id REQ-ENG-SAFE

# Fuzzy search across names, IDs, and docs
syscribe -m model_auto/ find throttle

# Requirement × Configuration coverage grid (variant-aware)
syscribe -m model_auto/ matrix

# Upstream / downstream impact of changing an element
syscribe -m model_auto/ impact REQ-ENG-SAFE-001
```

### Refactor: move an element or package

`move` relocates an element (or a whole package subtree) to a new qualified name and atomically rewrites every reference to it — frontmatter (including nested `connections`/`features`), multi-segment qualified names cited in Markdown bodies, and SVG diagram references (inline and companion `.svg` files: `sysml:ref`/`data-qname`/`href`). Stable IDs (`REQ-*`, `TC-*`, `ADR-*`) and references made through them are preserved.

```bash
# Preview the file move and every reference update — writes nothing
syscribe -m model_auto/ move System::Software::SafetyMonitor System::Safety::SafetyMonitor --dry-run

# Apply it (all-or-nothing; rolls back on any error)
syscribe -m model_auto/ move System::Software::SafetyMonitor System::Safety::SafetyMonitor
```

### Export the model graph

For CI gates, dashboards, and LLM agents that need the whole model without re-parsing Markdown:

```bash
# Versioned JSON document (schemaVersion + elements[] with typed frontmatter
# and resolved relationships: computed.verifiedBy / derivedChildren)
syscribe -m model_auto/ export

# Newline-delimited JSON (header line, then one element per line)
syscribe -m model_auto/ export --ndjson
```

Each element carries `qname`, `file`, `id`, `type`, `name`, its typed `frontmatter`, and — for requirements — a `computed` block with the resolved `verifiedBy` and `derivedChildren` reverse indices.

### Diagrams

```bash
# Generate PlantUML companion files, then render them to SVG
# (needs `plantuml` on PATH or PLANTUML_JAR set)
syscribe -m model_auto/ plantuml
syscribe -m model_auto/ plantuml render
```

Diagrams also render live in the web UI — server-side SVG plus client-side Mermaid — where non-Mermaid diagrams can be edited directly (create, connect, move, delete; every edit is validated before it is written).

### Browse in a web UI

```bash
syscribe-server -m model_auto/
# open http://localhost:3000   (tree browser, element detail, diagrams)
```

The server watches the model directory and reloads automatically on file changes.

### In-terminal format spec

```bash
syscribe spec               # table of contents
syscribe spec types         # all element types
syscribe spec validation    # all validation rule codes
syscribe spec traceability  # traceability rules
```

### Version

```bash
syscribe --version          # also -V, or `syscribe version` → "syscribe <semver>"
```

## Demo Models

| Model | Domain | Standards / method |
|---|---|---|
| `model_auto/` | Automotive Engine ECU | ISO 26262 (ASIL D), ISO/SAE 21434, AUTOSAR SecOC |
| `model_sil/` | SIL 4 Railway Interlocking | IEC 61508, EN 50128/50129, EN 50159 Cat 2 |
| `model/` | UAV Autonomous Flight System | General SysMLv2 element palette |
| `model_mg/` | EV DC Fast-Charging Station | MagicGrid (problem/solution × 4 pillars) |

All four demo models validate with 0 errors and demonstrate full requirements traceability, safety analysis (HARA, FTA, FMEA), and security analysis (TARA); CI gates on `validate` for every model.

## LLM Authoring

```bash
# Print the generation prompt
syscribe --agent-instructions

# Use it directly with your LLM tool
syscribe --agent-instructions | llm "Create a brake-by-wire model for ISO 26262 ASIL D"
```

The prompt and the validator are always in sync — `--agent-instructions` is embedded at compile time from `prompts/create-model.md`. For interactive, guarded authoring where the agent inspects each change before committing, run the [MCP server](#agent-native--the-mcp-server) instead. See the [LLM Workflow guide](https://sjames.github.io/syscribe/model-guide/llm-workflow/) for the full incremental authoring workflow.

## Full Feature List

- **40+ element types** covering SysMLv2 structural, behavioral, and requirements constructs
- **Native Requirement** elements (REQ-* stable IDs) with SIL/ASIL, lifecycle status, domain classification, derivation trees
- **Native TestCase** elements (TC-* IDs) with L1–L5 test levels and Gherkin scenarios
- **Architecture Decision Records** (ADR-*) — every requirement decomposition cites an accepted ADR
- **Safety analysis**: HARA, SafetyGoal, HazardousEvent, FaultTree (file-per-node), FMEA (exploded entries)
- **Security analysis**: TARA, DamageScenario, ThreatScenario, CybersecurityGoal, SecurityControl, VulnerabilityReport
- **Variability / product lines**: feature models (`FeatureDef`, `Configuration`), `appliesWhen:` conditioning, SAT-backed `feature-check`, and the `--config` projection lens
- **Multi-repository composition** (§14): import namespaces from peer repos via `[repos]` + `repoImports:`, resolve cross-repo references by global stable ID, and gate reproducibility on git ref drift / submodule gitlink (`W510`–`W512`)
- **Hierarchical product lines** (§14.7): a `Configuration` consolidates already-configured lower-tier `Configuration`s — local or in a `[repos]`-mounted product-line repo, at any depth — via `subConfigurations:`, with parameter bindings resolved across tiers (`examples/hple-multitier/`)
- **Work tracking** — native `PlanningItem` (`PI-*`) epics/stories/tasks tied to the `Requirement`s they achieve, with evidence-backed `done`, `blockedBy:`, `assignedTo:`, `syscribe set` for status/evidence edits, and `claim` / `release` advisory ownership for concurrent multi-agent work
- **User-defined link types** — declare project relationships (`mitigates`, `conflictsWith`, …) as `[linkTypes.*]` in `.syscribe.toml` with source/target types, cardinality and acyclicity; author them under `links:`, list the vocabulary with `link-types`, and walk them with `follow`
- **Foreign sources in one graph** — native SysMLv2 textual submodels (`sysmlSubmodel:`), any custom notation through stdio-subprocess plugins (`foreignFormat:`), and elements declared in ordinary source-code comments (`annotationFormat:`), all validated and traced like hand-written elements
- **IEC 62443 zones & conduits**, **review records**, **trade studies**, and **state-machine / sequence completeness** checks
- **Seven §12 traceability rules** enforced by the validator: OSLC link direction, breakdown ADR, leaf assignment, domain classification, HW/SW independence, deployment allocation, implementation trace (`implementedBy:`)
- **200+ validation rules** across parse-time, cross-reference, safety/security, behavior, and composition: cross-reference resolution, integrity level consistency, diagram annotation, documentation completeness
- **Suspect links** — content-baseline (`traceBaselines:`, BLAKE3) detection of *stale* trace links: when a reviewed relationship's target changes, it surfaces as `W090` and is cleared by re-review (`suspect accept`)
- **Release baselines** — first-class, git-anchored, content-hashed frozen release snapshots (`Baseline`, `BL-*`) with drift detection, scoped to the whole model, a package, a product-line variant, or a safety goal's trace closure
- **MCP server** — `syscribe mcp` exposes structured tools to LLM agents: read/query/trace/validate plus *guarded* writes (dry-run → validation delta → referential-integrity commit gate)
- **LSP server** — `syscribe lsp` gives editors live diagnostics and model-aware navigation over stdio (a VS Code extension lives in `editors/vscode/`)
- **Static HTML export** — `syscribe export-html` renders the whole model as a standalone, offline site for reviewers without the toolchain
- **Diagrams** — server-rendered SVG, client-side Mermaid, an editable diagram view in the web UI, and PlantUML companion generation/rendering
- **Coverage & product-line matrices** — Requirement × Configuration coverage grids, variant-aware verification depth, SAT-backed feature analysis
- **LLM-scale corpus tools** — `stats` / `digest` / `search-text` / `summarize` / `topics` / `clusters` for navigating large models, plus `impact` change analysis and ReqIF/SBOM export

## Repository Structure

```
crates/
  syscribe/           # CLI validator and query tool
  syscribe-model/     # core library: parser, walker, graph builder, resolver, renderer
  syscribe-server/    # Axum web server + Askama templates + HTMX frontend
model/                # UAV autonomous flight system demo model
model_auto/           # Engine ECU demo model (ISO 26262 / ISO/SAE 21434)
model_sil/            # SIL 4 railway interlocking demo model (IEC 61508 / EN 50128)
model_mg/             # EV DC fast-charging station demo model (MagicGrid)
prompts/              # LLM authoring prompt (embedded in the CLI binary)
spec/                 # Syscribe format specification
docs/                 # MkDocs documentation source
```

## Prior Work

Syscribe is an evolution of [assemblyline](https://github.com/sjames/assemblyline), an earlier experiment in structured systems modeling that used [Typst](https://typst.app) as the modeling language. Typst worked well for rendering and was extensible, but the language was unnecessarily complex for the authoring use case — requiring toolchain knowledge just to read or write a model element.

Markdown + YAML removes that barrier entirely. No special tools are needed to write or read the content. If you host the model on GitHub, the files render as-is. The ideas from assemblyline are reimplemented here in a format that any editor, any diff viewer, and any LLM can handle natively.

---

If you use Syscribe in a project or find it useful, feel free to tag [@sojan_james](https://twitter.com/sojan_james) on Twitter/X.
