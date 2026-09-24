tc_TRS_LINKTYPE_008() {
    local F="$1"; local FX="$F/TC-TRS-LINKTYPE-008"
    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }
    # has <output> <file-substring> <code>  — a finding line mentioning both
    has() { printf '%s' "$1" | grep -F -- "$2" | grep -qE -- "$3"; }

    local out
    _scn "text output lists each type and its rules"
    out=$("$SYSCRIBE" -m "$F/TC-TRS-LINKTYPE-001/good" link-types 2>&1) || true
    printf '%s' "$out" | grep -q 'mitigates' && printf '%s' "$out" | grep -q 'mitigatedBy' && printf '%s' "$out" | grep -q 'partiallySatisfies' && printf '%s' "$out" | grep -q 'E313' \
        && pass "text lists types, inverse and relax" || fail "text incomplete: $out"
    _scn "json output lists each type and its rules"
    out=$("$SYSCRIBE" -m "$F/TC-TRS-LINKTYPE-001/good" link-types --json 2>/dev/null) || true
    printf '%s' "$out" | jq -e '(.linkTypes|length)==2 and (.linkTypes[]|select(.name=="mitigates")|.inverse=="mitigatedBy" and .sourceTypes==["PartDef"] and .count==1) and (.linkTypes[]|select(.name=="partiallySatisfies")|.extends=="satisfies" and .relax==["E313"] and .coverage==false)' >/dev/null \
        && pass "json complete" || fail "json wrong: $out"
    _scn "an invalid entry is not listed as usable"
    out=$("$SYSCRIBE" -m "$F/TC-TRS-LINKTYPE-001/bad" link-types --json 2>/dev/null) || true
    printf '%s' "$out" | jq -e '[.linkTypes[].name]|index("badCard")==null' >/dev/null && pass "invalid entry omitted" || fail "invalid entry listed"
    _scn "an unconfigured model prints a hint and exits zero"
    out=$("$SYSCRIBE" -m "$F/TC-TRS-LINKTYPE-001/none" link-types 2>&1) && pass "exit zero" || fail "non-zero exit"
    printf '%s' "$out" | grep -qi 'no link types' && printf '%s' "$out" | grep -q 'linkTypes' && pass "hint shown" || fail "no hint: $out"
}
