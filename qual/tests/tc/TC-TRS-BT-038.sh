tc_TRS_BT_038() {
    local F="$1"
    local R; R="$(cd "$F/../.." && pwd)"
    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }
    # Every code introduced by the binding-time work.
    local NEW='(E75[0-9]|E76[0-2]|W75[0-4]|I750|E229|E230)'
    local models=() m out n=0 bad=0 hits
    for m in "$R"/model "$R"/model_* "$R"/examples/*/model "$R"/examples/*/model_* "$R"/examples/*/*/model "$R"/examples/*/*/*/model; do
        [ -d "$m" ] && models+=("$m")
    done

    _scn "no existing model gains a binding-time finding"
    for m in "${models[@]}"; do
        case "$m" in */examples/staged-configuration/*) continue ;; esac   # the one model that uses the feature
        n=$((n+1))
        hits=$("$SYSCRIBE" -m "$m" validate 2>&1 | grep -E "^\| ${NEW} \|" || true)
        if [ -n "$hits" ]; then bad=$((bad+1)); printf '    %s\n%s\n' "$m" "$hits"; fi
    done
    [ "$n" -ge 15 ] && pass "checked $n models" || fail "only $n models found"
    [ "$bad" -eq 0 ] && pass "none raises a binding-time code" || fail "$bad model(s) raise a binding-time code"

    _scn "the worked example validates clean and the feature is inert without it"
    out=$("$SYSCRIBE" -m "$R/examples/staged-configuration/model" validate 2>&1 || true)
    printf '%s' "$out" | grep -q '0 errors, 0 warnings' && pass "staged-configuration is clean" || fail "example not clean: $out"
    out=$("$SYSCRIBE" -m "$F/TC-TRS-BT-001/plain" validate 2>&1 || true)
    printf '%s' "$out" | grep -qE "^\| ${NEW} \|" && fail "a model with no windows raised a binding-time code" || pass "a model with no windows or stages raises none"
}
