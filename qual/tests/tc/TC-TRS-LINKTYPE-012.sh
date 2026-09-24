tc_TRS_LINKTYPE_012() {
    local F="$1"; local FX="$F/TC-TRS-LINKTYPE-012"
    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }
    # has <output> <file-substring> <code>  — a finding line mentioning both
    has() { printf '%s' "$1" | grep -F -- "$2" | grep -qE -- "$3"; }

    local out
    _scn "the general prompt documents links: and link-types"
    out=$("$SYSCRIBE" --agent-instructions 2>&1) || true
    grep -q 'links:' <<<"$out" && grep -q 'link-types' <<<"$out" && pass "prompt documents links" || fail "prompt missing links docs"
    _scn "a model declaring link types appends a Project link types section"
    out=$("$SYSCRIBE" -m "$F/TC-TRS-LINKTYPE-001/good" --agent-instructions 2>&1) || true
    grep -q '^## Project link types' <<<"$out" && grep -q 'mitigatedBy' <<<"$out" && grep -q 'partiallySatisfies' <<<"$out" && pass "section appended" || fail "section missing"
    _scn "a model declaring none appends nothing"
    out=$("$SYSCRIBE" -m "$F/TC-TRS-LINKTYPE-001/none" --agent-instructions 2>&1) || true
    grep -q '^## Project link types' <<<"$out" && fail "section appended for none" || pass "no section"
}
