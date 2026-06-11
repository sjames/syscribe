tc_TRS_XREF_006() {
    local F="$1"; local B="$F/TC-TRS-XREF-006"
    local out rc

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    # 1. a root-prefixed reference whose stripped form resolves gets a hint
    _scn "a root-prefixed reference whose stripped form resolves gets a hint"
    out=$("$SYSCRIBE" -m "$B/rootprefixed" validate 2>&1) && rc=0 || rc=$?
    if printf '%s' "$out" | grep -qF 'EvRoot::A::B'; then
        pass "unresolved-reference error raised for the root-prefixed ref EvRoot::A::B"
    else
        fail "no unresolved-reference error for the root-prefixed ref EvRoot::A::B"
    fi
    if printf '%s' "$out" | grep -q 'hint' && printf '%s' "$out" | grep -qF 'A::B'; then
        pass "finding carries a hint naming the stripped target A::B"
    else
        fail "finding did not carry a hint naming A::B"
    fi

    # 2. the correctly written reference resolves with no finding and no hint
    _scn "the correctly written reference resolves with no finding"
    out=$("$SYSCRIBE" -m "$B/correct" validate 2>&1) && rc=0 || rc=$?
    if printf '%s' "$out" | grep -q 'hint'; then
        fail "a hint appeared for the correctly written reference A::B"
    else
        pass "no hint for the correctly written reference A::B"
    fi

    # 3. an unresolved reference not starting with the root name gets no hint
    _scn "an unresolved reference not starting with the root name gets no hint"
    out=$("$SYSCRIBE" -m "$B/unknown" validate 2>&1) && rc=0 || rc=$?
    if printf '%s' "$out" | grep -qF 'Totally::Unknown'; then
        pass "unresolved-reference error raised for the unknown ref Totally::Unknown"
    else
        fail "no unresolved-reference error for the unknown ref Totally::Unknown"
    fi
    if printf '%s' "$out" | grep -q 'hint'; then
        fail "a root-name hint appeared for a wholly unknown reference"
    else
        pass "no hint for a wholly unknown reference"
    fi
}
