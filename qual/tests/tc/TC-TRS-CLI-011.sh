tc_TRS_CLI_011() {
    local F="$1"; local B="$F/TC-TRS-CLI-011"
    local ROOT; ROOT="$(cd "$F/../.." && pwd)"
    _scn() { _flush_scenario; SCENARIO_NAME="$1"; _SCEN_PASS=0; _SCEN_FAIL=0; printf "  ▶ %s\n" "$1"; }
    local out t id

    _scn "list finds Zone, Conduit and TestPlan elements"
    for t in Zone:ZN-CLI-001 Conduit:CD-CLI-001 TestPlan:TP-CLI-001; do
        id=${t#*:}; t=${t%%:*}
        out=$("$SYSCRIBE" -m "$B/model" list "$t" 2>&1) || true
        grep -qF "$id" <<<"$out" && pass "list $t lists $id" || fail "list $t does not list $id"
    done
    out=$("$SYSCRIBE" -m "$B/model" list zone 2>&1) || true
    grep -qF "ZN-CLI-002" <<<"$out" && pass "the type name matches case-insensitively" \
        || fail "list zone does not list ZN-CLI-002"
    out=$("$SYSCRIBE" -m "$B/model" show ZN-CLI-001 2>&1) || true
    grep -qF "| **type** | Zone |" <<<"$out" && pass "show labels the Zone as Zone" \
        || fail "show does not label the Zone as Zone"

    _scn "list finds an element of every type in the inventory"
    local S; S=$(mktemp -d)
    # The type inventory: every ElementType variant except the Unknown fallback.
    local types
    types=$(sed -n '/pub enum ElementType {/,/^}/p' "$ROOT/crates/syscribe-model/src/element.rs" \
        | grep -oE '^    [A-Z][A-Za-z0-9]*,' | tr -d ' ,' | grep -vx Unknown)
    local d f nm stem missing=""
    for t in $types; do
        [ "$t" = FMEAEntry ] && continue   # never authored standalone; exploded from FMEASheet rows
        out=$("$SYSCRIBE" template "$t" 2>/dev/null) || continue
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
    for t in $types; do
        out=$("$SYSCRIBE" -m "$S" list "$t" 2>&1) || true
        grep -qE "^# $t elements \(" <<<"$out" || missing="$missing $t"
    done
    if [ -z "$missing" ]; then pass "list finds every type ($(wc -w <<<"$types") types)"
    else fail "list finds no element for:$missing"; fi
    rm -rf "$S"
}
