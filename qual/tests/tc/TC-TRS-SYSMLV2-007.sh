tc_TRS_SYSMLV2_007() {
    local F="$1"; local FX="$F/TC-TRS-SYSMLV2-007/mixed"
    local out

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    # 1. a mapped construct survives alongside an unmapped one in the same file
    _scn "a mapped construct survives alongside an unmapped one in the same file"
    out=$("$SYSCRIBE" -m "$FX" show SysML2::Demo::MappedPart 2>&1 || true)
    printf '%s' "$out" | grep -q 'PartDef' \
        && pass "MappedPart resolves as a first-class PartDef" || fail "MappedPart did not resolve"

    # 2. the unmapped construct contributes nothing
    _scn "the unmapped construct contributes nothing"
    out=$("$SYSCRIBE" -m "$FX" export 2>&1 || true)
    printf '%s' "$out" | grep -q 'UnmappedCalc' \
        && fail "UnmappedCalc unexpectedly appears in the exported graph" \
        || pass "UnmappedCalc is absent from the exported graph"
    out=$("$SYSCRIBE" -m "$FX" validate 2>&1 || true)
    printf '%s' "$out" | grep -q 'UnmappedCalc' \
        && fail "a finding names the unmapped construct: $out" \
        || pass "no finding anywhere names the unmapped construct"
}
