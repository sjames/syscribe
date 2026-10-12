tc_TRS_ID_008() {
    local F="$1"; local B="$F/TC-TRS-ID-008"

    SCENARIO_NAME="descriptive and numeric Configuration ids validate without E200/W042"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$B/valid" validate 2>/dev/null || true)
    assert_no_code "E200"
    assert_no_code "W042"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$B/valid" validate --config CONF-GTI-US-M3 2>&1 || true)
    assert_output_contains "variant is valid"

    SCENARIO_NAME="malformed Configuration ids are still E200"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$B/invalid" validate 2>/dev/null || true)
    assert_count "E200" 4
}
