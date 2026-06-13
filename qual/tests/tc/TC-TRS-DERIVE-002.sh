tc_TRS_DERIVE_002() {
    local F="$1"

    SCENARIO_NAME="sum(children.custom_fields.wcet) aggregates child values"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    out=$("$SYSCRIBE" -m "$F/TC-TRS-DERIVE-002/model" show Sys::Parent 2>&1 || true)
    printf '%s' "$out" | grep -qE "totalWcet.*50" \
        && pass "totalWcet = 50 (30+20)" \
        || fail "totalWcet aggregation incorrect or missing"

    SCENARIO_NAME="count(children) returns number of direct children"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    printf '%s' "$out" | grep -qE "childCount.*2" \
        && pass "childCount = 2" \
        || fail "childCount incorrect or missing"
}
