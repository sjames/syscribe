tc_TRS_SYSMLV2_003() {
    local F="$1"; local FX="$F/TC-TRS-SYSMLV2-003/model"
    local out

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    out=$("$SYSCRIBE" -m "$FX" validate 2>&1 || true)

    # 1. satisfy by quoted REQ-* id resolves cleanly
    _scn "satisfy by quoted REQ-* id resolves cleanly"
    { ! printf '%s' "$out" | grep -q "REQ-DEMO-ONE-001' has no satisfying" \
        && ! printf '%s' "$out" | grep -q 'unresolved.*REQ-DEMO-ONE-001'; } \
        && pass "REQ-DEMO-ONE-001's satisfying-element warning is suppressed" \
        || fail "REQ-DEMO-ONE-001 still shows no satisfying element, or a dangling reference"

    # 2. satisfy by Syscribe qualified name resolves cleanly
    _scn "satisfy by Syscribe qualified name resolves cleanly"
    { ! printf '%s' "$out" | grep -q "REQ-DEMO-TWO-001' has no satisfying" \
        && ! printf '%s' "$out" | grep -q 'unresolved.*REQ-DEMO-TWO-001'; } \
        && pass "REQ-DEMO-TWO-001's satisfying-element warning is suppressed" \
        || fail "REQ-DEMO-TWO-001 still shows no satisfying element, or a dangling reference"

    # 3. verify targets a native Requirement
    _scn "verify targets a native Requirement"
    ! printf '%s' "$out" | grep -q 'unresolved.*REQ-DEMO-THREE-001' \
        && pass "no dangling-reference finding for the verify target REQ-DEMO-THREE-001" \
        || fail "verify target REQ-DEMO-THREE-001 raised a dangling-reference finding"
}
