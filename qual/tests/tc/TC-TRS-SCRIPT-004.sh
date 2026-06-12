tc_TRS_SCRIPT_004() {
    local F="$1"
    local M="$F/TC-TRS-SCRIPT-004"
    local DUP="$F/TC-TRS-SCRIPT-004/dup"

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    _scn "a file registers both a command and a check"
    local lst rc=0
    lst=$("$SYSCRIBE" -m "$M" scripts list 2>/dev/null) || rc=$?
    [ "$rc" -eq 0 ] && pass "scripts list exit 0" || fail "scripts list exit $rc"
    printf '%s' "$lst" | grep -qE "\| report .* command " \
        && pass "command 'report' listed as command" || fail "command 'report' not listed"
    printf '%s' "$lst" | grep -qE "\| naming .* check " \
        && pass "check 'naming' listed as check" || fail "check 'naming' not listed"

    _scn "a pure library is not runnable"
    printf '%s' "$lst" | grep -qiF "helper" \
        && fail "pure library function surfaced as runnable" || pass "pure library not surfaced"
    local prc=0
    "$SYSCRIBE" -m "$M" scripts run helper >/dev/null 2>&1 || prc=$?
    [ "$prc" -ne 0 ] && pass "running a library name exits non-zero" || fail "library name was runnable"

    _scn "duplicate name is a load error"
    local dout drc=0
    dout=$("$SYSCRIBE" -m "$DUP" scripts list 2>&1) || drc=$?
    [ "$drc" -ne 0 ] && pass "duplicate-name model exits non-zero" || fail "duplicate name did not error"
    printf '%s' "$dout" | grep -qiF "duplicate" \
        && pass "duplicate-name error message" || fail "no duplicate-name message"
    printf '%s' "$dout" | grep -qiF "coverage" \
        && pass "error names the colliding handle" || fail "colliding name not reported"
}
