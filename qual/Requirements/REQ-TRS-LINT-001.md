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

## Paths

Every `<path>` argument **shall** exist. A path that does not exist (e.g. a CI typo)
**shall** be a usage error: the tool **shall** name each missing path on stderr, scan
nothing, print nothing on stdout, and exit `1` — never exit `0` as if the (absent) file
were clean. (Issue #130.)

## Flags

The model root is the global `-m`/`--model` (there is no per-command model flag).

- `--deny <CODES>` — comma-separated (repeatable) `lint-docs` codes to treat as gate
  failures. The unresolvable-reference codes `W099`–`W102` already fail the run, so
  denying them is accepted and changes nothing; denying the advisory `W103` makes a
  `W103` finding fail the run (exit `1`). A code outside `W099`–`W103` **shall** be a
  usage error (exit `1`, message listing the valid codes). The flag's value **shall
  never** be taken as a path to scan. (Issue #130.)
- `--json` — emit findings as a JSON array instead of the human-readable form.
- Any other `-`/`--` option **shall** be a usage error (exit `1`).

**Acceptance criteria:**

- `lint-docs docs/` on a directory containing a Markdown file that references
  `REQ-TRS-OUT-015` (which exists in the model) produces no output and exits 0.
- A Markdown file referencing `REQ-TRS-NONEXIST-001` (not in model) causes W099 and
  exit 1.
- `--json` emits a JSON array of `{file, line, code, token}` objects.
- Files with no stable-ID tokens produce no output and exit 0.
- `lint-docs nonexist.md` (and `lint-docs <existing> nonexist.md`) exits 1 naming the
  missing path, with empty stdout.
- `lint-docs x.md --deny W099` scans only `x.md` (the code is not a path); `--deny W103`
  makes an `_index.md` enumerating its members exit 1; `--deny W999` exits 1 as a usage
  error.
