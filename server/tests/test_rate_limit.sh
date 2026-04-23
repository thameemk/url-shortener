#!/usr/bin/env bash
#
# Rate limit smoke-test for the URL shortener.
#
# Tests:
#   1. API tier  — POST /api/v1/urls hits 429 after 30 req/min
#   2. Global tier — GET /{code} hits 429 after 100 req/min
#   3. Recovery  — requests succeed again after waiting
#
# Usage:
#   ./test_rate_limit.sh [--base-url http://127.0.0.1:8000]

# --- Configuration & Colors ---
BASE_URL="http://127.0.0.1:8000"
SKIP_RECOVERY=0

GREEN='\033[92m'
RED='\033[91m'
YELLOW='\033[93m'
CYAN='\033[96m'
RESET='\033[0m'

# Temporary file to store the HTTP body for parsing
TEMP_BODY=$(mktemp)
trap 'rm -f "$TEMP_BODY"' EXIT

# --- Parse Arguments ---
while [[ "$#" -gt 0 ]]; do
    case $1 in
        --base-url) BASE_URL="$2"; shift ;;
        --skip-recovery) SKIP_RECOVERY=1 ;;
        *) echo "Unknown parameter: $1"; exit 1 ;;
    esac
    shift
done

# Strip trailing slash if present
BASE_URL="${BASE_URL%/}"

# --- Helper Functions ---

do_request() {
    local method="$1"
    local path="$2"
    local body="$3"
    local url="${BASE_URL}${path}"

    # We do not follow redirects (curl does not by default, matching Python's _NoRedirect)
    if [[ -n "$body" ]]; then
        curl -s -w "%{http_code}" -o "$TEMP_BODY" -m 5 -X "$method" \
             -H "Content-Type: application/json" -d "$body" "$url"
    else
        curl -s -w "%{http_code}" -o "$TEMP_BODY" -m 5 -X "$method" \
             -H "Content-Type: application/json" "$url"
    fi
}

check_server() {
    if ! curl -s -m 3 -o /dev/null "$BASE_URL/"; then
        echo -e "${RED}Server not reachable at ${BASE_URL}. Start it with: cargo run${RESET}"
        exit 1
    fi
}

# Global variables to pass data back from `burst`
# Using standard indexed array since status codes are integers (Bash 3 compatible)
TALLY=()
FIRST_429_AT=""
RETRY_AFTER=""

burst() {
    local method="$1"
    local path="$2"
    local count="$3"
    local body="$4"
    local delay="$5"

    # Reset globals
    TALLY=()
    FIRST_429_AT=""
    RETRY_AFTER=""

    for (( i=1; i<=count; i++ )); do
        local status
        status=$(do_request "$method" "$path" "$body")

        # Increment the tally for this status code
        TALLY[$status]=$(( ${TALLY[$status]:-0} + 1 ))

        if [[ "$status" == "429" && -z "$FIRST_429_AT" ]]; then
            FIRST_429_AT=$i
            # Extract retry_after_seconds robustly without jq
            RETRY_AFTER=$(grep -Eo '"retry_after_seconds"\s*:\s*[0-9]+' "$TEMP_BODY" | grep -Eo '[0-9]+' || echo "?")
        fi

        if [[ -n "$delay" ]]; then
            sleep "$delay"
        fi
    done
}

format_tally() {
    local res="{"
    local first=1
    for code in "${!TALLY[@]}"; do
        if [[ $first -eq 1 ]]; then
            first=0
        else
            res="${res}, "
        fi
        res="${res}${code}: ${TALLY[$code]}"
    done
    res="${res}}"
    echo "$res"
}

print_result() {
    local label="$1"
    local expect_429="$2"

    local err_count=${TALLY[429]:-0}
    local passed=0

    if [[ "$expect_429" == "true" ]]; then
        [[ $err_count -gt 0 ]] && passed=1
    else
        [[ $err_count -eq 0 ]] && passed=1
    fi

    local icon
    if [[ $passed -eq 1 ]]; then
        icon="${GREEN}PASS${RESET}"
        return 0
    else
        icon="${RED}FAIL${RESET}"
        return 1
    fi
}

render_result_output() {
    local label="$1"
    local icon_str="$2"
    echo -e "  [${icon_str}] ${label}"
    echo -e "         responses : $(format_tally)"
    if [[ -n "$FIRST_429_AT" ]]; then
        echo -e "         first 429  : request #${FIRST_429_AT}  (retry_after=${RETRY_AFTER}s)"
    fi
}

# --- Test Cases ---

test_api_rate_limit() {
    echo -e "\n${CYAN}Test 1 — API tier (POST /api/v1/urls, limit=30/min)${RESET}"
    burst "POST" "/api/v1/urls" 40 '{"long_url": "https://example.com"}' ""

    if print_result "40 rapid POSTs should trigger 429 after 30" "true"; then
        render_result_output "40 rapid POSTs should trigger 429 after 30" "${GREEN}PASS${RESET}"
        return 0
    else
        render_result_output "40 rapid POSTs should trigger 429 after 30" "${RED}FAIL${RESET}"
        return 1
    fi
}

test_redirect_rate_limit() {
    local code="$1"
    echo -e "\n${CYAN}Test 2 — Global tier (GET /${code}, limit=100/min)${RESET}"
    burst "GET" "/${code}" 110 "" ""

    if print_result "110 rapid GETs should trigger 429 after 100" "true"; then
        render_result_output "110 rapid GETs should trigger 429 after 100" "${GREEN}PASS${RESET}"
        return 0
    else
        render_result_output "110 rapid GETs should trigger 429 after 100" "${RED}FAIL${RESET}"
        return 1
    fi
}

test_docs_not_rate_limited_immediately() {
    echo -e "\n${CYAN}Test 3 — Docs endpoint not immediately rate-limited (GET /docs)${RESET}"
    burst "GET" "/docs" 10 "" ""

    if print_result "10 GETs to /docs should all succeed (well under limits)" "false"; then
        render_result_output "10 GETs to /docs should all succeed (well under limits)" "${GREEN}PASS${RESET}"
        return 0
    else
        render_result_output "10 GETs to /docs should all succeed (well under limits)" "${RED}FAIL${RESET}"
        return 1
    fi
}

test_recovery() {
    echo -e "\n${CYAN}Test 4 — Recovery after rate limit${RESET}"

    # Exhaust the limit
    burst "POST" "/api/v1/urls" 35 '{"long_url": "https://example.com"}' ""

    local wait=62
    echo -ne "  Waiting ${wait}s for the 1-minute window to reset …"
    for (( i=0; i<wait; i++ )); do
        sleep 1
        echo -ne "."
    done
    echo ""

    local status
    status=$(do_request "POST" "/api/v1/urls" '{"long_url": "https://example.com"}')

    if [[ "$status" == "200" || "$status" == "201" ]]; then
        echo -e "  [${GREEN}PASS${RESET}] First request after reset returned HTTP ${status} (expected 200/201)"
        return 0
    else
        echo -e "  [${RED}FAIL${RESET}] First request after reset returned HTTP ${status} (expected 200/201)"
        return 1
    fi
}

# --- Main Execution ---

echo "Target: $BASE_URL"
check_server

# Create a short URL before any test exhausts the rate limit.
init_status=$(do_request "POST" "/api/v1/urls" '{"long_url": "https://example.com"}')
REDIRECT_CODE=""
if [[ "$init_status" == "201" ]]; then
    REDIRECT_CODE=$(grep -Eo '"short_code"\s*:\s*"[^"]+"' "$TEMP_BODY" | awk -F '"' '{print $4}')
fi

if [[ -z "$REDIRECT_CODE" ]]; then
    echo -e "${RED}Setup failed — could not create a short URL (HTTP ${init_status})${RESET}"
    exit 1
fi

PASSED_COUNT=0
TOTAL_COUNT=0

run_test() {
    ((TOTAL_COUNT++))
    if "$@"; then
        ((PASSED_COUNT++))
    fi
}

run_test test_docs_not_rate_limited_immediately
run_test test_api_rate_limit
run_test test_redirect_rate_limit "$REDIRECT_CODE"

if [[ $SKIP_RECOVERY -eq 0 ]]; then
    run_test test_recovery
else
    echo -e "\n${YELLOW}Recovery test skipped (--skip-recovery)${RESET}"
fi

# Summary
if [[ $PASSED_COUNT -eq $TOTAL_COUNT ]]; then
    COLOR=$GREEN
else
    COLOR=$RED
fi

echo -e "\n${COLOR}────────────────────────────────────────${RESET}"
echo -e "${COLOR}${PASSED_COUNT}/${TOTAL_COUNT} tests passed${RESET}"

if [[ $PASSED_COUNT -eq $TOTAL_COUNT ]]; then
    exit 0
else
    exit 1
fi