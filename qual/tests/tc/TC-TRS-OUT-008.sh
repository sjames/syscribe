tc_TRS_OUT_008() {
    local F="$1"
    local SRC="$F/TC-TRS-OUT-008/results"

    # Scenario 1: no ingested results → no W010, clean model.
    run_scenario "no results means no W010" "$SRC"
    assert_no_code "W010"

    # Work on a copy so ingest-results' sidecar does not dirty the fixture.
    local W; W=$(mktemp -d)
    cp -r "$SRC"/. "$W"/

    # Scenario 2: ingest cargo-json → W010 for failed + missing, none for pass.
    SCENARIO_NAME="cargo-json flags failing and missing functions"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    "$SYSCRIBE" -m "$W" ingest-results --format cargo-json "$W/cargo.json" >/dev/null 2>&1
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$W" validate 2>/dev/null) && SCENARIO_EXIT=0 || SCENARIO_EXIT=$?
    assert_has_code "W010"
    assert_count "W010" 2
    printf '%s' "$SCENARIO_OUTPUT" | grep -qF "release_fails" && pass "W010 names the failing function" || fail "W010 missing failing function"
    printf '%s' "$SCENARIO_OUTPUT" | grep -qF "acquire_ok" && fail "passing function wrongly flagged" || pass "passing function not flagged"

    # Scenario 3: failing tests gate CI (exit 2) in an otherwise clean model.
    SCENARIO_NAME="failing tests gate CI with --deny W010"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    "$SYSCRIBE" -m "$W" validate --deny W010 >/dev/null 2>&1 && SCENARIO_EXIT=0 || SCENARIO_EXIT=$?
    [ "$SCENARIO_EXIT" -eq 2 ] && pass "exit 2 under --deny W010" || fail "exit $SCENARIO_EXIT (expected 2)"

    rm -rf "$W"

    # Scenario 4: JUnit XML via ad-hoc --results.
    SCENARIO_NAME="junit results supported via --results"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$SRC" validate --results "$SRC/junit.xml" --format junit 2>/dev/null) && SCENARIO_EXIT=0 || SCENARIO_EXIT=$?
    assert_has_code "W010"
}
