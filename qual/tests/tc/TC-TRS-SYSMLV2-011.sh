tc_TRS_SYSMLV2_011() {
    local F="$1"; local M="$F/TC-TRS-SYSMLV2-011/model"

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    # 1. scoped n2 on a SysMLv2 subtree lists its direct-child parts
    _scn "scoped n2 on a SysMLv2 subtree lists its direct-child parts"
    local out; out=$("$SYSCRIBE" -m "$M" n2 SysML2::Demo::Holder --format json 2>&1)
    local elems; elems=$(printf '%s' "$out" | jq -c '.elements | sort')
    [ "$elems" = '["a","b"]' ] && pass "axis is exactly [a, b]" || fail "axis was: $elems"

    # 2. a lifted connection populates the off-diagonal cell
    _scn "a lifted connection populates the off-diagonal cell"
    local cell; cell=$(printf '%s' "$out" | jq -c '.matrix.a.b // []')
    local has_conn; has_conn=$(printf '%s' "$cell" | jq '[.[] | select(.kind=="Connection")] | length')
    [ "$has_conn" -ge 1 ] && pass "a->b cell names a Connection edge" || fail "a->b cell had no Connection edge: $cell"

    # 3. unscoped n2 is unaffected (still lists every PartDef/Part)
    _scn "unscoped n2 is unaffected"
    local uout; uout=$("$SYSCRIBE" -m "$M" n2 --format json 2>&1)
    local uelems; uelems=$(printf '%s' "$uout" | jq -c '.elements | sort')
    [ "$uelems" = '["Ecu","Holder","a","b"]' ] && pass "unscoped axis lists every PartDef/Part" \
        || fail "unscoped axis was: $uelems"
}
