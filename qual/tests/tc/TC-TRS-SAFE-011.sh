tc_TRS_SAFE_011() {
    local F="$1"; local M="$F/TC-TRS-SAFE-011/model"

    # GH #217: the implicit chain is no longer dropped when a goal has an Argument. A
    # derived requirement NOT cited by the Argument is folded in (marked implicit);
    # one the Argument cites is shown once, under the Argument.
    SCENARIO_NAME="goal with explicit Argument still folds in uncited derived requirements"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    out=$("$SYSCRIBE" -m "$M" safety-case 2>/dev/null || true)
    sg_ctrl=$(printf '%s' "$out" | awk '/\[SafetyGoal\] SG-CTRL-001/{found=1; print; next} found && /^\[SafetyGoal\]/{found=0} found{print}')
    printf '%s' "$sg_ctrl" | grep -qE "^[├└]── \[evidence:Requirement\] REQ-CTRL-002.*\(implicit\)" \
        && pass "SG-CTRL-001 shows uncited REQ-CTRL-002 as an implicit requirement" \
        || fail "SG-CTRL-001 dropped the implicit REQ-CTRL-002 (has explicit Argument)"
    n=$(printf '%s' "$sg_ctrl" | grep -c "REQ-CTRL-001 " || true)
    [ "$n" -eq 1 ] \
        && pass "REQ-CTRL-001 (cited by the Argument) is shown once, not repeated as implicit" \
        || fail "REQ-CTRL-001 shown $n times under SG-CTRL-001 (expected 1)"

    SCENARIO_NAME="goal without Argument still shows implicit requirements fold-in"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    sg_prot=$(printf '%s' "$out" | awk '/\[SafetyGoal\] SG-PROT-001/{found=1} found && /^\[SafetyGoal\]/{if(!/SG-PROT-001/){found=0}} found{print}')
    printf '%s' "$sg_prot" | grep -qF "[evidence:Requirement] REQ-PROT-001" \
        && pass "SG-PROT-001 shows implicit fold-in (no Argument)" \
        || fail "SG-PROT-001 missing implicit fold-in (no Argument)"

    SCENARIO_NAME="--no-implicit suppresses fold-in for all goals"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    out_no_implicit=$("$SYSCRIBE" -m "$M" safety-case --no-implicit 2>/dev/null || true)
    # With --no-implicit no goal has a direct (un-indented) evidence:Requirement line.
    direct_reqs=$(printf '%s' "$out_no_implicit" | grep -cE "^[├└]── \[evidence:Requirement\]" || true)
    [ "$direct_reqs" -eq 0 ] \
        && pass "--no-implicit suppresses all direct implicit fold-in" \
        || fail "--no-implicit still shows $direct_reqs direct evidence:Requirement lines"

    SCENARIO_NAME="JSON lists the implicit requirement for a goal with an explicit Argument"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    json=$("$SYSCRIBE" -m "$M" safety-case --json 2>/dev/null || true)
    printf '%s' "$json" | python3 -c "
import json, sys
d = json.load(sys.stdin)
g = next(x for x in d['goals'] if x['id'] == 'SG-CTRL-001')
ids = [r['id'] for r in g['requirements']]
assert ids == ['REQ-CTRL-002'], ids
assert g['requirements'][0]['implicit'] is True
print('ok')
" 2>/dev/null && pass "JSON: SG-CTRL-001 requirements = [REQ-CTRL-002] (implicit)" || fail "JSON: SG-CTRL-001 implicit requirements wrong"

    SCENARIO_NAME="derivedChildren are walked transitively and the completeness summary is printed"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    printf '%s' "$out" | grep -qF "Completeness:" \
        && pass "text output ends with a Completeness summary" || fail "no Completeness summary"
    printf '%s' "$json" | python3 -c "
import json, sys
d = json.load(sys.stdin)
c = d['completeness']
assert c['goals'] == 2 and c['requirements'] == 3, c
assert d['goals'][0]['verdict'] in ('supported','incomplete','failing')
print('ok')
" 2>/dev/null && pass "JSON carries completeness and a per-goal verdict" || fail "JSON completeness missing"

    SCENARIO_NAME="unknown goal id exits non-zero"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    "$SYSCRIBE" -m "$M" safety-case SG-NOPE-999 >/dev/null 2>&1 && rc=0 || rc=$?
    [ "$rc" -ne 0 ] && pass "safety-case SG-NOPE-999 exits $rc" || fail "unknown goal id exited 0"
}
