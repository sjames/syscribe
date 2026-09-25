tc_TRS_DERIVE_006() {
    local F="$1" out c

    SCENARIO_NAME="a model raising both families reports each under its own code"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    out=$("$SYSCRIBE" -m "$F/TC-TRS-DERIVE-006/model" validate 2>/dev/null || true)

    grep -F "| E505 |" <<<"$out" | grep -qF "derive formula parse error for field 'broken'" \
        && pass "E505 is the derive parse error" || fail "E505 missing or not the derive parse error"
    grep -F "| E506 |" <<<"$out" | grep -qF "derive: element 'Gone::Thing' not found" \
        && pass "E506 is the derive unknown-element reference" || fail "E506 missing or not the derive reference"

    grep -F "| E500 |" <<<"$out" | grep -qF "Allocation feature \`allocatedFrom\` 'NoSuchSource'" \
        && pass "E500 is the Allocation-feature allocatedFrom" || fail "E500 missing or wrong meaning"
    grep -F "| E501 |" <<<"$out" | grep -qF "Allocation feature \`allocatedTo\` 'NoSuchTarget'" \
        && pass "E501 is the Allocation-feature allocatedTo" || fail "E501 missing or wrong meaning"
    grep -F "| E502 |" <<<"$out" | grep -qF "\`allocatedFrom\` 'NoSuchFrom'" \
        && pass "E502 is the element-level allocatedFrom" || fail "E502 missing or wrong meaning"
    grep -F "| E503 |" <<<"$out" | grep -qF "\`allocatedTo\` 'NoSuchTo'" \
        && pass "E503 is the element-level allocatedTo" || fail "E503 missing or wrong meaning"

    for c in E500 E501 E502 E503; do
        grep -F "| $c |" <<<"$out" | grep -qF "derive" \
            && fail "$c carries a derive finding" || pass "$c carries no derive finding"
    done
}
