tc_TRS_DERIVE_001() {
    local F="$1"

    SCENARIO_NAME="Basic derive block evaluates and appears in show output"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    out=$("$SYSCRIBE" -m "$F/TC-TRS-DERIVE-001/model" show Sys::Widget 2>&1 || true)
    printf '%s' "$out" | grep -qF "## Derived Fields" \
        && pass "Derived Fields section present in show output" \
        || fail "Derived Fields section missing from show output"

    SCENARIO_NAME="First derived field (baseValue = 15) is computed"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    printf '%s' "$out" | grep -qE "baseValue.*15" \
        && pass "baseValue = 15 in derived fields" \
        || fail "baseValue not 15 in derived fields"

    SCENARIO_NAME="Second field references first via self.baseValue (doubleBase = 30)"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    printf '%s' "$out" | grep -qE "doubleBase.*30" \
        && pass "doubleBase = 30 in derived fields" \
        || fail "doubleBase not 30 in derived fields"

    SCENARIO_NAME="Validation reports no errors for valid derive block"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    out=$("$SYSCRIBE" -m "$F/TC-TRS-DERIVE-001/model" validate 2>/dev/null || true)
    printf '%s' "$out" | grep -qE "^E[0-9]" \
        && fail "Unexpected errors for valid derive block" \
        || pass "No errors for valid derive block"
}
