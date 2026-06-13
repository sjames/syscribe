# CLI Reference

The `syscribe` binary is a command-line tool for validating, browsing, and querying Syscribe models. All examples on this page use the **Engine ECU** demo model (`model_auto/`).

## Pointing at a model

Every command needs to know where the model root is. Four ways, in priority order:

```bash
# 1. Flag (short or long)
syscribe -m model_auto/ validate
syscribe --model model_auto/ validate

# 2. Environment variable — useful in scripts and CI
export SYSCRIBE_MODEL=model_auto/
syscribe validate

# 3. Auto-discovery — with no flag/env, walk up from the current directory to the
#    nearest ancestor containing a .syscribe.toml, and use that as the model root.
#    Run any command from anywhere inside the model tree:
cd model_auto/System/Software && syscribe validate

# 4. Default — if none of the above, looks for model/ in the current directory
syscribe validate
```

**`.syscribe.toml` as the root marker.** Auto-discovery reuses the existing config file ([repo_root / matchers / remote hook](../format/spec.md)); an **empty** `.syscribe.toml` at the model root is a valid "mark this as the root" file. The marker is a locator only — it never changes qualified-name resolution. For CI, prefer the explicit `-m` for reproducibility.

---

## Getting help

Every command has a detailed man page (synopsis, options, examples, exit codes, see-also), reachable two ways — neither needs a model directory:

```bash
syscribe help              # index of every command with a one-line summary
syscribe help <command>    # the command's full page, e.g. `syscribe help audit`
syscribe <command> --help  # the same page, e.g. `syscribe validate --help` (also -h)
```

`syscribe spec [<section>]` browses the embedded **format** reference (types, fields, validation rules, …); `syscribe --agent-instructions` prints the LLM authoring prompt (`syscribe --agent-instructions magicgrid` prints a dedicated MagicGrid modeling prompt).

`syscribe --version` (also `-V`, or `syscribe version`) prints the tool version as `syscribe <semver>` and exits 0 — no model directory required.

**Command routing.** The top-level command line is parsed by a clap router whose subcommand registry is derived from the man-page list, so an **unknown command is rejected** with a clear error and a **non-zero** exit (`error: unrecognized subcommand '<name>'`), independent of whether a model directory is present. Each command's own flags are passed through to it unchanged.

---

## Validation

### Full report

Running with no subcommand — or the explicit `report` command — prints the full 10-section Markdown validation report: element inventory, requirements matrix, traceability summary, and findings.

```
$ syscribe -m model_auto/

...

Warnings (1):

| Code | File | Message |
|---|---|---|
| W803 | model_auto/Security/VR-ENG-002.md | VulnerabilityReport has status: open — ensure it is being tracked and mitigated |
```

### Findings only

`validate` prints just the errors and warnings table — useful for quick iteration.

```
$ syscribe -m model_auto/ validate

Warnings (1):

| Code | File | Message |
|---|---|---|
| W803 | model_auto/Security/VR-ENG-002.md | VulnerabilityReport has status: open — ensure it is being tracked and mitigated |
```

### Scoped to a single file

Useful when editing one file and you want instant feedback without re-validating the whole model.

```
$ syscribe -m model_auto/ validate --file model_auto/System/Software/SafetyMonitor.md

0 errors, 0 warnings — model is valid.
```

### Machine-readable output

`--json` emits a JSON array of findings — suitable for editor integrations and CI scripts.

```
$ syscribe -m model_auto/ validate --json

[
  {
    "code": "W803",
    "file": "model_auto/Security/VR-ENG-002.md",
    "message": "VulnerabilityReport has status: open — ensure it is being tracked and mitigated",
    ...
  }
]
```

### CI severity gating

`validate` is a CI gate. By default a model with only warnings exits `0`; the gating flags promote chosen warnings to failures:

| Flag | Effect |
|---|---|
| `--deny <CODES>` | Treat each comma-separated warning code as a gate failure. |
| `--max-warnings <N>` | Fail when the warning count exceeds `N`. |
| `--warnings-as-errors` | Treat every warning as a gate failure. |
| `--profile <name>` | Apply a named `[profiles.<name>]` policy from `.syscribe.toml` (see below). |

Exit-code contract: `0` clean · `1` one or more `Error`-severity findings (errors always dominate) · `2` warnings tripped a gate. All four flags compose additively.

### Named severity profiles

A **profile** is a reusable gating policy declared in `<model_root>/.syscribe.toml` and selected with `validate --profile <name>`. It promotes the listed warning codes to gating failures — optionally **scoped** to the integrity level / status / tag of the element each finding concerns.

```toml
# .syscribe.toml
[profiles.safety]
promote = ["W002", "W015", "W300"]   # warning codes promoted to gating failures
# OPTIONAL scope — promotion applies only to findings on an element matching ALL fields:
sil    = "4"          # element's silLevel stringifies to "4" OR asilLevel == "4" (as in `list --sil`)
status = "approved"   # element's status:
tag    = "safety"     # element's tags: contains this

[profiles.strict]
promote = ["W300"]    # no scope → every W300 is promoted
```

```
$ syscribe -m model/ validate --profile safety   # exit 2 if any scoped, promoted finding is present
```

Semantics:

- A finding trips the gate when its `code` is in `promote` **and** (the profile has no scope fields **or** the element whose `file_path` equals the finding's file matches **all** the provided scope fields).
- A finding whose file maps to no element is **not** promoted when any scope field is set. With no scope, all findings of the listed codes are promoted.
- `--profile` composes additively with `--deny` / `--max-warnings` / `--warnings-as-errors`.
- An **undefined** profile name (or a missing `.syscribe.toml`) prints an error to stderr and exits `1`.

Multiple profiles may be defined; the `[matchers]` / `[remote]` / `[ids]` tables and `repo_root` key continue to work alongside `[profiles.*]`.

### Stable-ID suffix width (`[ids]`)

Every stable ID (REQ-*, TC-*, TP-*, ADR-*, …) ends in a numeric suffix of **3 to 8 digits** by default. Raise or lower the maximum (minimum is fixed at 3) with:

```toml
# .syscribe.toml
[ids]
max_digits = 8   # default 8; e.g. 12 to allow wider counters, 4 to tighten
```

A suffix longer than the cap is error `E023`; a suffix shorter than 3 is `E006`. A reference to an over-long ID still resolves (the defect surfaces on the ID-bearing element, not as a dangling reference).

### Hosted source links (`[links]`)

A `[links]` table maps each **file-backed** element to a **hosted source URL** (GitHub/GitLab/a static site), making generated diagrams and reports link back into the hosted model. The feature is **opt-in and advisory** — it never affects validation findings or exit codes, and is **inert** (output byte-for-byte as before) when `[links]` is absent.

```toml
# .syscribe.toml
[links]
base_url = "https://github.com/<org>/<repo>/blob/main/model"   # the 90% case
# optional escape hatch for hosted HTML / custom refs / anchors:
url_template = "https://github.com/<org>/<repo>/blob/{ref}/model/{path}"
ref = "main"
```

- At least one of `base_url` / `url_template` enables the feature.
- For an element whose path **relative to the model root** is `<path>` (forward-slashed): with `url_template`, the placeholders `{path}` / `{qname}` / `{id}` / `{ref}` are substituted; otherwise the URL is `base_url` (one trailing `/` trimmed) + `/` + `<path>`. Path **segments** are percent-encoded (a space → `%20`); `/` separators are preserved. A **package** resolves to its `_index.md`; an attribute that is only a YAML key in a parent resolves to no URL.

When configured, the URL appears on each surface:

- **SVG** diagrams wrap each element shape in `<a xlink:href href target="_blank" rel="noopener">`.
- **Mermaid** diagrams append `click <nodeId> "<url>" _blank` directives.
- The **Markdown** validation report and the JSON `export` (`url` field) render element references as hosted links.

### Extension scripts (`[scripts]`)

A `[scripts]` table configures the directory of user-authored **Rhai extension scripts** run by the `scripts` command family. Scripts are **tooling, not model content** — they are never parsed as model elements and are **not** run by the built-in `validate` pass (the qualification boundary stays crisp). An absent scripts directory is **not** an error.

```toml
# .syscribe.toml
[scripts]
path = ".syscribe/scripts"   # default; resolved relative to the model root
```

The directory is also the Rhai **module-import root**, so a script can reuse a shared library (`import "lib/helpers" as h;`). Scripts run in a sandboxed, resource-limited, deterministic engine (no filesystem/network/clock/random/env; `eval` disabled; runaway scripts aborted by an operation budget; the only side effect is stdout/stderr text). A script registers a **command** (`register_command`) or a **check** (`register_check`).

| Command | Description |
|---|---|
| `scripts list [--json]` | Enumerate every registered command/check (name, kind, description, source file). |
| `scripts run <command> [--json]` | Invoke a command and print its returned string. Unknown name → non-zero; a check name reports that it is a check. |
| `scripts validate [--deny <codes>] [--max-warnings <n>] [--warnings-as-errors] [--json]` | Run every check; print findings as `<check>/<code>` with the source script. Exit 0 clean / 1 on an error-severity finding / 2 on a tripped gate. Independent of the built-in `validate`. |

See `syscribe help scripts` for the full read-only model API.

### Label field: `name` is the universal label (E025; E024 retired)

**`name` is the single human-readable label on every element type** — id-identified and name-identified alike. There is no longer a per-identity-class split.

- **Id-identified types** (`Requirement`, `TestCase`, `TestPlan`, `Configuration`, `ADR`, and the safety/security types — identity is a stable `id`) label via **`name`** (free prose — spaces/punctuation allowed; `W042` does not apply), which is **required** on them.
- **Name-identified types** (all SysML structural types, `Package`, `Diagram`, `FeatureDef` — identity is the `name`/path) label via **`name`**, which is also the identity segment and must be a basic name (`W042`).
- **`title` is removed.** Declaring `title:` on any element is error **`E025`** ("rename it to `name`"). Error **`E024`** (formerly: `name:` on an id-identified type) is **retired**.

`FeatureDef` also carries a **mandatory** `FEAT-*` `id` (a feature with no `id` is `E201`); it labels via `name`. These errors exit `1`.

---

## Browsing the model

### List root packages

```
$ syscribe -m model_auto/ ls

# (root)

| Name | Qualified Name | Type |
|---|---|---|
| Allocations | Allocations | Package |
| Decisions   | Decisions   | Package |
| Requirements | Requirements | Package |
| Safety      | Safety      | Package |
| Security    | Security    | Package |
| System      | System      | Package |
| Vehicle     | Vehicle     | Package |
| Verification | Verification | Package |
```

### List a namespace

```
$ syscribe -m model_auto/ ls System::Software

# System::Software

| Name | Qualified Name | Type |
|---|---|---|
| BootSequence           | System::Software::BootSequence           | ActionDef |
| CANSecurityModule      | System::Software::CANSecurityModule      | PartDef   |
| DiagnosticSecurityLayer | System::Software::DiagnosticSecurityLayer | PartDef  |
| EngineStallMonitor     | System::Software::EngineStallMonitor     | PartDef   |
| FuelControl            | System::Software::FuelControl            | PartDef   |
| SafetyMonitor          | System::Software::SafetyMonitor          | PartDef   |
| SecureBootManager      | System::Software::SecureBootManager      | PartDef   |
| ThrottleControl        | System::Software::ThrottleControl        | PartDef   |
```

### Recursive tree

```
$ syscribe -m model_auto/ tree System::Software

System::Software
├── BootSequence [ActionDef]
├── CANSecurityModule [PartDef]
├── DiagnosticSecurityLayer [PartDef]
├── EngineStallMonitor [PartDef]
├── FuelControl [PartDef]
├── SafetyMonitor [PartDef]
├── SecureBootManager [PartDef]
└── ThrottleControl [PartDef]
    └── canOut [Port]
```

### Show an element

`show` prints an element's fields, features, and documentation body.

```
$ syscribe -m model_auto/ show System::Software::SafetyMonitor

# System::Software::SafetyMonitor

| Field | Value |
|---|---|
| type    | PartDef  |
| file    | model_auto/System/Software/SafetyMonitor.md |
| domain  | software |
| ASIL    | D        |
| satisfies | REQ-ENG-SAFE-001, REQ-ENG-SAFE-004, REQ-ENG-SAFE-005 |

## Features

| Name | Type | Multiplicity |
|---|---|---|
| monitorCycleMs           | ScalarValues::Integer | 1 |
| tpsDivergenceThresholdPct | ScalarValues::Real   | 1 |

## Documentation

Safety monitoring software component (ASIL D). Supervises all safety-relevant
inputs and function outputs...
```

---

## Safety ↔ security co-analysis

`co-analysis` is the cross-domain view that links the functional-safety layer (`HazardousEvent`, `SafetyGoal`) to the cybersecurity layer (`DamageScenario`, `ThreatScenario`). It answers: *which cyber threats can violate each safety goal/hazard, and which safety-relevant damage scenarios are not yet linked?*

```bash
$ syscribe -m model/ co-analysis            # readable grouped report
$ syscribe -m model/ co-analysis --json     # structured document
```

It builds the chain `ThreatScenario --damageScenarios--> DamageScenario --hazardRef--> HazardousEvent/SafetyGoal` (plus a `ThreatScenario`'s own direct `hazardRef`). For each safety goal/hazard that is a `hazardRef` target it lists the safety-relevant damage scenarios and the threats that lead to them; a final section lists safety-tagged damage scenarios with no `hazardRef` (the **W030** gaps). `--json` emits `{ goals: [{ id, type, damageScenarios, threats }], unlinkedSafetyDamage: [...] }`. With no relevant content it prints a notice and exits 0.

Add `hazardRef:` (string or list) to a `DamageScenario`/`ThreatScenario` to declare the link. An unresolved or wrong-type `hazardRef` is error **E844**; a safety-tagged damage scenario lacking one warns **W030** (gate with `validate --deny W030`). See the [safety-analysis guide](../model-guide/safety-analysis.md).

---

## Cybersecurity risk determination

`cyber-risk` lists every `ThreatScenario` with its ISO/SAE 21434 §15.8 risk (severity × feasibility), treatment decision and treatment status. It answers the central 21434 question: *which threats are above the risk-acceptance line and how were they treated?*

```bash
$ syscribe -m model/ cyber-risk            # Markdown table
$ syscribe -m model/ cyber-risk --json     # JSON array
```

Severity = the max `damageSeverity` over the threat's `damageScenarios` (`negligible`=0 … `severe`=3); feasibility from `attackFeasibility` (`very_low`=0 … `high`=3); `score = severity + feasibility` → **low** (0–1), **medium** (2–3), **high** (4), **critical** (5–6), or **unknown** if either input is missing. Each row shows `severity`, `feasibility`, computed `risk`, `riskTreatment` (or `—`), whether a `CybersecurityGoal` addresses it, and a `flag` (`untreated` when it trips W031, `unknown`, or `ok`). `--json` emits an array of `{id, severity, feasibility, risk, treatment, addressed, flag}`. With no `ThreatScenario`s it prints a notice and exits 0.

Set `riskTreatment:` (`avoid`/`reduce`/`share`/`retain`; invalid → **E845**) and an optional free-text `residualRisk:` on the threat. A high/critical-risk threat with no treatment and no addressing `CybersecurityGoal` warns **W031** (gate with `validate --deny W031`); a `CybersecurityGoal` whose `calLevel` is below the expected CAL for its threats' max risk warns **W032**. See the [safety-analysis guide](../model-guide/safety-analysis.md).

---

## Quantitative HW safety metrics

`metrics` rolls up the ISO 26262-5 §8–9 hardware architectural metrics — SPFM, LFM, and PMHF — per `SafetyGoal`, from the `failureRate` and `diagnosticCoverage` of the `FaultTreeEvent`s under the `FaultTree`(s) whose `topEvent` resolves to that goal.

```bash
$ syscribe -m model/ metrics            # Markdown table: per-goal SPFM / LFM / PMHF + verdict
$ syscribe -m model/ metrics --json     # JSON array {id, asil, sil, spfm, lfm, pmhf, pass}
```

> **First-order FMEDA approximation** from user-supplied λ and diagnostic coverage — verify independently before use in a safety case.

`SPFM = 1 − λ_RF/Σλ` with `λ_RF = Σ λ_i·(1−DC_i)`; `LFM = 1 − λ_MPFL/(Σλ−λ_RF)` (only when an event sets `latentDiagnosticCoverage`); `PMHF = λ_RF + λ_MPFL` (/h). Targets by ASIL — SPFM ≥ {B 0.90, C 0.97, D 0.99}, LFM ≥ {B 0.60, C 0.80, D 0.90}, PMHF < {B/C 1e-7, D 1e-8}/h; SIL gates PMHF/PFH only. **Opt-in:** metrics are computed and gated only for goals whose contributing events declare `diagnosticCoverage` (others show `n/a`). A goal that misses its target raises **W033** (gate with `validate --deny W033`). A `diagnosticCoverage`/`latentDiagnosticCoverage` outside `0.0`–`1.0` is **E846**. See the [safety-analysis guide](../model-guide/safety-analysis.md).

---

## Connectivity

`connectivity` exports the **connected slice of the model reachable from a chosen element** — the elements plus the connections between them — as a focused subgraph. It walks outward from the root over the part-to-part wiring and structure edges, then renders the reachable nodes and edges as a text tree, a JSON document, or styled Graphviz DOT. Running it on the **model-root element dumps the whole model**.

```
syscribe -m <root> connectivity <element> [--depth N] [--format text|dot|json] [--kinds <csv>] [--undirected]
```

- **`<element>`** — the root of the walk (qualified name or stable id). An unknown element prints to stderr and exits non-zero.
- **`--format text|dot|json`** (default `text`):
  - **text** — an indented tree (`├──`/`└──`/`│  ` connectors) rooted at the element. Each node line is `<qualifiedName> [<Type>]`; each child line carries the traversed edge kind, e.g. `└── [connection] PortDemo::Motor [PartDef]`. A node already expanded elsewhere is shown but not re-expanded — marked ` (*)` — so cyclic models terminate.
  - **json** — `{ "root": "<qname>", "nodes": [{"qualifiedName","type","id"}], "edges": [{"from","to","kind"}] }`.
  - **dot** — Graphviz DOT styled so element families read by shape, definitions get a double border, and the wiring stands out (see the styling legend below). Pipe to Graphviz: `syscribe -m model/ connectivity UAV::Airframe --format dot | dot -Tsvg -o airframe.svg`.
- **`--depth N`** — bound the walk to N hops (default: unbounded). `--depth 0` yields only the root; `--depth 1` adds its direct neighbours.
- **`--kinds <csv>`** — override which edge kinds to follow (case-insensitive, comma-separated). Recognised: `connection,flow,binding,succession,featureTyped,contains,typedBy,supertype,subsets,redefines,satisfies,verifies,derivedFrom,allocatedFrom,allocatedTo,conditionalOn`. The default set is `connection,flow,binding,succession,featureTyped,contains,typedBy` — the wiring plus structure, so the model-root element reaches the whole containment tree and each part reaches its sub-part types.
- **`--undirected`** — follow edges in both directions (default: outbound only, following the wiring direction).

The wiring edges (`connection`/`flow`/`binding`/`succession`) come from the `connections:`/`flowConnections:`/`bindingConnections:`/`successionConnections:` frontmatter on `Part`/`PartDef`/`Action`/`ActionDef`; each endpoint feature chain is resolved to its owning element (a feature's `typedBy:` type, else the chain as a qualified name/id). The `featureTyped` edge links a part to the type of each inline `features:` entry, so a structural walk reaches a part's sub-part types.

```
$ syscribe -m model/ connectivity UAV::Airframe
UAV::Airframe [PartDef]
├── [featureTyped] UAV::Power::PowerSystem [PartDef]
│   └── ...
└── [featureTyped] UAV::Propulsion::PropulsionSystem [PartDef]
    └── [connection] ... (*)
```

> Deferred (MVP): rich edge labels — the `via` ConnectionDef/InterfaceDef and `fromEnd`/`toEnd` feature chains — are out of scope; edges carry their `kind` only.

### DOT styling legend

The DOT output is driven by a single source-of-truth style function so all renderers stay consistent. Three orthogonal encodings keep it scannable (and colorblind-safe — shape always disambiguates, colour never stands alone):

1. **Shape** encodes the element *family*: `box` (structure), `circle` (ports), `hexagon` (connections/interfaces), `cds` (flow), rounded `box` (behaviour), `note` (requirements/verification/ADR), `diamond`/`box3d` (variability), `folder` (packaging), `doubleoctagon`/`octagon` (safety/security), `tab` (views), `parallelogram` (allocation).
2. **Definition vs usage** — a definition (`*Def`) gets `peripheries=2` (double border); the matching usage shares the shape with a single border.
3. **Colour** encodes the domain/concern as a **pale fill + saturated border** (blue structure, cyan connections, green behaviour, purple variability, gray packaging, red/amber safety, violet security).

Edges are styled per kind: containment is dashed grey, typing/supertype is solid black, the **wiring stands out at `penwidth=2`**, and traceability (`verifies`/`satisfies`/`derivedFrom`) is coloured-dashed. A `Legend` subgraph at the bottom of the DOT documents the family→shape mapping.

---

## N² interface matrix (`n2`)

```bash
syscribe -m model/ n2 [<qname>] [--depth <N>] [--format text|html|json] [--interfaces-only] [--allocations]
```

Generates an N² (N-squared) interface matrix (§16): the in-scope `PartDef`/`Part` elements sit on the diagonal, and the off-diagonal cell **(row R, col C)** lists the interfaces directed from R to C — each a named interface from the connecting `connections:`/`flowConnections:` entry's `typedBy:` (ConnectionDef/InterfaceDef) or its `name`. Derived read-only from the wiring you already authored.

- No `<qname>` → every part in the model is on the axis; a `<qname>` → the composite's subpart types, expanded `--depth` levels (default 1).
- `--format` `text` (ASCII grid) / `html` (self-contained `<table>`) / `json` (`{ scope, elements, matrix: { R: { C: [{kind, name}] } } }`).
- `--allocations` adds `allocatedTo:` edges; `--interfaces-only` drops elements with no interfaces.

```text
N² Interface Matrix — UAV::Airframe (depth 1)

                    AvionicsBay  PayloadBay  PowerSystem  PropulsionSystem
AvionicsBay         ■            —           —            —
PayloadBay          —            ■           —            —
PowerSystem         PowerConn…   PowerConn…  ■            PowerConn…
PropulsionSystem    —            —           —            ■
```

## Change impact analysis (`impact`)

```bash
syscribe -m model/ impact <qname|id> [--direction downstream|upstream|both] [--depth N] [--format text|json|dot] [--kinds <csv>]
```

Traverses the traceability graph (§17) from an element and reports every reachable node, its hop distance, and the connecting edge kind — "if I change this, what else may need to change?". **Downstream** follows reverse links (`derivedChildren`, `verifiedBy`, `satisfiedBy`, `specializedBy`, `refinedBy`, `conditionalOn`, `allocatedFrom`, `safetyGoalChildren`); **upstream** follows forward links (`derivedFrom`, `verifies`, `satisfies`, `supertype`, `refines`, `allocatedTo`, `derivedFromSafetyGoal`). `--depth` limits hops, `--kinds` restricts edge kinds; `text` / `json` / `dot` output. Cycle-safe.

## Review records (`reviews`)

```bash
syscribe -m model/ reviews [<qname>] [--open-only] [--json]
syscribe -m model/ reviews --coverage [--json]
syscribe -m model/ review <RR-id> [--json]
```

Lists `ReviewRecord` elements (§19) and their coverage. A `ReviewRecord` is a baselined, thin traceability anchor for a formal review; the discussion lives in the tool named by `recordedAt:`. `reviews <qname>` filters to reviews covering that element; `--open-only` to reviews with open action items; `reviews --coverage` prints the native-Requirement review-coverage cross-table; `review <RR-id>` shows one record in full.

## Trade studies (`trade-study`)

```bash
syscribe -m model/ trade-study [<TRD-id>] [--json]
```

When the model contains `TradeStudy` elements (§15), lists them or prints one study's full **normalised, weighted, ranked** scoring table (computed, never written). Without `TradeStudy` elements, the command falls back to the MagicGrid MoE-weighted trade study (`--profile magicgrid`, REQ-TRS-MG-007). `template TradeStudy` prints a skeleton.

---

## Searching

### Fuzzy search

`find` searches element names, IDs, and documentation bodies. Results are ranked by relevance.

```
$ syscribe -m model_auto/ find throttle

# Search: `throttle`

| Score | Qualified Name | Type | Excerpt |
|---|---|---|---|
| 65 | System::Actuators::ThrottleActuator | PartDef | DC motor-driven electronic throttle body... |
| 65 | System::Sensors::ThrottlePositionSensor | PartDef | Dual-track potentiometric throttle position sensor... |
| 65 | System::Software::ThrottleControl | PartDef | Electronic throttle control (ETC) software component... |
| 50 | Requirements::Safety::REQ-ENG-SAFE-005 | Requirement | Throttle close command shall be verified by position feedback... |
| 50 | Safety::FMEA::FMEA-ENG-001::FM-ENG-001 | FMEAEntry | Throttle plate stuck open at >20 % position |
| ...  | | | |
```

### Look up by external reference

`extref <ref>` finds elements that represent an external artifact via the optional [`extRef`](../format/spec.md) common field — a requirement in DOORS Next, an element in a SysML tool, a ticket. Matching is exact on the whole reference value; all matches are returned (a duplicate `extRef` warns `W028`). It exits non-zero when nothing matches.

```
$ syscribe -m model/ extref "DNG:4521"

# extRef: `DNG:4521`

| Qualified Name | Type | id |
|---|---|---|
| System::Powertrain::Engine | PartDef |  |

1 match(es)

$ syscribe -m model/ extref "DNG:4521" --json   # machine-readable array
```

### List elements by type

```
$ syscribe -m model_auto/ list Requirement

# Requirement elements (14)

| Qualified Name | Name / ID |
|---|---|
| Requirements::Safety::REQ-ENG-SAFE-001 | Safety monitor shall detect all safety faults within 100 ms |
| Requirements::Safety::REQ-ENG-SAFE-002 | Hardware watchdog shall reset ECU within 30 ms of software failure |
| ...
```

Scope to a namespace with an optional second argument:

```
$ syscribe -m model_auto/ list PartDef System::Software

# PartDef elements in `System::Software` (7)

| Qualified Name | Name / ID |
|---|---|
| System::Software::CANSecurityModule | CAN Security Module |
| System::Software::SafetyMonitor     | Safety Monitor      |
| ...
```

Filter by free-text `tags:` with `--tag` (repeatable; orthogonal to the feature model):

```
$ syscribe -m model/ list Requirement --tag smoke
```

Filter by lifecycle status and safety integrity level, and emit machine-readable JSON:

```
$ syscribe -m model/ list Requirement --status approved   # keep only status: approved
$ syscribe -m model/ list Requirement --sil 4             # silLevel: 4 (integer, stringified)
$ syscribe -m model/ list Requirement --sil D             # asilLevel: D — one flag covers SIL and ASIL
$ syscribe -m model/ list Requirement --has-wcet          # only requirements with a wcet: timing claim
$ syscribe -m model/ list Requirement --status draft --json
```

- **`--status <s>`** keeps only elements whose `status:` equals `<s>` exactly.
- **`--sil <v>`** keeps only elements whose `silLevel:` (integer) stringifies to `<v>` **or** whose `asilLevel:` equals `<v>`. A single flag covers both IEC 61508 SIL and ISO 26262 ASIL.
- **`--has-wcet`** keeps only elements that declare a non-empty `wcet:` timing claim — pair it with the `W029` check ("WCET claimed but not measured", see [validation rules](../validation/rules.md)) to audit which timing claims are backed by an `L5`/timing-tagged test.
- **`--json`** emits a JSON array of the (filtered) elements — each object carries `qualifiedName`, `type`, `name`, `id`, `status`, `silLevel`, `asilLevel`, `wcet` (absent fields are `null`). All filters above apply to the JSON output too, and compose (AND) with `--tag`, `--feature` and the `--config` lens.

---

## Variability (product lines)

The variability dimension is **opt-in**: it stays dormant unless the model declares a `FeatureDef` and links something to it. See the [validation rules](../validation/rules.md) (E209, W015) and §9 of the format spec.

`appliesWhen:` conditions any element (including a `TestCase`) on a boolean expression over `FeatureDef` qualified names — `and` / `or` / `not` / parentheses, with a bare QName or a list (AND) also accepted:

```yaml
appliesWhen: "Features::CortexM and Features::Mpu"
appliesWhen: "not Features::Smp"
```

A `TestCase` *runs in* a `Configuration` iff its `appliesWhen:` is satisfied by that configuration's `features:` selections (no `appliesWhen:` ⇒ runs everywhere). There is no `runsIn` field.

### Coverage matrix

`matrix` emits a Requirement × Configuration grid. Columns are the model's `Configuration` elements; cells are covered (`✓`), gap (`✗`), or N/A (`—`, requirement not active in that variant):

```
$ syscribe -m model/ matrix
$ syscribe -m model/ matrix --json            # structured grid (schemaVersion, columns, rows, coverage)
$ syscribe -m model/ matrix --tag safety      # filter rows by tag
$ syscribe -m model/ matrix --status approved # restrict rows to requirements whose status: equals approved
$ syscribe -m model/ matrix --gaps-only       # keep only rows with at least one gap cell
$ syscribe -m model/ matrix --linked-only     # ignore ingested results: covered cells stay ✓ (today's linked-only view)
$ syscribe -m model/ matrix --features        # Feature × Configuration grid (which feature ships in which product)
```

- **`--status <s>`** restricts the requirement ROWS to those whose `status:` equals `<s>` (text and `--json`).
- **`--gaps-only`** drops rows that are fully covered or all-N/A, keeping only rows with at least one `gap` cell (text and `--json`).
- **Executed evidence (W010 results).** When a results sidecar (`<model_root>/.syscribe/results.json`, produced by [`ingest-results`](#validation)) is present, `matrix` reflects *executed-and-passed* evidence by default. Each covering TestCase is given an aggregate verdict over its `testFunctions[].function` (`Pass` if all passed, `Fail` if any failed, `Unknown` otherwise / when it declares no functions). A covered cell then becomes `✓` when at least one covering active TestCase that runs in that configuration passed, or `▣` (**covered, not passing**) when a linked TestCase runs there but none passed. The legend gains `▣ covered, not passing` and `--json` cells report `"passing"` vs `"covered"` (in addition to `"gap"`/`"na"`). With **no sidecar**, or under **`--linked-only`**, covered cells stay `✓` and the `--json` cell value stays `"covered"` exactly as before. The coverage-% footer always counts *linked* coverage (covered = linked), unchanged.
- Every run prints a **coverage footer**: per-configuration and overall `covered / applicable`, where `applicable = covered + gap` (N/A excluded) and the percentage is `covered*100/applicable` rounded to one decimal (`n/a` when nothing is applicable). Under `--json`, the same numbers appear in a `coverage` object: `{ "perConfig": { "<cfgId>": {"covered":N,"applicable":M,"pct":P}, ... }, "overall": {"covered":N,"applicable":M,"pct":P} }` (`pct` is `null` when `applicable == 0`). Coverage is plain/unweighted — SIL-weighted coverage is a planned follow-up.

With no feature model present, `matrix` prints a notice and falls back to a flat requirement/testcase view (exit 0); `--status`, `--gaps-only` and the coverage footer still apply. `matrix --features` swaps the rows for `FeatureDef`s and the cells for selected (`✓`) / not-selected — the product map complementing the Requirement × Configuration view.

`refs <CONF-id>` additionally lists the `TestCase`s that run in a given configuration.

### Feature-model check

`feature-check` runs holistic feature-model validation that is deliberately kept out of the per-element `validate` pass — `requires`/`excludes` resolution and satisfaction, dead/always-on optional features, `derivedFrom:` cycles, `bindTo:` propagation ranges, `parameterConstraints` path resolution **and numeric evaluation** (`E221`/`W025` — a constraint violated by a configuration whose `appliesWhen:` holds), and orphan features (`W024`) (see the [validation rules](../validation/rules.md)):

```
$ syscribe -m model/ feature-check
$ syscribe -m model/ feature-check --json
```

Exit code is `0` when there are no errors and `1` otherwise; with no `FeatureDef` present it prints a notice and exits `0`. `feature-check` also flags **orphan features** (`W024` — a `FeatureDef` referenced by no `appliesWhen:` and selected by no `Configuration`, so it gates nothing and ships in nothing); gate it with `feature-check --deny W024`.

Add `--deep` for SAT-backed whole-configuration-space analysis (over a propositional encoding of the feature model — deterministic; uses batsat, a pure-Rust CDCL solver, in-process):

```
$ syscribe -m model/ feature-check --deep
$ syscribe -m model/ feature-check --deep --json
```

`--deep` detects **void** models (`E223`), **dead** features (`E224`), **false-optional** features (`W018`), and **invalid configurations** under full group/cardinality semantics (`E225`), reports **core** features, explains each unsatisfiability with a minimal conflict set, and proposes **diagnoses** (minimal correction sets — how to fix a void model). It comfortably handles ~500 features, covers the Boolean feature layer only (parameter satisfiability is out of scope), and skips with a notice above a feature-count guard (1000).

Related solver-backed capabilities:

```
$ syscribe -m model/ feature-check --count          # number of valid configurations the model permits
$ syscribe -m model/ feature-check --enumerate      # list every valid configuration
$ syscribe -m model/ feature-check --deep --prove <dir>   # write DIMACS CNF of each UNSAT finding (re-checkable)
$ syscribe -m model/ configure <Configuration>      # from a partial selection: satisfiable? forced/free features?
```

`configure` treats a `Configuration`'s `features:` as a *partial* selection (set features fixed, absent open) and reports whether it can be completed plus which features are **forced** or still **free** — a feature configurator. (`--prove` emits the externally-checkable DIMACS CNF; a DRAT refutation proof is deferred — batsat does not expose one.)

### Authoring a gate (`applies-when`)

Show, set, replace, or remove an element's `appliesWhen:` gate from the CLI:

```
$ syscribe -m model/ applies-when REQ-UAV-NAV-001                       # read: own + effective gate
$ syscribe -m model/ applies-when REQ-UAV-NAV-001 --json
$ syscribe -m model/ applies-when REQ-UAV-NAV-001 --set "FEAT-QUAD"
$ syscribe -m model/ applies-when REQ-UAV-NAV-001 --set "Features::Propulsion::Quad"
$ syscribe -m model/ applies-when UAV::Payload::Camera --set "FEAT-SURVEY or FEAT-MAPPING"
$ syscribe -m model/ applies-when REQ-UAV-NAV-001 --clear
$ syscribe -m model/ applies-when REQ-UAV-NAV-001 --set "FEAT-QUAD" --dry-run
```

With **no flag** the command is a read-only display: it prints the element's **own** gate and its **effective** condition — the own gate, or, when the element declares none, the gate **inherited** from its nearest ancestor package (transitive package conditioning), naming that package; an element gated nowhere is reported as always applying. This is the static condition; `why-active <element> --config <CONF>` evaluates it for one product.

The element is resolved by qualified name or stable id; each operand of the expression resolves to a `FeatureDef` by its **`FEAT-*` id or its qualified name** (interchangeably). The edit is **refused without writing** if the expression is malformed or has an unresolved operand (`E209`), or the placement is forbidden (`E228` — a `FeatureDef`, a `Configuration`, the model-root package, a package whose subtree contains features, or a path that already declares `appliesWhen:`). Only the `appliesWhen:` key changes; the rest of the file is preserved byte-for-byte.

After a successful `--set`, the **feature model is checked for bad configurations** (`feature-check --deep`: void model `E223`, dead features `E224`, invalid configurations `E225`); the command exits non-zero if any are found, so a gate is never applied on top of an unsound feature model. `--dry-run` previews and runs the same check without writing.

### Feature discoverability

Four commands answer "what features exist, what does each gate, and why is this element in this product?"

```
$ syscribe -m model/ features                              # the whole feature model as a tree
$ syscribe -m model/ features --json
$ syscribe -m model/ feature Features::Payload::Delivery   # one feature's card
$ syscribe -m model/ list PartDef --feature Features::DualFlightController   # elements gated by a feature
$ syscribe -m model/ why-active <element> --config <CONF>  # is this element active in this product, and why?
```

- **`features`** prints the feature tree (indented by namespace), each node showing its `groupKind`, `requires`/`excludes`, parameters, and a *selected in N/M configs* rollup. Dormant (notice, exit 0) when no `FeatureDef` is present.
- **`feature <qname>`** is a single feature's "card": its doc, group, constraints, parameters, the `Configuration`s that select it, and every element it **gates** (whose `appliesWhen:` names it). Unknown/non-feature argument → non-zero.
- **`list <type> --feature <F>`** restricts the listing to elements whose `appliesWhen:` names `F` — orthogonal to `--tag` and `--config`.
- **`why-active <element> --config <C>`** explains a projection: it prints the element's `appliesWhen:`, the config's selections of the referenced features, and a `Verdict:` line (`active` / `inactive` / `always active`). `--config` is required.

### Configuration lens (`--config`)

The repository is a **150% model** of the product line; `--config` projects it onto one variant (the **100% model**) and runs the command over only the active elements. The argument is a stored `Configuration` (id/qname) or an ad-hoc feature set; the lens is inert when the model has no feature model.

```
$ syscribe -m model/ list Requirement --config CONF-MPS2-WDT
$ syscribe -m model/ export --config 'Features::Mps2,Features::Wdt' --json
$ syscribe -m model/ validate --config CONF-MPS2-WDT     # certify THIS variant
$ syscribe -m model/ validate --all-configs              # gate every stored variant (CI)
$ syscribe -m model/ diff --config CONF-MPS2-WDT --config CONF-M0-BASE
```

`validate --config` re-runs the full validation in the lens (coverage, traceability, safety) **and** flags **escaping references** — an active element pointing at one inactive in the variant: structural → `E226` (error), traceability → `W019` (warning). The complementary `feature-check --deep` rules prove this can't happen in *any* valid configuration (`E227`/`W020`), and report dead elements (`W021`) and family-wide coverage gaps (`W022`).

---

## Safety case (GSN argument tree)

`safety-case [<SG-id>] [--json]` renders the goal→argument→evidence tree for each `SafetyGoal` (or one given). It follows the GSN argument layer — `Argument` nodes (`claim`/`strategy`/`solution`) that `supports:` a goal and cite `evidence:` (Requirements, TestCases, sub-Arguments, `AssumptionOfUse`) — and also folds in the implicit `SafetyGoal ← Requirement (derivedFromSafetyGoal) ← TestCase (verifies)` chain, so it works even on models with no explicit `Argument` nodes. TestCase leaves show their ingested verdict when a results sidecar is present.

```
$ syscribe -m model/ safety-case SG-DEMO-001

[SafetyGoal] SG-DEMO-001 — Prevent unintended acceleration
├── [strategy] ARG-DEMO-001 — Argue over independent torque monitoring
│   └── [solution] ARG-DEMO-002 — Torque monitor is verified by test
│       └── [evidence:TestCase] TC-DEMO-001 — … [pass]
└── [AoU] AOU-DEMO-001 — Integrator provides a redundant torque sensor
```

`--json` emits `{ goals: [{ id, title, arguments, requirements, assumptions }] }`. Read-only; exit 0.

---

## Safety-readiness audit

`audit` is a read-only dashboard that aggregates existing data — it **reuses** `validate`, the `matrix` coverage computation and the [named severity profiles](#named-severity-profiles); it does not re-implement validation or coverage. It is the rollup an assessor reaches for first.

```
$ syscribe -m model/ audit
$ syscribe -m model/ audit --json              # the whole rollup as one structured document
$ syscribe -m model/ audit --profile safety    # add a named [profiles.<name>] gate to the verdict
$ syscribe -m model/ audit --config CONF-X     # variant-scoped: project onto one Configuration
$ syscribe -m model/ audit --all-configs       # gate every Configuration's variant (CI)
```

**Variant scoping (`--config` / `--all-configs`).** Certification is scoped to a *variant*. `audit --config <CONF|features>` projects the entire dashboard — verdict, W306, orphans, coverage — onto the elements **active** in that configuration (per `appliesWhen`), exactly like `validate --config`; a requirement gated out of the variant no longer trips the verdict. `audit --all-configs` audits every stored `Configuration` and exits non-zero if any fails. (The same `--config` lens is available on `metrics`, `cyber-risk`, `co-analysis`, `verification-depth`, and `safety-case`.)

The report (mirrored in `--json`) has five sections:

1. **Requirement status split** — counts of native `Requirement`s by `status:` (`draft` / `review` / `approved` / `implemented` / `verified`), **overall** and **per top-level package** (the first `::` segment of the qualified name).
2. **SIL / ASIL distribution** — counts by `silLevel` and by `asilLevel`, plus a `QM/none` bucket for requirements that declare neither.
3. **Per-configuration coverage %** — `covered / applicable` (N/A excluded) per `Configuration` and overall, computed by the same engine as `matrix`. With no feature model, it falls back to the flat requirement→TestCase coverage.
4. **Orphans** — counts and ids of: requirements with no active verifying `TestCase`; requirements that no element `satisfies:`; `TestCase`s whose `verifies:` is empty or resolves to nothing; and requirements with neither `derivedFrom` nor `derivedChildren`.
5. **Readiness verdict** — a single **PASS/FAIL** line that names *why* it failed.

### Verdict policy and exit code

| Exit code | Meaning |
|---|---|
| `0` | **PASS** — no `Error`-severity findings, no `W306`, and (under `--profile`) nothing the profile promotes. |
| `2` | **FAIL** — at least one `Error` finding, **or** at least one `W306` (the unsatisfied-safety-mechanism gate), **or** at least one finding promoted by `--profile <name>`. |
| `1` | The `--profile <name>` is undefined (or no `.syscribe.toml` exists). |

The default policy always fails on errors and on `W306`. Passing `--profile <name>` loads `[profiles.<name>]` from `<model_root>/.syscribe.toml` and additionally fails the audit if any finding that profile promotes is present, using the same promotion semantics as `validate --profile`. The JSON document has the shape `{ statusSplit, integrityDistribution, coverage, orphans, verdict: { pass, reasons } }`.

---

## TestPlans (`testplan`)

A `TestPlan` (`type: TestPlan`, stable `TP-*` id) is a curated, per-product verification artifact: it binds a set of TestCases (the **effective set** = explicit `testCases:` ∪ `selection:` matches) to zero or more `Configuration`s at one `scope`, and is optionally offered as evidence for the goals it `demonstrates:`. The read-only `testplan` command surfaces plans, their resolved membership, their coverage and a rolled-up verdict — it **reuses** the `matrix` coverage computation and the executed-results verdict fold, not a second engine.

```
$ syscribe -m model/ testplan                                 # list every TestPlan
$ syscribe -m model/ testplan --json
$ syscribe -m model/ testplan TP-DELIVERY-INTEGRATION-001     # detail for one plan
$ syscribe -m model/ testplan TP-DELIVERY-INTEGRATION-001 --json
```

- **List** — one row per plan: id, title, scope, bound configurations, effective-TestCase count, coverage %, and verdict.
- **Detail (`testplan TP-X`)** — the resolved member TestCases (each flagged `escaping` when active in **none** of the plan's configs), the **in-scope requirements**, a per-config coverage grid, and the roll-up verdict. An unknown id (or an id that is not a `TestPlan`) exits `1`.

**In-scope requirements.** With `demonstrates:` set, the scope is the **goal-closure**: each demonstrated `Requirement` plus the transitive closure of its `derivedChildren`, and for a demonstrated `SafetyGoal`/`CybersecurityGoal`, the requirements that `derivedFromSafetyGoal:`/`derivedFromSecurityGoal:` it (and their closure). Without `demonstrates:`, the scope is the union of the `verifies:` targets of the effective TestCase set.

**Verdict** ∈ `pass | fail | incomplete | empty`: `empty` when the effective set is empty; `fail` when any member's ingested verdict is `Fail`; `pass` when every member passes; otherwise `incomplete` (no/partial results). Load results with `ingest-results`.

### `--plan TP-X` lens

`matrix`, `verification-depth` and `audit` accept a `--plan TP-X` lens, symmetric to `--config`. On `matrix`/`verification-depth` it restricts the requirement rows to the plan's in-scope requirements and the TestCase universe to the plan's members. On `audit` it scopes the readiness verdict: validation runs over the **full** model (so no reference escaping the plan subset is mistaken for a defect) and only findings on the plan's in-scope elements count toward the verdict. It **composes** with `--config`, is dormant-safe, and exits `1` on an unknown plan id.

```
$ syscribe -m model/ matrix --plan TP-DELIVERY-INTEGRATION-001
$ syscribe -m model/ audit --plan TP-DELIVERY-INTEGRATION-001 --config CONF-X
$ syscribe -m model/ verification-depth --plan TP-DELIVERY-INTEGRATION-001 --sil 4
```

---

## Traceability

### Full trace for a requirement

`trace` shows a requirement's parents, breakdown ADR, safety goal, satisfying architecture elements, and covering test cases in one view.

```
$ syscribe -m model_auto/ trace REQ-ENG-SAFE-001

# Trace: REQ-ENG-SAFE-001

Title:  Safety monitor shall detect all safety faults within 100 ms
Status: approved · domain: software · ASIL: D

## Parents (derivedFrom)
- REQ-ENG-SAFE-000 — Engine ECU shall prevent safety hazards identified in HARA

## Breakdown ADR
- ADR-ENG-SAFE-001 — ASIL D decomposition for engine safety requirement (accepted)

## Safety Goal (derivedFromSafetyGoal)
- SG-ENG-001 — Prevent unintended engine acceleration (ASIL D)

## Satisfied by
- System::Software::SafetyMonitor [PartDef, software]

## Verified by
- TC-ENG-SAFE-002 — HIL — TPS dual-track divergence triggers safe state (L5)
```

When a results sidecar (`<model_root>/.syscribe/results.json`) is present, the **Verified by** listing annotates each TestCase with its ingested verdict — `[pass]`, `[fail]`, or `[unknown]` — aggregated over its `testFunctions[].function` (same rule as the [coverage matrix](#coverage-matrix)). Pass `--linked-only` to suppress the annotations; with no sidecar the listing is unchanged.

```
$ syscribe -m model/ trace REQ-ENG-SAFE-001 --linked-only   # ignore ingested results
```

### What does this component satisfy?

```
$ syscribe -m model_auto/ why System::Software::SafetyMonitor

# Why: System::Software::SafetyMonitor

## Satisfied requirements

| ID | Title | ASIL |
|---|---|---|
| REQ-ENG-SAFE-001 | Safety monitor shall detect all safety faults within 100 ms | D |
| REQ-ENG-SAFE-004 | Rev limiter shall cut fuel and retard ignition above 6500 rpm | A |
| REQ-ENG-SAFE-005 | Throttle close command shall be verified by position feedback | A |

## Verification coverage

| TC ID | Level | Covers |
|---|---|---|
| TC-ENG-SAFE-002 | L5 | REQ-ENG-SAFE-001 |
| TC-ENG-SAFE-005 | L5 | REQ-ENG-SAFE-004 |
| TC-ENG-SAFE-006 | L5 | REQ-ENG-SAFE-005 |
```

### Which test cases cover a requirement?

```
$ syscribe -m model_auto/ who-verifies REQ-ENG-SAFE-001

# Verification: REQ-ENG-SAFE-001

| TC ID | Level | Gherkin Scenarios | Status |
|---|---|---|---|
| TC-ENG-SAFE-002 | L5 | 1 | active |
```

### Verification-depth & independence report

`verification-depth` gives the fleet-wide view that `trace` shows one requirement at a time: for each requirement, the **distinct** `testLevel`s among its active verifying TestCases, a count, and a depth flag — `none` (no active test), `hil-only` (only L5), `single` (one level), or `ok` (≥2 levels). Diversity/independence of verification is a core SIL-4 expectation.

```
$ syscribe -m model_sil/ verification-depth --sil 4

# Verification depth (N requirements)

| Requirement | SIL/ASIL | Levels | Count | Flag |
|---|---|---|---|---|
| REQ-SIL-SW-002 | 4 | L2,L5 | 2 | ok |
| REQ-SIL-HW-003 | 4 | L5    | 1 | hil-only |
```

- **`--sil <v>` / `--status <s>`** filter the rows (same `--sil` semantics as `list`).
- **`--json`** emits an array of `{id, silLevel, asilLevel, levels, count, flag}`.
- **`--min-levels N`** turns it into a CI gate — exits non-zero when any reported requirement has fewer than `N` distinct verification levels. Combined with `--sil 4`, gates only that tier:

```
$ syscribe -m model/ verification-depth --sil 4 --min-levels 2   # fail the build if a SIL-4 req is verified at <2 levels
```

### All relationships on an element

`links` shows every outbound and inbound relationship — useful for impact analysis before editing a file.

```
$ syscribe -m model_auto/ links System::Software::SafetyMonitor

## Outbound relationships

| Relationship | Target | Type |
|---|---|---|
| satisfies | REQ-ENG-SAFE-001 | Requirement |
| satisfies | REQ-ENG-SAFE-004 | Requirement |
| satisfies | REQ-ENG-SAFE-005 | Requirement |

## Inbound relationships

| Source | Relationship | Type |
|---|---|---|
| Safety::FMEA::FMEA-ENG-001::FM-ENG-006 | subject | FMEAEntry |
| Safety::FMEA::FMEA-ENG-001::FM-ENG-010 | subject | FMEAEntry |
| Vehicle::PowertrainECU::softwareImage::safetyMonitor | typedBy | Part |
```

### What references this element?

```
$ syscribe -m model_auto/ refs System::Software::SafetyMonitor

# References to: System::Software::SafetyMonitor

| Source | Relationship | Type |
|---|---|---|
| Safety::FMEA::FMEA-ENG-001::FM-ENG-006 | subject | FMEAEntry |
| Safety::FMEA::FMEA-ENG-001::FM-ENG-010 | subject | FMEAEntry |
| Vehicle::PowertrainECU::softwareImage::safetyMonitor | typedBy | Part |
```

When the argument is **not** an element qualified name but a raw reference target — such as an `implementedBy:` source path — `refs` reverse-maps it back to the declaring element(s). A directory prefix matches every file beneath it:

```
$ syscribe -m model/ refs src/scheduler/

# References to: src/scheduler/

| Source | Relationship | Type |
|---|---|---|
| System::Software::Scheduler | implementedBy | PartDef |
```

---

## Authoring helpers

### Resolve a cross-reference

Verify that a qualified name or stable ID is resolvable before writing it into a `derivedFrom:` or `satisfies:` field.

```
$ syscribe -m model_auto/ check-ref System::Software::ThrottleControl

resolved  System::Software::ThrottleControl
type      PartDef
file      model_auto/System/Software/ThrottleControl.md
```

### Find the file path for an element

```
$ syscribe -m model_auto/ path-for REQ-ENG-SAFE-001

model_auto/Requirements/Safety/REQ-ENG-SAFE-001.md
```

### Get the next available ID

`next-id` scans the model for all IDs matching a prefix and returns the next unused one — prevents ID collisions when adding new elements.

```
$ syscribe -m model_auto/ next-id REQ-ENG-SAFE

REQ-ENG-SAFE-006
```

### Print a frontmatter template

`template` prints a ready-to-fill skeleton for any element type. Pipe it directly into a new file.

```
$ syscribe -m model_auto/ template Requirement

---
type: Requirement
id: REQ-PREFIX-001
name: "The system shall ..."
status: draft
reqDomain: system
verificationMethod: test
# silLevel: 1
# asilLevel: A
# derivedFrom:
#   - REQ-PARENT-001
# breakdownAdr: ADR-XXX-001
# derivedFromSafetyGoal: SG-PREFIX-001
---

The system shall ...

## Rationale

Why this requirement exists.
```

Create a new requirement file in one step:

```bash
syscribe -m model_auto/ template Requirement \
  > model_auto/Requirements/Safety/REQ-ENG-SAFE-006.md
```

---

## Format spec browser

The `spec` subcommand gives you the full Syscribe format specification without leaving the terminal. No model root needed.

```bash
syscribe spec                  # table of contents
syscribe spec types            # all element types and their schemas
syscribe spec fields           # complete frontmatter field reference
syscribe spec validation       # all validation rule codes (E001–W808)
syscribe spec traceability     # traceability rules R-001–R-007
syscribe spec safety           # safety/security analysis elements
```

---

## LLM authoring prompt

Print the prompt that instructs an LLM to produce valid Syscribe models:

```bash
syscribe --agent-instructions
```

Pipe it directly into your LLM tool of choice:

```bash
syscribe --agent-instructions | llm "Create a brake-by-wire model for ISO 26262 ASIL D"
```

---

## Use by agents and MCP servers

The CLI is designed to be driven by an agent (Claude Code, a CI script, or an MCP server tool) without any modification. Every command writes structured output to stdout and exits cleanly.

### Setting the model root from the environment

Agents that launch subprocesses should set `SYSCRIBE_MODEL` once rather than repeating the flag on every command:

```bash
export SYSCRIBE_MODEL=model_auto/
syscribe validate --json   # no -m needed
syscribe show System::Software::SafetyMonitor
syscribe next-id REQ-ENG-SAFE
```

### Machine-readable validation output

`validate --json` emits a JSON array, one object per finding. An agent reads this directly without parsing Markdown tables.

```json
[
  {
    "code": "W803",
    "severity": "warning",
    "file": "model_auto/Security/VR-ENG-002.md",
    "message": "VulnerabilityReport has status: open — ensure it is being tracked and mitigated"
  }
]
```

An empty array means the model is valid. A non-zero exit code is only set when the command itself fails (bad path, parse error) — a model with warnings still exits 0.

### The incremental authoring loop

The recommended workflow for an agent authoring a new model is to write files in batches and validate after each one. The validator output tells the agent exactly what to fix before moving on.

```
agent writes Batch 4 — Requirements
  → syscribe -m model_auto/ validate --json
  → reads [{"code":"E310","file":"...","message":"missing breakdownAdr"}]
  → fixes the file in the same turn
  → syscribe -m model_auto/ validate --json
  → reads []   ← clean; move to Batch 5
```

See the [LLM Workflow guide](../model-guide/llm-workflow.md) for the full eight-batch sequence and per-batch error checklist.

### Commands useful to agents

| Command | What it returns | When to use |
|---|---|---|
| `validate --json` | JSON findings array | After every batch of writes |
| `validate --file <path>` | Scoped findings | After editing a single file |
| `check-ref <qname\|id>` | Resolved type and file, or error | Before writing a cross-reference |
| `next-id <prefix>` | Next unused stable ID | Before writing a new REQ-*, TC-*, ADR-* |
| `show <qname>` | Full element fields and doc | To read an element before modifying it |
| `trace <req-id>` | Parents, ADR, safety goal, satisfiers, test cases | Impact analysis before changing a requirement |
| `links <qname>` | All outbound and inbound relationships | Impact analysis before changing an element |
| `path-for <qname\|id>` | Absolute file path | To open or overwrite the file for an element |
| `list <type> [scope]` | All elements of a type, optionally scoped | To enumerate IDs in use before authoring |
| `features` | The feature model as a tree (groupKind, constraints, params, config rollup) | To survey a product line's variation points |
| `feature <qname>` | One feature's card: constraints, params, configs, gated elements | To see what a feature means and gates |
| `matrix --features` | Feature × Configuration grid | To see which feature ships in which product |
| `list <type> --feature <F>` | Elements gated by feature `F` (via `appliesWhen:`) | To find what a feature controls |
| `why-active <el> --config <C>` | Whether an element is active in a product, and why | To debug a projection |
| `--agent-instructions [topic]` | Full generation prompt; `magicgrid` topic = a dedicated MagicGrid modeling prompt | System prompt for a model-authoring session |

### Exposing syscribe as an MCP tool

Any MCP server that can execute shell commands can expose syscribe as a set of tools. The simplest pattern is one tool per command group, using `SYSCRIBE_MODEL` to avoid passing the path on every call:

```json
{
  "tools": [
    {
      "name": "syscribe_validate",
      "description": "Validate the Syscribe model and return a JSON array of findings. Empty array means valid.",
      "inputSchema": {
        "type": "object",
        "properties": {
          "file": { "type": "string", "description": "Optional: scope validation to a single file path" }
        }
      }
    },
    {
      "name": "syscribe_show",
      "description": "Show all fields and documentation for a model element.",
      "inputSchema": {
        "type": "object",
        "properties": {
          "qname": { "type": "string", "description": "Qualified name or stable ID (e.g. System::Software::SafetyMonitor or REQ-ENG-SAFE-001)" }
        },
        "required": ["qname"]
      }
    },
    {
      "name": "syscribe_trace",
      "description": "Full traceability slice for a requirement: parents, ADR, safety goal, satisfiers, test cases.",
      "inputSchema": {
        "type": "object",
        "properties": {
          "id": { "type": "string", "description": "Stable requirement ID (e.g. REQ-ENG-SAFE-001)" }
        },
        "required": ["id"]
      }
    },
    {
      "name": "syscribe_next_id",
      "description": "Return the next unused stable ID for a given prefix.",
      "inputSchema": {
        "type": "object",
        "properties": {
          "prefix": { "type": "string", "description": "ID prefix, e.g. REQ-ENG-SAFE or TC-ENG-SEC" }
        },
        "required": ["prefix"]
      }
    }
  ]
}
```

Each tool handler runs the corresponding `syscribe` command, captures stdout, and returns it as the tool result. The agent calls `syscribe_validate` after every write and only proceeds when it returns `[]`.
