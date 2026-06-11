---
id: REQ-TRS-OUT-009
type: Requirement
title: matrix and trace shall surface executed-and-passed evidence (W010 results) by default
status: draft
reqDomain: software
verificationMethod: test
---

The `matrix` and `trace` commands today report *linked* TestCases (a non-draft TestCase whose `verifies:` names the requirement). When a results sidecar (`<model_root>/.syscribe/results.json`, produced by `ingest-results`, REQ-TRS-OUT-008) is present, these commands **shall** reflect *executed-and-passed* evidence by default; absent results, they **shall** behave exactly as today.

## Per-TestCase verdict

The tool **shall** derive a verdict for each TestCase by aggregating over its `testFunctions[].function` references via the ingested results:

- if the TestCase declares no `testFunctions` **or** no results are loaded → `Unknown`;
- else if **any** function's ingested verdict is `Fail` → `Fail`;
- else if **all** functions passed → `Pass`;
- else (some `Ignored`/`Missing`, none `Fail`) → `Unknown`.

## matrix glyphs

When results are present and `--linked-only` is not given, the `matrix` text grid **shall** refine covered cells:

- `—` N/A (requirement's `appliesWhen:` not satisfied by the configuration) — unchanged;
- `✗` gap (requirement active but no linked active TestCase runs here) — unchanged;
- `✓` covered **and** at least one covering active TestCase that runs in this configuration has verdict `Pass`;
- `▣` covered (a linked active TestCase runs here) but **no** passing evidence (every covering TestCase that runs here has verdict `Fail` or `Unknown`).

When results are absent, or `--linked-only` is given, covered cells **shall** stay `✓` exactly as today. The printed Legend **shall** gain `▣ covered, not passing` when results are present. The coverage-% footer semantics **shall** be unchanged (covered = linked).

For `--json`, when results are present the cell value **shall** be the richer `"passing"` (passing evidence), `"covered"` (linked but not passing), `"gap"`, or `"na"`; when results are absent the cell value **shall** stay `"covered"` / `"gap"` / `"na"` so existing consumers do not break.

## trace annotation

When results are present and `--linked-only` is not given, the `trace` command **shall** annotate each verifying TestCase in the "Verified by" listing with its ingested verdict: `[pass]`, `[fail]`, or `[unknown]`. When results are absent, or `--linked-only` is given, no such annotation **shall** appear.

## `--linked-only`

Both `matrix` and `trace` **shall** accept a `--linked-only` flag that forces today's linked-only view (results, if present, are ignored).

## Graceful degradation

With no sidecar present, `matrix` and `trace` output **shall** be byte-identical to the pre-feature behavior. The bundled `model/` (which has no `.syscribe/results.json`) **shall** show no `▣` glyph and no `[pass]`/`[fail]`/`[unknown]` annotations.

**Source:** Issue #21 (surface executed-evidence W010 results by default in `matrix` and `trace`)

**Acceptance criteria:** with a committed results sidecar, `matrix` shows `✓` for a passing-covered cell and `▣` for a covered-not-passing cell; `trace` on a requirement annotates its TestCases with `[pass]`/`[fail]`/`[unknown]`; `--linked-only` on either command reverts to the plain `✓` with no annotations; and a model with no `.syscribe/` directory produces no `▣` glyph and no verdict annotations.
