tc_TRS_MG_010() {
    local F="$1"; local B="$F/TC-TRS-MG-010"
    local out rc

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    # These are BASE checks — plain `validate`, no profile.

    # 1. an ActionDef refines a Requirement by id; the requirement back-links it
    _scn "an ActionDef refines link resolves and back-links the requirement"
    out=$("$SYSCRIBE" -m "$B/ok" validate 2>&1) && rc=0 || rc=$?
    if printf '%s' "$out" | grep -q 'E316'; then
        fail "well-formed behavioral refines unexpectedly produced E316"
    else
        pass "no E316 for well-formed behavioral refines"
    fi
    out=$("$SYSCRIBE" -m "$B/ok" show REQ-MG-BRAKE-001 2>&1) || true
    if printf '%s' "$out" | grep -q 'BrakeAction'; then
        pass "REQ-MG-BRAKE-001 show lists the refining BrakeAction (refinedBy)"
    else
        fail "REQ-MG-BRAKE-001 show did not list the refining BrakeAction (refinedBy)"
    fi

    # 2. resolution by qualified name also back-links (StateDef → Requirement by qname)
    _scn "resolution by qualified name also works"
    out=$("$SYSCRIBE" -m "$B/ok" show REQ-MG-MODE-001 2>&1) || true
    if printf '%s' "$out" | grep -q 'ModeState'; then
        pass "REQ-MG-MODE-001 show lists the refining ModeState (refinedBy)"
    else
        fail "REQ-MG-MODE-001 show did not list the refining ModeState (refinedBy)"
    fi

    # 3. an unresolved behavioral refines target raises E316 (StateDef → nothing)
    _scn "an unresolved behavioral refines target raises E316"
    out=$("$SYSCRIBE" -m "$B/e316unresolved" validate 2>&1) && rc=0 || rc=$?
    printf '%s' "$out" | grep -q 'E316' \
        && pass "E316 raised for a StateDef refines that resolves to nothing" \
        || fail "E316 not raised for a StateDef refines that resolves to nothing"

    # 4. a behavioral refines target that is not a requirement raises E316 (ActionDef → PartDef)
    _scn "a behavioral refines target that is not a requirement raises E316"
    out=$("$SYSCRIBE" -m "$B/e316nonreq" validate 2>&1) && rc=0 || rc=$?
    printf '%s' "$out" | grep -q 'E316' \
        && pass "E316 raised for an ActionDef refines naming a PartDef" \
        || fail "E316 not raised for an ActionDef refines naming a PartDef"

    # 5. a behavioral def with no refines raises no W307 against it
    _scn "a behavioral def with no refines raises no W307"
    out=$("$SYSCRIBE" -m "$B/ok" validate 2>&1) && rc=0 || rc=$?
    if printf '%s' "$out" | grep 'W307' | grep -q 'IdleAction'; then
        fail "W307 wrongly named the refines-less ActionDef IdleAction"
    else
        pass "no W307 finding names IdleAction"
    fi
}
