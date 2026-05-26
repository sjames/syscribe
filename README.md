# Syscribe

> An experiment in LLM-native systems modeling — full SysMLv2 semantics expressed as Markdown + YAML.

---

## The Idea

Systems modeling tools have traditionally been built around proprietary binary formats or complex XML schemas. They are powerful but opaque — hard for humans to read in raw form, and nearly impossible for LLMs to generate or reason about reliably.

This project explores a different premise: **what if the model format were just Markdown files with YAML frontmatter?**

Every model element is a `.md` file. The directory structure encodes the namespace hierarchy. YAML frontmatter declares the element type and its structural relationships. The Markdown body is the documentation. Cross-references between elements use `::` qualified names, just like SysMLv2.

```
model/
  UAV/
    _index.md          # package UAV
    Avionics/
      FlightController.md   # part def FlightController
      IMU.md
  Requirements/
    SafetyReqs.md      # requirement def SafetyReqs  {isAbstract}
    FaultTolerantFCReq.md
  Diagrams/
    SafetyRequirementsD.md  # diagram with layout: block
```

```yaml
---
type: RequirementDef
name: FaultTolerantFCReq
supertype: Requirements::SafetyReqs
text: "The flight controller shall detect a single sensor failure within 50 ms."
verifies: [Verification::FCFaultInjectionTest]
---

Derived from the top-level safety requirement. Applies to all flight-critical
sensor inputs. Design assurance level: DAL A.
```

## Why This Format Is Interesting

**For humans** — every element is a readable, diffable, grep-able text file. You can edit the model in any text editor, navigate it with standard file-system tools, and review changes in any git diff viewer.

**For LLMs** — Markdown and YAML are the two formats that large language models handle best. An LLM can generate a complete model element from a natural-language description, refactor relationships across files, or answer questions about the architecture by reading the files directly. No special tooling required on the LLM side.

**For teams** — the model lives in a git repository alongside the code. Pull requests, blame, branches, and merges all work as expected. The model and the implementation evolve together.

## The Validation Problem

Flexibility without structure becomes noise. If an LLM (or a human) can write anything into a frontmatter field, you lose the semantic guarantees that make a model useful for analysis, simulation, or certification.

The Rust tooling in this repository provides that structure. It:

- **Parses and validates** every `.md` file against the Syscribe schema
- **Resolves cross-references** — catches dangling `::` qualified names at load time
- **Builds an in-memory graph** (via petgraph) representing containment, specialisation, allocation, and verification relationships
- **Renders diagrams** from `layout:` blocks in frontmatter — no hand-crafted SVG required
- **Serves a live browser UI** with an infinite canvas, diagram tabs, and in-place editing that writes back to the source files

The combination means LLMs can freely author and edit model files, and the Rust layer immediately tells you whether the result is structurally valid.

## Repository Structure

```
crates/
  syscribe-model/       # core library: parser, walker, graph builder, resolver, SVG renderer
  syscribe-server/      # Axum web server + Askama templates + HTMX frontend
model/               # the sample UAV model (Syscribe source files)
temp/                # reference PDFs (SysMLv2 spec, OMG formal documents)
```

## Running

```bash
cargo build
MODEL_ROOT=model ./target/debug/syscribe-server
# open http://localhost:3000
```

The server watches the `model/` directory for changes and reloads automatically.

## Status

This is an early-stage experiment. The format is largely stabilised; the tooling is functional but incomplete. Current capabilities:

- Full model tree browser with namespace drill-down
- Diagram rendering from `shapes` / `edges` / `layout` frontmatter blocks
- Infinite canvas with pan, zoom, and per-tab state
- Drag-to-reposition shapes (positions written back to the `.md` file)
- Element property editing (name and documentation) via the browser UI
- Live reload over WebSocket on file save
- REST API for elements, containment tree, and connection graph

Planned:
- Auto-layout for diagrams that have no `layout:` block
- Structural editing: adding and connecting elements from the browser
- Validation report UI (unresolved references, type errors)
- Additional diagram kinds (IBD, BDD, state machine, sequence)
