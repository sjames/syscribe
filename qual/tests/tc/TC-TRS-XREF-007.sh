tc_TRS_XREF_007() {
    local F="$1"; local B="$F/TC-TRS-XREF-007"
    local out rc c

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }
    _count() { grep -cF "| $1 |" <<<"$out" || true; }

    # 1. every structural field with a missing target raises its own code
    _scn "every structural field with a missing target raises its own code"
    out=$("$SYSCRIBE" -m "$B/unresolved" validate 2>&1) && rc=0 || rc=$?
    for c in E110 E112 E113 E114; do
        [ "$(_count "$c")" = "1" ] && pass "$c raised once" || fail "$c count $(_count "$c") (expected 1)"
    done
    [ "$(_count E111)" = "2" ] && pass "E111 raised for the usage and the inline feature" \
        || fail "E111 count $(_count E111) (expected 2)"
    grep -qF "'Nope::FeatureType' on inline feature 'gear'" <<<"$out" \
        && pass "inline-feature E111 names the feature" || fail "inline-feature E111 does not name the feature"
    [ "$rc" -ne 0 ] && pass "exit code non-zero" || fail "exit code 0 (expected non-zero)"

    # 2. references resolving by any §11.5 form raise nothing
    _scn "references resolving by any §11.5 form raise nothing"
    out=$("$SYSCRIBE" -m "$B/resolved" validate 2>&1) && rc=0 || rc=$?
    for c in E110 E111 E112 E113 E114; do
        [ "$(_count "$c")" = "0" ] && pass "no $c" || fail "unexpected $c: $(grep -F "| $c |" <<<"$out")"
    done
    [ "$rc" -eq 0 ] && pass "exit code 0" || fail "exit code $rc (expected 0)"

    # 3. a root-prefixed supertype carries the root-name hint
    _scn "a root-prefixed supertype carries the root-name hint"
    out=$("$SYSCRIBE" -m "$B/rootprefixed" validate 2>&1) && rc=0 || rc=$?
    grep -F "| E110 |" <<<"$out" | grep -qF "did you mean 'Lib::Base'" \
        && pass "E110 carries the hint naming Lib::Base" || fail "E110 missing the root-name hint"

    # 4. a [repos] model reports an unresolved reference once, as E512
    _scn "a [repos] model reports an unresolved reference once, as E512"
    out=$("$SYSCRIBE" -m "$B/repos/model" validate 2>&1) && rc=0 || rc=$?
    grep -qF "'Lib::PeerBase'" <<<"$out" \
        && fail "the peer-resolving supertype was reported" || pass "peer-resolving supertype raises nothing"
    grep -F "| E512 |" <<<"$out" | grep -qF "cross-repo supertype reference 'Lib::NotInPeer'" \
        && pass "missing supertype raises E512" || fail "missing supertype did not raise E512"
    [ "$(_count E110)" = "0" ] && pass "no E110 alongside E512" || fail "E110 double-reported with E512"
    [ "$(_count E114)" = "0" ] && pass "no E114 alongside E512" || fail "E114 double-reported with E512"
}
