tc_TRS_DERIVE_003() {
    local F="$1"

    SCENARIO_NAME="Arithmetic on custom_fields: headroom = budget - used = 40"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    out=$("$SYSCRIBE" -m "$F/TC-TRS-DERIVE-003/model" show Sys::Task 2>&1 || true)
    printf '%s' "$out" | grep -qE "headroom.*40" \
        && pass "headroom = 40" \
        || fail "headroom arithmetic incorrect or missing"

    SCENARIO_NAME="Division: ratio = used / budget = 0.6"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    printf '%s' "$out" | grep -qE "ratio.*0\.6" \
        && pass "ratio = 0.6" \
        || fail "ratio division incorrect or missing"

    SCENARIO_NAME="No errors for valid arithmetic derive block"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    out=$("$SYSCRIBE" -m "$F/TC-TRS-DERIVE-003/model" validate 2>/dev/null || true)
    printf '%s' "$out" | grep -qE "^E[0-9]" \
        && fail "Unexpected errors for arithmetic derive block" \
        || pass "No errors for arithmetic derive block"
}
