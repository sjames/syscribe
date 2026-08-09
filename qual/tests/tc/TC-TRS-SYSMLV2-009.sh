tc_TRS_SYSMLV2_009() {
    local F="$1"; local M="$F/TC-TRS-SYSMLV2-009/model"

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    # 1. a single doc block lifts onto a part def and clears W600
    _scn "a single doc block lifts onto a part def and clears W600"
    local out; out=$("$SYSCRIBE" -m "$M" show SysML2::Demo::SingleDocPart 2>&1)
    printf '%s' "$out" | grep -q '^Explanation\.$' \
        && pass "SingleDocPart's doc body is 'Explanation.'" || fail "SingleDocPart's doc body did not match: $out"

    local vout; vout=$("$SYSCRIBE" -m "$M" validate 2>&1 || true)
    local w600_count; w600_count=$(printf '%s' "$vout" | grep -c 'W600' || true)
    [ "$w600_count" -eq 1 ] && pass "exactly one W600 raised (PlainPart only)" \
        || fail "W600 count=$w600_count (expected 1 — SingleDocPart/TwoDocPart should be clear)"

    # 2. two doc blocks concatenate in source order
    _scn "two doc blocks concatenate in source order"
    local two_out; two_out=$("$SYSCRIBE" -m "$M" show SysML2::Demo::TwoDocPart 2>&1)
    local first_line; first_line=$(printf '%s\n' "$two_out" | grep -n '^First\.$' | cut -d: -f1)
    local second_line; second_line=$(printf '%s\n' "$two_out" | grep -n '^Second\.$' | cut -d: -f1)
    if [ -n "$first_line" ] && [ -n "$second_line" ] && [ "$first_line" -lt "$second_line" ]; then
        pass "TwoDocPart's doc body has First. before Second. (concatenated in source order)"
    else
        fail "TwoDocPart's doc body did not concatenate as expected: $two_out"
    fi

    # 3. a part def with no doc member is unaffected — no Documentation
    # section on show, and (per scenario 1's w600_count == 1) it's the one
    # element still tripping W600.
    _scn "a part def with no doc member is unaffected"
    local plain_out; plain_out=$("$SYSCRIBE" -m "$M" show SysML2::Demo::PlainPart 2>&1)
    printf '%s' "$plain_out" | grep -q '## Documentation' \
        && fail "PlainPart unexpectedly has a Documentation section: $plain_out" \
        || pass "PlainPart has no Documentation section (empty doc, no regression)"
}
