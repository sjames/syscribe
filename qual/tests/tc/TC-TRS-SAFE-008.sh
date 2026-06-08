tc_TRS_SAFE_008() {
    local F="$1"
    local D="$F/TC-TRS-SAFE-008"

    # ── the GSN argument-layer types validate clean in main/ ───────────────────
    run_scenario "main: Argument + AssumptionOfUse validate with no errors" "$D/main"
    assert_exit_zero
    assert_no_code "E852"
    assert_no_code "E853"
    assert_no_code "E854"
    assert_no_code "E855"
    assert_no_code "E856"
    assert_no_code "E857"
    assert_no_code "E858"

    # ── safety-case text shows the goal, its strategy/claim/evidence and the AoU ─
    printf "  ▶ %s\n" "safety-case text shows the GSN tree and the AoU"
    local out rc
    out=$("$SYSCRIBE" -m "$D/main" safety-case 2>/dev/null) && rc=0 || rc=$?
    [ "$rc" -eq 0 ] && pass "safety-case exit 0" || fail "safety-case exit $rc (expected 0)"
    echo "$out" | grep -q "SG-DEMO-001" && pass "names the SafetyGoal" || fail "missing SafetyGoal"
    echo "$out" | grep -q "\[strategy\]" && pass "shows [strategy] node" || fail "missing [strategy]"
    echo "$out" | grep -q "ARG-DEMO-001" && pass "names the strategy Argument" || fail "missing ARG-DEMO-001"
    echo "$out" | grep -q "TC-DEMO-001" && pass "shows the TestCase evidence leaf" || fail "missing TC-DEMO-001"
    echo "$out" | grep -q "\[AoU\]" && pass "shows the [AoU] node" || fail "missing [AoU]"
    echo "$out" | grep -q "AOU-DEMO-001" && pass "names the AssumptionOfUse" || fail "missing AOU-DEMO-001"

    # ── safety-case --json is valid JSON with goals[].arguments and assumptions ─
    printf "  ▶ %s\n" "safety-case --json is valid JSON with arguments and assumptions"
    local json keys
    json=$("$SYSCRIBE" -m "$D/main" safety-case --json 2>/dev/null) && rc=0 || rc=$?
    [ "$rc" -eq 0 ] && pass "safety-case --json exit 0" || fail "safety-case --json exit $rc (expected 0)"
    keys=$(printf '%s' "$json" | jq -r '(.goals|type=="array") and (.goals[0]|has("arguments") and has("assumptions") and has("requirements"))' 2>/dev/null)
    [ "$keys" = "true" ] && pass "valid JSON with goals[].arguments/assumptions/requirements" \
        || fail "safety-case --json missing required keys"
    keys=$(printf '%s' "$json" | jq -r '.goals[0].arguments[0].argumentType' 2>/dev/null)
    [ "$keys" = "strategy" ] && pass "first argument is the strategy node" \
        || fail "first argument argumentType = $keys (expected strategy)"

    # ── named SafetyGoal selector renders just that goal ───────────────────────
    printf "  ▶ %s\n" "safety-case SG-DEMO-001 selects a single goal"
    out=$("$SYSCRIBE" -m "$D/main" safety-case SG-DEMO-001 2>/dev/null) && rc=0 || rc=$?
    [ "$rc" -eq 0 ] && pass "safety-case <SG> exit 0" || fail "safety-case <SG> exit $rc"
    echo "$out" | grep -q "SG-DEMO-001" && pass "selected goal present" || fail "selected goal missing"

    # ── an unresolved Argument.supports yields E855 in badref/ ─────────────────
    run_scenario "badref: unresolved Argument.supports/evidence yields E855" "$D/badref"
    assert_has_code "E855"

    # ── an orphan claim Argument yields W040 in orphan/ ────────────────────────
    run_scenario "orphan: claim Argument with no supports/evidence yields W040" "$D/orphan"
    assert_exit_zero
    assert_has_code "W040"
}
