# Syscribe

> Markdown-native SysMLv2 modeling — human-readable, LLM-friendly, version-controlled, fully traceable.

**[Documentation →](https://sjames.github.io/syscribe)**

---

## The Idea

Systems modeling tools have traditionally been built around proprietary binary formats or complex XML schemas. They are powerful but opaque — hard for humans to read in raw form, and nearly impossible for LLMs to generate or reason about reliably.

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

## What It Supports

- **40+ element types** covering SysMLv2 structural, behavioral, and requirements constructs
- **Native Requirement** elements (REQ-* stable IDs) with SIL/ASIL, lifecycle status, domain classification, derivation trees
- **Native TestCase** elements (TC-* IDs) with L1–L5 test levels and Gherkin scenarios
- **Architecture Decision Records** (ADR-*) — every requirement decomposition cites an accepted ADR
- **Safety analysis**: HARA, SafetyGoal, HazardousEvent, FaultTree (file-per-node), FMEA (exploded entries)
- **Security analysis**: TARA, DamageScenario, ThreatScenario, CybersecurityGoal, SecurityControl, VulnerabilityReport
- **Variability / product lines**: feature models (`FeatureDef`, `Configuration`), `appliesWhen:` conditioning, SAT-backed `feature-check`, and the `--config` projection lens
- **Multi-repository composition** (§14): import namespaces from peer repos via `[repos]` + `repoImports:`, resolve cross-repo references by global stable ID, and gate reproducibility on git ref drift / submodule gitlink (`W510`–`W512`)
- **IEC 62443 zones & conduits**, **review records**, **trade studies**, and **state-machine / sequence completeness** checks
- **Six §12 traceability rules** enforced by the validator: OSLC link direction, breakdown ADR, leaf assignment, domain classification, HW/SW independence, deployment allocation
- **200+ validation rules** across parse-time, cross-reference, safety/security, behavior, and composition: cross-reference resolution, integrity level consistency, diagram annotation, documentation completeness

## Repository Structure

```
crates/
  syscribe/           # CLI validator and query tool
  syscribe-model/     # core library: parser, walker, graph builder, resolver, renderer
  syscribe-server/    # Axum web server + Askama templates + HTMX frontend
model/                # UAV autonomous flight system demo model
model_auto/           # Engine ECU demo model (ISO 26262 / ISO/SAE 21434)
model_sil/            # SIL 4 railway interlocking demo model (IEC 61508 / EN 50128)
prompts/              # LLM authoring prompt (embedded in the CLI binary)
spec/                 # Syscribe format specification
docs/                 # MkDocs documentation source
```

## Running

```bash
cargo build --workspace
```

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

### Browse in a web UI

```bash
syscribe-server -m model_auto/
# open http://localhost:3000        (tree browser)
# open http://localhost:3000/canvas (interactive model canvas)
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

The prompt and the validator are always in sync — `--agent-instructions` is embedded at compile time from `prompts/create-model.md`. See the [LLM Workflow guide](https://sjames.github.io/syscribe/model-guide/llm-workflow/) for the full incremental authoring workflow.

## Prior Work

Syscribe is an evolution of [assemblyline](https://github.com/sjames/assemblyline), an earlier experiment in structured systems modeling that used [Typst](https://typst.app) as the modeling language. Typst worked well for rendering and was extensible, but the language was unnecessarily complex for the authoring use case — requiring toolchain knowledge just to read or write a model element.

Markdown + YAML removes that barrier entirely. No special tools are needed to write or read the content. If you host the model on GitHub, the files render as-is. The ideas from assemblyline are reimplemented here in a format that any editor, any diff viewer, and any LLM can handle natively.

---

If you use Syscribe in a project or find it useful, feel free to tag [@sojan_james](https://twitter.com/sojan_james) on Twitter/X.
