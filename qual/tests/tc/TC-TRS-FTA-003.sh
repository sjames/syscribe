tc_TRS_FTA_003() {
    local F="$1"; local M="$F/TC-TRS-FTA-003/model"; local C="$F/TC-TRS-FTA-003/ccf"
    local o

    SCENARIO_NAME="cut sets and top probability follow the gate logic"; _SCEN_PASS=0; _SCEN_FAIL=0
    printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$M" fault-tree analyze FT-FA3-001 2>/dev/null) && SCENARIO_EXIT=0 || SCENARIO_EXIT=$?
    assert_exit_zero
    assert_output_contains "| 1 | 1 | FTE-FA3-003 |"
    assert_output_contains "| 2 | 2 | FTE-FA3-001, FTE-FA3-002 |"
    assert_output_contains "6.900e-2"

    SCENARIO_NAME="--json output"; _SCEN_PASS=0; _SCEN_FAIL=0
    printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$M" fault-tree analyze FT-FA3-001 --json 2>/dev/null) && SCENARIO_EXIT=0 || SCENARIO_EXIT=$?
    assert_output_contains "\"cutSets\""
    assert_output_contains "\"topProbability\""
    assert_output_contains "\"fussellVesely\""

    SCENARIO_NAME="beta-factor CCF adds an order-1 cut set; --no-ccf removes it"; _SCEN_PASS=0; _SCEN_FAIL=0
    printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$C" fault-tree analyze FT-FC3-001 2>/dev/null) && SCENARIO_EXIT=0 || SCENARIO_EXIT=$?
    assert_output_contains "CCF:PAIR"
    o=$("$SYSCRIBE" -m "$C" fault-tree analyze FT-FC3-001 --no-ccf 2>/dev/null || true)
    grep -qF "CCF:PAIR" <<<"$o" && fail "--no-ccf still reports CCF:PAIR" || pass "--no-ccf drops the CCF event"

    SCENARIO_NAME="unknown fault tree exits non-zero"; _SCEN_PASS=0; _SCEN_FAIL=0
    printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$M" fault-tree analyze FT-NOPE-001 2>&1) && SCENARIO_EXIT=0 || SCENARIO_EXIT=$?
    assert_exit_nonzero
}
