tc_TRS_DIAG_004() {
    local F="$1"; local B="$F/TC-TRS-DIAG-004"
    _scn() { _flush_scenario; SCENARIO_NAME="$1"; _SCEN_PASS=0; _SCEN_FAIL=0; printf "  ▶ %s\n" "$1"; }
    local S; S=$(mktemp -d)
    local out err ec

    _scn "measure an unknown element"
    out=$("$SYSCRIBE" -m "$B/model" diagram measure Nope::X 2>"$S/err") && ec=0 || ec=$?
    err=$(cat "$S/err")
    [ "$ec" -eq 1 ] && pass "exit 1" || fail "exit $ec (expected 1)"
    grep -qF "error: element 'Nope::X' not found" <<<"$err" && pass "stderr names Nope::X" \
        || fail "stderr lacks the not-found error: $err"
    [ -z "$out" ] && pass "nothing on stdout" || fail "stdout not empty: $(head -1 <<<"$out")"

    _scn "measure a mix of known and unknown elements"
    out=$("$SYSCRIBE" -m "$B/model" diagram measure Arch::Pump,Nope::X 2>"$S/err") && ec=0 || ec=$?
    err=$(cat "$S/err")
    [ "$ec" -eq 1 ] && pass "exit 1" || fail "exit $ec (expected 1)"
    grep -qF "'Nope::X' not found" <<<"$err" && pass "names Nope::X" || fail "does not name Nope::X"
    grep -qF "'Arch::Pump' not found" <<<"$err" && fail "reports the known element" \
        || pass "the known element is not reported"

    _scn "measure a known element"
    out=$("$SYSCRIBE" -m "$B/model" diagram measure Arch::Pump 2>/dev/null) && ec=0 || ec=$?
    [ "$ec" -eq 0 ] && pass "exit 0" || fail "exit $ec (expected 0)"
    jq -e '.[0].qname == "Arch::Pump"' <<<"$out" >/dev/null 2>&1 && pass "JSON names Arch::Pump" \
        || fail "JSON does not name Arch::Pump"

    _scn "layout and compose files naming an unknown element"
    out=$("$SYSCRIBE" -m "$B/model" diagram layout "$B/placement.json" 2>"$S/err") && ec=0 || ec=$?
    err=$(cat "$S/err")
    [ "$ec" -eq 1 ] && pass "layout exits 1" || fail "layout exits $ec (expected 1)"
    grep -qF "error: element 'Nope::X' not found" <<<"$err" && pass "layout names Nope::X" \
        || fail "layout does not name Nope::X"
    out=$("$SYSCRIBE" -m "$B/model" diagram compose "$B/stale-layout.json" 2>"$S/err") && ec=0 || ec=$?
    err=$(cat "$S/err")
    [ "$ec" -eq 1 ] && pass "compose exits 1" || fail "compose exits $ec (expected 1)"
    grep -qF "error: element 'Nope::X' not found" <<<"$err" && pass "compose names Nope::X" \
        || fail "compose does not name Nope::X"
    rm -rf "$S"
}
