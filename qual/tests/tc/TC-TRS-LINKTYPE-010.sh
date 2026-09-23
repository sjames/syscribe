tc_TRS_LINKTYPE_010() {
    local F="$1"; local FX="$F/TC-TRS-LINKTYPE-010"
    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }
    # has <output> <file-substring> <code>  — a finding line mentioning both
    has() { printf '%s' "$1" | grep -F -- "$2" | grep -qE -- "$3"; }

    local M="$F/TC-TRS-LINKTYPE-007/model" out
    local init='{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"qual","version":"1"}}}'
    out=$(printf '%s\n' "$init" \
      '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' \
      '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"link_types","arguments":{}}}' \
      '{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"follow","arguments":{"element":"REQ-LT7-001","link":"mitigates","transitive":true}}}' \
      | timeout 20 "$SYSCRIBE" -m "$M" mcp --read-only 2>/dev/null)
    _scn "tools/list advertises link_types and follow"
    printf '%s' "$out" | jq -se '.[]|select(.id==2)|[.result.tools[].name]|(index("link_types")!=null and index("follow")!=null)' >/dev/null && pass "both tools listed" || fail "tools not listed"
    _scn "link_types returns the declared vocabulary"
    printf '%s' "$out" | jq -se '.[]|select(.id==3)|.result|tostring|test("mitigatedBy")' >/dev/null && pass "link_types data" || fail "link_types wrong"
    _scn "follow returns traversal results"
    printf '%s' "$out" | jq -se '.[]|select(.id==4)|.result|tostring|test("REQ-LT7-003")' >/dev/null && pass "follow data" || fail "follow wrong"
}
