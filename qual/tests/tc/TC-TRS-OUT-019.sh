tc_TRS_OUT_019() {
    local F="$1"
    local M="$F/TC-TRS-OUT-019/proj"

    run_sbom() {
        _flush_scenario
        SCENARIO_NAME="$1"; _SCEN_PASS=0; _SCEN_FAIL=0
        printf "  ▶ %s\n" "$SCENARIO_NAME"
        shift
        SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$M" sbom "$@" 2>/dev/null); SCENARIO_EXIT=$?
    }

    run_sbom "CycloneDX file + package components"
    assert_output_contains "\"CycloneDX\""
    assert_output_contains "\"1.6\""
    assert_output_contains "pkg:cargo/tokio@1.38.0"
    assert_output_contains "scheduler"

    run_sbom "local component links to the requirement"
    assert_output_contains "syscribe://REQ-SBOM-001"

    run_sbom "registry URIs become PURLs"
    assert_output_contains "pkg:npm/lodash@4.17.21"
    assert_output_contains "pkg:github/embedded/embedded-hal@v1.0.0"

    run_sbom "SPDX 2.3 output" --format spdx
    assert_output_contains "SPDX-2.3"
    assert_output_contains "GENERATED_FROM"

    run_sbom "--include-tests adds test components" --include-tests
    assert_output_contains "scheduler_test"

    # --output writes a file
    _flush_scenario
    SCENARIO_NAME="--output writes a file"; _SCEN_PASS=0; _SCEN_FAIL=0
    printf "  ▶ %s\n" "$SCENARIO_NAME"
    local tmp="${TMPDIR:-/tmp}/sbom-out-$$.json"
    "$SYSCRIBE" -m "$M" sbom --output "$tmp" >/dev/null 2>&1; SCENARIO_EXIT=$?
    SCENARIO_OUTPUT=$(cat "$tmp" 2>/dev/null); rm -f "$tmp"
    assert_output_contains "bomFormat"
}
