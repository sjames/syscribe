tc_TRS_SYSMLV2_006() {
    local F="$1"; local FX="$F/TC-TRS-SYSMLV2-006"
    local out rc

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    # 1. a malformed sysmlSubmodel: value does not abort validation
    _scn "a malformed sysmlSubmodel: value does not abort validation"
    out=$("$SYSCRIBE" -m "$FX/malformed" validate 2>&1) && rc=0 || rc=$?
    printf '%s' "$out" | grep -q 'E002' \
        && pass "E002 names the malformed frontmatter" || fail "E002 not raised"
    printf '%s' "$out" | grep -qi 'panic' \
        && fail "process panicked" || pass "no panic"

    # 2. a .sysml parse failure does not abort validation
    _scn "a .sysml parse failure does not abort validation"
    out=$("$SYSCRIBE" -m "$FX/parsefail" validate 2>&1) && rc=0 || rc=$?
    printf '%s' "$out" | grep -q 'W541' \
        && pass "W541 names the broken file" || fail "W541 not raised"
    printf '%s' "$out" | grep -qi 'panic' \
        && fail "process panicked" || pass "no panic"

    # 3. an unmapped construct produces no finding at all
    _scn "an unmapped construct produces no finding at all"
    out=$("$SYSCRIBE" -m "$FX/unmapped" validate 2>&1) && rc=0 || rc=$?
    [ "$rc" -eq 0 ] \
        && pass "validate exits 0 on a subtree containing only unmapped constructs" \
        || fail "validate exited non-zero (rc=$rc) on a subtree containing only unmapped constructs"
    printf '%s' "$out" | grep -qE '^\| [EW][0-9]' \
        && fail "unexpected finding for an unmapped-only subtree: $out" \
        || pass "zero findings for an unmapped-only subtree"
}
