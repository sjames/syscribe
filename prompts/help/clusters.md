# clusters — topical clustering (TF-IDF cosine k-means)

## SYNOPSIS
    syscribe -m <root> clusters [--json] [--k <N>] [--type <T>] [--config <C>]

## DESCRIPTION
Groups elements into topical clusters by the similarity of their normative text,
surfacing cross-package themes (e.g. "safety-shutdown" requirements scattered
across subsystems). Each element is a TF-IDF vector; clustering is k-means over
cosine similarity. Deterministic and offline — bag-of-words TF-IDF (no neural
embeddings, no network), with a farthest-first centroid initialisation seeded
from a fixed element order, so runs are reproducible.

## OPTIONS
    --k <N>        Number of clusters (default min(8, element count)). Clamped to
                   the element count; --k 0 → exit 1.
    --type <T>     Element type to cluster (default Requirement).
    --config <C>   Project onto a Configuration before clustering. Bad → exit 1.
    --json         Emit { k, clusters: [ { label, size, members } … ] }.

## EXAMPLES
    syscribe -m model/ clusters --k 6
    syscribe -m model/ clusters --json
    syscribe -m model/ clusters --type Requirement --k 10

## EXIT CODES
    0  ok    1  usage error (--k < 1, or unresolvable --config)

## SEE ALSO
    topics, summarize, search-text
