# trace-code — find untraced code: code units no implementedBy or sourceFile names

## SYNOPSIS
    syscribe -m <root> trace-code [--root <repo>] [--json] [--all]
    syscribe -m <root> trace-code --write-baseline [--dry-run]

## DESCRIPTION
`implementedBy:` (on a design element) and `sourceFile:` (on a TestCase) say which code realises
or verifies an element. `trace-code` reports the reverse direction: code in the repository that
**no** element names — a crate, a build directory, a script or a test file added without a design
element or TestCase. It is the mechanical check for "design before code" (GH #451):
find code without design, and baseline the backlog so that only new untraced code fails.

**Code units.** The opt-in `[traceCode]` table in `<root>/.syscribe.toml` names the source roots
(repository-relative: resolved against the git work tree, or the model root's parent without git)
and how a unit is recognised under them:

    [traceCode]
    roots = ["crates", "scripts", "tests"]
    exclude = ["crates/*/vendor/**"]          # globs; target/, node_modules/, .git/ always skipped
    baseline = "trace-code-baseline.toml"     # relative to the model root (default)
    validate = false                          # true: W326/W327 in `validate`
    units = [
      { kind = "cargo-crate", under = "crates" },
      { kind = "file", under = "scripts", extensions = ["sh", "py"] },
      { kind = "file", under = "tests", extensions = ["rs"] },
    ]

- `cargo-crate` — a directory holding a `Cargo.toml` with a `[package]` table;
- `build-dir` — a directory holding any of `build_files` (default `Cargo.toml`, `package.json`,
  `CMakeLists.txt`, `Makefile`, `go.mod`, `pyproject.toml`, `setup.py`);
- `file` — every file, or only those with one of `extensions`.

A rule applies below its `under` (anywhere under the roots without it); `units` may also be a
single kind (`units = "cargo-crate"`); the default is `build-dir`. Units do not nest: below a
directory unit nothing else is a unit. Keys may be snake_case or camelCase. A malformed entry is
`W328` and ignored by `validate`; `trace-code` itself refuses to run on it (exit 1).

**The containment rule.** A unit is *traced* when some `implementedBy:` or `sourceFile:` entry
names the unit path itself, a file inside it, or a directory containing it. Entries are resolved
exactly as `W023` resolves them (`repo:`/`model:` prefixes, `file://`, absolute, otherwise
model-root-relative); remote URIs and package-registry references (`crates.io:…@…`) name no local
code.

**The baseline only shrinks.** The baseline file lists known untraced units with a reason:

    [[untraced]]
    path = "editors/vscode"
    reason = "VS Code extension; design element pending"

The command fails (exit 2) on an untraced unit missing from the baseline (NEW), and on a baseline
entry that is now traced, no longer exists, or is no longer a unit (stale). `--write-baseline`
writes or refreshes the file: it keeps every existing reason, adds new entries with
`reason = "TODO"` and drops stale ones. It is a guarded write — the agent policy's
`protected_paths` applies — and honours `--dry-run`, `--by`/`--commit`.

**Composed repositories.** With `[repos]` configured, each peer that has its own `[traceCode]`
table is checked against **its own** elements (an element of this model naming peer code does
not count) and reported separately; a peer without the table is listed as not checked.
`--write-baseline` writes only this model's baseline — refresh a peer's in the peer.

**In `validate`.** `[traceCode] validate = true` makes `validate` raise `W326` for each new
untraced unit and `W327` for each stale baseline entry, gateable with `--deny W326 --deny W327`.

## OPTIONS
    --root <repo>     Check this repository instead of the model's own.
    --json            Machine-readable: {schemaVersion, repoRoot, baseline,
                      untraced:[{path, unit, inBaseline}], stale:[{path, reason, why}],
                      traced?:[{path, unit, namedBy:[{element, via}]}], peers?:[...]}.
                      `why` is traced | missing | not-a-unit. Schema:
                      crates/syscribe/schemas/trace-code.schema.json.
    --all             Also list the traced units and the elements that name them.
    --write-baseline  Write/refresh the baseline (keeps reasons, new entries say TODO).

## EXIT CODES
    0  no new untraced unit and no stale baseline entry (or the baseline was written)
    1  usage or configuration error: no [traceCode] table, a malformed entry, a missing root,
       an unreadable baseline
    2  a new untraced unit, or a stale baseline entry

## EXAMPLES
    syscribe -m model/ trace-code
    syscribe -m model/ trace-code --all --json
    # record today's backlog, then give each TODO a reason
    syscribe -m <root> trace-code --write-baseline

## SEE ALSO
    impact (--files: from changed code to elements), validate, coverage, explain W326
