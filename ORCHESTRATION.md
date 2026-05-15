# Orchestration: how to drive the calibration loop

This file is the operator's playbook. The "orchestrator" is the top-level Claude Code session you start when you `cd` into this repo and run `claude`. It reads `AGENTS.md`, this file, and then dispatches to subagents via the Task tool.

You can also drive this manually — the subagents work fine if you invoke them by hand from a single session. The state machine below is the same either way.

## Before you start (one-time setup)

```bash
# 1. Confirm verus is installed and on PATH
verus --version

# 2. Confirm Claude Code sees the subagents
ls .claude/agents/   # should list architect.md, implementer.md, reviewer.md

# 3. Tag the frozen baseline for each exercise. Do this BEFORE any implementation.
git tag spec-frozen-binary_search HEAD
git tag spec-frozen-bounded_log HEAD
git tag spec-frozen-quorum_count HEAD

# 4. Set a session-level token budget alert in your head: hard cap $50 for the weekend.
```

If any of those tags already exist with implementation commits past them, the experiment is invalid — re-baseline only against a commit where every exercise still has `unimplemented!()` bodies.

## The per-exercise state machine

```
state = DESIGN
  → Task(architect): "design exercises/<ex>.rs"
  → check exercises/<ex>.design.md exists
  → state = IMPLEMENT

state = IMPLEMENT
  → Task(implementer): "implement exercises/<ex>.rs"
  → wait for one of: verus-passed | escalation | blocked
  → verus-passed   → state = REVIEW
  → escalation     → state = ESCALATED
  → blocked        → state = BLOCKED

state = ESCALATED
  → Task(architect): "revise exercises/<ex>.rs given logs/<ex>/escalation.md"
  → state = IMPLEMENT  (with the revised design)

state = REVIEW
  → Task(reviewer): "audit exercises/<ex>.rs against spec-frozen-<ex>"
  → check logs/<ex>/review.md → APPROVE or REJECT
  → APPROVE        → state = DONE
  → REJECT         → state = IMPLEMENT  (with the rejection note as context)

state = BLOCKED   → record in writeup, move to next exercise
state = DONE      → record in writeup, move to next exercise
```

Run this state machine for `binary_search`, then `bounded_log`, then `quorum_count`. Do not parallelize across exercises — the experiment measures per-exercise convergence.

## Invocation prompts

When dispatching subagents via the Task tool, use these exact prompts. The subagent definitions already contain their full system prompts; the Task prompt is the one-shot task description.

**Architect (first pass):**
```
Design the implementation of exercises/<ex>.rs. The frozen spec is in that
file with unimplemented!() bodies. Read AGENTS.md and the spec, then write
exercises/<ex>.design.md per the format in your role description. Stop when
the design note is committed.
```

**Architect (revision):**
```
The implementer escalated on exercises/<ex>.rs. Read logs/<ex>/escalation.md,
the current state of exercises/<ex>.rs, and the existing design note. Update
the design note in place with a new "## Revision (escalation N)" section.
Commit and stop.
```

**Implementer:**
```
Implement exercises/<ex>.rs against the design in exercises/<ex>.design.md.
Follow the attempt loop in your role description. Stop when verus exits 0,
when you escalate, or when you hit the iteration cap.
```

**Reviewer:**
```
Audit exercises/<ex>.rs against spec-frozen-<ex>. Run the five-point checklist
in your role description, write logs/<ex>/review.md, and commit. Do not run
verus or edit any source file.
```

## Cost discipline

- Architect runs are cheap per call (Opus reading a spec, writing a design note: 1-3 minutes, low token count). Budget: 1-3 calls per exercise.
- Implementer runs are the largest cost driver (Sonnet, many tool calls per attempt, attempts caps 10-20). Budget: most of your tokens go here.
- Reviewer runs are short (Opus reading a diff, writing a checklist). Budget: 1 call per exercise, possibly 2 if a REJECT happens.

If you're burning faster than expected, the lever is the iteration cap, not the model. Cut bounded_log and quorum_count caps from 20 to 12 if needed and document the change in the writeup as a limitation.

## What to record per exercise

By the time you mark `DONE` or `BLOCKED`, the following must exist:

- `exercises/<ex>.rs` — final state (verified-and-approved, or last attempt at blocker)
- `exercises/<ex>.design.md` — original + any revisions
- `logs/<ex>/attempts.md` — every attempt with verifier output summary
- `logs/<ex>/raw/attempt-N.txt` — raw verifier output per attempt
- `logs/<ex>/review.md` — present only if you reached REVIEW state
- `logs/<ex>/escalation.md` — present only if escalation fired (may exist multiple times; preserve all)
- `logs/<ex>/blocked.md` — present only if you hit the cap

Each artifact should be committed. The git history is part of the result.

## Manual override

You are the operator. If a subagent gets stuck in an obviously-bad loop (e.g. implementer keeps trying the same broken approach), stop it and intervene by hand. Note the intervention in `logs/<ex>/attempts.md` as `## Operator intervention — <timestamp>` so the writeup is honest about where the loop needed help. The writeup's honesty about intervention is the methodology's credibility.

## When all three exercises are done

Fill in `writeup/results_template.md` from the logs. That fills Post B. Then write Post A from `writeup/outline.md` independently — methodology must not be back-fit to results.
