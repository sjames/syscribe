tc_TRS_SYSMLV2_001() {
    local F="$1"; local FX="$F/TC-TRS-SYSMLV2-001"
    local out

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    # 1. the marked package's own _index.md still parses as a normal element
    _scn "the package's own _index.md still parses as a normal element"
    out=$("$SYSCRIBE" -m "$FX/marked" show SysML2 2>&1 || true)
    printf '%s' "$out" | grep -q 'Package' \
        && pass "SysML2 package element parses normally" || fail "SysML2 package element missing/malformed"

    # 2. a stray nested _index.md is excluded and warned (W540)
    _scn "a stray nested _index.md is excluded and warned"
    out=$("$SYSCRIBE" -m "$FX/marked" validate 2>&1 || true)
    printf '%s' "$out" | grep -q 'W540' \
        && pass "W540 raised for the stray nested _index.md" || fail "W540 not raised"
    printf '%s' "$out" | grep -q 'Nested/_index.md' \
        && pass "W540 names the stray file" || fail "W540 does not name the stray file"

    # 3. a hand-authored .md sibling still parses normally
    _scn "a hand-authored .md sibling still parses normally"
    out=$("$SYSCRIBE" -m "$FX/marked" show SysML2::Extra 2>&1 || true)
    printf '%s' "$out" | grep -q 'PartDef' \
        && pass "hand-authored sibling Extra resolves under SysML2's namespace" || fail "hand-authored sibling did not resolve"

    # 4. a model with no sysmlSubmodel package is unaffected
    _scn "a model with no sysmlSubmodel package is unaffected"
    out=$("$SYSCRIBE" -m "$FX/nomarker" validate 2>&1 || true)
    { ! printf '%s' "$out" | grep -q 'W540' && ! printf '%s' "$out" | grep -q 'W541'; } \
        && pass "no SysMLv2-related finding on a model that never mentions sysmlSubmodel" \
        || fail "unexpected SysMLv2-related finding on an unrelated model"
}
