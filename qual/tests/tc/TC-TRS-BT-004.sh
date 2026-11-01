tc_TRS_BT_004() {
    local F="$1"
    local R; R="$(cd "$F/../.." && pwd)"
    local hits
    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    _scn "no stage-name literal is tested against a bindingTime outside the BindingTime type"
    hits=$(grep -rnE 'bindingTime' "$R"/crates/*/src --include='*.rs' \
        | grep -v 'binding_time.rs' | grep -E '"(compile|load|runtime)"' || true)
    [ -z "$hits" ] && pass "no literal stage comparison" || fail "literal stage names remain: $hits"

    _scn "no binding-time rank number is compared outside the BindingTime type"
    hits=$(grep -rnE 'binding_time.*[!=]= *Some\([0-9]+\)|binding_time_rank' "$R"/crates/*/src --include='*.rs' \
        | grep -v 'binding_time.rs' || true)
    [ -z "$hits" ] && pass "no rank literal comparison" || fail "rank literals remain: $hits"

    _scn "the existing parameter binding-time behaviour is unchanged"
    local out
    out=$("$SYSCRIBE" -m "$F/TC-TRS-PARAM-004/valid" validate 2>&1 || true)
    printf '%s' "$out" | grep -q 'E230' && fail "E230 on the valid PARAM-004 fixture" || pass "valid fixture is clean"
    out=$("$SYSCRIBE" -m "$F/TC-TRS-PARAM-004/e230" validate 2>&1 || true)
    printf '%s' "$out" | grep -q 'E230' && pass "E230 still raised for an unknown stage" || fail "E230 lost: $out"
}
