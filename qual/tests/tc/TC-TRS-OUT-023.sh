tc_TRS_OUT_023() {
    # Shares the stats fixture model (see TC-TRS-OUT-021.sh header). summarize writes
    # .syscribe/cache/ into the model tree, so every run works on a scratch copy.
    local F="$1"; local BASE="$F/TC-TRS-OUT-021/model" out rc W
    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }
    _jq() { printf '%s' "$3" | jq -e "$2" >/dev/null 2>&1 && pass "$1" || fail "$1 — got: $(printf '%s' "$3" | head -c 600)"; }
    local init='{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"qual","version":"1"}}}'
    W=$(mktemp -d); cp -r "$BASE"/. "$W"/; rm -rf "$W/.syscribe"
    local SHAPE='["children","count","qname","representative","statusSplit","terms"]'

    _scn "The digest is a nested per-package rollup"
    local full; full=$("$SYSCRIBE" -m "$W" summarize --json --no-cache 2>/dev/null) || true
    _jq "root node has exactly qname,count,statusSplit,terms,representative,children" "(keys)==$SHAPE" "$full"
    _jq "root qname is (root)" '.qname=="(root)"' "$full"
    _jq "root count = 9 requirements in scope" '.count==9' "$full"
    _jq "root statusSplit approved 4 / draft 4 / other 1 (deprecated)" '.statusSplit=={"approved":4,"draft":4,"other":1}' "$full"
    _jq "children are packages Comms(4), Power(5)" '[.children[]|[.qname,.count]]==[["Comms",4],["Power",5]]' "$full"
    _jq "every child node (any depth) has the same shape" "[..|objects|select(has(\"qname\"))|keys]|all(.==$SHAPE)" "$full"
    _jq "nesting: Power::Cells(1) is a child of Power" '.children[]|select(.qname=="Power")|[.children[]|[.qname,.count]]==[["Power::Cells",1]]' "$full"
    _jq "counts roll up: root 9 = Comms 4 + Power 5" '.count==([.children[].count]|add)' "$full"
    _jq "representative entries are {id,text} one-liners of member requirements" '.children[]|select(.qname=="Comms")|.representative|length>0 and all(.[]; (keys)==["id","text"] and (.text|contains("\n")|not))' "$full"
    _jq "Comms representatives are Comms requirements" '.children[]|select(.qname=="Comms")|[.representative[].id]|all(startswith("REQ-Q21-01"))' "$full"

    _scn "About-terms are content words, not stopwords"
    _jq "root terms non-empty" '(.terms|length)>0' "$full"
    local sw
    for sw in the shall and of to a when; do
        _jq "no stopword '$sw' in any node's terms" "[..|objects|select(has(\"terms\"))|.terms[]]|index(\"$sw\")==null" "$full"
    done
    _jq "Power's top term is the content word 'battery'" '.children[]|select(.qname=="Power")|.terms[0]=="battery"' "$full"

    _scn "Output is deterministic and cached"
    rm -rf "$W/.syscribe"
    local a b c
    a=$("$SYSCRIBE" -m "$W" summarize 2>/dev/null) || true
    [ -f "$W/.syscribe/cache/summaries.json" ] && pass "cache .syscribe/cache/summaries.json exists after first run" || fail "cache file missing after first run"
    jq -e 'type=="object" or type=="array"' "$W/.syscribe/cache/summaries.json" >/dev/null 2>&1 && pass "cache file is valid JSON" || fail "cache file not valid JSON"
    b=$("$SYSCRIBE" -m "$W" summarize 2>/dev/null) || true
    [ -n "$a" ] && [ "$a" = "$b" ] && pass "second (cached) run prints identical output" || fail "runs differ"
    printf '%s' "$a" | grep -q 'battery' && pass "text output carries the digest (term 'battery')" || fail "text output lacks digest content: $a"
    c=$("$SYSCRIBE" -m "$W" summarize --json 2>/dev/null) || true
    [ "$(printf '%s' "$c" | jq -S . 2>/dev/null)" = "$(printf '%s' "$full" | jq -S . 2>/dev/null)" ] \
        && pass "cached --json == --no-cache --json" || fail "cached json differs from recomputed"

    _scn "--scope and --config restrict the digest"
    out=$("$SYSCRIBE" -m "$W" summarize --json --config CONF-Q21-001 2>/dev/null) || true
    _jq "--config: root count 8 (gated REQ-Q21-012 excluded)" '.count==8' "$out"
    _jq "--config: Comms count 3, REQ-Q21-012 not a representative" '.children[]|select(.qname=="Comms")|.count==3 and ([.representative[].id]|index("REQ-Q21-012")==null)' "$out"
    out=$("$SYSCRIBE" -m "$W" summarize --json --scope Power 2>/dev/null) || true
    _jq "--scope Power: root is Power with count 5" '.qname=="Power" and .count==5' "$out"
    _jq "--scope Power: no Comms node anywhere" '[..|objects|select(has("qname"))|.qname]|index("Comms")==null' "$out"
    out=$("$SYSCRIBE" -m "$W" summarize --json --depth 1 2>/dev/null) || true
    _jq "--depth 1: Power's children omitted but count still 5" '.children[]|select(.qname=="Power")|(.children|length)==0 and .count==5' "$out"
    "$SYSCRIBE" -m "$W" summarize --scope NoSuchPackage >/dev/null 2>&1 && rc=0 || rc=$?
    [ "$rc" -eq 1 ] && pass "--scope NoSuchPackage exits 1" || fail "--scope NoSuchPackage exit $rc (expected 1)"
    "$SYSCRIBE" -m "$W" summarize --config bogus >/dev/null 2>&1 && rc=0 || rc=$?
    [ "$rc" -eq 1 ] && pass "--config bogus exits 1" || fail "--config bogus exit $rc (expected 1)"

    _scn "The MCP summarize tool matches the CLI"
    out=$(printf '%s\n' "$init" \
        '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' \
        '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"summarize","arguments":{}}}' \
        | timeout 20 "$SYSCRIBE" -m "$W" mcp 2>/dev/null) || true
    printf '%s' "$out" | jq -se '[.[]|select(.id==2)|.result.tools[]|select(.name=="summarize")][0].annotations.readOnlyHint==true' >/dev/null 2>&1 \
        && pass "summarize tool advertised with readOnlyHint true" || fail "summarize tool not advertised read-only"
    local tool cli
    tool=$(printf '%s' "$out" | jq -S 'select(.id==3)|.result.content[0].text|fromjson' 2>/dev/null) || true
    cli=$("$SYSCRIBE" -m "$W" summarize --json 2>/dev/null | jq -S . 2>/dev/null) || true
    [ -n "$tool" ] && [ "$tool" = "$cli" ] && pass "MCP summarize document == summarize --json" || fail "MCP summarize differs: $(printf '%s' "$tool" | head -c 400)"
    rm -rf "$W"
}
