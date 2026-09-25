tc_TRS_CLI_009() {
    local F="$1"
    local ROOT; ROOT="$(cd "$F/../.." && pwd)"
    _scn() { _flush_scenario; SCENARIO_NAME="$1"; _SCEN_PASS=0; _SCEN_FAIL=0; printf "  ▶ %s\n" "$1"; }
    local S; S=$(mktemp -d)
    # The type inventory: every ElementType variant except the Unknown fallback.
    local types
    types=$(sed -n '/pub enum ElementType {/,/^}/p' "$ROOT/crates/syscribe-model/src/element.rs" \
        | grep -oE '^    [A-Z][A-Za-z0-9]*,' | tr -d ' ,' | grep -vx Unknown)

    _scn "every element type has a template"
    local t out ec failed=0 id nm stem d f
    for t in $types; do
        out=$("$SYSCRIBE" template "$t" 2>&1) && ec=0 || ec=$?
        if [ "$t" = FMEAEntry ]; then
            if [ "$ec" -ne 0 ] && grep -qF "template FMEASheet" <<<"$out"; then pass "FMEAEntry points at FMEASheet"; else fail "FMEAEntry: exit $ec"; fi
            continue
        fi
        if [ "$ec" -ne 0 ]; then fail "template $t exits $ec: $(head -1 <<<"$out")"; failed=1; continue; fi
        # Place the skeleton the way its own comments direct.
        case $t in
            FaultTreeGate|FaultTreeEvent) d="$S/FaultTree/FT-PREFIX-001" ;;
            AttackTreeGate|AttackStep)    d="$S/AttackTree/AT-PREFIX-001" ;;
            *)                            d="$S/$t" ;;
        esac
        mkdir -p "$d"
        id=$(sed -n '2,/^---$/p' <<<"$out" | grep -m1 -E '^id:' | sed 's/^id: *//; s/"//g; s/ *#.*//' || true)
        nm=$(sed -n '2,/^---$/p' <<<"$out" | grep -m1 -E '^name:' | sed 's/^name: *//; s/"//g; s/ *#.*//' || true)
        stem=${id:-$nm}; [[ "$stem" =~ ^[A-Za-z_][A-Za-z0-9_-]*$ ]] || stem=$t
        case $t in Package|LibraryPackage|Namespace) f="$d/_index.md" ;; *) f="$d/$stem.md" ;; esac
        printf '%s' "$out" > "$f"
    done
    [ "$failed" -eq 0 ] && pass "template succeeds for every type ($(wc -w <<<"$types") types)"

    _scn "the Baseline template carries the baseline create fields"
    out=$("$SYSCRIBE" template Baseline 2>/dev/null) || true
    for k in "type: Baseline" "gitTag:" "gitCommit:" "frozenScope:" "seal:" "aggregateHash:"; do
        grep -qF "$k" <<<"$out" && pass "Baseline template has $k" || fail "Baseline template lacks $k"
    done

    _scn "all templates validate together with only placeholder findings"
    local findings bad
    findings=$("$SYSCRIBE" -m "$S" validate --json 2>/dev/null | jq -r '.[] | "\(.code)\t\(.file)\t\(.message)"') || true
    bad=$(awk -F'\t' '
        $1 ~ /^(W005|W007|W039|W613|W803|W804|W805)$/ { next }
        tolower($3) ~ /does not resolve|unresolved/ { next }
        { print }' <<<"$findings")
    if [ -z "$bad" ]; then pass "only allowed findings"; else fail "unexpected findings:"; printf '%s\n' "$bad" | sed 's/^/        /'; fi
    grep -P '^(W073|W075)\t' <<<"$findings" | grep -qF "StateDef" && fail "StateDef raises W073/W075" || pass "StateDef: no W073/W075"
    grep -P '^(W030|W032)\t' <<<"$findings" | grep -qF "TARASheet" && fail "TARASheet raises W030/W032" || pass "TARASheet: no W030/W032"

    _scn "the unknown-type error lists every known type"
    out=$("$SYSCRIBE" template NoSuchType 2>&1) && ec=0 || ec=$?
    [ "$ec" -ne 0 ] && pass "unknown type exits $ec" || fail "unknown type exits 0"
    for k in ReviewRecord TradeStudy Zone Conduit Baseline OccurrenceDef; do
        grep -qw "$k" <<<"$out" && pass "known types list $k" || fail "known types omit $k"
    done
    rm -rf "$S"
}
