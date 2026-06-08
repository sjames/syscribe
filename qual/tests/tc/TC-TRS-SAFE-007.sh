tc_TRS_SAFE_007() {
    local F="$1"
    local D="$F/TC-TRS-SAFE-007"

    # ── W038 fires on a non-draft work product with no responsibility ──────────
    run_scenario "dia: non-draft work product with no responsibility" "$D/dia"
    assert_exit_zero
    assert_has_code "W038"
    assert_output_contains "SensorFusion"

    # ── assigning responsibility everywhere clears W038 ────────────────────────
    run_scenario "dia_clean: every work product has responsibility, no W038" "$D/dia_clean"
    assert_exit_zero
    assert_no_code "W038"

    # ── W039 fires on an ASIL-D goal lacking its I3 FS assessment ──────────────
    run_scenario "confirm: ASIL D goal without I3 functional_safety_assessment" "$D/confirm"
    assert_exit_zero
    assert_has_code "W039"
    assert_output_contains "SG-BRK-001"

    # ── an I3 functional_safety_assessment clears W039 ─────────────────────────
    run_scenario "confirmed: I3 functional_safety_assessment present, no W039" "$D/confirmed"
    assert_exit_zero
    assert_no_code "W039"

    # ── invalid ConfirmationMeasure enums yield E849/E850 ──────────────────────
    run_scenario "badenum: invalid measureType/independenceLevel" "$D/badenum"
    assert_has_code "E849"
    assert_has_code "E850"

    # ── both checks dormant without their practice adopted ─────────────────────
    run_scenario "dormant: no responsibility and no ConfirmationMeasure" "$D/dormant"
    assert_exit_zero
    assert_no_code "W038"
    assert_no_code "W039"

    # ── --deny W038 exits non-zero on the dia model ────────────────────────────
    SCENARIO_NAME="--deny W038 exits non-zero"; _SCEN_PASS=0; _SCEN_FAIL=0
    printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$D/dia" validate --deny W038 2>/dev/null) && SCENARIO_EXIT=0 || SCENARIO_EXIT=$?
    assert_exit_nonzero

    # ── --deny W039 exits non-zero on the confirm model ────────────────────────
    SCENARIO_NAME="--deny W039 exits non-zero"; _SCEN_PASS=0; _SCEN_FAIL=0
    printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$D/confirm" validate --deny W039 2>/dev/null) && SCENARIO_EXIT=0 || SCENARIO_EXIT=$?
    assert_exit_nonzero
}
