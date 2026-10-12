tc_TRS_PORTTYPE_001() {
    local F="$1"
    local G="$F/TC-TRS-PORTTYPE-001/good"
    local B="$F/TC-TRS-PORTTYPE-001/bad"

    SCENARIO_NAME="a sender and a conjugated receiver typed by one PortDef are clean"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$G" validate 2>/dev/null || true)
    assert_no_code "E932"
    assert_no_code "E935"
    assert_no_code "W047"

    SCENARIO_NAME="each event and payload mismatch is reported"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$B" validate 2>/dev/null || true)
    assert_count "E932" 4
    assert_output_contains "carries no Bogus (it carries Sample, Trip)"
    assert_output_contains "carries Trip inward"
    assert_output_contains "with payload field 'x', which Sample does not have"
    assert_output_contains "uses 's.okay', but Sample has no field 'okay'"
    assert_count "E935" 2
    assert_output_contains "payload 'v' of Sample is a number"
    assert_output_contains "\`last := s.ok\` assigns a boolean to a number"
}
