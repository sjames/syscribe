tc_TRS_MG_014() {
    local F="$1"; local FX="$F/TC-TRS-MG-014"
    local out

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }
    # `|| true`: a fixture may carry an incidental MagicGrid error (e.g. MG041 on an
    # unrealised logical part) that exits non-zero; we assert on code strings, not exit.
    _gate() { "$SYSCRIBE" -m "$1" validate --profile magicgrid 2>&1 || true; }

    # MG080 — orphan B1 need
    _scn "an orphan B1 need raises MG080"
    out=$(_gate "$FX/mg080")
    printf '%s' "$out" | grep -q 'MG080' \
        && pass "MG080 raised for an orphan B1 need" || fail "MG080 not raised"

    _scn "a refined need clears MG080"
    out=$(_gate "$FX/mg080cleared")
    printf '%s' "$out" | grep -q 'MG080' \
        && fail "MG080 still raised after refinement" || pass "MG080 cleared by a refining use case"

    # MG081 — unallocated W2 function
    _scn "an unallocated W2 function raises MG081"
    out=$(_gate "$FX/mg081")
    printf '%s' "$out" | grep -q 'MG081' \
        && pass "MG081 raised for an unallocated W2 function" || fail "MG081 not raised"

    _scn "allocating the W2 function clears MG081"
    out=$(_gate "$FX/mg081cleared")
    printf '%s' "$out" | grep -q 'MG081' \
        && fail "MG081 still raised after allocation" || pass "MG081 cleared by a functional allocation"

    # MG082 — missing System of Interest
    _scn "a system context with no SoI raises MG082"
    out=$(_gate "$FX/mg082")
    printf '%s' "$out" | grep -q 'MG082' \
        && pass "MG082 raised when an external actor exists but no SoI" || fail "MG082 not raised"

    _scn "marking the SoI clears MG082"
    out=$(_gate "$FX/mg082cleared")
    printf '%s' "$out" | grep -q 'MG082' \
        && fail "MG082 still raised after marking the SoI" || pass "MG082 cleared by mg_soi"

    # MG083 — MoE without a MoP
    _scn "a MoE with no MoP raises MG083"
    out=$(_gate "$FX/mg083")
    printf '%s' "$out" | grep -q 'MG083' \
        && pass "MG083 raised for a MoE with no MoP" || fail "MG083 not raised"

    # inert without the gate
    _scn "the completeness checks are inert without the gate"
    out=$("$SYSCRIBE" -m "$FX/mg080" validate 2>&1)
    printf '%s' "$out" | grep -qE 'MG08[0-9]' \
        && fail "MG08x raised without the magicgrid profile" || pass "no MG08x finding without the gate"
}
