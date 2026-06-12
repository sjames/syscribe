tc_TRS_MG_013() {
    local F="$1"; local FX="$F/TC-TRS-MG-013"
    local out rc

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    # 1. a clean MagicGrid model passes
    _scn "a clean MagicGrid model passes"
    out=$("$SYSCRIBE" -m "$FX/clean" magicgrid --audit 2>&1) && rc=0 || rc=$?
    { [ "$rc" -eq 0 ] && printf '%s' "$out" | grep -qi 'verdict' && printf '%s' "$out" | grep -q 'PASS'; } \
        && pass "clean model audit Verdict PASS, exit 0" || fail "clean audit did not PASS (rc=$rc)"

    # 2. readiness names the system of interest
    _scn "the audit reports readiness (system of interest)"
    out=$("$SYSCRIBE" -m "$FX/clean" magicgrid --audit 2>&1) || true
    printf '%s' "$out" | grep -q 'Station' \
        && pass "readiness names the SoI (Station)" || fail "readiness did not name the SoI"

    # 3. a MagicGrid error lists the code and fails
    _scn "a MagicGrid error lists the code and fails"
    out=$("$SYSCRIBE" -m "$FX/error" magicgrid --audit 2>&1) && rc=0 || rc=$?
    { [ "$rc" -ne 0 ] && printf '%s' "$out" | grep -q 'MG013' && printf '%s' "$out" | grep -q 'FAIL'; } \
        && pass "MG013 listed, Verdict FAIL, non-zero exit" || fail "error audit did not FAIL on MG013 (rc=$rc)"

    # 4. plain magicgrid has no verdict
    _scn "plain magicgrid has no verdict"
    out=$("$SYSCRIBE" -m "$FX/clean" magicgrid 2>&1) || true
    { printf '%s' "$out" | grep -q 'B2' && ! printf '%s' "$out" | grep -qi 'verdict'; } \
        && pass "grid printed, no Verdict line" || fail "plain magicgrid printed a verdict or no grid"

    # 5. the audit emits JSON
    _scn "the audit emits JSON"
    out=$("$SYSCRIBE" -m "$FX/clean" magicgrid --audit --json 2>&1) && rc=0 || rc=$?
    { [ "$rc" -eq 0 ] && printf '%s' "$out" | grep -q '"verdict"'; } \
        && pass "magicgrid --audit --json emits a verdict field" || fail "audit JSON malformed (rc=$rc)"
}
