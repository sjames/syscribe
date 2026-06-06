tc_TRS_VAR_005() {
    local F="$1"; local M="$F/TC-TRS-VAR-005/model"

    run_scenario "per-configuration coverage emits W015" "$M"
    assert_has_code "W015"

    # W015 line helper: true if a W015 finding mentions the given requirement id.
    _w015_names() { printf '%s' "$SCENARIO_OUTPUT" | grep -F "| W015 |" | grep -qF "$1"; }

    SCENARIO_NAME="W015 names the active uncovered requirement and its configuration"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    if printf '%s' "$SCENARIO_OUTPUT" | grep -F "| W015 |" | grep -F "REQ-V5-WDT-002" | grep -qF "CONF-MPS2-WDT-001"; then
        pass "W015 names REQ-V5-WDT-002 and CONF-MPS2-WDT-001"
    else
        fail "W015 does not name both REQ-V5-WDT-002 and CONF-MPS2-WDT-001"
    fi

    SCENARIO_NAME="covered requirement yields no W015"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    _w015_names "REQ-V5-WDT-001" && fail "covered REQ-V5-WDT-001 wrongly flagged" || pass "covered requirement not flagged"

    SCENARIO_NAME="draft requirement is not flagged"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    _w015_names "REQ-V5-WDT-003" && fail "draft REQ-V5-WDT-003 wrongly flagged" || pass "draft requirement not flagged"

    SCENARIO_NAME="draft TestCase does not count as coverage"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    _w015_names "REQ-V5-WDT-004" && pass "REQ-V5-WDT-004 flagged (draft test not coverage)" || fail "REQ-V5-WDT-004 not flagged"

    SCENARIO_NAME="W015 is gateable with --deny"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    "$SYSCRIBE" -m "$M" validate --deny W015 >/dev/null 2>&1 && SCENARIO_EXIT=0 || SCENARIO_EXIT=$?
    [ "$SCENARIO_EXIT" -eq 2 ] && pass "--deny W015 exits 2" || fail "exit $SCENARIO_EXIT (expected 2)"

    # Scenario: dormant model emits no W015.
    run_scenario "dormant model emits no W015" "$F/TC-TRS-VAR-005/flat"
    assert_no_code "W015"
}
