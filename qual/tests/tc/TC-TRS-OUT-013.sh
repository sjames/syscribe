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

    # --- Scenario: --config projects the verdict onto a variant (GH #35) ---
    printf "  ▶ %s\n" "--config projects the audit onto a variant"
    local VAR="$F/TC-TRS-OUT-013/variant"
    "$SYSCRIBE" -m "$VAR" audit >/dev/null 2>&1 && rc=0 || rc=$?
    [ "$rc" -eq 2 ] && pass "whole-model audit FAILs (gated SIL-4 req trips W306)" \
        || fail "whole-model variant audit exit $rc (expected 2)"
    "$SYSCRIBE" -m "$VAR" audit --config CONF-SINGLE-001 >/dev/null 2>&1 && rc=0 || rc=$?
    [ "$rc" -eq 0 ] && pass "audit --config CONF-SINGLE-001 PASSes (req projected out)" \
        || fail "audit --config exit $rc (expected 0)"

    # --- Scenario: audit --config agrees with validate --config (GH #36) ------
    # TC-AMP-001 verifies REQ-AMP-001, which is projected OUT of CONF-SINGLE-001.
    # The TestCase itself stays in the variant; its escaping verifies: ref must
    # NOT be mis-counted as a dangling/error finding. audit must match validate.
    printf "  ▶ %s\n" "audit --config agrees with validate --config (no phantom dangling)"
    "$SYSCRIBE" -m "$VAR" validate --config CONF-SINGLE-001 >/dev/null 2>&1 && vrc=0 || vrc=$?
    [ "$vrc" -eq 0 ] && pass "validate --config CONF-SINGLE-001 is clean (rc 0)" \
        || fail "validate --config exit $vrc (expected 0)"
    local ajson
    ajson=$("$SYSCRIBE" -m "$VAR" audit --config CONF-SINGLE-001 --json 2>/dev/null)
    local dangling
    dangling=$(printf '%s' "$ajson" | jq -r '.orphans.danglingTestCases.count')
    [ "$dangling" = "0" ] && pass "TestCase verifying a config-inactive req is not dangling" \
        || fail "phantom dangling under --config (count=$dangling, GH #36)"
    local averdict
    averdict=$(printf '%s' "$ajson" | jq -r '.verdict.pass')
    [ "$averdict" = "true" ] && pass "audit --config verdict agrees (no phantom error finding)" \
        || fail "audit --config verdict FAIL while validate --config is clean (GH #36)"

    # --- Scenario: --all-configs + bad-config error path ---
    printf "  ▶ %s\n" "--all-configs and bad --config"
    out=$("$SYSCRIBE" -m "$VAR" audit --all-configs 2>/dev/null) && rc=0 || rc=$?
    printf '%s' "$out" | grep -q "CONF-SINGLE-001" && [ "$rc" -eq 0 ] \
        && pass "--all-configs lists CONF-SINGLE-001 and PASSes" \
        || fail "--all-configs unexpected (exit $rc)"
    "$SYSCRIBE" -m "$VAR" audit --config NOPE-X >/dev/null 2>&1 && rc=0 || rc=$?
    [ "$rc" -ne 0 ] && pass "unresolvable --config exits non-zero" \
        || fail "bad --config did not error"

    # --- Scenario: parent requirement excluded from orphan sets (GH #37) -------
    # The parent (REQ-PR-PARENT-000) has two leaves, each satisfied by an arch
    # element and verified by an active TestCase. The parent is satisfied/verified
    # only transitively through its leaves and can never be satisfied directly
    # (§12.4 / E312). audit must not list it as unsatisfied or unverified, matching
    # validate which suppresses W002/W300 for parents.
    printf "  ▶ %s\n" "parent requirement is excluded from the orphan sets (GH #37)"
    local PR="$F/TC-TRS-OUT-013/parent-rollup"
    local PARENT="REQ-PR-PARENT-000"
    "$SYSCRIBE" -m "$PR" validate >/dev/null 2>&1 && rc=0 || rc=$?
    [ "$rc" -eq 0 ] && pass "parent-rollup validate is clean (rc 0)" \
        || fail "parent-rollup validate exit $rc (expected 0)"
    local pjson
    pjson=$("$SYSCRIBE" -m "$PR" audit --json 2>/dev/null)
    printf '%s' "$pjson" | jq -e --arg p "$PARENT" \
        '.orphans.unsatisfiedRequirements.ids | index($p) == null' >/dev/null \
        && pass "parent absent from unsatisfiedRequirements" \
        || fail "parent $PARENT mis-listed as unsatisfied (GH #37)"
    printf '%s' "$pjson" | jq -e --arg p "$PARENT" \
        '.orphans.unverifiedRequirements.ids | index($p) == null' >/dev/null \
        && pass "parent absent from unverifiedRequirements" \
        || fail "parent $PARENT mis-listed as unverified (GH #37)"
    local pvalidate
    pvalidate=$("$SYSCRIBE" -m "$PR" validate 2>/dev/null)
    printf '%s' "$pvalidate" | grep "$PARENT" | grep -Eq 'W300|W002' \
        && fail "validate emits W300/W002 for parent $PARENT" \
        || pass "validate emits no W300/W002 for the parent (audit agrees)"
}
