tc_TRS_ALLOC_001() {
    local F="$1"; local FX="$F/TC-TRS-ALLOC-001"
    local out

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }
    _gate() { "$SYSCRIBE" -m "$1" validate --profile magicgrid 2>&1 || true; }

    # 1. allocatedTo on a W2 function clears MG081
    _scn "allocatedTo on a W2 function clears MG081"
    out=$(_gate "$FX/form1")
    printf '%s' "$out" | grep -q 'MG081' \
        && fail "MG081 raised despite allocatedTo" || pass "MG081 cleared by allocatedTo-on-source"

    # 2. allocatedTo on a logical part clears MG041
    _scn "allocatedTo on a logical part clears MG041"
    out=$(_gate "$FX/form1")
    printf '%s' "$out" | grep -q 'MG041' \
        && fail "MG041 raised despite allocatedTo" || pass "MG041 cleared by allocatedTo-on-source"

    # 3. the allocatedTo edge appears in the matrix
    _scn "the allocatedTo edge appears in the matrix"
    out=$("$SYSCRIBE" -m "$FX/form1" matrix --allocations 2>&1 || true)
    { printf '%s' "$out" | grep -q 'Func' && printf '%s' "$out" | grep -q 'Logical'; } \
        && pass "form-1 edge present in matrix" || fail "allocatedTo edge missing from matrix"

    # 4. allocatedFrom is derived on the target
    _scn "allocatedFrom is derived on the target"
    out=$("$SYSCRIBE" -m "$FX/form1" show Logical 2>&1 || true)
    printf '%s' "$out" | grep -q 'Func' \
        && pass "derived allocatedFrom lists Func on Logical" || fail "derived allocatedFrom not surfaced in show"

    # 5. a legacy features entry without type is still an edge (parity)
    _scn "a legacy features entry without type is still an edge"
    out=$(_gate "$FX/form2")
    local mat; mat=$("$SYSCRIBE" -m "$FX/form2" matrix --allocations 2>&1 || true)
    { ! printf '%s' "$out" | grep -q 'MG081' && ! printf '%s' "$out" | grep -q 'MG041' \
        && printf '%s' "$mat" | grep -q 'Func' && printf '%s' "$mat" | grep -q 'Logical'; } \
        && pass "type-less legacy features are edges (MG041/MG081 clear, matrix shows them)" \
        || fail "type-less legacy features not recognised as edges"

    # 6. an unresolved allocatedTo raises E503
    _scn "an unresolved allocatedTo raises E503"
    out=$("$SYSCRIBE" -m "$FX/e503" validate 2>&1 || true)
    printf '%s' "$out" | grep -q 'E503' \
        && pass "E503 raised for unresolved allocatedTo" || fail "E503 not raised"

    # 7. the same edge in both forms raises the redundancy warning
    _scn "the same edge in both forms raises W503"
    out=$("$SYSCRIBE" -m "$FX/redundant" validate 2>&1 || true)
    printf '%s' "$out" | grep -q 'W503' \
        && pass "W503 raised for a redundant duplicate edge" || fail "W503 not raised"
}
