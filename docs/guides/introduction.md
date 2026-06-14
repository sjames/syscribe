# Introduction: A New Approach to System Modeling

`GUIDE · INTRODUCTION`

### Why the existing tools are not enough

The tools that dominate safety-critical systems engineering today — DOORS, Polarion,
Rhapsody, Cameo — were designed in an era defined by two assumptions: that models live
in proprietary databases managed by specialist tool operators, and that the primary
interface for authoring and reviewing that content is a graphical user interface driven
by a human expert.

Both assumptions are now obsolete.

Models locked in proprietary databases cannot be diffed, cannot be reviewed in a pull
request, cannot be audited without a tool licence, and cannot be read by an automated
agent without a bespoke integration. The GUI-first interface, designed for human point-
and-click interaction, is precisely the kind of interface that automated reasoning
systems — including modern LLMs — handle worst. The result is a category of tooling that
is simultaneously expensive, opaque, and resistant to the automation that modern
engineering demands.

A different starting point is possible: treat the *model itself* as structured text,
and design the interface for the three consumers that actually need to read and write it —
**humans**, **LLMs**, and **automated validators**.

### The model-as-text insight

syscribe's foundational choice is that every element in the model — a requirement, a
safety goal, a fault tree event, an architecture block, a test case — is a plain
Markdown file with a structured YAML frontmatter. The directory tree is the namespace.
The content is human-readable prose. The structure is machine-checkable.

This is not a documentation format that happens to have some structure. It is a formal
model format that happens to be readable without a special tool. The difference matters:
the formal structure enables a validator to check every reference, enforce every
integrity rule, compute every metric, and produce a deterministic pass/fail verdict
against a defined gate policy. The human-readable format means that every artefact
produced by the system is directly reviewable in a standard code review — no tool
licence, no export, no rendering pipeline required.

The choice of Markdown + YAML is not accidental. It is the native medium of both human
software engineers (who live in text editors and review tools) and LLMs (which are
trained on vast corpora of structured text and excel at generating and reasoning about
it). The model format is tuned for access by all three parties simultaneously.

### A new participant: the LLM as a contributing engineer

The emergence of capable LLMs changes what is possible in model-based systems
engineering. An LLM can:

- Draft a complete set of requirements from a design description in seconds
- Populate a HARA by reasoning over a system description and a hazard taxonomy
- Author FMEA entries by connecting fault modes to architecture elements it can read
- Write Gherkin test specifications that align with requirements it has just authored
- Maintain consistency across hundreds of model elements as the system evolves

But an LLM used without a formal check on its outputs introduces a new and serious
risk: **confident incorrectness**. An LLM that cannot be contradicted by an authoritative
source will hallucinate — it will assert that a requirement exists when it does not,
claim a test case covers a requirement that it does not verify, or write a safety argument
that references a SafetyGoal that was never defined. In safety-critical engineering, a
confident but wrong assertion in the model is more dangerous than a gap, because it creates
false assurance.

syscribe's validator is the direct answer to this risk. It is the objective arbiter that
the LLM cannot convince and cannot bypass. If an LLM writes `verifies: REQ-X-999` and
REQ-X-999 does not exist, the validator emits an error and the gate fails — regardless
of how confidently the LLM stated the link. If the LLM derives a requirement from a
safety goal and forgets the mandatory `breakdownAdr:`, the validator catches it. If the
LLM authors a TestPlan whose `selection: tags:` matches no TestCase, W612 fires. The
LLM's output is continuously tested against a formal specification of what a valid model
looks like.

This makes syscribe an instance of a broader principle: **the LLM authors, the validator
judges, the human reviews the diff**. Each party does what it does best. The LLM generates
structured, coherent, voluminous content quickly. The validator enforces formal correctness
mechanically and objectively. The human applies engineering judgement to the substance —
not to checking whether every QName resolves or every mandatory field is present, because
the tool already did that.

### Keeping the LLM grounded: guard rails, not a leash

The metaphor of "guard rails" is precise. A guard rail does not steer the vehicle — it
lets the driver go wherever the road permits, but prevents them from going off a cliff.
syscribe's validation rules are guard rails in exactly this sense: they do not constrain
what the model can express, but they prevent the model from expressing things that are
formally inconsistent.

The guard rails operate at several levels:

**Reference integrity.** Every cross-reference in the model (`derivedFrom:`, `verifies:`,
`hazardousEvents:`, `children:`) is validated at parse time. An LLM cannot invent a
reference that does not exist. A renamed element immediately surfaces every stale
reference pointing to the old name. The model is always internally consistent, or the
gate fails.

**Structural completeness.** Mandatory fields are enforced. A `SafetyGoal` without
`hazardousEvents:` is incomplete (W806). A `TestCase` without a Gherkin body is invalid
(E011). A derived requirement without a breakdown rationale is refused (E310). The LLM
must produce complete artefacts, or the validator rejects them.

**Integrity level propagation.** ASIL and SIL levels flow through the derivation chain
and are checked for consistency. An LLM cannot elevate a derived requirement above its
parent's integrity level (E841). It cannot mix `asilLevel:` and `silLevel:` on the same
element (W006). The mathematical structure of the integrity level hierarchy is enforced
automatically.

**Coverage accounting.** The coverage matrix is computed from actual links, not from
assertions. An LLM cannot declare that a requirement is covered — it must write a
`TestCase` with a `verifies:` that points to the requirement. The matrix then shows
coverage as a fact, not a claim. If the LLM says "all requirements are covered" but has
not written the test cases, `matrix --gaps-only` shows the truth.

**Metric consistency.** The safety metrics (SPFM, LFM, PMHF) are computed from the
`failureRate` and `diagnosticCoverage` fields on the fault tree events — numbers the LLM
must justify with engineering rationale. The result is compared against the SIL/ASIL
target and the gate either passes or fails. The LLM cannot fabricate a passing metric.

### Formal Spec Driven Development

The practice that emerges from combining syscribe with LLM-assisted authoring is best
understood as **Formal Spec Driven Development** — a structured analogue of Test Driven
Development applied at the specification and requirements level.

In TDD, the test is written before the implementation; the test fails; the implementation
is written to make the test pass; the cycle repeats. The test is the authority, and
the implementation must conform to it.

In Formal Spec Driven Development:

1. **The specification is written first** — requirements, architecture, safety goals,
   interface contracts. These live in the model and are validated by syscribe.
2. **The specification is the authority** — not the implementation, not the documentation,
   not the engineer's memory. The model is the single source of truth.
3. **The implementation must conform to the specification** — not by assertion, but by
   verified traceability. Test cases link to requirements. Architecture elements satisfy
   requirements. The coverage matrix shows the state of conformance.
4. **The specification gate runs in CI** — every commit is validated. Drift between
   specification and implementation (missing test coverage, unallocated requirements,
   broken references) is detected automatically and reported as a gate failure.

LLMs accelerate this cycle dramatically. An LLM can author a first-draft specification
from a design intent expressed in natural language. The validator immediately tells it
what is incomplete. The LLM revises. The human reviews the substantive content, not
the mechanical completeness. The gap between "design intent" and "validated formal
specification" collapses from days to minutes.

The result is a model that is simultaneously:
- **Formal** — every link is validated, every integrity rule is enforced, every metric
  is computed from the model
- **Human-readable** — any engineer can read, review, and understand every artefact in
  a text editor or a code-review tool
- **LLM-accessible** — an LLM can author, query, update, and reason about the model
  without tool licences or bespoke integrations
- **CI-gated** — the gate is objective, reproducible, and automatic

This combination — formal rigour delivered through a text-first, LLM-native, CI-gated
interface — is what makes syscribe a genuinely new approach to systems modelling, rather
than a repackaging of existing ideas in a new syntax.

### The role of the human engineer

None of the above makes human engineers redundant. The validator enforces structural
correctness; it does not evaluate engineering judgement. The LLM authors content
quickly; it does not understand the physics, the regulatory context, or the operational
environment. The human brings the things neither can provide:

- **Domain knowledge** — knowing that a failure rate of 5e-9/h is a reasonable estimate
  for a well-tested software module, and why
- **Regulatory context** — understanding what an assessor will actually look for in an
  ISO 26262 ASIL D submission, and where the current model falls short
- **Integration judgement** — deciding that two subsystems at different integrity levels
  need a documented FFI argument, not just an allocation link
- **Review authority** — approving changes to the model by reviewing the git diff,
  exercising the same scrutiny as a code review

The workflow is therefore not "LLM replaces engineer" but "LLM + validator + CI remove
the mechanical overhead, leaving the engineer's time for engineering judgement." The
three-way collaboration — LLM authors, validator checks, human reviews — produces
higher-quality formal models faster and more consistently than any single party alone.

### To the sceptic

If you are approaching syscribe from a background in traditional toolchains, the most
common concern is: *is text in git really rigorous enough for a safety submission?*

The answer is that the rigour comes from the validator and the process, not from the
database. DOORS does not make requirements correct — it stores them. Correctness comes
from the review that approved them and the tests that verified them. syscribe provides
the same evidence — requirements with stable IDs, formal traceability links, reviewed
changes, test coverage records, metric computations — in a format that is auditable
without the tool, reproducible without the database, and reviewable without a licence.

The git history *is* the audit trail. The diff *is* the change record. The CI gate
*is* the consistency check. The `--json` output of `validate`, `matrix`, and `metrics`
*is* the evidence package.

What syscribe cannot provide: tool qualification in the IEC 61508-3 Annex D / ISO 26262-8
§11 sense for a T3 tool. syscribe is a T1 tool — its outputs are reviewed before use,
not trusted blindly. This is the same classification as any documentation tool. Your
compiler remains the T3 tool that requires qualification; syscribe is closer to your
word processor, used to produce work products that engineers review.

**A path toward T2.** There is, however, a substantive argument that syscribe's tool
confidence level can be raised above T1. The reason is that syscribe is developed using
itself — its own requirements are managed in a syscribe model, its own test cases are
linked to those requirements, and its own CI gate runs `syscribe validate` against its
own model to check coverage and traceability consistency. In other words, the Formal Spec
Driven Development methodology described above is applied to syscribe's own development
process.

This matters for qualification because IEC 61508-3 Annex D §D.4 and ISO 26262-8 §11.4.2
provide a path to increased tool confidence through evidence of the *tool's own development
process*: if the tool developer can demonstrate that the tool was developed to a defined
software quality process — with traceable requirements, test coverage, and a CI gate — the
qualification burden on the tool user is correspondingly reduced. A tool developed with
full requirements traceability, multi-level tests (unit, integration, formal proofs), and
automated coverage measurement is in a meaningfully different position than a tool whose
development process is undocumented.

The practical consequence: syscribe may be able to support a **T2 confidence classification**
under both standards — meaning tool errors are detectable (its outputs are human-reviewed
and its findings are reproducible from the same model), and the tool's own development
artefacts provide evidence of a disciplined process. This is an active area of work. Users
who need formal tool qualification evidence should request the syscribe development model
and CI artefacts as part of their tool assessment, and evaluate them against the §D.4 /
§11.4.2 criteria for their specific SIL or ASIL target.
