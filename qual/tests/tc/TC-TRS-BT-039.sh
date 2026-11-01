tc_TRS_BT_039() {
    local F="$1"
    local R; R="$(cd "$F/../.." && pwd)"
    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }
    local codes="E750 E751 E752 E753 E754 E755 E756 E757 E758 E759 E760 E761 E762 W750 W751 W752 W753 W754 I750"
    local c missing

    _scn "every code is in the catalogue and the spec"
    missing=""
    for c in $codes; do
        grep -qE "^\| \`?${c}\`? \|" "$R/prompts/spec/validation.md" || missing="$missing validation.md:$c"
        grep -qE "^\| \`?${c}\`? \|" "$R/docs/validation/rules.md" || missing="$missing rules.md:$c"
        grep -qE "^\| \`${c}\` \|" "$R/spec/markdown-sysml-format.md" || missing="$missing spec:$c"
    done
    [ -z "$missing" ] && pass "all $(echo $codes | wc -w) codes are documented in the three places" || fail "undocumented:$missing"

    _scn "every new command has a help page"
    missing=""
    for c in binding-times residual params; do
        "$SYSCRIBE" help "$c" 2>&1 | grep -q 'SYNOPSIS' || missing="$missing $c"
    done
    "$SYSCRIBE" help matrix 2>&1 | grep -q -- '--stage' || missing="$missing matrix(--stage)"
    "$SYSCRIBE" help configure 2>&1 | grep -q -- '--stage' || missing="$missing configure(--stage)"
    [ -z "$missing" ] && pass "binding-times, residual, params, matrix --stage, configure --stage" || fail "no help for:$missing"

    _scn "the guide, the authoring prompt and the worked example describe the feature"
    grep -q 'Binding time and staged configuration' "$R/docs/model-guide/variability.md" && pass "variability guide section" || fail "guide section missing"
    grep -q 'params export' "$R/prompts/create-model.md" && grep -q 'residual' "$R/prompts/create-model.md" && grep -q 'binding-times' "$R/prompts/create-model.md" && pass "authoring prompt teaches the commands" || fail "prompt incomplete"
    [ -f "$R/examples/staged-configuration/README.md" ] && "$SYSCRIBE" -m "$R/examples/staged-configuration/model" validate 2>&1 | grep -q '0 errors, 0 warnings' && pass "the staged-configuration example validates clean" || fail "example missing or not clean"
}
