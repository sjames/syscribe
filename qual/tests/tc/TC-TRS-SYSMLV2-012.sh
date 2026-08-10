tc_TRS_SYSMLV2_012() {
    local F="$1"; local M="$F/TC-TRS-SYSMLV2-012/model"

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    # 1. a connection usage's trailing doc body lifts
    _scn "a connection usage's trailing doc body lifts"
    local out; out=$("$SYSCRIBE" -m "$M" show SysML2::Demo::Holder::withDoc 2>&1)
    printf '%s' "$out" | grep -q '^Explanation\.$' \
        && pass "withDoc's doc body is 'Explanation.'" || fail "withDoc's doc body did not match: $out"

    # 2. a connection usage with no trailing body is unaffected
    _scn "a connection usage with no trailing body is unaffected"
    local nout; nout=$("$SYSCRIBE" -m "$M" show SysML2::Demo::Holder::noBody 2>&1)
    printf '%s' "$nout" | grep -q '## Documentation' \
        && fail "noBody unexpectedly has a Documentation section: $nout" \
        || pass "noBody has no Documentation section (empty doc, no regression)"
}
