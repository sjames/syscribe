tc_TRS_EXTREF_002() {
    local F="$1"; local B="$F/TC-TRS-EXTREF-002"; local M="$B/model"

    # extref finds the element declaring a reference, exit 0.
    SCENARIO_NAME="extref finds the element declaring a reference"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$M" extref "DNG:4521" 2>/dev/null) && ec=0 || ec=$?
    printf '%s' "$SCENARIO_OUTPUT" | grep -qF "Engine" && pass "Engine printed" || fail "Engine not printed"
    [ "${ec:-0}" -eq 0 ] && pass "exit 0 on a hit" || fail "exit ${ec} (expected 0)"

    # extref returns all elements sharing a duplicate reference.
    SCENARIO_NAME="extref returns all elements sharing a duplicate reference"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$M" extref "DUP:1" 2>/dev/null || true)
    printf '%s' "$SCENARIO_OUTPUT" | grep -qF "A" && printf '%s' "$SCENARIO_OUTPUT" | grep -qF "B" \
        && pass "both A and B printed" || fail "did not print both matches"

    # extref on an unknown reference: no match, exit non-zero.
    SCENARIO_NAME="extref on an unknown reference reports no match and exits non-zero"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$M" extref "NOPE:0" 2>/dev/null) && ec=0 || ec=$?
    [ "${ec:-0}" -ne 0 ] && pass "exit non-zero on a miss" || fail "exit 0 (expected non-zero)"

    # extref --json emits an array containing the element.
    SCENARIO_NAME="extref --json emits an array of matches"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$M" extref "DNG:4521" --json 2>/dev/null || true)
    printf '%s' "$SCENARIO_OUTPUT" | grep -qF "[" && printf '%s' "$SCENARIO_OUTPUT" | grep -qF "Engine" \
        && pass "JSON array contains the element" || fail "JSON output missing array or element"

    # show surfaces extRef.
    SCENARIO_NAME="show surfaces extRef on an element that declares it"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$M" show "Engine" 2>/dev/null || true)
    printf '%s' "$SCENARIO_OUTPUT" | grep -qF "DNG:4521" && pass "extRef shown by show" || fail "extRef not shown"

    # spec fields lists extRef.
    SCENARIO_NAME="spec fields lists extRef"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" spec fields 2>/dev/null || true)
    printf '%s' "$SCENARIO_OUTPUT" | grep -qF "extRef" && pass "extRef in spec fields" || fail "extRef missing from spec fields"
}
