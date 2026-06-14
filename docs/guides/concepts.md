# Part I — Concepts and the Model-as-Code Shift

`GUIDE · PART I — CONCEPTS`


### 1.1 What syscribe is

syscribe is a **model validator and analysis engine** that works against a collection of
Markdown files stored in a directory tree. Each file is an element — a requirement, a test
case, an architecture block, a fault tree event, an asset. The files are your model; syscribe
reads them, builds a graph, validates it, and runs analyses on demand.

There is no proprietary database. There is no server. The model is plain text in your
version-control system. Every analysis command (`validate`, `matrix`, `metrics`, `cyber-risk`)
reads the files fresh and writes its output to stdout. Reports are *reproducible* — the same
model always produces the same output.

### 1.2 Mental model shift from traditional tools

| Concept | DOORS / Polarion | syscribe |
|---|---|---|
| **Element** | Row in a module | `.md` file with YAML frontmatter + prose body |
| **Attribute** | Column in a module | Frontmatter field (`status:`, `asilLevel:`, etc.) |
| **Trace link** | Link type between modules | `derivedFrom:`, `verifies:`, `satisfies:` fields |
| **Module / folder** | Module with an attribute scheme | Directory + `_index.md` (`type: Package`) |
| **Baseline** | Named snapshot in the tool | Git tag (`git tag v1.0-cert-2026-06`) |
| **Change management** | Change workflow / ECO | Pull request + review + merge |
| **Report** | Configurable export / PDF | `syscribe validate`, `matrix`, `metrics`, `audit`, etc. |
| **Tool qualification** | T2/T3 with tool evidence | syscribe is a documentation tool (T1); document its role |
| **Coverage matrix** | Trace matrix report | `syscribe matrix` (live, from the model) |
| **Attribute filter** | Query builder in GUI | `syscribe list Requirement --status draft --sil 4` |

The important difference: **traceability is enforced by the validator, not by GUI link types**.
When you write `derivedFrom: [REQ-SAFE-001]` in a requirement file, syscribe checks that
`REQ-SAFE-001` exists, is in scope, and is of the right type. If the target is renamed or
deleted, `validate` emits an error on the next run. You cannot have a dangling link.

### 1.3 File and namespace layout

Every element lives at a path under a model root directory. The directory path becomes the
**qualified name** (QName), with `::` replacing `/` and the filename stem as the final segment:

```
model/
  _index.md                          ← type: Namespace  (model root)
  Requirements/
    _index.md                        ← type: Package
    Safety/
      _index.md                      ← type: Package
      SchedulingIntegrity.md         ← QName: Requirements::Safety::SchedulingIntegrity
  Safety/
    SG-KERNEL-001-SchedulingIntegrity.md  ← QName: Safety::SG-KERNEL-001-SchedulingIntegrity
```

**Rule**: use the full QName in cross-references unless the element is in the same package.
The `check-ref` command confirms a reference resolves:

```bash
syscribe -m model check-ref Safety::SG-KERNEL-001-SchedulingIntegrity
```

### 1.4 The validation gate

```bash
syscribe -m model validate
```

Exits 0 if no errors. Exit 1 = errors present. The output is a structured Markdown table of
findings. Promote specific warnings to gate failures with `--deny`:

```bash
syscribe -m model validate --deny W015,W031   # coverage gaps and untreated threats fail the gate
```

Or apply a named policy from `.syscribe.toml`:

```bash
syscribe -m model validate --profile sil4     # all codes in [profiles.sil4] are hard failures
```

**Fit to your CI pipeline**: syscribe runs in seconds. Put `validate` and the relevant analysis
commands (`matrix`, `metrics`, `cyber-risk`) in your gate. The exit codes are clean for
scripting.
