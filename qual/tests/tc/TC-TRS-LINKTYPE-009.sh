tc_TRS_LINKTYPE_009() {
    local F="$1"; local FX="$F/TC-TRS-LINKTYPE-009"
    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }
    # has <output> <file-substring> <code>  — a finding line mentioning both
    has() { printf '%s' "$1" | grep -F -- "$2" | grep -qE -- "$3"; }

    local M="$F/TC-TRS-LINKTYPE-007/model" out
    _scn "links shows outbound and inbound custom links"
    out=$("$SYSCRIBE" -m "$M" links REQ-LT7-002 2>&1) || true
    printf '%s' "$out" | grep 'mitigates' | grep -q 'REQ-LT7-003' && pass "outbound custom link" || fail "outbound missing: $out"
    printf '%s' "$out" | grep 'mitigatedBy' | grep -q 'REQ-LT7-001' && pass "inbound under inverse" || fail "inbound missing: $out"
    out=$("$SYSCRIBE" -m "$M" links REQ-LT7-011 2>&1) || true
    printf '%s' "$out" | grep -q 'loops (inbound)' && pass "inbound without inverse" || fail "no '(inbound)' label: $out"
    _scn "refs shows inbound custom links"
    out=$("$SYSCRIBE" -m "$M" refs REQ-LT7-003 2>&1) || true
    printf '%s' "$out" | grep -q 'REQ-LT7-002' && pass "refs includes custom" || fail "refs missing: $out"
    _scn "impact traverses custom links and filters by kind"
    out=$("$SYSCRIBE" -m "$M" impact REQ-LT7-003 --direction downstream 2>&1) || true
    printf '%s' "$out" | grep -q 'REQ-LT7-001' && pass "impact downstream via custom" || fail "impact missing: $out"
    out=$("$SYSCRIBE" -m "$M" impact REQ-LT7-001 --direction upstream --kinds mitigates --format json 2>/dev/null) || true
    printf '%s' "$out" | grep -q 'REQ-LT7-003' && ! printf '%s' "$out" | grep -q 'TC-LT7-001' && pass "--kinds accepts custom" || fail "--kinds wrong: $out"
    _scn "trace lists custom links"
    out=$("$SYSCRIBE" -m "$M" trace REQ-LT7-002 2>&1) || true
    printf '%s' "$out" | grep -q 'mitigates' && printf '%s' "$out" | grep -q 'REQ-LT7-001' && pass "trace lists custom" || fail "trace missing: $out"
    _scn "show displays links:"
    out=$("$SYSCRIBE" -m "$M" show REQ-LT7-001 2>&1) || true
    printf '%s' "$out" | grep -q 'mitigates' && pass "show lists links" || fail "show missing: $out"
}
