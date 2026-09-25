tc_TRS_SEC_009() {
    local F="$1"; local B="$F/TC-TRS-SEC-009"
    _scn() { _flush_scenario; SCENARIO_NAME="$1"; _SCEN_PASS=0; _SCEN_FAIL=0; printf "  ▶ %s\n" "$1"; }
    local out hits

    # The root OR gate ATG-ROOT-002 sorts after the AND sub-gate ATG-ROOT-001.
    # Rolled up from the true root the tree is medium, matching TS-ROOT-001.
    _scn "the roll-up starts at the gate no other gate lists as an input, not the first in file order"
    out=$("$SYSCRIBE" -m "$B/main" validate 2>/dev/null) || true
    grep -qF "| W035 |" <<<"$out" \
        && fail "spurious W035 — the roll-up did not start at the root gate: $(grep -F '| W035 |' <<<"$out")" \
        || pass "no W035: computed medium matches declared medium"
    grep -qF "0 errors" <<<"$out" && pass "no errors" || fail "unexpected errors"

    _scn "a mismatch is still reported against the root's value"
    out=$("$SYSCRIBE" -m "$B/mismatch" validate 2>/dev/null) || true
    hits=$(grep -F "| W035 |" <<<"$out" || true)
    grep -qF "computed feasibility 'medium'" <<<"$hits" && grep -qF "declared attackFeasibility 'high'" <<<"$hits" \
        && pass "W035 names computed medium vs declared high" \
        || fail "W035 missing or wrong computed value: $hits"
}
