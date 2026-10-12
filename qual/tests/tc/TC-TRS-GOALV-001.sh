tc_TRS_GOALV_001() {
    local F="$1/TC-TRS-GOALV-001"

    SCENARIO_NAME="rate-only verdict"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$F/rate" metrics 2>&1 || true)
    assert_output_contains "| SG-SYS-001 | — | 2 | continuous | n/a | n/a | PFH 2.996e-7 /h | 1e-6 (built-in default) | rate only (top event / mission time) | pass (unverified rates) |"

    SCENARIO_NAME="demand mode"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$F/demand" metrics 2>&1 || true)
    assert_output_contains "| demand | n/a | n/a | PFDavg 1.314e-3 | 1e-2 (built-in default) | cut sets (PFDavg) |"

    SCENARIO_NAME="implausible claimed rate"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$F/implausible" validate 2>&1 || true)
    assert_output_contains "'FTE-SYS-001' failureRate 1.000e-4 /h (verified) fails with probability 0.63"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$F/implausible" metrics 2>&1 || true)
    assert_output_contains "| not credible |"

    SCENARIO_NAME="tree versus campaign"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$F/campaign" inject TC-SYS-001 --ft FT-SYS-001 --consistency --report-only 2>&1 || true)
    assert_output_contains "FTE-SYS-001                                      1    9.995e-4  defeated             confirmed"
    assert_output_contains "FTE-SYS-002                                      1    9.995e-4  survived             conservative or a model gap"
    assert_output_contains "FM-TX-004                                        1           —  defeated             missing from the tree"
}
