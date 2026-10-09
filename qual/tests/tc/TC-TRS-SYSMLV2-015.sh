tc_TRS_SYSMLV2_015() {
    local F="$1"; local M="$F/TC-TRS-SYSMLV2-015/model"

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    local out; out=$("$SYSCRIBE" -m "$M" validate 2>&1 || true)

    _scn "a non-redeclared two-segment endpoint raises no W542 and resolves"
    local w542_count; w542_count=$(printf '%s' "$out" | grep -c 'W542' || true)
    # Only Inherited's a.p1/b.p1 truncate -- Resolved (redeclared), Bare
    # (undotted), and ThreeSegment (three-plus segments, its own separate
    # deliberately-unwarned fallback) all contribute none.
    [ "$w542_count" -eq 0 ] && pass "no W542 raised for connect endpoints" \
        || fail "W542 count=$w542_count (expected 0): $out"
    printf '%s' "$out" | grep 'W056' | grep -q "Inherited" \
        && fail "Inherited's a.p1/b.p1 should resolve through the inherited ports: $out" \
        || pass "Inherited::a::p1 / b::p1 resolve (no W056)"

    _scn "a redeclared two-segment endpoint raises no W542"
    printf '%s' "$out" | grep 'W542' | grep -q "fooProvider" \
        && fail "unexpected W542 for the redeclared fooProvider/fooClient endpoints: $out" \
        || pass "no W542 for the redeclared Resolved::a::fooProvider/b::fooClient edge"

    _scn "a bare endpoint raises no W542 and a three-segment endpoint raises no W542"
    # Covered by the count==0 assertion above; restated as its own scenario for TVR traceability.
    [ "$w542_count" -eq 0 ] && pass "Bare and ThreeSegment contribute no W542" \
        || fail "W542 count=$w542_count includes an unexpected Bare/ThreeSegment finding"
}
