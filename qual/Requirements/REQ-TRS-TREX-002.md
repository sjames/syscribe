---
id: REQ-TRS-TREX-002
type: Requirement
name: The export can be projected onto a configuration, omitting inactive elements and recording the configuration
status: verified
reqDomain: software
verificationMethod: test
---

With `--config <C>` (a stored `Configuration` id or qualified name, or an ad-hoc
`Features::A,Features::B` set — a `FEAT-*` id is accepted and normalised) the export **shall**
project the model exactly as `trace --config` and `export --config` do: requirements,
satisfiers, verifiers, parents and children that are inactive in `C` are omitted from every
list, and `coverage` is computed over the projected lists. The document's `config` field
**shall** then carry `{ "id", "qname", "name", "activeFeatures": [qualified names] }` (`id`
and `qname` null for an ad-hoc set, whose `name` is the argument as written). An unresolvable
or invalid configuration **shall** be a usage error: a message on stderr naming it, nothing on
stdout, exit 1. Without `--config`, `config` **shall** be `null` and nothing is filtered.

**Source:** `REQ-TRS-TREX-002` (product model).

**Acceptance criteria:** on the `REQ-TRS-TREX-001` model with a `FeatureDef` `Features::Opt`,
one child requirement and its TestCases gated by `appliesWhen: Features::Opt`, and
`CONF-TX-001` deselecting it: (a) without `--config` the `config` field is null and all four
requirements are listed; (b) `--config CONF-TX-001` (by id or by qualified name) omits the
gated requirement from the list and from the parent's `derivedChildren`, records
`{CONF-TX-001, Configurations::CONF-TX-001, <name>, []}` and the summary drops to `{3, 2, 1,
1, 1}`; (c) `--config Features::Opt` records `{null, null, "Features::Opt",
["Features::Opt"]}` and filters nothing; (d) `--config CONF-NOPE-001` exits 1 with the
configuration named on stderr and nothing on stdout.
