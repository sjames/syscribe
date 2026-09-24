tc_TRS_LINKTYPE_006() {
    local F="$1"; local FX="$F/TC-TRS-LINKTYPE-006"
    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }
    # has <output> <file-substring> <code>  — a finding line mentioning both
    has() { printf '%s' "$1" | grep -F -- "$2" | grep -qE -- "$3"; }

    local out
    out=$("$SYSCRIBE" -m "$FX/model" validate 2>&1 || true)
    _scn "a relaxed code is not raised for the extending link"
    has "$out" "HwA.md" "E313" && fail "E313 raised despite relax" || pass "E313 relaxed on HwA"
    _scn "an unrelaxed extending link raises the base rule"
    has "$out" "HwB.md" "E313" && pass "E313 inherited by strictSat" || fail "E313 not inherited"
    _scn "a built-in link is never relaxed"
    has "$out" "HwC.md" "E313" && pass "E313 on built-in satisfies" || fail "built-in satisfies was relaxed"
    _scn "coverage=true counts toward the base coverage rule"
    has "$out" "REQ-LT6-004.md" "W300" && fail "W300 despite coverage link" || pass "extending link satisfies REQ-LT6-004"
    _scn "coverage=false does not count toward coverage"
    has "$out" "REQ-LT6-005.md" "W300" && pass "W300 kept with coverage=false" || fail "coverage=false link counted"
    _scn "relaxing E310 on a derivedFrom extension"
    has "$out" "REQ-LT6-011.md" "E310" && fail "E310 raised despite relax" || pass "E310 relaxed"
    has "$out" "REQ-LT6-012.md" "E310" && pass "E310 inherited by refinedFrom" || fail "E310 not inherited"
    _scn "reports treat a coverage=true extending link as its base link"
    out=$("$SYSCRIBE" -m "$FX/model" trace REQ-LT6-004 2>&1 || true)
    printf '%s' "$out" | sed -n '/^## Satisfied by/,/^## Verified by/p' | grep -q 'Arch::SwD' \
        && pass "trace lists coverage=true extending satisfier" || fail "trace Satisfied by misses Arch::SwD: $out"
    out=$("$SYSCRIBE" -m "$FX/model" trace REQ-LT6-005 2>&1 || true)
    printf '%s' "$out" | sed -n '/^## Satisfied by/,/^## Verified by/p' | grep -q 'Arch::SwE' \
        && fail "trace lists coverage=false satisfier" || pass "coverage=false satisfier not listed"
    out=$("$SYSCRIBE" -m "$FX/model" who-verifies REQ-LT6-020 2>&1 || true)
    printf '%s' "$out" | grep -q 'TC-LT6-001' && pass "who-verifies lists extending verifier" || fail "who-verifies misses TC-LT6-001: $out"

    _scn "mutation does not rewrite an extending link into the base field"
    local tmp; tmp=$(mktemp -d); cp -r "$FX/model" "$tmp/m"
    "$SYSCRIBE" -m "$tmp/m" set REQ-LT6-011 status=approved >/dev/null 2>&1 || true
    local fl="$tmp/m/Requirements/REQ-LT6-011.md"
    grep -q 'status: approved' "$fl" && pass "set applied" || fail "set did not apply"
    grep -q 'inspiredBy' "$fl" && ! grep -q '^derivedFrom' "$fl" && pass "links: preserved, no derivedFrom written" || fail "extending link rewritten"
    rm -rf "$tmp"
}
