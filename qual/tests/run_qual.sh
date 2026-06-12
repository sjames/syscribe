#!/usr/bin/env bash
# qual/tests/run_qual.sh — Syscribe Tool Qualification Test Runner
#
# Usage:
#   ./qual/tests/run_qual.sh [--no-build] [TC-TRS-PARSE-001 ...]
#
# With no arguments: runs all test cases discovered in qual/TestCases/.
# With TC IDs: runs only those test cases.
# --no-build: skip cargo build (use existing binary).
#
# Requirements:
#   - jq       (JSON parsing)
#   - cargo    (build, unless --no-build)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

export SYSCRIBE="$REPO_ROOT/target/debug/syscribe"
FIXTURES="$REPO_ROOT/qual/fixtures"
TC_DIR="$SCRIPT_DIR/tc"
TVR_DIR="$SCRIPT_DIR/tvr"
export RESULTS_FILE="$TVR_DIR/results.ndjson"

source "$SCRIPT_DIR/lib.sh"

# ── Argument parsing ──────────────────────────────────────────────────────────

BUILD=1
FILTER=()
for arg in "$@"; do
    case "$arg" in
        --no-build) BUILD=0 ;;
        TC-TRS-*) FILTER+=("$arg") ;;
        *) echo "Unknown argument: $arg" >&2; exit 1 ;;
    esac
done

# ── Build ─────────────────────────────────────────────────────────────────────

if [ "$BUILD" -eq 1 ]; then
    printf "${BOLD}Building syscribe...${NC}\n"
    cd "$REPO_ROOT"
    cargo build --package syscribe 2>&1 | tail -3
fi

if [ ! -x "$SYSCRIBE" ]; then
    printf "${RED}Binary not found: %s${NC}\n" "$SYSCRIBE" >&2
    exit 1
fi

SYSCRIBE_VERSION=$("$SYSCRIBE" --version 2>/dev/null | head -1 || echo "unknown")
printf "${BOLD}Binary:${NC} %s (%s)\n" "$SYSCRIBE" "$SYSCRIBE_VERSION"

# ── Discover test cases ───────────────────────────────────────────────────────

mkdir -p "$TVR_DIR"
> "$RESULTS_FILE"   # truncate

TC_FILES=()
while IFS= read -r f; do
    TC_FILES+=("$f")
done < <(find "$REPO_ROOT/qual/TestCases" -name "TC-TRS-*.md" | sort)

if [ "${#TC_FILES[@]}" -eq 0 ]; then
    printf "${RED}No test case files found in qual/TestCases/${NC}\n" >&2
    exit 1
fi

printf "${BOLD}Discovered %d test cases${NC}\n" "${#TC_FILES[@]}"

# ── Run tests ─────────────────────────────────────────────────────────────────

TOTAL=0; PASSED=0; FAILED=0; SKIPPED=0

for tc_file in "${TC_FILES[@]}"; do
    # Read metadata from frontmatter (TestCases label via `name`, REQ-TRS-NAME-002)
    tc_id=$(fm_get "$tc_file" "id")
    tc_title=$(fm_get "$tc_file" "name")
    tc_verifies=$(fm_get_list "$tc_file" "verifies" | paste -sd', ')

    [ -z "$tc_id" ] && continue

    # Apply filter
    if [ "${#FILTER[@]}" -gt 0 ]; then
        match=0
        for f in "${FILTER[@]}"; do
            [ "$f" = "$tc_id" ] && match=1 && break
        done
        [ "$match" -eq 0 ] && continue
    fi

    # Find the test function file
    tc_script="$TC_DIR/${tc_id}.sh"
    if [ ! -f "$tc_script" ]; then
        printf "${YELLOW}⚠ SKIP${NC}  [%s] — no script at %s\n" "$tc_id" "$tc_script"
        SKIPPED=$((SKIPPED + 1))
        continue
    fi

    # Derive function name: TC-TRS-PARSE-001 → tc_TRS_PARSE_001
    fn_name=$(echo "$tc_id" | tr '-' '_' | sed 's/^TC_/tc_/')

    # Run
    TOTAL=$((TOTAL + 1))
    start_tc "$tc_id" "${tc_title:-$tc_id}"
    source "$tc_script"
    "$fn_name" "$FIXTURES"
    end_tc "$tc_verifies"

    if [ "$TC_FAILED" -eq 0 ]; then
        PASSED=$((PASSED + 1))
    else
        FAILED=$((FAILED + 1))
    fi
done

# ── Summary ───────────────────────────────────────────────────────────────────

printf "\n${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}\n"
printf "${BOLD}Results:${NC}  %d total  ${GREEN}%d passed${NC}  ${RED}%d failed${NC}  ${YELLOW}%d skipped${NC}\n" \
    "$TOTAL" "$PASSED" "$FAILED" "$SKIPPED"
printf "${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}\n"

# Generate TVR
"$SCRIPT_DIR/tvr/generate_tvr.sh" "$SYSCRIBE_VERSION"

printf "\n${BOLD}TVR written to:${NC} %s\n" "$TVR_DIR/TVR.md"

[ "$FAILED" -eq 0 ]
