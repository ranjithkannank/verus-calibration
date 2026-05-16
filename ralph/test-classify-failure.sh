#!/usr/bin/env bash
# Unit tests for classify_failure() in run-exercise.sh. Builds fixture log
# files containing each known signature and confirms the classifier picks
# the right category.

set -u

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

# Inlined copy of classify_failure — kept in sync with run-exercise.sh by
# convention. If the script's classifier changes shape, update this one too.
classify_failure() {
  local log="$1"
  [ -f "$log" ] || { echo unknown; return; }
  if grep -qiE "hit your limit|resets at [0-9]|five_hour|rate.limit|rate_limit_event" "$log"; then
    echo rate_limit
  elif grep -qiE "Exceeded USD budget|max-budget-usd" "$log"; then
    echo budget
  elif grep -qiE "ECONNREFUSED|getaddrinfo|connect ETIMEDOUT|network unreachable|EHOSTUNREACH" "$log"; then
    echo network
  elif grep -qiE "Input must be provided|unrecognized option|unknown flag" "$log"; then
    echo invocation
  else
    echo unknown
  fi
}

pass=0
fail=0

assert_kind() {
  local label="$1" expected="$2" content="$3"
  local tmp
  tmp=$(mktemp)
  printf '%s\n' "$content" > "$tmp"
  local actual
  actual=$(classify_failure "$tmp")
  rm -f "$tmp"
  if [ "$actual" = "$expected" ]; then
    printf '  [%-44s] PASS  (kind=%s)\n' "$label" "$actual"
    pass=$((pass + 1))
  else
    printf '  [%-44s] FAIL  expected=%s got=%s\n' "$label" "$expected" "$actual"
    fail=$((fail + 1))
  fi
}

echo
echo "classify_failure tests"
echo "======================"

assert_kind "rate-limit / 'hit your limit'"        rate_limit  "Error: You've hit your limit · resets 10:30pm"
assert_kind "rate-limit / five_hour"                rate_limit  '{"rate_limit_event":{"rateLimitType":"five_hour"}}'
assert_kind "rate-limit / rate_limit_event"         rate_limit  '{"type":"rate_limit_event"}'
assert_kind "budget / Exceeded USD"                 budget      "Error: Exceeded USD budget (2)"
assert_kind "budget / max-budget-usd"               budget      "max-budget-usd was set to 2.00"
assert_kind "network / ECONNREFUSED"                network     "fetch error: ECONNREFUSED 127.0.0.1:443"
assert_kind "network / getaddrinfo"                 network     "Error: getaddrinfo ENOTFOUND api.anthropic.com"
assert_kind "invocation / Input must be provided"   invocation  "Error: Input must be provided either through stdin or as a prompt argument"
assert_kind "invocation / unrecognized option"      invocation  "claude: unrecognized option: --bogus"
assert_kind "unknown / verus error"                 unknown     "error: invariant not satisfied before loop"
assert_kind "unknown / empty"                       unknown     ""
assert_kind "unknown / missing log"                 unknown     "@@MISSING@@"  # placeholder unused; missing-file path tested below

# Missing-file path: classify_failure on a path that does not exist should
# return unknown.
actual=$(classify_failure /tmp/nonexistent-classify-failure-fixture)
if [ "$actual" = "unknown" ]; then
  printf '  [%-44s] PASS  (kind=%s)\n' "missing file path → unknown" "$actual"
  pass=$((pass + 1))
else
  printf '  [%-44s] FAIL  expected=unknown got=%s\n' "missing file path → unknown" "$actual"
  fail=$((fail + 1))
fi

echo
echo "Summary: passed=$pass failed=$fail"
[ "$fail" -eq 0 ] && exit 0 || exit 1
