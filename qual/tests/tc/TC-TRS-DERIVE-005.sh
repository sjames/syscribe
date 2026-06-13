tc_TRS_DERIVE_005() {
    local F="$1"

    SCENARIO_NAME="Cross-element ref to unknown element emits E502"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    out=$("$SYSCRIBE" -m "$F/TC-TRS-DERIVE-005/model" validate 2>/dev/null || true)
    printf '%s' "$out" | grep -qF "E502" \
        && pass "E502 emitted for unknown element reference" \
        || fail "E502 not emitted for unknown element reference"

    SCENARIO_NAME="E502 message names the missing element"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    printf '%s' "$out" | grep -qF "NonExistent::Thing" \
        && pass "E502 message contains element name" \
        || fail "E502 message does not contain element name"
}
