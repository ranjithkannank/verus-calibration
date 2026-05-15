#!/usr/bin/env bash
# Ralph-style outer loop for a single verus-calibration exercise.
#
# Reads state from filesystem artifacts (design.md, attempts.md, status,
# review.md). Fires one `claude -p` call per iteration with fresh context,
# selecting the role (architect / implementer / reviewer) and model based
# on the current state.
#
# Usage:
#   ./ralph/run-exercise.sh <exercise>           # run the full state machine
#   ./ralph/run-exercise.sh <exercise> --dry-run # show state transitions only,
#                                                # no claude calls
#
# Exit codes:
#   0   exercise verified AND reviewed APPROVE
#   1   exercise blocked (cap hit or unrecoverable)
#   2   usage / environment error

set -u

DRY_RUN=0
if [ "${2:-}" = "--dry-run" ]; then
  DRY_RUN=1
fi

if [ $# -lt 1 ]; then
  echo "usage: $0 <exercise> [--dry-run]" >&2
  echo "  e.g. $0 binary_search" >&2
  exit 2
fi

EXERCISE="$1"
REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

EXFILE="exercises/${EXERCISE}.rs"
DESIGN="exercises/${EXERCISE}.design.md"
LOGDIR="logs/${EXERCISE}"
RAWDIR="${LOGDIR}/raw"
ATTEMPTS="${LOGDIR}/attempts.md"
STATUS="${LOGDIR}/status"
ESCALATION="${LOGDIR}/escalation.md"
REVIEW="${LOGDIR}/review.md"
BLOCKED="${LOGDIR}/blocked.md"
DONE_FLAG="${LOGDIR}/done.flag"
SPEC_TAG="spec-frozen-${EXERCISE}"

# Per-exercise iteration caps (matches AGENTS.md).
case "$EXERCISE" in
  binary_search) ATTEMPT_CAP=10 ;;
  bounded_log)   ATTEMPT_CAP=20 ;;
  quorum_count)  ATTEMPT_CAP=20 ;;
  *) echo "unknown exercise: $EXERCISE" >&2; exit 2 ;;
esac

# Hard outer-iteration ceiling (covers THINK/REVIEW/ESCALATE overhead).
MAX_OUTER_ITERATIONS=$((ATTEMPT_CAP + 8))

# Models — keep in sync with .claude/agents/*.md frontmatter.
MODEL_ARCHITECT="claude-opus-4-7"
MODEL_IMPLEMENTER="claude-sonnet-4-6"
MODEL_REVIEWER="claude-opus-4-7"

# Per-call dollar safety net. No-op for subscription users; takes effect if
# the install ever falls back to API metering.
PER_CALL_BUDGET=2.00

# --- preflight ----------------------------------------------------------------

[ -f "$EXFILE" ] || { echo "missing: $EXFILE" >&2; exit 2; }
[ -d ".claude/agents" ] || { echo "missing: .claude/agents (subagent defs)" >&2; exit 2; }
git rev-parse --verify --quiet "$SPEC_TAG" >/dev/null \
  || { echo "missing git tag: $SPEC_TAG (run setup-check.sh)" >&2; exit 2; }

if [ "$DRY_RUN" -eq 0 ]; then
  command -v claude >/dev/null || { echo "claude not on PATH" >&2; exit 2; }
  command -v verus  >/dev/null || { echo "verus not on PATH"  >&2; exit 2; }
fi

mkdir -p "$RAWDIR"

# --- helpers ------------------------------------------------------------------

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

log_blocked() {
  local reason="$1"
  cat > "$BLOCKED" <<EOF
# Blocked: $EXERCISE

**Reason:** $reason
**Attempts completed:** $(count_attempts) / $ATTEMPT_CAP
**Outer iterations consumed:** $iteration / $MAX_OUTER_ITERATIONS

See \`$ATTEMPTS\` for the per-attempt history and \`$RAWDIR/\` for raw verifier output.
EOF
  git add -A && git commit -m "${EXERCISE}: blocked — ${reason}" >/dev/null 2>&1 || true
}

# Determine state from filesystem artifacts. Echoes the state name.
classify_state() {
  if [ -f "$DONE_FLAG" ]; then echo DONE; return; fi
  if [ -f "$BLOCKED" ]; then echo BLOCKED; return; fi

  if [ ! -f "$DESIGN" ]; then echo THINK; return; fi
  if [ -f "$ESCALATION" ]; then echo THINK_REVISE; return; fi

  if [ -f "$REVIEW" ]; then
    if grep -qE '^\*\*Conclusion:\*\*[[:space:]]*APPROVE' "$REVIEW"; then
      echo APPROVED; return
    elif grep -qE '^\*\*Conclusion:\*\*[[:space:]]*REJECT' "$REVIEW"; then
      echo WORK_AFTER_REJECT; return
    fi
  fi

  case "$(read_status)" in
    verus_passed) echo REVIEW; return ;;
    escalated)    echo THINK_REVISE; return ;;
  esac

  if [ "$(count_attempts)" -ge "$ATTEMPT_CAP" ]; then
    echo CAP_HIT; return
  fi

  echo WORK
}

# Per-role allowed tool lists. Strict whitelist — anything off these lists is
# denied. Bash patterns use Claude Code's glob syntax: `Bash(verus *)` matches
# any command that starts with `verus`.
ALLOWED_ARCHITECT=(
  Read Write Glob Grep
  "Bash(git add *)" "Bash(git commit *)"
  "Bash(git diff *)" "Bash(git log *)" "Bash(git status *)"
  "Bash(git show *)" "Bash(git rev-parse *)"
  "Bash(ls *)" "Bash(cat *)" "Bash(wc *)"
)

ALLOWED_IMPLEMENTER=(
  Read Edit Write Glob Grep
  "Bash(verus *)"
  "Bash(git add *)" "Bash(git commit *)"
  "Bash(git diff *)" "Bash(git log *)" "Bash(git status *)"
  "Bash(git show *)" "Bash(git rev-parse *)"
  "Bash(ls *)" "Bash(cat *)" "Bash(mkdir -p logs/*)" "Bash(mkdir -p logs/*/*)"
  "Bash(echo *)" "Bash(head *)" "Bash(tail *)" "Bash(wc *)"
  "Bash(grep *)" "Bash(rg *)"
)

ALLOWED_REVIEWER=(
  Read Write Glob Grep
  "Bash(git diff *)" "Bash(git log *)" "Bash(git status *)"
  "Bash(git show *)" "Bash(git rev-parse *)"
  "Bash(git add *)" "Bash(git commit *)"
  "Bash(ls *)" "Bash(cat *)" "Bash(grep *)" "Bash(wc *)"
)

# Always-denied. Includes hook-bypass and any-network/install patterns.
# Explicit denies even though the allowlist already rejects them — defense
# in depth and clearer error messages.
DISALLOWED_TOOLS=(
  WebFetch WebSearch Task NotebookEdit
  "Bash(rm *)" "Bash(rmdir *)" "Bash(mv *)" "Bash(chmod *)" "Bash(chown *)"
  "Bash(git push*)" "Bash(git reset*)" "Bash(git rebase*)"
  "Bash(git checkout*)" "Bash(git restore*)" "Bash(git revert*)"
  "Bash(git config*)" "Bash(git -c*)"
  "Bash(*--no-verify*)" "Bash(*-n -m*)"
  "Bash(curl*)" "Bash(wget*)" "Bash(nc *)" "Bash(ssh*)" "Bash(scp*)"
  "Bash(brew*)" "Bash(npm*)" "Bash(pip*)" "Bash(cargo install*)"
  "Bash(sudo*)" "Bash(su *)"
)

# Fire a single claude -p call with role-scoped permissions.
# Args:
#   $1 = role label (for logging)
#   $2 = model
#   $3 = agent name (must match a file in .claude/agents/)
#   $4 = task prompt (string)
#   $5 = allowed-tools array name (ALLOWED_ARCHITECT | ALLOWED_IMPLEMENTER | ALLOWED_REVIEWER)
fire_claude() {
  local role="$1" model="$2" agent="$3" prompt="$4" allowed_var="$5"
  echo
  echo "  > role=$role  model=$model  agent=$agent  allowed=$allowed_var"
  if [ "$DRY_RUN" -eq 1 ]; then
    echo "  > (dry-run; not invoking claude)"
    return 0
  fi
  # Dereference the array name into a local array (bash 3.2 compatible).
  eval "local allowed=(\"\${${allowed_var}[@]}\")"
  claude -p \
    --agent "$agent" \
    --model "$model" \
    --no-session-persistence \
    --permission-mode acceptEdits \
    --max-budget-usd "$PER_CALL_BUDGET" \
    --allowedTools "${allowed[@]}" \
    --disallowedTools "${DISALLOWED_TOOLS[@]}" \
    "$prompt"
}

# --- prompts ------------------------------------------------------------------

prompt_think() {
  cat <<EOF
You have fresh context. Read in this order:

  1. AGENTS.md
  2. .claude/agents/architect.md (your role definition)
  3. $EXFILE (the frozen spec)

Then write $DESIGN per the architect role spec — representation choice,
key invariants, loop-invariant sketches, predicted helper lemmas, SMT
trouble spots, and a suggested order of operations.

Do NOT edit $EXFILE. Do NOT run verus. Commit the design note with
message "architect: design for ${EXERCISE}" and stop.
EOF
}

prompt_think_revise() {
  cat <<EOF
You have fresh context. The implementer escalated. Read:

  1. AGENTS.md
  2. .claude/agents/architect.md
  3. $EXFILE (current state)
  4. $DESIGN (existing design note)
  5. $ESCALATION (the implementer's blocker description)

Update $DESIGN in place — append a "## Revision (escalation $(date -u +%Y%m%dT%H%M%SZ))"
section addressing what the implementer got stuck on and what should change
about the strategy.

Then delete $ESCALATION so it does not retrigger. Commit with message
"architect: revision for ${EXERCISE}" and stop.
EOF
}

prompt_work() {
  local attempt_num=$(( $(count_attempts) + 1 ))
  cat <<EOF
You have fresh context. This is attempt $attempt_num of $ATTEMPT_CAP for ${EXERCISE}.

Read in this order:

  1. AGENTS.md
  2. .claude/agents/implementer.md (your role definition)
  3. $EXFILE (current state of the exercise)
  4. $DESIGN (the architect's strategy)
  5. $ATTEMPTS if it exists (your prior attempts — do not repeat them)
  6. The most recent file in $RAWDIR/ if any (last verifier output)

Make EXACTLY ONE new attempt:

  a. Edit $EXFILE — implement or refine.
  b. Run: verus $EXFILE --crate-type=lib > $RAWDIR/attempt-${attempt_num}.txt 2>&1
     (You can `cat` the file afterwards to inspect the output.)
  c. Append an entry to $ATTEMPTS using the format in AGENTS.md.
  d. Write $STATUS with exactly one of these tokens:
       - verus_passed     (verus exited 0)
       - verus_failed     (verus exited non-zero; you will iterate)
       - escalated        (you wrote $ESCALATION; you are stuck)
  e. Commit with message "${EXERCISE} attempt-${attempt_num}: <one line>".

Then stop. Do not start another attempt — the outer loop drives iteration.

Escalation rule: if you have 3 prior attempts failing on the same proof
obligation, do NOT try a fourth variant — write $ESCALATION instead,
explaining the obligation, the three approaches tried, and your best guess
at what is missing. Set status to "escalated" and commit.

Forbidden under all circumstances (these are audited by the reviewer):
  - Modifying any \`requires\`, \`ensures\`, or pre-existing \`spec fn\` body
  - Adding \`assume(\`, \`#[verifier::external_body]\`, \`unreachable!()\` in
    reachable positions, or panicking stubs to dodge cases
EOF
}

prompt_work_after_reject() {
  cat <<EOF
You have fresh context. The previous successful verification was REJECTED
by the reviewer.

Read in this order:

  1. AGENTS.md
  2. .claude/agents/implementer.md
  3. $EXFILE
  4. $DESIGN
  5. $ATTEMPTS
  6. $REVIEW (the rejection — pay attention to which rule was violated)

Make ONE new attempt that fixes the reviewer's finding without re-introducing
the original verification failure. Follow the same per-attempt protocol as a
normal WORK iteration (edit, run verus, log, write status, commit).

After your commit, delete $REVIEW (it is consumed). Then stop.
EOF
}

prompt_review() {
  cat <<EOF
You have fresh context. The implementer reports verus exited 0. Your job is
the no-cheating audit — NOT correctness.

Read in this order:

  1. AGENTS.md
  2. .claude/agents/reviewer.md (your role definition, including the
     five-point checklist)
  3. The diff: \`git diff ${SPEC_TAG}..HEAD -- $EXFILE\`
  4. $EXFILE (current state, for line references)

Run the five-point checklist verbatim. Write $REVIEW in the format specified
in your role definition (Conclusion: APPROVE | REJECT, plus checklist
answers and a justification).

Commit with message "${EXERCISE} review: APPROVE" or "${EXERCISE} review: REJECT"
and stop. Do not edit any file under exercises/. Do not run verus.
EOF
}

# --- main loop ----------------------------------------------------------------

echo "=================================="
echo " Ralph loop: ${EXERCISE}"
echo " Attempt cap: ${ATTEMPT_CAP}"
echo " Outer ceiling: ${MAX_OUTER_ITERATIONS}"
echo " Dry-run: ${DRY_RUN}"
echo "=================================="

iteration=0
while [ $iteration -lt $MAX_OUTER_ITERATIONS ]; do
  iteration=$((iteration + 1))
  state=$(classify_state)
  attempts=$(count_attempts)

  echo
  echo "--- iteration $iteration  |  state=$state  |  attempts=$attempts/$ATTEMPT_CAP"

  case "$state" in
    DONE)
      echo "DONE — ${EXERCISE} verified and approved."
      exit 0
      ;;

    APPROVED)
      touch "$DONE_FLAG"
      git add -A && git commit -m "${EXERCISE}: DONE" >/dev/null 2>&1 || true
      echo "DONE — ${EXERCISE} verified and approved."
      exit 0
      ;;

    BLOCKED)
      echo "BLOCKED — see $BLOCKED."
      exit 1
      ;;

    CAP_HIT)
      log_blocked "iteration cap of $ATTEMPT_CAP attempts hit without verification"
      echo "BLOCKED — cap hit."
      exit 1
      ;;

    THINK)
      fire_claude THINK "$MODEL_ARCHITECT" architect "$(prompt_think)" ALLOWED_ARCHITECT
      ;;

    THINK_REVISE)
      fire_claude THINK_REVISE "$MODEL_ARCHITECT" architect "$(prompt_think_revise)" ALLOWED_ARCHITECT
      ;;

    WORK)
      fire_claude WORK "$MODEL_IMPLEMENTER" implementer "$(prompt_work)" ALLOWED_IMPLEMENTER
      ;;

    WORK_AFTER_REJECT)
      fire_claude WORK_AFTER_REJECT "$MODEL_IMPLEMENTER" implementer "$(prompt_work_after_reject)" ALLOWED_IMPLEMENTER
      ;;

    REVIEW)
      fire_claude REVIEW "$MODEL_REVIEWER" reviewer "$(prompt_review)" ALLOWED_REVIEWER
      # Status is consumed once the reviewer has run.
      rm -f "$STATUS"
      ;;

    *)
      echo "unknown state: $state" >&2
      exit 1
      ;;
  esac

  # Tiny pause so filesystem writes settle before the next classification.
  sleep 1
done

log_blocked "hit hard outer ceiling of $MAX_OUTER_ITERATIONS iterations"
echo "BLOCKED — outer ceiling hit."
exit 1
