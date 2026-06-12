---
id: REQ-TRS-FMA-008
type: Requirement
name: Tool shall provide a configure command reporting satisfiability, forced and free features for a partial selection
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** provide an assisted-configuration command, `configure <Configuration>` (with `--json`), that turns the analysis engine into a product configurator: given a partial selection it reports whether the selection can be completed and what it forces.

### Partial-selection semantics

- The named `Configuration`'s `features:` entries are treated as a **partial assignment** (assumptions): a feature set `true`/`false` is **fixed**; a feature **absent** from the map is **open** (free to be decided). This deliberately differs from the closed-world configuration-validity check ([[REQ-TRS-FMA-004]]), which treats absent features as deselected.

### Output

For the encoded model `Φ` and the partial assignment `P`:

| Result | Definition |
|---|---|
| **satisfiable** | `Φ ∧ P` is SAT — the partial selection extends to at least one valid configuration |
| **forced-true** feature `f` | `Φ ∧ P ∧ ¬f` is UNSAT (an open `f` that every completion must select) |
| **forced-false** feature `f` | `Φ ∧ P ∧ f` is UNSAT (an open `f` no completion may select) |
| **free** feature `f` | open and neither forced |

- If `Φ ∧ P` is **unsatisfiable**, the command **shall** report it with a minimal explanation ([[REQ-TRS-FMA-007]]) and exit `1`; otherwise exit `0`.
- Output (text + `--json`: `satisfiable`, `forcedTrue`, `forcedFalse`, `free`, and `explanation` when unsatisfiable) **shall** be in deterministic (qualified-name) order.
- Dormant when no feature model is present (notice, exit `0`); Boolean layer only ([[REQ-TRS-FMA-006]]).
- The command **shall** be discoverable in `--help`.

**Source:** ADR-FM-002.

**Acceptance criteria:** Given `A requires B` with a partial selection `{A: true}` (B open), `configure` reports satisfiable, `B` forced-true, and any unrelated optional feature free. A contradictory partial selection (e.g. `{A: true, B: false}` with `A requires B`) is reported unsatisfiable with an explanation naming the conflict and exits `1`. With no `FeatureDef`, it prints a notice and exits `0`. Output is identical across runs.
