# lint-docs — scan external Markdown/SVG for unresolvable model references

## SYNOPSIS
    syscribe -m <root> lint-docs <path>... [--json] [--deny <CODES>]

## DESCRIPTION
Scans `.md` and `.svg` files (or directories, recursively) for references to model
elements that do not resolve in the loaded model: stable-ID tokens in prose, qualified
names inside Mermaid blocks, SVG `sysml:ref` manifests, and local image/diagram embed
paths.

It also flags a package `_index.md` that hand-enumerates its own members (`W103`,
advisory): membership is generated from the directory by `show <package>`, the web UI
and `export-html`, so a hand-maintained list only drifts.

Exits non-zero if any unresolvable references are found (`W099`–`W102`), enabling CI
gating. `W103` alone never makes it exit non-zero unless you opt in with `--deny W103`.

Every path must exist: a nonexistent path is a usage error (each missing path is
named on stderr, nothing is scanned, exit 1), so a typo in a CI path cannot pass as a
clean run.

## OPTIONS
    <path>...    Files or directories to scan (.md and .svg).
    --json       Emit findings as a JSON array of {file, line, code, token|ref|path|detail}
                 (`detail` carries the W103 message).
    --deny <CODES>
                 Comma-separated lint-docs codes (W099–W103; repeatable, or --deny=<CODES>)
                 treated as failures. W099–W102 already fail the run, so this matters for
                 the advisory W103: `--deny W103` makes it exit 1. Any other code, or an
                 unknown option, is a usage error (exit 1).

## EXAMPLES
    syscribe -m model/ lint-docs docs/
    syscribe -m model/ lint-docs docs/ --json
    syscribe -m model/ lint-docs docs/demo-model/PowerSystemIBD.svg

    # Also fail on package _index.md files that hand-enumerate their members
    syscribe -m model/ lint-docs model/ --deny W103

## EXIT CODES
    0  no W099–W102 finding (W103 is advisory unless denied)
    1  a W099–W102 finding, a finding whose code is in --deny, or a usage error
       (nonexistent path, unknown --deny code, unknown option, no path)

## NOTES
Findings (W099–W102 always fail the run; W103 only with `--deny W103`):

- **W099** — an unresolvable stable-ID token (`REQ-*`, `TC-*`, …) in prose.
- **W100** — a qualified name (`A::B::C`) inside a ` ```mermaid ` block that does not
  resolve. (Qualified names in *prose* are deliberately not resolved — false-positive prone.)
- **W101** — an SVG `sysml:ref="…"` that does not resolve. An SVG with no `sysml:ref`
  attributes is treated as opaque (no findings).
- **W102** — a local image/diagram embed path (`![](path)`, `<img src>`) that does not
  exist. Remote URIs (`https://…`) are accepted as external.
- **W103** (advisory) — a package's own `_index.md` whose body (frontmatter excluded)
  mentions three or more distinct stable ids of that package's *direct* members. The
  message names the count and points at `syscribe show <package>`; state the package's
  purpose and scope in `_index.md` instead of listing its members. Ids of other
  packages' elements, fewer than three members, and files that are not a package's
  `_index.md` never raise it. Directory scans apply it too.

## SEE ALSO
    validate, check-ref, refs, show
