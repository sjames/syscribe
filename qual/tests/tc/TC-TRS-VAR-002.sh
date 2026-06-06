tc_TRS_VAR_002() {
    local F="$1"; local M="$F/TC-TRS-VAR-002/model"

    # Scenario: links shows the TestCase's appliesWhen condition.
    SCENARIO_NAME="links shows TestCase appliesWhen condition"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local lk; lk=$("$SYSCRIBE" -m "$M" links TC-V2-WDT-001 2>/dev/null || true)
    printf '%s' "$lk" | grep -qF "Wdt" \
        && pass "links references the Wdt FeatureDef" || fail "links missing the Wdt condition"

    # Scenario: refs of a selecting configuration lists the conditioned TestCase.
    SCENARIO_NAME="refs of selecting config lists conditioned TestCase"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local rw; rw=$("$SYSCRIBE" -m "$M" refs CONF-MPS2-WDT-001 2>/dev/null || true)
    printf '%s' "$rw" | grep -qF "TC-V2-WDT-001" \
        && pass "Wdt config lists TC-V2-WDT-001" || fail "Wdt config missing TC-V2-WDT-001"

    # Scenario: refs of a deselecting configuration excludes the conditioned TestCase
    # but still lists the configuration-agnostic one.
    SCENARIO_NAME="refs of deselecting config excludes conditioned TestCase"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local rn; rn=$("$SYSCRIBE" -m "$M" refs CONF-M0-BASE-001 2>/dev/null || true)
    printf '%s' "$rn" | grep -qF "TC-V2-WDT-001" \
        && fail "non-Wdt config wrongly lists TC-V2-WDT-001" || pass "non-Wdt config excludes TC-V2-WDT-001"
    printf '%s' "$rn" | grep -qF "TC-V2-CORE-001" \
        && pass "non-Wdt config lists agnostic TC-V2-CORE-001" || fail "non-Wdt config missing agnostic TestCase"
}
