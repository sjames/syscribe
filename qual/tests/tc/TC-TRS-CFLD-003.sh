tc_TRS_CFLD_003() {
    local F="$1"; local M="$F/TC-TRS-CFLD-003/model"
    local out

    SCENARIO_NAME="show renders custom fields when present"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    out=$("$SYSCRIBE" -m "$M" show Engine 2>/dev/null || true)
    printf '%s' "$out" | grep -qi "custom" && pass "show has a custom-fields section" \
        || fail "show missing custom-fields section"
    printf '%s' "$out" | grep -q "Bosch" && pass "show prints a scalar value" || fail "scalar value missing"
    printf '%s' "$out" | grep -q "A-1001" && pass "show prints a list value" || fail "list value missing"

    SCENARIO_NAME="show omits the section when absent"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    out=$("$SYSCRIBE" -m "$M" show Wheel 2>/dev/null || true)
    printf '%s' "$out" | grep -qF "## Custom Fields" && fail "custom section shown when absent" \
        || pass "no custom section for an element without custom fields"
}
