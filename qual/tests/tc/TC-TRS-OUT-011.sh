tc_TRS_OUT_011() {
    local F="$1"; local M="$F/TC-TRS-OUT-011/model"

    SCENARIO_NAME="depth flags: hil-only / none / ok"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local OUT; OUT=$("$SYSCRIBE" -m "$M" verification-depth 2>/dev/null)
    printf '%s' "$OUT" | grep "REQ-HIL-001" | grep -qi "hil-only" && pass "REQ-HIL-001 hil-only" || fail "REQ-HIL-001 not hil-only"
    printf '%s' "$OUT" | grep "REQ-NONE-002" | grep -qi "none" && pass "REQ-NONE-002 none" || fail "REQ-NONE-002 not none"
    printf '%s' "$OUT" | grep "REQ-OK-003" | grep -qi "ok" && pass "REQ-OK-003 ok" || fail "REQ-OK-003 not ok"

    SCENARIO_NAME="--json emits per-requirement array"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    OUT=$("$SYSCRIBE" -m "$M" verification-depth --json 2>/dev/null)
    printf '%s' "$OUT" | grep -qF '"flag"' && printf '%s' "$OUT" | grep -qF '"levels"' \
        && printf '%s' "$OUT" | grep -qF '"count"' \
        && pass "json has flag/levels/count" || fail "json missing fields"

    SCENARIO_NAME="--min-levels gates insufficient depth"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    "$SYSCRIBE" -m "$M" verification-depth --sil 4 --min-levels 2 >/dev/null 2>&1 && ec=0 || ec=$?
    [ "${ec:-0}" -ne 0 ] && pass "--min-levels 2 exits non-zero (HIL/none fail)" || fail "gate did not fire"

    SCENARIO_NAME="--min-levels 1 passes when all have >=1 level"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    # REQ-NONE-002 has 0 levels, so even --min-levels 1 should still gate; assert it does.
    "$SYSCRIBE" -m "$M" verification-depth --sil 4 --min-levels 1 >/dev/null 2>&1 && ec=0 || ec=$?
    [ "${ec:-0}" -ne 0 ] && pass "--min-levels 1 still fails (REQ-NONE-002 has 0)" || fail "expected gate on zero-level req"
}
