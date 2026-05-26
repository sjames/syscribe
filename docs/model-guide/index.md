# Modeling Guide

`GUIDE · OVERVIEW`

This guide walks through building a Syscribe model from scratch — from a directory skeleton to a fully traced, validated system description.

## Prerequisites

- Rust toolchain (stable, 1.75+)
- `cargo` available on `$PATH`
- A text editor with YAML syntax highlighting

## Directory skeleton

```
model/
  _index.md          # root namespace (type: Namespace or Package)
  Requirements/
    _index.md        # Requirements package
  Verification/
    _index.md
  Decisions/
    _index.md
  MySystem/
    _index.md
```

Each `_index.md` declares the package:

```yaml
---
type: Package
name: Requirements
---

System requirements for the XYZ project.
```

## Running the validator

```bash
cargo run --package syscribe -- model/
```

This prints a full Markdown report to stdout with all errors, warnings, and a traceability matrix. Pipe it to a file for review:

```bash
cargo run --package syscribe -- model/ > reports/validation.md
```

## Running the web browser

```bash
cargo run --package syscribe-server -- model/
# Listening on http://0.0.0.0:3000
```

The server watches the model directory for changes and reloads automatically.

## Typical workflow

1. **Identify stakeholder goals** — write parent `Requirement` elements with `REQ-SYS-000` style IDs
2. **Decompose requirements** — create child requirements with `derivedFrom:` and `breakdownAdr:`
3. **Write an ADR** for each decomposition decision — the validator enforces this (E310)
4. **Build the architecture** — create `PartDef`, `PortDef`, `InterfaceDef` elements
5. **Assign requirements** — add `satisfies:` and `domain:` to architecture elements
6. **Write test cases** — create `TestCase` elements with `verifies:` and Gherkin bodies
7. **Draw diagrams** — create `Diagram` elements; add `layout:` for SVG rendering
8. **Validate** — run the validator; fix all errors; review warnings

## Further reading

- [Requirements & Test Cases](requirements.md) — stable IDs, lifecycle, Gherkin
- [Traceability](traceability.md) — the six §12 rules
- [Architecture Decisions](adrs.md) — ADR lifecycle and breakdown rules
