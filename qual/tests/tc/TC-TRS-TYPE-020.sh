tc_TRS_TYPE_020() {
    local F="$1"
    local B="$F/TC-TRS-TYPE-020"

    run_scenario "well-formed zones/conduit are clean" "$B/clean"
    for c in E950 E951 E952 E953 E954 E955 E956 W950; do assert_no_code "$c"; done

    run_scenario "E950 — zone missing targetSL" "$B/e950"; assert_has_code "E950"
    run_scenario "E951 — bad zone id" "$B/e951"; assert_has_code "E951"
    run_scenario "E954 — conduit endpoint not a zone" "$B/e954"; assert_has_code "E954"
    run_scenario "E956 — part inZone not a zone" "$B/e956"; assert_has_code "E956"
    run_scenario "W950 — SL gap" "$B/w950"; assert_has_code "W950"
    run_scenario "W953 — approved zone with no conduit" "$B/w953"; assert_has_code "W953"

    run_cli() {
        _flush_scenario
        SCENARIO_NAME="$1"; _SCEN_PASS=0; _SCEN_FAIL=0
        printf "  ▶ %s\n" "$SCENARIO_NAME"
        shift
        SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$B/clean" "$@" 2>/dev/null); SCENARIO_EXIT=$?
    }

    run_cli "zones lists the zones" zones
    assert_output_contains "ZN-CL-001"

    run_cli "conduits lists the conduit" conduits
    assert_output_contains "CD-CL-001"

    run_cli "zones --coverage shows the control" zones --coverage
    assert_output_contains "SC-FW-001"

    run_cli "template Zone" template Zone
    assert_output_contains "type: Zone"

    run_cli "template Conduit" template Conduit
    assert_output_contains "type: Conduit"
}
