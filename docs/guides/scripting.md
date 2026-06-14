# Appendix D — Scripting and Extensibility

`GUIDE · APPENDIX D — SCRIPTING`

syscribe is designed to be composed. Every analytical command emits structured JSON;
the full model graph is exportable as a streaming data source; test results flow in
through a defined ingestion interface; gate policies are configurable in a project-local
TOML file. The tool is deliberately a Unix-style filter — it reads a model and writes
structured output — so any language that can run a subprocess and parse JSON can extend
or integrate with it.

### D.1 The JSON output surface

Every command that produces a report has a `--json` flag. The output is stable structured
data suitable for consumption by scripts, dashboards, LLM sessions, or external reporting
pipelines. The format never changes without a version bump.

| Command | `--json` output shape | Primary use |
|---|---|---|
| `validate --json` | Array of `{code, severity, file, message}` | Parse findings in CI; route to issue tracker |
| `list <type> --json` | Array of `{qualifiedName, type, name, id, status, silLevel, asilLevel, wcet}` | Inventory; filter; feed to dashboards |
| `matrix --json` | `{columns, rows, coverage: {overall, perConfig}}` | Coverage dashboards; trend tracking |
| `audit --json` | Full readiness rollup with status split, coverage, orphans, verdict | Safety dashboard; go/no-go gate |
| `metrics --json` | Array of `{id, sil, asil, spfm, lfm, pmhf, pass}` | PMHF trend charts; FMEDA evidence |
| `cyber-risk --json` | Array of `{id, severity, feasibility, risk, treatment, addressed, flag}` | Security dashboards; untreated-threat alerting |
| `verification-depth --json` | Array of `{id, silLevel, asilLevel, count, levels[], flag}` | Single-level coverage gap detection |
| `co-analysis --json` | Safety↔security overlap per goal | Co-engineering reporting |
| `safety-case --json` | GSN tree per SafetyGoal with evidence nodes | Safety case document generation |
| `features --json` | Feature model with groupKind, requires/excludes, selected-in counts | Product line dashboards |
| `configure <CONF> --json` | Satisfiability verdict + forced/free/bound features | Build-system variant selection |
| `connectivity --json` | `{root, nodes[], edges[]}` subgraph | Custom graph queries |
| `trade-study --json` | MoE × Configuration score matrix | Architecture decision records |
| `magicgrid --json` | B/W/S × 1-4 cell population | MBSE completeness dashboards |

**Worked example — count gaps per configuration with `jq`**:

```bash
syscribe -m model matrix --json \
  | jq '.rows[] | select(.cells | any(. == "gap")) | .id' \
  | sort | uniq -c | sort -rn
```

**Worked example — list all requirements failing the PMHF target**:

```bash
syscribe -m model metrics --json \
  | jq '.[] | select(.pass == false) | {id, pmhf, sil}'
```

**Worked example — find every untreated high/critical security threat**:

```bash
syscribe -m model cyber-risk --json \
  | jq '.[] | select(.flag == "untreated") | {id, risk, severity}'
```

**Worked example — count approved vs draft per package (for a status report)**:

```bash
syscribe -m model list Requirement --json \
  | jq 'group_by(.qualifiedName | split("::")[0])
        | map({
            package: .[0].qualifiedName | split("::")[0],
            approved: map(select(.status == "approved")) | length,
            draft:    map(select(.status == "draft")) | length
          })'
```

### D.2 Full model graph export

`export` emits the complete parsed model as a single JSON document (or as streaming
NDJSON). Every element appears with its file path, qualified name, type, and full
frontmatter as parsed. This gives a script complete, structured access to the model
without having to parse Markdown files directly.

```bash
# Full graph as JSON (all 405 elements in one document)
syscribe -m model export --json > model-graph.json

# Streaming NDJSON — one JSON object per line; memory-efficient for large models
# First line is a header: {"kind":"header","elementCount":N,"schemaVersion":"1.0"}
syscribe -m model export --ndjson | grep '"type":"Requirement"' | jq '.frontmatter.id'
```

**NDJSON schema** (each element line):

```json
{
  "file": "model/Requirements/Safety/SchedulingIntegrity.md",
  "qname": "Requirements::Safety::SchedulingIntegrity",
  "type": "Requirement",
  "name": "Scheduler shall guarantee priority ordering",
  "frontmatter": { /* complete raw frontmatter as parsed */ }
}
```

**Use cases for `export`**:

- **Custom report generation**: process the full graph in Python/Ruby/Node to produce
  Word documents, Excel workbooks, or DOCX evidence packages for auditors
- **External synchronisation**: feed elements with `extRef:` fields into DOORS, Polarion,
  or Jira to keep external tools in sync with the syscribe model as the source of truth
- **Graph analysis**: load the full element graph into a network-analysis library (NetworkX,
  igraph) to run custom connectivity, centrality, or shortest-path queries beyond what the
  CLI commands expose
- **LLM context injection**: pipe `export --json` for a specific package into an LLM
  session to give it precise, current model state without hallucinating stale content

**Python example — generate a custom safety requirements PDF**:

```python
import subprocess, json, markdown, weasyprint

# Get all approved ASIL-D requirements as JSON
result = subprocess.run(
    ["syscribe", "-m", "model", "list", "Requirement",
     "--status", "approved", "--sil", "D", "--json"],
    capture_output=True, text=True
)
reqs = json.loads(result.stdout)

# Get the full frontmatter for each (via export)
model = json.loads(subprocess.run(
    ["syscribe", "-m", "model", "export", "--json"],
    capture_output=True, text=True
).stdout)
by_qname = {e["qname"]: e for e in model["elements"]}

# Render to HTML then PDF
html = "<h1>ASIL D Requirements</h1>"
for r in reqs:
    elem = by_qname.get(r["qualifiedName"], {})
    html += f"<h2>{r['id']} — {r['name']}</h2>"
    # frontmatter['body'] contains the Markdown prose
html += "</body></html>"
weasyprint.HTML(string=html).write_pdf("asil-d-requirements.pdf")
```

### D.3 Test result ingestion

The `.syscribe/results.json` sidecar enables the matrix to distinguish between a
requirement that is *linked to a test case* (the test case exists in the model) and a
requirement that is *verified and passing* (the linked test case actually passed in the
most recent run).

**Ingesting results in CI**:

```bash
# After running Rust tests with --format json
cargo test -- -Z unstable-options --format json > test-output.json
syscribe -m model ingest-results --format cargo-json test-output.json
# Writes .syscribe/results.json

# Matrix now shows ✓ (linked + passing), ▣ (linked + failing), ✗ (gap)
syscribe -m model matrix

# One-shot: inject for this run without persisting the sidecar
syscribe -m model validate --results test-output.json
```

**JUnit results** (Python pytest, Java, most CI systems):

```bash
pytest --junit-xml=results.xml
syscribe -m model ingest-results --format junit results.xml
```

**Why this matters**: a traceability matrix that shows "covered" based only on model
links is a claim. A matrix that shows "covered + passing" based on actual test results
is evidence. The ingestion step converts the model from a static design document into a
live verification record that updates on every CI run.

### D.4 Project configuration with `.syscribe.toml`

`.syscribe.toml` in the model root (or any parent directory — auto-discovered) controls
gate policies, remote source hooks, and MagicGrid enablement. It is the primary
extension point for project-specific behaviour.

```toml
# .syscribe.toml

# Named gate profiles — apply with: syscribe validate --profile <name>
[profiles.sil4]
promote = ["W015", "W031", "W033", "W300"]  # these warnings become hard failures

[profiles.aspice-swe2]
promote = ["W300", "W015"]    # unallocated requirements + coverage gaps

[profiles.security]
promote = ["W031", "W032", "W810"]  # untreated threats, CAL level, unreferenced assets

[profiles.magicgrid]
magicgrid = true
promote = ["W307"]            # every use case must refine a stakeholder need

# Remote source file hook — runs when: syscribe validate --fetch-remote
# Useful when test source files live in a separate repository or artifact store
[remote]
command = ["./scripts/fetch-test-sources.sh"]
# Script is responsible for populating the paths referenced by TestCase.sourceFile
```

**Gate composition in CI**: run multiple profiles in sequence, each with a different
scope:

```bash
# In your CI pipeline:
syscribe -m model validate --profile sil4          # safety gate
syscribe -m model validate --profile security       # security gate
syscribe -m model validate --profile aspice-swe2   # ASPICE process gate
syscribe -m model validate --all-configs            # product-line completeness gate
```

Each gate is independent and produces a distinct exit code and JSON artefact. You can
parallelise them and aggregate results in your CI system.

### D.5 Diagram and graph generation

`connectivity` can emit Graphviz DOT format for any element in the model, enabling
automated diagram generation as part of the documentation pipeline:

```bash
# Generate a data-flow diagram centred on the Scheduler, 2 hops deep
syscribe -m model connectivity Architecture::Logical::Scheduler \
  --format dot --depth 2 \
  > docs/diagrams/scheduler-connectivity.dot

# Render to SVG (requires graphviz installed)
dot -Tsvg docs/diagrams/scheduler-connectivity.dot \
  -o docs/diagrams/scheduler-connectivity.svg
```

```bash
# Full fault tree as Mermaid (built-in command)
syscribe -m model fault-tree render FT-KERNEL-001
# Paste output into any Markdown file — renders in GitHub, GitLab, Notion

# Allocation matrix as a table
syscribe -m model matrix --allocations
```

**Automated diagram update in CI**: add a step that regenerates diagrams from the model
and commits the SVG files. Diagrams are always consistent with the model because they
are derived from it, not hand-drawn alongside it.

```bash
# In a docs-update CI job:
syscribe -m model connectivity Architecture::Logical::KernelSoftware \
  --format dot --depth 3 | dot -Tsvg > docs/diagrams/architecture-l2.svg

syscribe -m model fault-tree render FT-KERNEL-001 > docs/diagrams/ft-kernel-001.mmd
syscribe -m model fault-tree render FT-KERNEL-002 > docs/diagrams/ft-kernel-002.mmd

git add docs/diagrams/ && git commit -m "docs: regenerate diagrams from model"
```

### D.6 Code generation and scaffolding helpers

syscribe provides several commands designed for use in code-generation and model-authoring
pipelines — whether driven by a human writing a script, or an LLM writing model elements.

**Generate a frontmatter skeleton**:

```bash
# Print a ready-to-fill skeleton for any element type
syscribe -m model template Requirement
syscribe -m model template FaultTreeEvent
syscribe -m model template Asset
```

Output is a complete YAML frontmatter block with every field, its type annotation, and
a placeholder value. Use this as the basis for a code generator that creates model
elements programmatically.

**Generate and align Gherkin blocks**:

```bash
# Given a TestCase with testFunctions: declared but no Gherkin body yet:
syscribe -m model scaffold-gherkin TC-SCHED-001

# --fix rewrites the file in place, adding Gherkin stubs for each testFunction
syscribe -m model scaffold-gherkin TC-SCHED-001 --fix
```

**Allocate stable IDs without conflicts**:

```bash
# Get the next available ID for a prefix — safe for concurrent use
syscribe -m model next-id REQ-SCHED
# → REQ-SCHED-007

# In a shell script that creates a batch of elements:
for feature in "Priority ordering" "Preemption" "Round-robin"; do
  id=$(syscribe -m model next-id REQ-SCHED)
  # create the file using $id and the feature name
done
```

**Resolve a file path from a QName**:

```bash
# Where is this element on disk? (useful in scripts that patch frontmatter)
syscribe -m model path-for REQ-SCHED-001
# → model/Requirements/Safety/SchedulingIntegrity.md

syscribe -m model path-for Architecture::Logical::Scheduler
# → model/Architecture/Logical/Scheduler.md
```

**Practical pattern — batch-create requirements from a CSV**:

```bash
#!/usr/bin/env bash
# Create requirements from a spreadsheet export

while IFS=, read -r name domain asil; do
  id=$(syscribe -m model next-id REQ-SCHED)
  file="model/Requirements/Safety/${id}.md"
  cat > "$file" <<EOF
---
type: Requirement
id: ${id}
name: "${name}"
status: draft
reqDomain: ${domain}
asilLevel: ${asil}
verificationMethod: test
---

\${name} **shall** ...
EOF
done < requirements-import.csv

# Validate immediately — catch any structural issues before review
syscribe -m model validate
```

### D.7 LLM integration

syscribe has a first-class integration point for LLM authoring sessions: the
`--agent-instructions` command emits a self-contained authoring prompt that teaches an
LLM the complete syscribe model format and the conventions for a specific domain.

```bash
# Base authoring context — format spec, conventions, cross-reference rules
syscribe --agent-instructions

# MagicGrid-specific authoring context
syscribe --agent-instructions magicgrid
```

This prompt is designed to be injected at the start of an LLM session working with a
syscribe model. It covers:
- The element type inventory and when to use each type
- Frontmatter field reference with types and validation rules
- Directory and namespace conventions
- Cross-reference syntax and resolution rules
- Common errors and how to avoid them

**Practical workflow for LLM-driven model authoring**:

1. Start an LLM session with `syscribe --agent-instructions` as the system context
2. Feed the LLM the current model state for the area being worked on:
   ```bash
   syscribe -m model export --ndjson | grep '"type":"SafetyGoal"\|"type":"HazardousEvent"'
   ```
3. Give the LLM a task: "Author FMEA entries for FT-KERNEL-002, connecting each basic
   event to its detection mechanism and to the TCB data structure."
4. The LLM produces Markdown files with YAML frontmatter
5. Run `syscribe -m model validate` — any structural errors are caught immediately
6. The LLM sees the error output and corrects the elements
7. When the gate is green, commit the files for human review

The LLM–validator feedback loop typically converges in 1–3 iterations. The human review
step focuses on the engineering substance (are the failure rates reasonable? is the DC
claim defensible?) rather than structural correctness (the validator already handled that).

### D.8 External tool integration via `extRef`

`extRef:` on any element creates a pointer to a record in an external system. The
`extref` command resolves these pointers back to model elements, enabling bidirectional
navigation between syscribe and external trackers.

```yaml
# On a Requirement that originated in DOORS:
extRef: "DOORS://Safety_Requirements#REQ-UAV-SAFE-001"

# On a TestCase linked to a Jira ticket:
extRef: "JIRA://SAFE-1234"
```

```bash
# Find the syscribe element for a DOORS reference (during migration)
syscribe -m model extref "DOORS://Safety_Requirements#REQ-UAV-SAFE-001"

# Find all elements with Jira references (for a sync script)
syscribe -m model export --json \
  | jq '.elements[] | select(.frontmatter.extRef | type == "string"
        and startswith("JIRA://")) | {qname, extRef: .frontmatter.extRef}'
```

**Bidirectional sync pattern**: syscribe is the source of truth; DOORS/Polarion holds
a read-only copy for stakeholders who cannot use git. A nightly CI job exports the syscribe
model, diffs it against the DOORS snapshot, and posts changes to the external tool:

```bash
# Export current model state for DOORS-linked requirements
syscribe -m model export --json \
  | jq '.elements[] | select(.frontmatter.extRef | strings | startswith("DOORS://"))
        | {id: .frontmatter.id, name: .frontmatter.name,
           status: .frontmatter.status, extRef: .frontmatter.extRef}' \
  > doors-sync-payload.json

# Feed to a DOORS REST API client, DNG client, or OSLC adapter
python3 scripts/sync-to-doors.py doors-sync-payload.json
```

**Migration from DOORS**: run the sync in reverse during a migration — read from DOORS,
create syscribe `.md` files, then use `validate` to catch any structural issues in the
imported content before switching over entirely.

### D.9 Summary: the extensibility model

syscribe's extensibility rests on four surfaces that compose cleanly:

| Surface | Interface | Use for |
|---|---|---|
| **JSON output** | `--json` on every command | Scripts, dashboards, LLM context, CI parsing |
| **Model graph** | `export --json` / `--ndjson` | Custom reports, external sync, graph analysis |
| **Result ingestion** | `ingest-results` / `--results` | Live coverage tracking from CI test runs |
| **Gate policy** | `.syscribe.toml` profiles + `--profile` | Project-specific CI gates, ASPICE/SIL/ASIL profiles |
| **Scaffolding** | `template`, `scaffold-gherkin`, `next-id`, `path-for` | Code generation, LLM authoring pipelines |
| **LLM context** | `--agent-instructions [topic]` | LLM system prompt for model authoring sessions |
| **External refs** | `extRef:` field + `extref` command | DOORS/Polarion/Jira bridges |
| **Graph rendering** | `connectivity --format dot`, `fault-tree render` | Automated diagram generation |

The design principle is that syscribe remains a focused validator and analysis engine —
it does not try to be a report generator, a dashboard, or an external tool adapter.
Those responsibilities belong to the scripts and pipelines that consume its JSON output.
This keeps the tool simple, fast, and predictable, and means any language or framework
can integrate with it without waiting for a plugin system.

---
