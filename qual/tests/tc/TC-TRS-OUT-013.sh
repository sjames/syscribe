tc_TRS_OUT_013() {
    local F="$1"
    local READY="$F/TC-TRS-OUT-013/ready"
    local NOTREADY="$F/TC-TRS-OUT-013/notready"

    # --- Scenario: ready model audits PASS (exit 0) and prints the sections ---
    printf "  ▶ %s\n" "ready model audits PASS (exit 0)"
    local out
    out=$("$SYSCRIBE" -m "$READY" audit 2>/dev/null) && rc=0 || rc=$?
    [ "$rc" -eq 0 ] && pass "ready audit exit 0" || fail "ready audit exit $rc (expected 0)"
    echo "$out" | grep -qi "status split" && pass "prints status split section" \
        || fail "missing status split section"
    echo "$out" | grep -qi "coverage" && pass "prints coverage section" \
        || fail "missing coverage section"
    echo "$out" | grep -qi "PASS" && pass "verdict PASS" || fail "verdict not PASS"

    # --profile strict on the ready model promotes an absent code → still PASS.
    out=$("$SYSCRIBE" -m "$READY" audit --profile strict 2>/dev/null) && rc=0 || rc=$?
    [ "$rc" -eq 0 ] && pass "ready audit --profile strict exit 0" \
        || fail "ready audit --profile strict exit $rc (expected 0)"

    # --- Scenario: notready model audits FAIL (exit 2) naming W306 ---
    printf "  ▶ %s\n" "notready model audits FAIL (exit 2) naming W306"
    out=$("$SYSCRIBE" -m "$NOTREADY" audit 2>/dev/null) && rc=0 || rc=$?
    [ "$rc" -eq 2 ] && pass "notready audit exit 2" || fail "notready audit exit $rc (expected 2)"
    echo "$out" | grep -qi "FAIL" && pass "verdict FAIL" || fail "verdict not FAIL"
    echo "$out" | grep -q "W306" && pass "verdict names W306" || fail "verdict does not name W306"

    # --- Scenario: audit --json on ready is valid JSON with the rollup keys ---
    printf "  ▶ %s\n" "audit --json is valid JSON with statusSplit/coverage/verdict"
    local json
    json=$("$SYSCRIBE" -m "$READY" audit --json 2>/dev/null) && rc=0 || rc=$?
    [ "$rc" -eq 0 ] && pass "audit --json exit 0" || fail "audit --json exit $rc (expected 0)"
    local keys
    keys=$(printf '%s' "$json" | jq -r 'has("statusSplit") and has("coverage") and has("verdict")' 2>/dev/null)
    [ "$keys" = "true" ] && pass "valid JSON with statusSplit/coverage/verdict" \
        || fail "audit --json is not valid JSON with the required keys"
}
