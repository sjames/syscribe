tc_TRS_PLANITEM_008() {
    local F="$1"; local FX="$F/TC-TRS-PLANITEM-008"
    local out

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    out=$("$SYSCRIBE" -m "$FX/model" validate 2>&1 || true)

    # 1. a well-formed username in a configured roster validates cleanly
    _scn "a well-formed username in a configured roster validates cleanly"
    printf '%s' "$out" | grep 'Declared.md' | grep -qE 'E72[23]' \
        && fail "unexpected assignedTo error on Declared.md" \
        || pass "Declared.md raises no assignedTo error"

    # 2. a well-formed username not in the roster is rejected
    _scn "a well-formed username not in the roster is rejected"
    printf '%s' "$out" | grep 'Undeclared.md' | grep -q 'E722' \
        && pass "E722 raised for an undeclared user" || fail "E722 not raised for an undeclared user"

    # 3. a malformed username is rejected regardless of roster configuration
    _scn "a malformed username is rejected regardless of roster configuration"
    if printf '%s' "$out" | grep 'Malformed.md' | grep -q 'E723'; then
        pass "E723 raised for a malformed username"
    else
        fail "E723 not raised for a malformed username"
    fi
    printf '%s' "$out" | grep 'Malformed.md' | grep -q 'E722' \
        && fail "a malformed value must not also be reported as undeclared (double-reporting)" \
        || pass "no redundant E722 alongside E723"

    # 4. assignedTo is unchecked for roster membership when [users] is not configured
    _scn "assignedTo is unchecked for roster membership when [users] is not configured"
    local nr_out
    nr_out=$("$SYSCRIBE" -m "$FX/no_roster" validate 2>&1 || true)
    printf '%s' "$nr_out" | grep -qE 'E72[23]' \
        && fail "unexpected assignedTo error with no [users] table at all" \
        || pass "no assignedTo error with no [users] table configured"

    # 5. a malformed [users] key is flagged and excluded from the roster
    _scn "a malformed [users] key is flagged and excluded from the roster"
    if printf '%s' "$out" | grep -q 'W309'; then
        pass "W309 raised for the malformed [users] key"
    else
        fail "W309 not raised for the malformed [users] key"
    fi
    printf '%s' "$out" | grep 'Declared.md' | grep -qE 'E72[23]' \
        && fail "the well-formed roster entry must still validate correctly alongside a bad one" \
        || pass "well-formed roster entries are unaffected by the one malformed key"
}
