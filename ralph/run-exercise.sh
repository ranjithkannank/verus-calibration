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
ONCE=0
for arg in "${@:2}"; do
  case "$arg" in
    --dry-run) DRY_RUN=1 ;;
    --once)    ONCE=1 ;;
    *) echo "unknown flag: $arg" >&2; exit 2 ;;
  esac
done

if [ $# -lt 1 ]; then
  echo "usage: $0 <exercise> [--dry-run] [--once]" >&2
  echo "  --once: exit cleanly after one iteration (probe mode)" >&2
  exit 2
fi

EXERCISE="$1"
REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

# Layout detection. Two shapes are supported:
#   single-file:  exercises/<name>.rs + exercises/<name>.design.md
#   multi-file:   exercises/<name>/main.rs + exercises/<name>/<module>.rs + design.md
#
# EXFILE is always the verus entry point. EXSCOPE describes the
# editable scope (a single file or a directory). EX_DIFF_PATH is what
# `git diff` operates on for reviewer audits.
if [ -d "exercises/${EXERCISE}" ]; then
  LAYOUT="multi"
  EXDIR="exercises/${EXERCISE}"
  EXFILE="${EXDIR}/main.rs"
  DESIGN="${EXDIR}/design.md"
  EX_DIFF_PATH="$EXDIR"
  EXSCOPE="the files under ${EXDIR}/ (verus entry: ${EXFILE})"
elif [ -f "exercises/${EXERCISE}.rs" ]; then
  LAYOUT="single"
  EXDIR="exercises"
  EXFILE="exercises/${EXERCISE}.rs"
  DESIGN="exercises/${EXERCISE}.design.md"
  EX_DIFF_PATH="$EXFILE"
  EXSCOPE="$EXFILE"
else
  echo "no such exercise: ${EXERCISE} (neither exercises/${EXERCISE}.rs nor exercises/${EXERCISE}/ exists)" >&2
  exit 2
fi
LOGDIR="logs/${EXERCISE}"
RAWDIR="${LOGDIR}/raw"
ATTEMPTS="${LOGDIR}/attempts.md"
STATUS="${LOGDIR}/status"
ESCALATION="${LOGDIR}/escalation.md"
REVIEW="${LOGDIR}/review.md"
BLOCKED="${LOGDIR}/blocked.md"
DONE_FLAG="${LOGDIR}/done.flag"
INFRA_FAILURE="${LOGDIR}/infra_failure.md"
SPEC_TAG="spec-frozen-${EXERCISE}"

# Per-exercise iteration caps (matches AGENTS.md).
case "$EXERCISE" in
  binary_search) ATTEMPT_CAP=10 ;;
  bounded_log)   ATTEMPT_CAP=20 ;;
  quorum_count)  ATTEMPT_CAP=20 ;;
  quorum_cert)   ATTEMPT_CAP=20 ;;
  ft_midpoint)   ATTEMPT_CAP=20 ;;
  marzullo)      ATTEMPT_CAP=20 ;;
  cross_module_counter) ATTEMPT_CAP=15 ;;
  counter_multifile) ATTEMPT_CAP=10 ;;
  counter_producer) ATTEMPT_CAP=12 ;;
  sensor_poll) ATTEMPT_CAP=18 ;;
  sensor_poll_signed) ATTEMPT_CAP=18 ;;
  sensor_poll_honest) ATTEMPT_CAP=20 ;;
  counter_filler) ATTEMPT_CAP=15 ;;
  vec_swap) ATTEMPT_CAP=25 ;;
  vec_swap_v2) ATTEMPT_CAP=25 ;;
  *) echo "unknown exercise: $EXERCISE" >&2; exit 2 ;;
esac

# Hard outer-iteration ceiling (covers THINK/REVIEW/ESCALATE overhead).
MAX_OUTER_ITERATIONS=$((ATTEMPT_CAP + 8))
if [ $ONCE -eq 1 ]; then
  MAX_OUTER_ITERATIONS=1
fi

# Per-iteration agent output goes here so the operator's terminal stays
# readable and (when invoked from another claude session) doesn't blow up
# the parent's context.
mkdir -p "logs/${EXERCISE}/ralph"

# Models — keep in sync with .claude/agents/*.md frontmatter.
#
# The implementer was originally Sonnet 4.6 — it handled the three
# calibration exercises (binary_search, bounded_log, quorum_count)
# competently. Quorum_cert and subsequent BFT-path exercises require
# genuine proof reasoning (cardinality bounds, pigeonhole arguments,
# composing helper lemmas) where the model needs to plan over many
# tokens of internal thinking. Switching to Opus 4.7 for the
# implementer role aligns with the principle that the calls doing the
# hardest reasoning should run on the strongest model — even if it
# makes the implementer the most expensive role.
MODEL_ARCHITECT="claude-opus-4-7"
MODEL_IMPLEMENTER="claude-opus-4-7"
MODEL_REVIEWER="claude-opus-4-7"

# No per-call USD budget on the claude invocations: this loop runs against
# a Claude Code subscription, which halts on its own when quota is hit.
# An explicit --max-budget-usd is a no-op at best for subscription users
# and (we saw on quorum_count) can produce confusing partial-output
# behavior on long calls. Removing it.

# --- preflight ----------------------------------------------------------------

[ -f "$EXFILE" ] || { echo "missing: $EXFILE" >&2; exit 2; }
[ -f "$DESIGN" ] || { echo "missing: $DESIGN" >&2; exit 2; }
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
#
# Note: escalation.md and status are checked with `-s` (non-empty) not
# `-f` (exists). The architect cannot `rm` the escalation marker (the
# tool whitelist denies it), so the agent has historically truncated
# the file instead of deleting. The non-empty test makes the state
# classifier agnostic to which the agent does. The orchestrator also
# explicitly cleans these files in the post-THINK_REVISE block —
# defense in depth.
classify_state() {
  if [ -f "$DONE_FLAG" ]; then echo DONE; return; fi
  if [ -f "$BLOCKED" ]; then echo BLOCKED; return; fi

  if [ ! -f "$DESIGN" ]; then echo THINK; return; fi
  if [ -s "$ESCALATION" ]; then echo THINK_REVISE; return; fi

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
  # Witness files are operator territory. Block every tool path the agent
  # could use to peek at the reference implementation. Multiple shapes to
  # cover both single-file (`exercises/<name>_witness.rs`) and multi-file
  # (`exercises/<name>_witness/...`) layouts.
  "Read(**/exercises/*_witness*)" "Read(**/exercises/*_witness/**)"
  "Read(**/*_witness.rs)" "Read(**/*_witness/**)"
  "Glob(**/exercises/*_witness*)" "Glob(**/*_witness*)"
  "Grep(**/exercises/*_witness*)" "Grep(**/*_witness.rs)"
  "Bash(cat *_witness*)" "Bash(cat *_witness/*)"
  "Bash(grep *_witness*)" "Bash(rg *_witness*)"
  "Bash(head *_witness*)" "Bash(tail *_witness*)"
  "Bash(ls *_witness*)"
)

# Classify a failed claude call by inspecting its iteration log. Echoes one
# of: rate_limit | budget | network | invocation | unknown.
#
# This is what makes the orchestrator signal-aware: instead of treating every
# non-zero exit code as "verus failed, iterate," we recognise infrastructure
# signatures and surface them as a distinct failure mode so the loop can stop
# rather than burn iterations against a transient problem.
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

# Write the infrastructure-failure marker. Records what happened, names the
# iteration log so the operator can read the raw evidence, and exits the loop
# cleanly. The marker is informational; classify_state does not treat it as a
# stop condition, so re-running the script (after the underlying issue is
# resolved) resumes naturally.
write_infra_failure() {
  local kind="$1" iter_log="$2" rc="$3"
  cat > "$INFRA_FAILURE" <<EOF
# Infrastructure failure: $kind

Iteration $iteration of ${EXERCISE} hit an infrastructure-class failure
rather than a verification failure. The agent did not get to run.

- Failure kind: \`$kind\`
- Exit code from claude: $rc
- Iteration log: \`$iter_log\`
- Detected at: $(date -u +%Y-%m-%dT%H:%M:%SZ)

The orchestrator stopped to avoid burning iterations against a transient
problem. To resume after the underlying issue clears (rate-limit reset,
plan upgrade, network restored), simply re-run
\`./ralph/run-exercise.sh ${EXERCISE}\` — state is on disk; the loop
picks up at the next state.
EOF
}

# Fire a single claude -p call with role-scoped permissions.
# Args:
#   $1 = role label (for logging)
#   $2 = model
#   $3 = agent name (must match a file in .claude/agents/)
#   $4 = task prompt (string)
#   $5 = allowed-tools array name (ALLOWED_ARCHITECT | ALLOWED_IMPLEMENTER | ALLOWED_REVIEWER)
#
# Return codes:
#   0  call succeeded; any agent-produced changes were committed
#   1  call failed for a content reason (verus rejection, agent error)
#      — caller should iterate
#   2  call failed for an infrastructure reason (rate limit, budget, etc.)
#      — caller should stop; INFRA_FAILURE was written
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
  local role_lc
  role_lc=$(echo "$role" | tr '[:upper:]' '[:lower:]')
  local iter_log="logs/${EXERCISE}/ralph/iter-${iteration}-${role_lc}.log"
  echo "  > log: $iter_log"
  # The `--` is load-bearing: --allowedTools/--disallowedTools are variadic
  # and will eat the prompt arg without it.
  claude -p \
    --agent "$agent" \
    --model "$model" \
    --no-session-persistence \
    --permission-mode acceptEdits \
    --allowedTools "${allowed[@]}" \
    --disallowedTools "${DISALLOWED_TOOLS[@]}" \
    -- "$prompt" > "$iter_log" 2>&1
  local rc=$?
  echo "  > exit: $rc  (head of log:)"
  head -5 "$iter_log" | sed 's/^/      /'

  if [ $rc -ne 0 ]; then
    local kind
    kind=$(classify_failure "$iter_log")
    case "$kind" in
      rate_limit|budget|network|invocation)
        echo "  > INFRA FAILURE: $kind (see $INFRA_FAILURE)"
        write_infra_failure "$kind" "$iter_log" "$rc"
        return 2
        ;;
      unknown|*)
        echo "  > failure kind: unknown (treating as content failure; will iterate)"
        return 1
        ;;
    esac
  fi

  # Auto-commit any agent-produced changes. The pre-commit hook runs here
  # and is what makes spec weakening / cheat tokens load-bearing — if the
  # hook rejects, we surface the rejection and return non-zero. The operator
  # (or next iteration) can decide how to recover; we deliberately do NOT
  # auto-clean the working tree, so the rejection state remains inspectable.
  if [ -n "$(git status --porcelain)" ]; then
    git add -A
    local commit_msg="${role_lc}: ${EXERCISE} iter-${iteration}"
    if git commit -m "$commit_msg" > "${iter_log}.commit" 2>&1; then
      echo "  > committed: $(git log --oneline -1)"
      rm -f "${iter_log}.commit"
    else
      echo "  > COMMIT REJECTED by pre-commit hook:"
      sed 's/^/      /' "${iter_log}.commit"
      echo "  > working tree left as-is; inspect and resolve before re-running"
      return 1
    fi
  fi
  return 0
}

# --- prompts ------------------------------------------------------------------

prompt_think() {
  cat <<EOF
You have fresh context. Read in this order:

  1. AGENTS.md
  2. .claude/agents/architect.md (your role definition)
  3. $EXSCOPE (the frozen spec)

Then write $DESIGN per the architect role spec — representation choice,
key invariants, loop-invariant sketches, predicted helper lemmas, SMT
trouble spots, and a suggested order of operations.

Do NOT edit anything in $EXSCOPE. Do NOT run verus. Commit the design
note with message "architect: design for ${EXERCISE}" and stop.
EOF
}

prompt_think_revise() {
  cat <<EOF
You have fresh context. The implementer escalated. Read:

  1. AGENTS.md
  2. .claude/agents/architect.md
  3. $EXSCOPE (current state)
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
  3. $EXSCOPE (current state of the exercise)
  4. $DESIGN (the architect's strategy)
  5. $ATTEMPTS if it exists (your prior attempts — do not repeat them)
  6. The most recent file in $RAWDIR/ if any (last verifier output)

Make EXACTLY ONE new attempt. Scope it narrowly:

  - If $DESIGN contains a `## Sub-tasks` section (or equivalent,
    such as "Suggested order of operations"), work on the smallest
    unfinished sub-task. Pick the next one in order. The list is
    ordered easiest to hardest; do not skip ahead.
  - If the most recent raw verifier output names a specific failing
    function or assertion, scope your edits to that failure. Do
    NOT rewrite unrelated parts of the file in the same attempt.
  - The orchestrator will iterate. You do not need to fix
    everything in one call. Small, surgical, focused edits.

Then perform the per-attempt protocol:

  a. Edit $EXSCOPE — implement or refine the chosen sub-task only.
     For multi-file exercises you may edit any file in the directory;
     verus walks the rest via the \`mod\` declarations.
  b. Run: verus $EXFILE --crate-type=lib > $RAWDIR/attempt-${attempt_num}.txt 2>&1
     (You can `cat` the file afterwards to inspect the output.)
  c. Append an entry to $ATTEMPTS using the format in AGENTS.md.
     Include WHICH sub-task you worked on this iteration.
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
  3. $EXSCOPE
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
  3. The diff: \`git diff ${SPEC_TAG}..HEAD -- $EX_DIFF_PATH\`
  4. $EXSCOPE (current state, for line references)

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
      [ $? -eq 2 ] && exit 2
      ;;

    THINK_REVISE)
      fire_claude THINK_REVISE "$MODEL_ARCHITECT" architect "$(prompt_think_revise)" ALLOWED_ARCHITECT
      [ $? -eq 2 ] && exit 2
      # Defensive cleanup: the architect prompt says to delete escalation.md
      # after writing the revision, but the agent cannot `rm` (the tool
      # whitelist denies it) and historically falls back to truncating the
      # file. Truncation does not satisfy the state classifier's old
      # `-f` test, leading to a THINK_REVISE loop. The classifier now uses
      # `-s` (non-empty) and we also explicitly remove both the empty
      # escalation marker and the prior `escalated` status so the next
      # classification routes to WORK.
      rm -f "$ESCALATION" "$STATUS"
      ;;

    WORK)
      fire_claude WORK "$MODEL_IMPLEMENTER" implementer "$(prompt_work)" ALLOWED_IMPLEMENTER
      [ $? -eq 2 ] && exit 2
      ;;

    WORK_AFTER_REJECT)
      fire_claude WORK_AFTER_REJECT "$MODEL_IMPLEMENTER" implementer "$(prompt_work_after_reject)" ALLOWED_IMPLEMENTER
      [ $? -eq 2 ] && exit 2
      ;;

    REVIEW)
      fire_claude REVIEW "$MODEL_REVIEWER" reviewer "$(prompt_review)" ALLOWED_REVIEWER
      [ $? -eq 2 ] && exit 2
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

if [ $ONCE -eq 1 ]; then
  echo "Exited cleanly after one iteration (--once)."
  exit 0
fi

log_blocked "hit hard outer ceiling of $MAX_OUTER_ITERATIONS iterations"
echo "BLOCKED — outer ceiling hit."
exit 1
