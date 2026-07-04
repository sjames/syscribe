# digest — one compact line per requirement (LLM bulk scan)

## SYNOPSIS
    syscribe -m <root> digest [--json] [--limit <N>] [--offset <N>]
        [--where <predicate>]... [--status <s>] [--tag <t>] [--config <C>]

## DESCRIPTION
Emits one compact JSON object per native Requirement — id, name, status,
reqDomain, sil/asil (when set), a one-line text summary, and a verified flag —
tuned to ~30 tokens/row so an LLM can bulk-scan a narrowed slice. The "dump the
slice" companion to `stats` (which gives the shape). Distinct from `export`,
which dumps the full element. Default output is NDJSON (one row per line).

## OPTIONS
    --limit <N>          Emit at most N rows (cursor paging).
    --offset <N>         Skip the first N rows.
    --where <predicate>  Restrict the set (custom-field predicate, e.g.
                         custom.k=v). Repeatable (AND).
    --status <s>         Restrict to requirements with status: <s>.
    --tag <t>            Restrict to requirements carrying tag <t>.
    --config <C>         Only requirements active in a Configuration (id/qname or
                         'Features::A,Features::B'). Unresolvable → exit 1.
    --json               Emit one { total, offset, rows } document instead of NDJSON.

## EXAMPLES
    syscribe -m model/ digest
    syscribe -m model/ digest --status approved --limit 100
    syscribe -m model/ digest --where custom.supplier=Bosch --json
    syscribe -m model/ digest --config CONF-LM3S-QEMU-001

## EXIT CODES
    0  ok    1  usage error (unresolvable --config)

## SEE ALSO
    stats, search-text, export, list
