tc_TRS_MG_006() {
    local F="$1"; local FX="$F/TC-TRS-MG-006"
    local out rc

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    # 1. allocations render as a matrix
    _scn "allocations render as a matrix (sources x targets)"
    out=$("$SYSCRIBE" -m "$FX/flat" matrix --allocations 2>&1) && rc=0 || rc=$?
    { [ "$rc" -eq 0 ] && printf '%s' "$out" | grep -q 'ActA' && printf '%s' "$out" | grep -q 'FC'; } \
        && pass "ActA row and FC column present" || fail "allocation matrix not rendered (rc=$rc)"

    # 2. the rollup reports gaps (unallocated source + unused target)
    _scn "the rollup reports an unallocated source and an unused target"
    out=$("$SYSCRIBE" -m "$FX/flat" matrix --allocations 2>&1) || true
    { printf '%s' "$out" | grep -q 'ActC' && printf '%s' "$out" | grep -q 'Sensor'; } \
        && pass "unallocated ActC and unused Sensor reported" || fail "gap rollup missing ActC/Sensor"

    # 3. mg_layer partitions logical to physical
    _scn "mg_layer partitions logical sources from physical targets"
    out=$("$SYSCRIBE" -m "$FX/layered" matrix --allocations 2>&1) && rc=0 || rc=$?
    { [ "$rc" -eq 0 ] && printf '%s' "$out" | grep -q 'LogicalSub' && printf '%s' "$out" | grep -q 'PhysComp' \
        && printf '%s' "$out" | grep -qi 'logical'; } \
        && pass "logical->physical partition rendered" || fail "mg_layer partition missing (rc=$rc)"

    # 4. flat fallback without mg_layer
    _scn "flat fallback when no mg_layer is present"
    out=$("$SYSCRIBE" -m "$FX/flat" matrix --allocations 2>&1) && rc=0 || rc=$?
    { [ "$rc" -eq 0 ] && printf '%s' "$out" | grep -q 'ActA'; } \
        && pass "flat Allocation-derived matrix produced" || fail "flat fallback failed (rc=$rc)"

    # 5. the matrix emits JSON
    _scn "the matrix emits JSON"
    out=$("$SYSCRIBE" -m "$FX/flat" matrix --allocations --json 2>&1) && rc=0 || rc=$?
    { [ "$rc" -eq 0 ] && printf '%s' "$out" | grep -q '"cells"'; } \
        && pass "matrix --allocations --json emits a grid" || fail "allocation matrix JSON malformed (rc=$rc)"
}
