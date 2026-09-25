tc_TRS_DERIVE_005() {
    local F="$1" out

    SCENARIO_NAME="Cross-element ref to unknown element emits E506"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    out=$("$SYSCRIBE" -m "$F/TC-TRS-DERIVE-005/model" validate 2>/dev/null || true)
    grep -qF "| E506 |" <<<"$out" \
        && pass "E506 emitted for unknown element reference" \
        || fail "E506 not emitted for unknown element reference"
    grep -qF "| E502 |" <<<"$out" \
        && fail "E502 (the Allocation allocatedFrom code) emitted for a derive reference" \
        || pass "no E502 for a derive reference"

    SCENARIO_NAME="E506 message names the missing element"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    grep -F "| E506 |" <<<"$out" | grep -qF "NonExistent::Thing" \
        && pass "E506 message contains element name" \
        || fail "E506 message does not contain element name"
}
