tc_TRS_SAFE_011() {
    local F="$1"; local M="$F/TC-TRS-SAFE-011/model"

    SCENARIO_NAME="goal with explicit Argument suppresses implicit requirements fold-in"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    out=$("$SYSCRIBE" -m "$M" safety-case 2>/dev/null || true)
    # Implicit fold-in would show a top-level "├── [evidence:Requirement]" line without the
    # argument indentation. The argument's own evidence line is indented deeper.
    # Check that there is no un-indented "[evidence:Requirement]" line directly under SG-CTRL-001.
    sg_ctrl_direct=$(printf '%s' "$out" | awk '/\[SafetyGoal\] SG-CTRL-001/{found=1; next} found && /^\[SafetyGoal\]/{found=0} found && /^[├└]── \[evidence:Requirement\]/{print}')
    [ -z "$sg_ctrl_direct" ] \
        && pass "SG-CTRL-001 does not show direct implicit fold-in (has explicit Argument)" \
        || fail "SG-CTRL-001 unexpectedly shows direct implicit REQ fold-in (has explicit Argument)"

    SCENARIO_NAME="goal without Argument still shows implicit requirements fold-in"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    sg_prot=$(printf '%s' "$out" | awk '/\[SafetyGoal\] SG-PROT-001/{found=1} found && /^\[SafetyGoal\]/{if(!/SG-PROT-001/){found=0}} found{print}')
    printf '%s' "$sg_prot" | grep -qF "[evidence:Requirement] REQ-PROT-001" \
        && pass "SG-PROT-001 shows implicit fold-in (no Argument)" \
        || fail "SG-PROT-001 missing implicit fold-in (no Argument)"

    SCENARIO_NAME="--no-implicit suppresses fold-in for all goals"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    out_no_implicit=$("$SYSCRIBE" -m "$M" safety-case --no-implicit 2>/dev/null || true)
    # With --no-implicit, only direct top-level evidence:Requirement lines should be absent.
    # Argument-internal evidence may still show (indented deeper), but SG-PROT-001 has no Arguments
    # so its direct fold-in should be suppressed.
    direct_reqs=$(printf '%s' "$out_no_implicit" | grep -cE "^[├└]── \[evidence:Requirement\]" || true)
    [ "$direct_reqs" -eq 0 ] \
        && pass "--no-implicit suppresses all direct implicit fold-in" \
        || fail "--no-implicit still shows $direct_reqs direct evidence:Requirement lines"

    SCENARIO_NAME="JSON has empty requirements array for goal with explicit Argument"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    json=$("$SYSCRIBE" -m "$M" safety-case --json 2>/dev/null || true)
    printf '%s' "$json" | python3 -c "
import json, sys
d = json.load(sys.stdin)
g = next(x for x in d['goals'] if x['id'] == 'SG-CTRL-001')
reqs = g.get('requirements', None)
assert reqs is not None, 'requirements key missing for SG-CTRL-001'
assert reqs == [], f'requirements not empty for SG-CTRL-001 (has explicit Argument): {reqs}'
print('ok')
" 2>/dev/null && pass "JSON: SG-CTRL-001 has empty 'requirements' array" || fail "JSON: SG-CTRL-001 'requirements' not empty"
}
