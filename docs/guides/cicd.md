# Part VIII — Integration and CI/CD

`GUIDE · PART VIII — CI/CD`


### 8.1 Minimum CI gate

```bash
# Validate model — errors are hard failures
syscribe -m model validate

# Gate W031 (untreated high-risk threats) and W015 (coverage gaps for critical reqs)
syscribe -m model validate --deny W031,W015

# Matrix for drift detection (non-zero exit if errors)
syscribe -m model matrix --gaps-only
```

### 8.2 Named gate profiles in `.syscribe.toml`

Define a named policy that captures your project's gate rules:

```toml
# .syscribe.toml

[profiles.sil4]
magicgrid = false
promote = ["W015", "W031", "W033", "W300"]   # these warnings become hard failures

[profiles.magicgrid]
magicgrid = true
promote = ["W307", "W300"]
```

Run the gate:

```bash
syscribe -m model validate --profile sil4     # SIL4 project gate
syscribe -m model validate --profile magicgrid  # MagicGrid project gate
```

### 8.3 Safety-readiness dashboard

`audit` gives a one-command readiness summary:

```bash
syscribe -m model audit
```

It reports: requirement status distribution (approved/draft per package), SIL/ASIL
distribution, per-configuration coverage %, orphan requirements, and a PASS/FAIL verdict.
This is the first thing to run at the start of a certification sprint to understand what
is missing.

### 8.4 Per-configuration certification gate

```bash
# Validate a specific product configuration
syscribe -m model validate --config CONF-RP-PICO2-HIL

# Validate every stored configuration; fail if any has errors
syscribe -m model validate --all-configs
```

Stored `Configuration` elements (`type: Configuration`) define feature selections. The
`--config` flag projects the model onto that variant — elements with `appliesWhen:` that
don't hold in this configuration disappear from the view, and escaping cross-references
are flagged (E226).

### 8.5 Persisting evidence

The IEC 61508 / ISO 26262 audit trail requires evidence that analyses were run and
produced specific outputs. Use `--json` to capture structured output:

```bash
syscribe -m model validate --json > artifacts/validation-$(git describe --tags).json
syscribe -m model metrics --json > artifacts/metrics-$(git describe --tags).json
syscribe -m model matrix --json > artifacts/matrix-$(git describe --tags).json
```

Store these under version control or in your artifact repository. The model at the same
git commit always reproduces the same output, so the `git describe` tag in the filename
ties the evidence to the exact model state.

### 8.6 Linking to external tools with `extRef`

If your model elements correspond to items in DOORS, Jira, or another tool, record the
external reference:

```yaml
extRef: "DOORS://UAV_Safety_Requirements#REQ-UAV-SAFE-001"
```

Look up all elements referencing a DOORS ID:

```bash
syscribe -m model extref "DOORS://UAV_Safety_Requirements#REQ-UAV-SAFE-001"
```

This is the integration point for hybrid toolchains where syscribe coexists with DOORS
or Polarion during a migration phase.

### 8.7 Safe refactoring with `move`

Renaming a package or element in a traditional tool is risky because links use opaque IDs.
In syscribe, QNames are path-derived — a directory rename changes all QNames. Use the
`move` command to safely rename and rewrite all references atomically:

```bash
# Preview what would change
syscribe -m model move Safety::FTA::FT-KERNEL-001 Safety::FaultTrees::FT-KERNEL-001 --dry-run

# Execute the move
syscribe -m model move Safety::FTA::FT-KERNEL-001 Safety::FaultTrees::FT-KERNEL-001
```

`move` updates every `derivedFrom:`, `verifies:`, `children:`, and other reference field
in the model. Run `validate` afterwards to confirm no dangling references remain.

---
