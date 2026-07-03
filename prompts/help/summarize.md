# summarize — hierarchical content digest (LLM corpus digestion)

## SYNOPSIS
    syscribe -m <root> summarize [--json] [--scope <qname>] [--depth <N>]
        [--no-cache] [--config <C>]

## DESCRIPTION
Bottom-up per-package rollup so an LLM can read a handful of package summaries
instead of every requirement file. For each package (namespace) node: a
requirement count, a status split, an "about" line (TF-IDF distinctive terms),
and the one-line extract of a few representative requirements — nested through
the hierarchy. Deterministic and offline (extractive, not an LLM summary). Each
node's fields are cached under .syscribe/cache/summaries.json keyed by a content
hash of its subtree, so unchanged subtrees are served from cache (incremental).

## OPTIONS
    --scope <qname>  Restrict to the subtree rooted at a package. Unknown → exit 1.
    --depth <N>      Bound the nesting depth reported (counts still roll up fully).
    --no-cache       Bypass and rewrite the cache (recompute everything).
    --config <C>     Project onto a Configuration before summarising. Bad → exit 1.
    --json           Emit the nested { qname, count, statusSplit, terms,
                     representative, children } document.

## EXAMPLES
    syscribe -m model/ summarize
    syscribe -m model/ summarize --scope VehicleSystem --depth 2
    syscribe -m model/ summarize --json

## EXIT CODES
    0  ok    1  usage error (unresolvable --scope or --config)

## SEE ALSO
    stats, topics, clusters, digest
