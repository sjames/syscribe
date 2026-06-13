---
id: REQ-TRS-TYPE-022
type: Requirement
name: Tool shall detect peer-repository ref drift and allow gating validation on it
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** detect when a peer repository's git work tree has **drifted** from the
`ref:` pinned in the `[repos]` table (§14, GH #62), so a composition can be gated on
reproducibility, and **shall** report it as warning **`W511`**.

- For each repo with a configured `ref:`, the tool **shall** compare the peer work tree's
  `HEAD` commit with the commit the `ref:` resolves to. When they differ it **shall** emit
  `W511` naming the repo, the configured ref, and the resolved `HEAD`.
- `W511` **shall** be gateable to a hard failure with `--deny W511` (a CI reproducibility
  gate), consistent with `W510`. It is **not** raised when drift cannot be determined —
  git unavailable, the peer is not a git work tree, the `ref:` does not resolve, the path is
  absent, or no `ref:` is configured — so drift is never falsely reported.
- The `repos status` command **shall** report the same drift (`out of sync`) and exit `2`.

### Validation rules

| Code | Condition |
|---|---|
| `W511` | A peer repo's `HEAD` differs from its configured `ref:` (drift; opt-in, `--deny W511`). |

**Source:** §14 (Multi-Repository Model Composition), GH #62.

**Acceptance criteria:**

- A composition whose peer is pinned to a ref the peer `HEAD` no longer matches emits `W511`;
  `--deny W511` exits non-zero.
- A composition whose peer `HEAD` is at the configured ref is silent (no `W511`).
- Drift that cannot be determined (no git / unresolved ref) does not emit `W511`.
