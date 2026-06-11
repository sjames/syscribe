tc_TRS_MG_012() {
    local F="$1"; local FX="$F/TC-TRS-MG-012"
    local out rc

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    # 1. colliding final-segment bindings make the cell n/a
    _scn "two bindings sharing a final segment make the cell n/a"
    out=$("$SYSCRIBE" -m "$FX/ambiguous" trade-study 2>&1) && rc=0 || rc=$?
    { [ "$rc" -eq 0 ] && printf '%s' "$out" | grep -q 'n/a'; } \
        && pass "ambiguous bare-token match reported n/a" || fail "ambiguous match not n/a (rc=$rc)"

    # 2. an exact key match wins despite the collision
    _scn "an exact key match wins despite a colliding segment"
    out=$("$SYSCRIBE" -m "$FX/exact" trade-study 2>&1) && rc=0 || rc=$?
    { [ "$rc" -eq 0 ] && printf '%s' "$out" | grep -q '99' && ! printf '%s' "$out" | grep -q 'n/a'; } \
        && pass "exact speed binding resolves to 99, not n/a" || fail "exact key did not win (rc=$rc)"

    # 3. a single segment match still resolves
    _scn "a single segment match still resolves"
    out=$("$SYSCRIBE" -m "$FX/single" trade-study 2>&1) && rc=0 || rc=$?
    { [ "$rc" -eq 0 ] && printf '%s' "$out" | grep -q '30' && ! printf '%s' "$out" | grep -q 'n/a'; } \
        && pass "single segment match resolves to 30" || fail "single segment match failed (rc=$rc)"
}
