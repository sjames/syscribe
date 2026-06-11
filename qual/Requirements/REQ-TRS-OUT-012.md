---
id: REQ-TRS-OUT-012
type: Requirement
title: Tool shall support named, integrity-level-scopable validation severity profiles
status: draft
reqDomain: software
verificationMethod: test
---

The `validate` subcommand **shall** support **named, reusable severity profiles** declared in `<model_root>/.syscribe.toml` and selected with `validate --profile <name>`, so that a project can encode a reusable gating policy (e.g. a "safety" profile) instead of repeating ad-hoc gating flags on every invocation.

This requirement refines the configurable warning gating and exit-code contract of REQ-TRS-OUT-006.

## Profile declaration

A profile is a `[profiles.<name>]` table in `<model_root>/.syscribe.toml`:

```toml
[profiles.safety]
promote = ["W002", "W015", "W300"]   # warning codes promoted to gating failures
# OPTIONAL scope — promotion applies only to findings on an element matching ALL given fields:
sil = "4"            # element's silLevel stringifies to "4" OR asilLevel == "4"
status = "approved"  # element's status:
tag = "safety"       # element's tags: contains this
```

The tool **shall**:

- Allow multiple profiles (`[profiles.foo]`, `[profiles.bar]`) in one file.
- Continue to parse the existing `[matchers]`, `[remote]` tables and the `repo_root` key without regression when a `[profiles]` table is also present.

## Behaviour

The tool **shall**:

1. On `validate --profile <name>`, load `[profiles.<name>]` from `<model_root>/.syscribe.toml`. If the name is not defined (or no `.syscribe.toml` exists), print an error to stderr and exit `1`.
2. Treat a finding as a **gate failure** (contributing to exit `2`, exactly like `--deny`) when its `code` is in the profile's `promote` list **AND** (the profile declares no scope fields, **OR** the element the finding concerns matches **ALL** provided scope fields).
3. Map a finding to its element by `file_path` (the element whose `file_path` equals `Finding.file`). A finding whose file maps to no element does **not** match a scope (so it is not promoted when any scope field is set). With no scope fields, every finding whose code is listed is promoted regardless of element lookup.
4. Evaluate the scope using the same matching semantics as `list --sil` for `sil` (element's `silLevel` stringified, or `asilLevel`), exact equality for `status`, and membership for `tag` (the element's `tags` contains the value).
5. Compose `--profile` additively with `--deny`, `--max-warnings`, and `--warnings-as-errors` (their gate-failure sets are unioned).

## Exit-code contract

The tool **shall** preserve the REQ-TRS-OUT-006 contract:

| Exit code | Meaning |
|---|---|
| `0` | No `Error`-severity findings and no gate failure |
| `1` | One or more `Error`-severity findings (errors always dominate), **or** an unknown / undefined profile name |
| `2` | One or more `Warning`-severity findings tripped the gate (including profile-promoted findings) |

When no `--profile` is supplied, the bundled models and any model without a `[profiles]` table **shall** behave exactly as before.

**Source:** Issue #18 (named, SIL/ASIL-scopable validation severity profiles); refines REQ-TRS-OUT-006; §11.12

**Acceptance criteria:**

- A warning-only model exits `0` under plain `validate`.
- `validate --profile <p>` where `<p>` promotes a code present in the model exits `2`.
- A profile scoped to a SIL/status/tag promotes **only** findings on elements matching that scope: an unscoped profile and a scope that matches a strict subset of those findings produce distinguishable gate outcomes.
- A profile whose scope matches no element exits `0` (nothing promoted), while an unscoped profile over the same code exits `2`.
- `validate --profile nonexistent` (undefined profile or missing `.syscribe.toml`) exits `1`.
