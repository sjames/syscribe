tc_TRS_SYSMLV2_004() {
    local F="$1"; local FX="$F/TC-TRS-SYSMLV2-004/model"
    local out

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    out=$("$SYSCRIBE" -m "$FX" validate 2>&1 || true)

    # 1. verifies: resolves against a SysMLv2-mapped element by qname
    _scn "verifies: resolves against a SysMLv2-mapped element by qname"
    { printf '%s' "$out" | grep -F 'TC-DEMO-001.md' | grep -qE 'E102|E104'; } \
        && fail "unexpected dangling/wrong-type finding for TC-DEMO-001" \
        || pass "TC-DEMO-001 (verifies a SysMLv2 element) raises no dangling/wrong-type finding"

    # 2. verifying a native Requirement still works unchanged
    _scn "verifying a native Requirement still works unchanged"
    { printf '%s' "$out" | grep -F 'TC-DEMO-002.md' | grep -qE 'E102|E104'; } \
        && fail "unexpected dangling/wrong-type finding for TC-DEMO-002" \
        || pass "TC-DEMO-002 (verifies a native Requirement) raises no dangling/wrong-type finding"

    # 3. verifying a hand-authored non-Requirement element is still rejected
    _scn "verifying a hand-authored non-Requirement element is still rejected"
    { printf '%s' "$out" | grep -F 'TC-DEMO-003.md' | grep -q 'E104'; } \
        && pass "TC-DEMO-003 (verifies a hand-authored PartDef) still raises E104" \
        || fail "E104 not raised for TC-DEMO-003"
}
