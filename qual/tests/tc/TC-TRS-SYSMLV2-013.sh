tc_TRS_SYSMLV2_013() {
    local F="$1"; local M="$F/TC-TRS-SYSMLV2-013/model"

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    # 1. a redeclared feature resolves to the finer-grained edge
    _scn "a redeclared feature resolves to the finer-grained edge"
    local out; out=$("$SYSCRIBE" -m "$M" connectivity SysML2::Demo::Resolved::a::fooProvider --format json 2>&1)
    local has_edge; has_edge=$(printf '%s' "$out" | jq '[.edges[] | select(.to=="SysML2::Demo::Resolved::b::fooClient" and .kind=="connection")] | length')
    [ "$has_edge" -ge 1 ] && pass "real edge from a::fooProvider to b::fooClient" \
        || fail "no edge from a::fooProvider to b::fooClient found: $out"

    # 2. an inherited-only feature keeps its full path (GH #206)
    _scn "an inherited-only feature keeps its full path"
    local eout; eout=$("$SYSCRIBE" -m "$M" export --ndjson 2>&1)
    local inh_conns; inh_conns=$(printf '%s' "$eout" | jq -c --arg q "SysML2::Demo::Inherited" \
        'select(.qname==$q) | .frontmatter.connections[0]')
    local inh_from; inh_from=$(printf '%s' "$inh_conns" | jq -r '.from')
    local inh_to; inh_to=$(printf '%s' "$inh_conns" | jq -r '.to')
    [ "$inh_from" = "SysML2::Demo::Inherited::a::p1" ] && [ "$inh_to" = "SysML2::Demo::Inherited::b::p1" ] \
        && pass "Inherited's connection keeps the full path (a::p1 -> b::p1)" \
        || fail "Inherited's connection was: from=$inh_from to=$inh_to"

    # 3. a three-segment chain keeps its full path
    _scn "a three-segment chain keeps its full path"
    local three_conns; three_conns=$(printf '%s' "$eout" | jq -c --arg q "SysML2::Demo::ThreeSegment" \
        'select(.qname==$q) | .frontmatter.connections[0]')
    local three_from; three_from=$(printf '%s' "$three_conns" | jq -r '.from')
    [ "$three_from" = "SysML2::Demo::ThreeSegment::a::fooProvider::deep" ] \
        && pass "ThreeSegment's connection keeps the full path (a.fooProvider.deep)" \
        || fail "ThreeSegment's connection from was: $three_from (expected the full path)"
}
