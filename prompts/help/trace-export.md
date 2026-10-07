# trace-export — one JSON document of every requirement's traceability

## SYNOPSIS
    syscribe -m <root> trace-export [--config <C>] [--sort <order>] [--out <file>]

## DESCRIPTION
Emits one versioned, machine-readable JSON document listing every requirement in
the model — native `Requirement` and SysML `RequirementDef`/`Requirement` — with
its identity, derivation (`derivedFrom`, `derivedChildren`, `breakdownAdr`),
satisfaction (`satisfiedBy`), verification (`verifiedBy`, with the ingested
verdict when a results sidecar is present), refinements (`refinedBy`) and a
computed `coverage` block, for coverage checks in CI and for external tools
(ADR-SYS-TREX-001). Read-only; never modifies the model.

The lists are the same reverse indices `validate` maintains, and `coverage` is
computed by the same rules `W300`, `W002` and `W305` apply: a leaf (no
`derivedChildren`) is `satisfied` when at least one element satisfies it;
`verified` when at least one `active` TestCase verifies it; `integrationVerified`
when an `active` L3/L4/L5 TestCase verifies it. A `retired` TestCase is listed
but never counts. A user-defined link type that `extends` `satisfies`,
`verifies`, `derivedFrom` or `refines` with `coverage = true` contributes to the
same lists.

Every reference is the element's full qualified name plus its stable id. A
reference that does not resolve is kept as `{ "qname": "<as written>",
"unresolved": true }`, never dropped. Field order within objects is fixed and
the output is byte-identical across runs, so two exports diff cleanly.

### Document

    {
      "version": 1,
      "modelRoot": "<root>",
      "config": null | { "id", "qname", "name", "activeFeatures": [...] },
      "sort": "directory" | "asc" | "desc",
      "requirements": [ <entry>, ... ],
      "summary": { "requirements", "leaves", "satisfied", "verified", "integrationVerified" }
    }

| Entry field | Content |
|---|---|
| `qname`, `id`, `name`, `type` | identity: full qualified name, stable id (or null), label, `Requirement` or `RequirementDef` |
| `status`, `reqClass`, `reqDomain`, `file` | lifecycle status, class, domain, model-root-relative path |
| `derivedFrom`, `derivedChildren` | `[{qname, id}]` — parents (authored) and children (reverse index) |
| `breakdownAdr` | `{qname, id, status}` or null |
| `satisfiedBy` | `[{qname, id, type, domain}]` — every satisfying element |
| `verifiedBy` | `[{qname, id, testLevel, status, verdict}]` — `verdict` is `pass`/`fail`/`unknown`, or null without a results sidecar |
| `refinedBy` | `[{qname, id}]` — use cases / behaviours that `refines:` it |
| `coverage` | `{leaf, satisfied, verified, integrationVerified}` (booleans) |

## OPTIONS
    --config <C>    Project onto a configuration exactly as `export --config` does
                    (a stored Configuration id/qname or an ad-hoc
                    'Features::A,Features::B' set): requirements, satisfiers,
                    verifiers, parents and children inactive in C are omitted
                    from every list and `config` records the configuration and
                    its active features. An unresolvable configuration is a
                    usage error (exit 1). Without it `config` is null.
    --sort <order>  directory (default: the walker's file order, as `export` and
                    `ls` use) | asc | desc (by full qualified name). Applies to
                    the requirement list and every nested list; recorded in the
                    document's `sort` field.
    --out <file>    Write the document to <file> (parent directories are created)
                    instead of stdout.

## EXAMPLES
    syscribe -m model/ trace-export > trace.json
    syscribe -m model/ trace-export --sort asc --out build/trace.json
    syscribe -m model/ trace-export --config CONF-UAV-SURVEY-001
    syscribe -m model/ trace-export | jq '.requirements[] | select(.coverage.leaf and (.coverage.satisfied | not)) | .qname'

## SEE ALSO
    trace, matrix, export, who-verifies, verification-depth, validate
