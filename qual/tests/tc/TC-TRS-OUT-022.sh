tc_TRS_OUT_022() {
    # Shares the stats fixture model (9 requirements; see TC-TRS-OUT-021.sh header).
    local F="$1"; local M="$F/TC-TRS-OUT-021/model" out rc
    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }
    _jq() { printf '%s' "$3" | jq -e "$2" >/dev/null 2>&1 && pass "$1" || fail "$1 — got: $(printf '%s' "$3" | head -c 600)"; }
    local init='{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"qual","version":"1"}}}'
    local ALL='["REQ-Q21-001","REQ-Q21-002","REQ-Q21-003","REQ-Q21-010","REQ-Q21-011","REQ-Q21-012","REQ-Q21-013","REQ-Q21C-001","REQ-Q21P-001"]'

    _scn "Default output is one compact NDJSON row per requirement"
    out=$("$SYSCRIBE" -m "$M" digest 2>/dev/null) || true
    local nlines; nlines=$(printf '%s\n' "$out" | grep -c . || true)
    [ "$nlines" -eq 9 ] && pass "9 lines for 9 requirements" || fail "$nlines lines (expected 9)"
    local bad=0 line
    while IFS= read -r line; do
        printf '%s' "$line" | jq -se 'length==1 and (.[0]|type)=="object"' >/dev/null 2>&1 || bad=$((bad+1))
    done <<< "$out"
    [ "$bad" -eq 0 ] && pass "every line is a JSON object on its own" || fail "$bad lines are not standalone JSON objects"
    local rows; rows=$(printf '%s' "$out" | jq -sc . 2>/dev/null) || true
    _jq "row ids = the 9 requirement ids (each once)" "([.[].id]|sort)==$ALL" "$rows"
    _jq "every row carries id,name,status,reqDomain,text,verified" 'all(.[]; has("id") and has("name") and has("status") and has("reqDomain") and has("text") and (.verified|type=="boolean"))' "$rows"
    _jq "verified flag true exactly for REQ-Q21-001 and REQ-Q21-010" '[.[]|select(.verified)|.id]|sort==["REQ-Q21-001","REQ-Q21-010"]' "$rows"
    _jq "row values match frontmatter (REQ-Q21-002)" '.[]|select(.id=="REQ-Q21-002")|.name=="Battery thermal cutoff" and .status=="approved" and .reqDomain=="hardware" and .asil=="B"' "$rows"
    _jq "no text field contains a newline" 'all(.[]; .text|contains("\n")|not)' "$rows"
    _jq "long first line bounded to 200 chars + ellipsis (REQ-Q21-013)" '.[]|select(.id=="REQ-Q21-013")|(.text|length)==201 and (.text|endswith("…"))' "$rows"
    _jq "second body line not leaked into text (REQ-Q21-013)" '.[]|select(.id=="REQ-Q21-013")|.text|contains("Second line")|not' "$rows"
    _jq "short text kept verbatim (REQ-Q21-010)" '.[]|select(.id=="REQ-Q21-010")|.text=="The radio shall transmit telemetry frames at one hertz."' "$rows"

    _scn "--json emits a paged document with a pre-paging total"
    local whole; whole=$("$SYSCRIBE" -m "$M" digest --json 2>/dev/null) || true
    out=$("$SYSCRIBE" -m "$M" digest --json --limit 3 --offset 2 2>/dev/null) || true
    _jq "one document with total, offset, rows" '(keys)==["offset","rows","total"]' "$out"
    _jq "offset = 2" '.offset==2' "$out"
    _jq "rows has 3 entries" '(.rows|length)==3' "$out"
    _jq "total = 9 (full in-scope count, not page)" '.total==9' "$out"
    local want; want=$(printf '%s' "$whole" | jq -c '[.rows[2:5][].id]' 2>/dev/null) || true
    _jq "page = rows 2..4 of the unpaged order" "[.rows[].id]==$want" "$out"
    out=$("$SYSCRIBE" -m "$M" digest --json --limit 3 --offset 7 2>/dev/null) || true
    _jq "tail page truncated to 2 rows, total still 9" '(.rows|length)==2 and .total==9' "$out"

    _scn "Scoping filters restrict the rows and the total"
    out=$("$SYSCRIBE" -m "$M" digest --json --status approved 2>/dev/null) || true
    _jq "total = 4 approved" '.total==4' "$out"
    _jq "every row has status approved" '(.rows|length)==4 and all(.rows[]; .status=="approved")' "$out"
    _jq "approved ids exact" '[.rows[].id]|sort==["REQ-Q21-001","REQ-Q21-002","REQ-Q21-010","REQ-Q21P-001"]' "$out"
    out=$("$SYSCRIBE" -m "$M" digest --json --tag comms 2>/dev/null) || true
    _jq "--tag comms: exactly REQ-Q21-010, REQ-Q21-011" '.total==2 and ([.rows[].id]|sort)==["REQ-Q21-010","REQ-Q21-011"]' "$out"

    _scn "--config projects the rows onto a variant"
    out=$("$SYSCRIBE" -m "$M" digest --json --config CONF-Q21-001 2>/dev/null) || true
    _jq "gated REQ-Q21-012 absent from rows" '[.rows[].id]|index("REQ-Q21-012")==null' "$out"
    _jq "total 8, all other requirements kept" '.total==8 and (.rows|length)==8' "$out"
    "$SYSCRIBE" -m "$M" digest --config bogus >/dev/null 2>&1 && rc=0 || rc=$?
    [ "$rc" -eq 1 ] && pass "--config bogus exits 1" || fail "--config bogus exit $rc (expected 1)"

    _scn "The MCP digest tool returns the same document as the CLI"
    out=$(printf '%s\n' "$init" \
        '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' \
        '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"digest","arguments":{}}}' \
        | timeout 20 "$SYSCRIBE" -m "$M" mcp 2>/dev/null) || true
    printf '%s' "$out" | jq -se '[.[]|select(.id==2)|.result.tools[]|select(.name=="digest")][0].annotations.readOnlyHint==true' >/dev/null 2>&1 \
        && pass "digest tool advertised with readOnlyHint true" || fail "digest tool not advertised read-only"
    local tool cli
    tool=$(printf '%s' "$out" | jq -S 'select(.id==3)|.result.content[0].text|fromjson' 2>/dev/null) || true
    cli=$(printf '%s' "$whole" | jq -S . 2>/dev/null) || true
    [ -n "$tool" ] && [ "$tool" = "$cli" ] && pass "MCP digest document == digest --json" || fail "MCP digest differs: $(printf '%s' "$tool" | head -c 400)"
}
