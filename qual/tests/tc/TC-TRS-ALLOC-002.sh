tc_TRS_ALLOC_002() {
    local F="$1"; local D="$F/TC-TRS-ALLOC-002"
    local out

    run_scenario "allocatedTo on the deployment package clears E314" "$D/e314-allocatedto"
    assert_no_code "E314"
    assert_exit_zero

    run_scenario "a features-form Allocation element clears E314" "$D/e314-features"
    assert_no_code "E314"
    assert_exit_zero

    run_scenario "a legacy authored allocatedFrom on the hardware target clears E314" "$D/e314-legacy"
    assert_no_code "E314"
    assert_exit_zero

    run_scenario "an allocation to a software element still raises E314" "$D/e314-software-target"
    assert_has_code "E314"
    assert_exit_nonzero

    run_scenario "W034 sees the sources of standalone Allocation elements" "$D/w034-elements"
    assert_has_code "W034"
    grep -F "| W034 |" <<<"$SCENARIO_OUTPUT" | grep -qF "allocation target 'Ecu'" \
        && pass "W034 names the shared ECU" || fail "W034 does not name the shared ECU"
    grep -F "| W034 |" <<<"$SCENARIO_OUTPUT" | grep -qF "'SafetyCore' (D)" \
        && pass "W034 names SafetyCore" || fail "W034 does not name SafetyCore"
    grep -F "| W034 |" <<<"$SCENARIO_OUTPUT" | grep -qF "'Infotainment' (B)" \
        && pass "W034 names Infotainment" || fail "W034 does not name Infotainment"

    _flush_scenario
    SCENARIO_NAME="a legacy authored allocatedFrom reaches the matrix and W503"; _SCEN_PASS=0; _SCEN_FAIL=0
    printf "  ▶ %s\n" "$SCENARIO_NAME"
    out=$("$SYSCRIBE" -m "$D/legacy-matrix" matrix --allocations --json 2>&1 || true)
    grep -qF '"source": "Compute"' <<<"$out" && grep -qF '"target": "Controller"' <<<"$out" \
        && pass "matrix --allocations lists the Compute → Controller edge" \
        || fail "matrix --allocations misses the legacy edge: $out"
    out=$("$SYSCRIBE" -m "$D/legacy-redundant" validate 2>&1 || true)
    grep -F "| W503 |" <<<"$out" | grep -qF "an allocatedTo and an authored allocatedFrom on the target" \
        && pass "W503 names both forms" || fail "W503 missing for the legacy + allocatedTo duplicate"
}
