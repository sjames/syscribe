tc_TRS_IMPL_003() {
    local F="$1"; local M="$F/TC-TRS-IMPL-003/model"
    local out sbom w023
    run_scenario "registry references raise no W023" "$M"
    w023=$(grep -F "| W023 |" <<<"$SCENARIO_OUTPUT" || true)
    grep -qF "Runtime.md" <<<"$w023" && fail "W023 wrongly flags registry references: $w023" || pass "no W023 for Runtime (crates.io/npm/github)"

    _flush_scenario; SCENARIO_NAME="a missing local path still raises W023"; _SCEN_PASS=0; _SCEN_FAIL=0
    printf "  ▶ %s\n" "$SCENARIO_NAME"
    assert_count "W023" 1
    grep -qF "Ghost.md" <<<"$w023" && pass "W023 names Ghost" || fail "W023 does not name Ghost"

    _flush_scenario; SCENARIO_NAME="sbom still maps the registry references to purls"; _SCEN_PASS=0; _SCEN_FAIL=0
    printf "  ▶ %s\n" "$SCENARIO_NAME"
    sbom=$("$SYSCRIBE" -m "$M" sbom 2>/dev/null) || true
    for p in "pkg:cargo/tokio@1.38.0" "pkg:npm/lodash@4.17.21" "pkg:github/org/repo@v1"; do
        grep -qF "$p" <<<"$sbom" && pass "sbom has $p" || fail "sbom lacks $p"
    done
}
