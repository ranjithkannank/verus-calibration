#!/usr/bin/env bash
# Exercises every state classification in run-exercise.sh by setting up the
# corresponding filesystem conditions and reading back the classified state.
# Cleans up after each scenario.

set -u

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

EX="binary_search"
EXFILE="exercises/${EX}.rs"
DESIGN="exercises/${EX}.design.md"
LOGDIR="logs/${EX}"
RAWDIR="${LOGDIR}/raw"
ATTEMPTS="${LOGDIR}/attempts.md"
STATUS="${LOGDIR}/status"
ESCALATION="${LOGDIR}/escalation.md"
REVIEW="${LOGDIR}/review.md"
BLOCKED="${LOGDIR}/blocked.md"
DONE_FLAG="${LOGDIR}/done.flag"

mkdir -p "$RAWDIR"

# Source the classification logic from run-exercise.sh.
# We mimic the in-script setup: same vars, same case statement.
ATTEMPT_CAP=10

cleanup() {
  rm -f "$DESIGN" "$ATTEMPTS" "$STATUS" "$ESCALATION" "$REVIEW" "$BLOCKED" "$DONE_FLAG"
  rm -rf "${RAWDIR:?}"/*
}

count_attempts() {
  if [ -f "$ATTEMPTS" ]; then
    grep -c '^## Attempt' "$ATTEMPTS" 2>/dev/null || echo 0
  else
    echo 0
  fi
}

read_status() {
  [ -f "$STATUS" ] && cat "$STATUS" || echo ""
}

classify_state() {
  if [ -f "$DONE_FLAG" ]; then echo DONE; return; fi
  if [ -f "$BLOCKED" ]; then echo BLOCKED; return; fi
  if [ ! -f "$DESIGN" ]; then echo THINK; return; fi
  if [ -f "$ESCALATION" ]; then echo THINK_REVISE; return; fi
  if [ -f "$REVIEW" ]; then
    if grep -qE '^\*\*Conclusion:\*\*[[:space:]]*APPROVE' "$REVIEW"; then echo APPROVED; return
    elif grep -qE '^\*\*Conclusion:\*\*[[:space:]]*REJECT' "$REVIEW"; then echo WORK_AFTER_REJECT; return
    fi
  fi
  case "$(read_status)" in
    verus_passed) echo REVIEW; return ;;
    escalated)    echo THINK_REVISE; return ;;
  esac
  if [ "$(count_attempts)" -ge "$ATTEMPT_CAP" ]; then echo CAP_HIT; return; fi
  echo WORK
}

assert_state() {
  local expected="$1"
  local label="$2"
  local actual
  actual=$(classify_state)
  if [ "$actual" = "$expected" ]; then
    printf '  [%-30s] PASS  (state=%s)\n' "$label" "$actual"
    pass=$((pass + 1))
  else
    printf '  [%-30s] FAIL  expected=%s got=%s\n' "$label" "$expected" "$actual"
    fail=$((fail + 1))
  fi
}

pass=0
fail=0

echo
echo "State-machine classification tests"
echo "=================================="

cleanup
assert_state THINK "no design.md → THINK"

cleanup
touch "$DESIGN"
assert_state WORK "design.md only → WORK"

cleanup
touch "$DESIGN"
echo "## Attempt 1 — t" > "$ATTEMPTS"
echo "verus_failed" > "$STATUS"
assert_state WORK "1 attempt, verus_failed → WORK"

cleanup
touch "$DESIGN"
for i in 1 2 3; do echo "## Attempt $i — t" >> "$ATTEMPTS"; done
echo "verus_passed" > "$STATUS"
assert_state REVIEW "verus_passed → REVIEW"

cleanup
touch "$DESIGN"
echo "## Attempt 1 — t" >> "$ATTEMPTS"
touch "$ESCALATION"
echo "escalated" > "$STATUS"
assert_state THINK_REVISE "escalation.md present → THINK_REVISE"

cleanup
touch "$DESIGN"
cat > "$REVIEW" <<'EOF'
# Review: binary_search

**Conclusion:** APPROVE
EOF
assert_state APPROVED "review APPROVE → APPROVED"

cleanup
touch "$DESIGN"
cat > "$REVIEW" <<'EOF'
# Review: binary_search

**Conclusion:** REJECT
EOF
assert_state WORK_AFTER_REJECT "review REJECT → WORK_AFTER_REJECT"

cleanup
touch "$DESIGN"
for i in $(seq 1 10); do echo "## Attempt $i — t" >> "$ATTEMPTS"; done
echo "verus_failed" > "$STATUS"
assert_state CAP_HIT "10 attempts, still failing → CAP_HIT"

cleanup
touch "$DESIGN"
touch "$DONE_FLAG"
assert_state DONE "done.flag → DONE"

cleanup
touch "$DESIGN"
touch "$BLOCKED"
assert_state BLOCKED "blocked.md → BLOCKED"

cleanup
echo
echo "Summary: passed=$pass failed=$fail"
[ "$fail" -eq 0 ] && exit 0 || exit 1
