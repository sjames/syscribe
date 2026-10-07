tc_TRS_VIS_013() {
    local F="$1"
    local M="$F/TC-TRS-VIS-013/model"

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    _scn "a diagram subcommand is rejected"
    local out err ec
    out=$("$SYSCRIBE" -m "$M" diagram list 2>/dev/null) && ec=0 || ec=$?
    err=$("$SYSCRIBE" -m "$M" diagram list 2>&1 >/dev/null) || true
    [ "$ec" -ne 0 ] && pass "diagram list exits non-zero ($ec)" || fail "diagram list exited 0"
    [ -z "$out" ] && pass "nothing on stdout" || fail "stdout not empty: $out"
    printf '%s' "$err" | grep -qi "unrecognized subcommand" \
        && pass "stderr names the unrecognized subcommand" \
        || fail "stderr did not name the unrecognized subcommand: $err"

    _scn "the diagram man page no longer exists"
    out=$("$SYSCRIBE" help diagram 2>/dev/null) && ec=0 || ec=$?
    [ "$ec" -ne 0 ] && pass "help diagram exits non-zero" || fail "help diagram exited 0"
    printf '%s' "$out" | grep -qF "SYNOPSIS" && fail "help diagram still prints a SYNOPSIS" || pass "no SYNOPSIS for diagram"

    _scn "the surviving diagram commands keep their man pages"
    for c in render plantuml; do
        out=$("$SYSCRIBE" help "$c" 2>/dev/null) && ec=0 || ec=$?
        printf '%s' "$out" | grep -qF "SYNOPSIS" && [ "$ec" -eq 0 ] \
            && pass "help $c prints a SYNOPSIS" || fail "help $c: no SYNOPSIS / exit $ec"
    done

    _scn "a Mermaid-kind diagram still validates"
    local summary; summary=$("$SYSCRIBE" -m "$M" validate 2>/dev/null | grep -iE "^[0-9]+ errors")
    printf '%s' "$summary" | grep -q "^0 errors" && pass "0 errors ($summary)" || fail "validate: $summary"
}
