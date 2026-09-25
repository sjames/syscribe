tc_TRS_DERIVE_007() {
    local F="$1"; local B="$F/TC-TRS-DERIVE-007"
    _scn() { _flush_scenario; SCENARIO_NAME="$1"; _SCEN_PASS=0; _SCEN_FAIL=0; printf "  ▶ %s\n" "$1"; }
    local out hits show

    out=$("$SYSCRIBE" -m "$B/recognised" validate 2>/dev/null) || true

    _scn "a derive: block raises no W047"
    grep -F "| W047 |" <<<"$out" | grep -qF "'derive'" \
        && fail "W047 raised for derive:" || pass "no W047 for derive:"

    _scn "cross-element derived fields evaluate in dependency order"
    show=$("$SYSCRIBE" -m "$B/recognised" show Sys::Assembly 2>/dev/null) || true
    grep -qE "\| total \| 43(\.0)? \|" <<<"$show" \
        && pass "Assembly.total = 43" || fail "Assembly.total is not 43: $(grep -F "total" <<<"$show" || true)"

    _scn "a malformed derive: block raises E505"
    hits=$(grep -F "| E505 |" <<<"$out" || true)
    grep -qF "NotAMap.md" <<<"$hits" && pass "E505 for derive: 5" || fail "no E505 for a non-mapping derive: $hits"
    grep -qF "NumFormula.md" <<<"$hits" && grep -qF "'fixed'" <<<"$hits" \
        && pass "E505 names the non-string formula field" || fail "no E505 for a non-string formula: $hits"

    out=$("$SYSCRIBE" -m "$B/cycle" validate 2>/dev/null) || true
    hits=$(grep -F "| E504 |" <<<"$out" || true)

    _scn "a self-referential formula raises E504"
    grep -qF "SelfRef.md" <<<"$hits" && grep -qF "Sys::SelfRef.fieldA" <<<"$hits" \
        && pass "E504 names Sys::SelfRef.fieldA" || fail "E504 missing for the self-reference: $hits"
    show=$("$SYSCRIBE" -m "$B/cycle" show Sys::SelfRef 2>/dev/null) || true
    grep -qE "\| fieldA \|" <<<"$show" && fail "cyclic fieldA was evaluated" || pass "cyclic fieldA skipped"

    _scn "mutually dependent elements raise E504 on both"
    grep -qF "Xray.md" <<<"$hits" && pass "E504 on Xray" || fail "E504 missing on Xray: $hits"
    grep -qF "Yankee.md" <<<"$hits" && pass "E504 on Yankee" || fail "E504 missing on Yankee: $hits"
    grep -F "Xray.md" <<<"$hits" | grep -qF "Sys::Yankee.b" \
        && pass "E504 names the cycle through Sys::Yankee.b" || fail "E504 on Xray does not name the cycle: $hits"

    _scn "a valid chain and a dependent of a cycle raise no E504"
    grep -qF "Chain.md" <<<"$hits" && fail "E504 raised for a valid chain" || pass "no E504 for the valid chain"
    grep -qF "Downstream.md" <<<"$hits" && fail "E504 raised for a dependent outside the cycle" || pass "no E504 for the dependent"
    show=$("$SYSCRIBE" -m "$B/cycle" show Sys::Chain 2>/dev/null) || true
    grep -qE "\| first \| 9(\.0)? \|" <<<"$show" && grep -qE "\| second \| 8(\.0)? \|" <<<"$show" \
        && pass "chain evaluates first=9, second=8" || fail "chain did not evaluate in dependency order: $show"
    show=$("$SYSCRIBE" -m "$B/cycle" show Sys::Downstream 2>/dev/null) || true
    grep -qE "\| c \| 7(\.0)? \|" <<<"$show" && pass "dependent falls back to 7" || fail "Downstream.c is not 7"
}
