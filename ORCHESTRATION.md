# Orchestration: the Ralph loop

The autonomous loop lives in `ralph/run-exercise.sh`. It is a bash state
machine that fires one `claude -p` call per iteration, with **fresh context
every time**, selecting the role (architect / implementer / reviewer) and
model from the current state read out of filesystem artifacts.

This matches Huntley's Ralph pattern: each iteration is independent, memory
lives in `AGENTS.md`, the design note, `logs/<ex>/attempts.md`, and git
history. Nothing about the loop's reasoning survives a context reset — which
is the point. The implementer cannot get worse over time from accumulated
confused context, and every attempt is reproducible from clean state.

## Quickstart

```bash
# from the repo root, after running scripts/setup-check.sh passes:
./ralph/run-exercise.sh binary_search        # drive one exercise to done/blocked
./ralph/run-all.sh                           # run all three in sequence
./ralph/run-exercise.sh binary_search --dry-run  # state transitions only, no claude calls
```

Both scripts exit 0 on DONE (verified and approved), 1 on BLOCKED, 2 on
usage/environment error.

## The state machine

Per-exercise states, with the role fired and the artifact each writes:

| State              | Role        | Model            | Writes                                |
|--------------------|-------------|------------------|---------------------------------------|
| THINK              | architect   | claude-opus-4-7  | `exercises/<ex>.design.md`            |
| WORK               | implementer | claude-sonnet-4-6 | edits `<ex>.rs`, appends attempts.md, writes `status`, runs verus into `raw/attempt-N.txt`, commits |
| REVIEW             | reviewer    | claude-opus-4-7  | `logs/<ex>/review.md` (APPROVE or REJECT) |
| WORK_AFTER_REJECT  | implementer | claude-sonnet-4-6 | same as WORK, plus deletes `review.md` after success |
| THINK_REVISE       | architect   | claude-opus-4-7  | appends revision section to `design.md`, deletes `escalation.md` |
| APPROVED           | (no call)   | —                | touches `done.flag`, commits             |
| CAP_HIT / BLOCKED  | (no call)   | —                | writes `blocked.md`, commits, exits 1    |

The state is **inferred from the filesystem each iteration**, not stored
explicitly. That makes the loop resumable — kill it, re-run, it picks up where
it left off.

## State transitions

```
                  ┌─────────────────────────────────┐
                  ▼                                 │
START ──► THINK ──► WORK ──► (verus passes) ──► REVIEW
                     │                            │   │
                     │                            │   ├─► APPROVE ──► DONE
                     │                            │   │
                     │                            │   └─► REJECT ──► WORK_AFTER_REJECT ──┐
                     │                            │                                       │
                     │                            └───────────────────────────────────────┘
                     │
                     ├─► (escalation written by worker) ──► THINK_REVISE ──► WORK
                     │
                     └─► (cap hit) ──► CAP_HIT / BLOCKED
```

## Per-iteration claude invocation

Every call looks like this (managed by `fire_claude` in `run-exercise.sh`):

```bash
claude -p \
  --agent <architect|implementer|reviewer> \
  --model <claude-opus-4-7|claude-sonnet-4-6> \
  --no-session-persistence \
  --permission-mode acceptEdits \
  --max-budget-usd 2.00 \
  "<role-specific prompt for this state>"
```

- `--agent` selects the subagent definition from `.claude/agents/` (role identity, tool restrictions).
- `--model` overrides the model from the agent frontmatter, so the bash script holds the source of truth.
- `--no-session-persistence` is the Ralph discipline: the session is not saved, the next call starts from zero.
- `--permission-mode acceptEdits` auto-approves file edits but still gates risky operations.
- `--max-budget-usd` is a safety net per call. No-op on subscription billing; takes effect if you ever fall back to API metering.

## Before the first run (one-time)

```bash
cd /path/to/verus-calibration
bash scripts/setup-check.sh   # must exit 0 — installs Verus/Rust if missing
```

`setup-check.sh` will tell you exactly what to fix if anything is off. It
also creates the `spec-frozen-<exercise>` git tags the reviewer audits
against.

## Resumability

The loop reads state from the filesystem on every iteration. To pause:
`Ctrl-C`. To resume: re-run the same command. It will pick up at the next
state — no special resume flag.

If a state gets stuck (e.g. the implementer never writes `status`), inspect
`logs/<ex>/` to see the last artifact written, decide whether to manually
unstick or declare blocked, and either delete a stale file or write
`logs/<ex>/blocked.md` to halt the loop cleanly.

## Cost discipline

- Architect (Opus) runs once per exercise, plus once per escalation. Budget: 1-3 calls.
- Reviewer (Opus) runs once per exercise, plus once per REJECT round. Budget: 1-2 calls.
- Implementer (Sonnet) is the cost driver: one call per attempt, cap 10/20/20.

Worst case for the weekend: ~60 claude calls total (3 exercises × ~20 calls).
Realistic case: 25-35 calls. With Pro subscription, that lives well within
the weekly Opus quota provided you do not also run Opus interactively for
other work.

If you start to feel rate-limit pressure: cut the iteration cap from 20 to
12 for `bounded_log` and `quorum_count`, and document the change in the
writeup as a limitation. Do not raise `--max-budget-usd` to push through —
on a Pro plan that flag is moot; the limit is the weekly quota.

## What success looks like

After `./ralph/run-all.sh` exits 0, every exercise has:

- `exercises/<ex>.rs` — verified
- `exercises/<ex>.design.md` — original + any revisions
- `logs/<ex>/attempts.md` — every attempt logged
- `logs/<ex>/raw/attempt-N.txt` — raw verifier output per attempt
- `logs/<ex>/review.md` — Conclusion: APPROVE
- `logs/<ex>/done.flag` — sentinel
- Per-attempt commits in `git log`

Then fill `writeup/results_template.md` from the logs and write the blog
posts from `writeup/outline.md`.

## Manual override

You are the operator. If the loop is clearly stuck in a bad pattern (e.g.
the implementer keeps trying minor variants of the same broken approach),
stop it with `Ctrl-C` and intervene by hand. Then either:

- Write `logs/<ex>/escalation.md` yourself with what you observed, then
  re-run — the loop will fire THINK_REVISE on the next iteration.
- Or write `logs/<ex>/blocked.md` to halt cleanly.

Note manual interventions in `logs/<ex>/attempts.md` with a header like
`## Operator intervention — <timestamp>` so the writeup is honest about
where the loop needed human help. The writeup's credibility depends on
this.

## Tests

The state-machine classification is unit-tested:

```bash
./ralph/test-state-machine.sh
```

Should pass 10/10 before any real Ralph run.
