tc_TRS_SCRIPT_002() {
    local F="$1"
    local M="$F/TC-TRS-SCRIPT-002"

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    _scn "a runaway loop is aborted by the operation limit"
    local out rc=0
    out=$(timeout 30 "$SYSCRIBE" -m "$M" scripts run runaway 2>&1) || rc=$?
    [ "$rc" -ne 124 ] && pass "did not time out (no hang)" || fail "hung (timed out)"
    [ "$rc" -ne 0 ] && pass "runaway exits non-zero ($rc)" || fail "runaway exited 0"
    printf '%s' "$out" | grep -qiE "runaway|operation|limit|error" \
        && pass "abort reported with an error" || fail "no error message on abort"

    _scn "eval is disabled"
    rc=0; out=$("$SYSCRIBE" -m "$M" scripts run try_eval 2>&1) || rc=$?
    [ "$rc" -ne 0 ] && pass "try_eval exits non-zero" || fail "try_eval exited 0"
    printf '%s' "$out" | grep -qiF "eval succeeded" \
        && fail "eval unexpectedly executed" || pass "eval refused"

    _scn "a module import escaping the scripts dir fails"
    rc=0; out=$("$SYSCRIBE" -m "$M" scripts run try_fs 2>&1) || rc=$?
    [ "$rc" -ne 0 ] && pass "try_fs exits non-zero" || fail "try_fs exited 0"
    printf '%s' "$out" | grep -qiF "escaped" \
        && fail "import escaped the scripts dir" || pass "escape refused"

    _scn "a parse error is reported with the script name and does not break siblings"
    local lst; lst=$("$SYSCRIBE" -m "$M" scripts list 2>&1) || true
    printf '%s' "$lst" | grep -qiF "broken" \
        && pass "broken.rhai reported by name" || fail "parse error not attributed to broken script"
    printf '%s' "$lst" | grep -qF "| ok " \
        && pass "healthy 'ok' command still listed" || fail "'ok' command lost after sibling parse error"
    local okout okrc=0
    okout=$("$SYSCRIBE" -m "$M" scripts run ok 2>/dev/null) || okrc=$?
    [ "$okrc" -eq 0 ] && pass "'ok' runs despite sibling failures" || fail "'ok' run failed ($okrc)"
    printf '%s' "$okout" | grep -qF "ok:2" \
        && pass "'ok' produced expected output" || fail "'ok' output wrong: $okout"

    _scn "deterministic output"
    local a b
    a=$("$SYSCRIBE" -m "$M" scripts run ok 2>/dev/null) || true
    b=$("$SYSCRIBE" -m "$M" scripts run ok 2>/dev/null) || true
    [ "$a" = "$b" ] && pass "two runs identical (deterministic)" || fail "output differed between runs"
}
