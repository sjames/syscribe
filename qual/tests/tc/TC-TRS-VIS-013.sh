tc_TRS_VIS_013() {
    local F="$1"
    local M="$F/TC-TRS-VIS-013/model"

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    _scn "the retired toolkit subcommands are rejected"
    local out err ec
    for sub in list measure compose layout seq req; do
        out=$("$SYSCRIBE" -m "$M" diagram "$sub" 2>/dev/null) && ec=0 || ec=$?
        err=$("$SYSCRIBE" -m "$M" diagram "$sub" 2>&1 >/dev/null) || true
        [ "$ec" -ne 0 ] && pass "diagram $sub exits non-zero ($ec)" || fail "diagram $sub exited 0"
        [ -z "$out" ] && pass "diagram $sub: nothing on stdout" || fail "diagram $sub: stdout not empty: $out"
        grep -qi "unrecognized subcommand" <<<"$err" \
            && pass "diagram $sub: stderr names the unrecognized subcommand" \
            || fail "diagram $sub: stderr did not name the unrecognized subcommand: $err"
    done

    _scn "the diagram man page describes only export"
    out=$("$SYSCRIBE" help diagram 2>/dev/null) && ec=0 || ec=$?
    [ "$ec" -eq 0 ] && grep -qF "SYNOPSIS" <<<"$out" \
        && pass "help diagram prints a SYNOPSIS" || fail "help diagram: no SYNOPSIS / exit $ec"
    grep -qF "diagram export" <<<"$out" && pass "the page documents diagram export" || fail "the page does not mention diagram export"
    local synopsis; synopsis=$(sed -n '/^## SYNOPSIS/,/^## /p' <<<"$out")
    for sub in list measure compose layout seq req; do
        grep -qE "diagram $sub\b" <<<"$synopsis" \
            && fail "the SYNOPSIS still offers diagram $sub" \
            || pass "the SYNOPSIS does not offer diagram $sub"
    done

    _scn "the surviving diagram commands keep their man pages"
    for c in render plantuml; do
        out=$("$SYSCRIBE" help "$c" 2>/dev/null) && ec=0 || ec=$?
        grep -qF "SYNOPSIS" <<<"$out" && [ "$ec" -eq 0 ] \
            && pass "help $c prints a SYNOPSIS" || fail "help $c: no SYNOPSIS / exit $ec"
    done

    _scn "a Mermaid-kind diagram still validates"
    local summary; summary=$("$SYSCRIBE" -m "$M" validate 2>/dev/null | grep -iE "^[0-9]+ errors")
    grep -q "^0 errors" <<<"$summary" && pass "0 errors ($summary)" || fail "validate: $summary"
}
