tc_TRS_TYPE_021() {
    local F="$1"; local B="$F/TC-TRS-TYPE-021"
    local out rc

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    _has()    { printf '%s' "$1" | grep -qF "$2"; }
    _has_re() { printf '%s' "$1" | grep -qE "$2"; }

    # 1. valid composition: no E5xx/W510, cross-repo verifies neither E102 nor E512
    _scn "valid composition with a cross-repo verifies is clean"
    out=$("$SYSCRIBE" -m "$B/clean" validate 2>&1) && rc=0 || rc=$?
    if _has_re "$out" 'E51[0-9]|W510'; then
        fail "valid composition emitted an E51x/W510 finding"
    else
        pass "valid composition is free of E510–E515 and W510"
    fi
    if _has "$out" 'E102' || _has "$out" 'E512'; then
        fail "cross-repo verifies raised E102/E512 in a valid composition"
    else
        pass "cross-repo verifies REQ-PEER-AA-001 resolved without E102/E512"
    fi

    # 2. E510 — circular repo import
    _scn "E510 — circular repo import"
    out=$("$SYSCRIBE" -m "$B/e510" validate 2>&1) && rc=0 || rc=$?
    if _has "$out" 'E510'; then pass "E510 emitted for circular repo import"
    else fail "E510 not emitted for circular repo import"; fi

    # 3. E511 — repo path missing and no ref
    _scn "E511 — repo path missing and no ref"
    out=$("$SYSCRIBE" -m "$B/e511" validate 2>&1) && rc=0 || rc=$?
    if _has "$out" 'E511'; then pass "E511 emitted for missing repo path"
    else fail "E511 not emitted for missing repo path"; fi

    # 4. E512 — dangling cross-repo reference
    _scn "E512 — dangling cross-repo reference"
    out=$("$SYSCRIBE" -m "$B/e512" validate 2>&1) && rc=0 || rc=$?
    if _has "$out" 'E512'; then pass "E512 emitted for a reference present in no repo"
    else fail "E512 not emitted for a dangling cross-repo reference"; fi

    # 5. E513 — repoImports names an unknown alias
    _scn "E513 — repoImports names an unknown alias"
    out=$("$SYSCRIBE" -m "$B/e513" validate 2>&1) && rc=0 || rc=$?
    if _has "$out" 'E513'; then pass "E513 emitted for an unknown repo alias"
    else fail "E513 not emitted for an unknown repo alias"; fi

    # 6. E514 — repoImports qname not in the peer
    _scn "E514 — repoImports qname not in the peer"
    out=$("$SYSCRIBE" -m "$B/e514" validate 2>&1) && rc=0 || rc=$?
    if _has "$out" 'E514'; then pass "E514 emitted for an unresolved import qname"
    else fail "E514 not emitted for an unresolved import qname"; fi

    # 7. E515 — duplicate stable id across repos
    _scn "E515 — duplicate stable id across repos"
    out=$("$SYSCRIBE" -m "$B/e515" validate 2>&1) && rc=0 || rc=$?
    if _has "$out" 'E515'; then pass "E515 emitted for a stable id shared with the peer"
    else fail "E515 not emitted for a duplicate stable id"; fi

    # 8. W510 — repo with no ref
    _scn "W510 — repo with no ref"
    out=$("$SYSCRIBE" -m "$B/w510" validate 2>&1) && rc=0 || rc=$?
    if _has "$out" 'W510'; then pass "W510 emitted for an unpinned repo"
    else fail "W510 not emitted for an unpinned repo"; fi

    # 9. repos list command
    _scn "repos list command"
    out=$("$SYSCRIBE" -m "$B/clean" repos list 2>&1) && rc=0 || rc=$?
    if _has "$out" 'peer' && _has "$out" '../peer' && _has "$out" 'main'; then
        pass "repos list reports the peer alias, path, and ref"
    else
        fail "repos list did not report the configured peer"
    fi
}
