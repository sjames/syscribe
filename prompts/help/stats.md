# stats — corpus-shape digest (fast LLM scan)

## SYNOPSIS
    syscribe -m <root> stats [--json] [--group-by <facet>]
        [--where <predicate>]... [--status <s>] [--tag <t>]
        [--config <C>] [--package-top-n <N>]

## DESCRIPTION
Aggregates the native Requirement population into per-facet histograms plus
coverage and orphan rollups, so an LLM (or a human) can grasp the shape of a
model with tens of thousands of requirements in one call — without reading the
element files. Facets: status, reqDomain, silLevel, asilLevel, top-level
package (top-N + `other`), and tags. Reuses the coverage/matrix computation and
the validator reverse indices, so its numbers agree with `coverage`/`matrix`.

## OPTIONS
    --group-by <facet>   Re-key the primary histogram by <facet> crossed with the
                         top-level package (a per-package histogram under
                         `byPackage`). <facet> ∈ status | reqDomain | silLevel |
                         asilLevel | tags. Unknown facet → exit 1.
    --where <predicate>  Restrict the requirement set (custom-field predicate,
                         e.g. custom.supplier=Bosch). Repeatable (AND).
    --status <s>         Restrict to requirements with status: <s>.
    --tag <t>            Restrict to requirements carrying tag <t>.
    --config <C>         Aggregate only the elements active in a Configuration
                         (id/qname or 'Features::A,Features::B'). Unresolvable → 1.
    --package-top-n <N>  Packages to report before the `other` bucket (default 20).
    --json               Emit the whole rollup as one JSON document.

## OUTPUT (--json)
    { "total", "facets": { status, reqDomain, silLevel, asilLevel, package, tags },
      "coverage": { verified, unverifiedLeaves, parentsMissingIntegration },
      "orphans":  { unverifiedRequirements, unsatisfiedRequirements, untraced, ids } }
Note: `coverage` always reflects the full active model (so it equals `coverage`
/`matrix`); `--where`/`--status`/`--tag` scope only the facet/orphan set.

## EXAMPLES
    syscribe -m model/ stats
    syscribe -m model/ stats --json
    syscribe -m model/ stats --group-by status
    syscribe -m model/ stats --status approved --tag safety
    syscribe -m model/ stats --config CONF-LM3S-QEMU-001

## EXIT CODES
    0  ok    1  usage error (unknown --group-by facet, or unresolvable --config)

## SEE ALSO
    audit, coverage, matrix, list
