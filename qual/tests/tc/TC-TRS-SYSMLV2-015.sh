tc_TRS_SYSMLV2_015() {
    local F="$1"; local M="$F/TC-TRS-SYSMLV2-015/model"

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    local out; out=$("$SYSCRIBE" -m "$M" validate 2>&1 || true)

    _scn "a non-redeclared two-segment endpoint raises W542 for each truncated end"
    local w542_count; w542_count=$(printf '%s' "$out" | grep -c 'W542' || true)
    # Only Inherited's a.p1/b.p1 truncate -- Resolved (redeclared), Bare
    # (undotted), and ThreeSegment (three-plus segments, its own separate
    # deliberately-unwarned fallback) all contribute none.
    [ "$w542_count" -eq 2 ] && pass "exactly two W542 raised (Inherited's a.p1, b.p1)" \
        || fail "W542 count=$w542_count (expected 2): $out"
    printf '%s' "$out" | grep 'W542' | grep -q "'a.p1'" \
        && pass "one W542 identifies endpoint 'a.p1'" || fail "no W542 for 'a.p1': $out"
    printf '%s' "$out" | grep 'W542' | grep -q "'b.p1'" \
        && pass "one W542 identifies endpoint 'b.p1'" || fail "no W542 for 'b.p1': $out"

    _scn "a redeclared two-segment endpoint raises no W542"
    printf '%s' "$out" | grep 'W542' | grep -q "fooProvider" \
        && fail "unexpected W542 for the redeclared fooProvider/fooClient endpoints: $out" \
        || pass "no W542 for the redeclared Resolved::a::fooProvider/b::fooClient edge"

    _scn "a bare endpoint raises no W542 and a three-segment endpoint raises no W542"
    # Already covered by the exact count==2 assertion above (Bare/ThreeSegment
    # would push the count past 2 if either incorrectly raised W542); restated
    # here as its own scenario for TVR traceability.
    [ "$w542_count" -eq 2 ] && pass "Bare and ThreeSegment contribute no W542" \
        || fail "W542 count=$w542_count includes an unexpected Bare/ThreeSegment finding"
}
