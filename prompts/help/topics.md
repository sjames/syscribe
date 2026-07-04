# topics — distinctive per-package keywords (TF-IDF)

## SYNOPSIS
    syscribe -m <root> topics [--json] [--top <N>] [--type <T>] [--config <C>]

## DESCRIPTION
Surfaces, per package, the terms that distinguish it from the rest of the corpus
via TF-IDF over the elements' normative text (body + name). Names what each
package is about without reading its contents. Deterministic and offline;
demotes vocabulary common to every package (low IDF).

## OPTIONS
    --top <N>      Terms per package (default 10), ordered by descending score.
    --type <T>     Element type to analyse (default Requirement).
    --config <C>   Project onto a Configuration before computing. Bad → exit 1.
    --json         Emit { packages: { <pkg>: [ {term, score} … ] } }.

## EXAMPLES
    syscribe -m model/ topics
    syscribe -m model/ topics --top 5 --json
    syscribe -m model/ topics --type TestCase

## EXIT CODES
    0  ok    1  usage error (unresolvable --config)

## SEE ALSO
    summarize, clusters, search-text
