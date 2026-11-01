tc_TRS_BT_034() {
    local F="$1"; local B="$F/TC-TRS-BT-034"
    local out
    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }
    cnt() { printf '%s' "$1" | grep "$2" | grep -c -- "$3" || true; }
    has() { printf '%s' "$1" | grep -q "$2"; }
    val() { "$SYSCRIBE" -m "$1" validate --config "$2" 2>&1 || true; }

    chk() { "$SYSCRIBE" -m "$B/$1" validate 2>&1 || true; }

    _scn "late-bound variability reaching a safety requirement with no covering test is flagged"
    out=$(chk uncovered)
    [ "$(cnt "$out" W753 "REQ-AKV-001")" -eq 1 ] && printf '%s' "$out" | grep W753 | grep 'REQ-AKV-001' | grep -q 'Features::Motor.kv' && pass "a parameter in the text (ASIL B)" || fail "no W753 for REQ-AKV-001: $out"
    [ "$(cnt "$out" W753 "REQ-ASENS-001")" -eq 1 ] && printf '%s' "$out" | grep W753 | grep 'REQ-ASENS-001' | grep -q 'appliesWhen' && pass "a feature in the appliesWhen (ASIL C)" || fail "no W753 for REQ-ASENS-001: $out"

    _scn "what is not late, not safety-relevant or not final is silent"
    [ "$(cnt "$out" W753 "REQ-AOLD-001")" -eq 0 ] && pass "a compile-bound parameter" || fail "W753 for a compile-bound parameter"
    [ "$(cnt "$out" W753 "REQ-QMKV-001")" -eq 0 ] && pass "a requirement with no safety level" || fail "W753 for a QM requirement"
    [ "$(cnt "$out" W753 "REQ-ADRAFT-001")" -eq 0 ] && pass "a draft requirement" || fail "W753 for a draft requirement"

    _scn "a verifying TestCase that covers the item clears it"
    out=$(chk covered)
    [ "$(printf '%s' "$out" | grep -c W753)" -eq 0 ] && pass "no W753 once covered (by FEAT id and by qualified name)" || fail "still flagged: $(printf '%s' "$out" | grep W753)"
    [ "$(cnt "$out" W754 "Motor.nope")" -eq 1 ] && pass "a coversVariability entry naming nothing is W754" || fail "no W754 for the typo: $out"

    _scn "a model with no binding windows is unaffected"
    out=$(chk nowindows); has "$out" 'W753\|W754' && fail "finding on a model without windows: $out" || pass "no W753"
    rc=0; "$SYSCRIBE" -m "$B/uncovered" validate --deny W753 >/dev/null 2>&1 || rc=$?
    [ "$rc" -eq 2 ] && pass "--deny W753 gates it" || fail "exit $rc under --deny W753"
}
