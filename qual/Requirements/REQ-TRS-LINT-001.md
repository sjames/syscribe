---
id: REQ-TRS-LINT-001
type: Requirement
name: "lint-docs command shall scan external Markdown for unresolvable stable ID tokens"
status: draft
reqDomain: software
verificationMethod: test
---

The `lint-docs` sub-command **shall** accept one or more paths (files or directories)
and scan all `.md` files found under them for tokens that match a stable-ID pattern.

## Stable-ID patterns scanned

- `REQ-*` (Requirement)
- `TC-*` (TestCase)
- `ADR-*` (ADR)
- `FEAT-*` (FeatureDef)
- `FM-*` (FMEAEntry)
- `FTE-*` (FaultTreeEvent)
- `AOU-*`, `SG-*`, `CM-*` (safety/security elements with stable ids)

Any token matching `^(REQ|TC|ADR|FEAT|FM|FTE|AOU|SG|CM)(-[A-Z0-9]{2,12})+(-[0-9]{3,8})?$`
is a candidate.

## Resolution and reporting

Each candidate token **shall** be looked up against the loaded model. If the token does
not resolve to a known element the tool **shall** emit a warning in the form:

```
<file>:<line>: W099: unresolvable ID token '<token>' referenced in external doc
```

Exit code **shall** be non-zero if any unresolvable tokens are found (enabling CI gating).

## Flags

- `--model-dir <path>` — model root to load (defaults to `-m` value if the global flag
  is already provided).
- `--deny W099` — escalate to error (exit non-zero) even if tokens are not found
  (already implied by W099 being the only emitted code).
- `--json` — emit findings as a JSON array instead of the human-readable form.

**Acceptance criteria:**

- `lint-docs docs/` on a directory containing a Markdown file that references
  `REQ-TRS-OUT-015` (which exists in the model) produces no output and exits 0.
- A Markdown file referencing `REQ-TRS-NONEXIST-001` (not in model) causes W099 and
  exit 1.
- `--json` emits a JSON array of `{file, line, code, token}` objects.
- Files with no stable-ID tokens produce no output and exit 0.
