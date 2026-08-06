tc_TRS_SYSMLV2_002() {
    local F="$1"; local FX="$F/TC-TRS-SYSMLV2-002"
    local out

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    # 1. two files contributing to the same package merge into one namespace
    _scn "two files contributing to the same package merge into one namespace"
    out=$("$SYSCRIBE" -m "$FX/merge" show SysML2::Widgets::Alpha 2>&1 || true)
    printf '%s' "$out" | grep -q 'PartDef' \
        && pass "Alpha (from A.sysml) resolves under the merged Widgets namespace" || fail "Alpha did not resolve"
    out=$("$SYSCRIBE" -m "$FX/merge" show SysML2::Widgets::Beta 2>&1 || true)
    printf '%s' "$out" | grep -q 'PartDef' \
        && pass "Beta (from B.sysml) resolves under the same merged Widgets namespace" || fail "Beta did not resolve"

    # 2. a nested SysML v2 package derives a full-depth qualified name
    _scn "a nested SysML v2 package derives a full-depth qualified name"
    out=$("$SYSCRIBE" -m "$FX/merge" show SysML2::Widgets::Sub::Deep 2>&1 || true)
    printf '%s' "$out" | grep -q 'PartDef' \
        && pass "Deep resolves at its full nested qualified name" || fail "nested qname derivation failed"

    # 3. a parse failure in one file does not abort the rest of the subtree
    _scn "a parse failure in one file does not abort the rest of the subtree"
    out=$("$SYSCRIBE" -m "$FX/parsefail" validate 2>&1 || true)
    printf '%s' "$out" | grep -q 'W541' \
        && pass "W541 raised for the broken file" || fail "W541 not raised"
    printf '%s' "$out" | grep -q 'Bad.sysml' \
        && pass "W541 names the broken file" || fail "W541 does not name the broken file"
    out=$("$SYSCRIBE" -m "$FX/parsefail" show SysML2::Good::GoodPart 2>&1 || true)
    printf '%s' "$out" | grep -q 'PartDef' \
        && pass "the good file's element still resolves despite the sibling parse failure" \
        || fail "good file's element missing after sibling parse failure"
}
