tc_TRS_SYSMLV2_010() {
    local F="$1"; local M="$F/TC-TRS-SYSMLV2-010/model"

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    # 1. a binary connect lifts and resolves to a real edge
    _scn "a binary connect lifts and resolves to a real edge"
    local out; out=$("$SYSCRIBE" -m "$M" connectivity SysML2::Demo::BinaryHolder::a --format json 2>&1)
    local has_edge; has_edge=$(printf '%s' "$out" | jq '[.edges[] | select(.to=="SysML2::Demo::BinaryHolder::b" and .kind=="connection")] | length')
    [ "$has_edge" -ge 1 ] && pass "real edge from BinaryHolder::a to BinaryHolder::b" \
        || fail "no edge from BinaryHolder::a to BinaryHolder::b found: $out"

    # 2. an n-ary connect lifts to the ends: shape and every end resolves
    _scn "an n-ary connect lifts to the ends: shape and every end resolves"
    local nout; nout=$("$SYSCRIBE" -m "$M" connectivity SysML2::Demo::NaryHolder::a --format json 2>&1)
    local edge_b; edge_b=$(printf '%s' "$nout" | jq '[.edges[] | select(.to=="SysML2::Demo::NaryHolder::b" and .kind=="connection")] | length')
    local edge_c; edge_c=$(printf '%s' "$nout" | jq '[.edges[] | select(.to=="SysML2::Demo::NaryHolder::c" and .kind=="connection")] | length')
    [ "$edge_b" -ge 1 ] && pass "real edge from NaryHolder::a to NaryHolder::b" \
        || fail "no edge from NaryHolder::a to NaryHolder::b found: $nout"
    [ "$edge_c" -ge 1 ] && pass "real edge from NaryHolder::a to NaryHolder::c" \
        || fail "no edge from NaryHolder::a to NaryHolder::c found: $nout"

    # 3. a connection usage with no connect clause contributes no entry
    _scn "a connection usage with no connect clause contributes no entry"
    local eout; eout=$("$SYSCRIBE" -m "$M" export --ndjson 2>&1)
    local conns; conns=$(printf '%s' "$eout" | jq -c --arg q "SysML2::Demo::NoConnectHolder" \
        'select(.qname==$q) | .frontmatter.connections')
    [ "$conns" = "null" ] && pass "NoConnectHolder carries no connections: field" \
        || fail "NoConnectHolder unexpectedly carries connections:: $conns"
}
