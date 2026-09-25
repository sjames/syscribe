---
id: REQ-TRS-CLI-009
type: Requirement
name: Tool shall reject invalid option values and unknown options with a usage error
status: draft
reqDomain: software
verificationMethod: test
---

A command-line mistake **shall never** be silently absorbed into a default: a CI job or
an agent that mistypes an option must get an error, not output that looks valid but was
computed with different settings. (Issue #133.)

### Enumerated and numeric option values

An option whose value is drawn from a fixed set **shall** reject any other value with a
usage error that names the option, the offending value and the valid values:

| Command | Option | Valid values |
|---|---|---|
| `impact` | `--direction` | `downstream`, `upstream`, `both` |
| `impact` | `--format` | `text`, `json`, `dot` |
| `n2` | `--format` | `text`, `html`, `json` |
| `behavioral-coverage` | `--format` | `text`, `json` |
| `sbom` | `--format` | `cyclonedx`, `spdx` |
| `build-config` | `--format` | `cmake`, `c-header`, `makefile`, `env`, `json`, `kconfig` (only `json` with `--all-configs`) |
| `validate` | `--format` (with `--results`) | `cargo-json`, `junit`, `session-log` |
| `diagram render`/`compose`/`layout` | `--kind` | `bdd`, `ibd`, `arch` |
| `diagram` | `--view` | `full`, `ports`, `features`, `compact`, `name`, `requirement` (`req`), any case |

An option that takes a count or depth (`--depth` on `impact`, `n2`, `behavioral-coverage`,
`summarize`; `--package-top-n` on `stats`; `--limit`/`--offset` on `digest`; `--limit` on
`search-text`; `--top` on `topics`; `--k` on `clusters`; `--min-levels` on
`verification-depth`; `--min-width` on `diagram`) **shall** reject a value that is not a
non-negative integer (a number, for `--min-width`) instead of silently using the default.
A `validate --results <file>` that cannot be read or parsed **shall** likewise be a usage
error rather than a run without results.

### Unknown options

The following commands **shall** check every `-`/`--` token against their own documented
option list and reject an unknown option (or a value-taking option given no value) with
a usage error naming it: `validate`, `list`, `show`, `trace`, `why`, `who-verifies`,
`impact`, `links`, `refs`, `export`, `find`, `ls`, `tree`, `extref`, `n2`,
`behavioral-coverage`, `sbom`, `build-config`, `stats`, `digest`, `search-text`,
`summarize`, `topics`, `clusters`, `verification-depth`. (`lint-docs`, `follow`,
`connectivity` and the clap-parsed `diagram` family already do so with their own parsers.)
Positional arguments are not affected, and an option given as `--opt=value` is accepted
only where the command documents that spelling.

### Contract

- A usage error **shall** print a message to stderr, print nothing to stdout, and exit
  `1` (for `validate`, consistent with [[REQ-TRS-OUT-006]]: `2` stays reserved for a
  tripped gate). The `diagram` family keeps its clap parser's exit code `2`.
- The check **shall** happen before the model is loaded, so the error is the same with
  or without a valid model directory.
- Valid invocations **shall** behave exactly as before: this requirement only turns
  silent acceptance of invalid input into errors.

**Acceptance criteria:** `impact X --direction sideways`, `impact X --format xml`,
`n2 --format xml`, `behavioral-coverage --format xml`, `sbom --format xml`,
`build-config --config C --format xml`, `n2 --depth abc` and `validate --bogus` each exit
`1` with empty stdout and a message naming the option; the error for an enumerated option
lists its valid values; `list Requirement --bogus`, `show X --bogus`, `trace X --bogus`,
`export --bogus`, `find x --bogus`, `ls --bogus` exit `1`; every documented option of the
checked commands is still accepted; `diagram render X --view bogus` is rejected.
