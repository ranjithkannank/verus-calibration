# verus-calibration

A weekend experiment: how reliably can an autonomous coding loop produce
**formally verified** Verus code without cheating?

Three Verus exercises of increasing difficulty, run through a Ralph-style
autonomous loop with two separate boundaries forbidding the agent from
weakening specs or bypassing the verifier. The goal is four numbers:

- First-try success rate
- Attempts to convergence on the rest
- Tokens per verified function
- A taxonomy of recurring failure modes

The methodology and results land as two posts in an ongoing series on
autonomous-loop trust infrastructure (mutation testing → audit/decision
split → integration contracts → **vericoding as proof-grade feedback**).

## Status

All three exercises landed `DONE` (verus passed + reviewer
`APPROVE`). Calibration is complete; the writeup is drafted.

| Exercise         | Status | Attempts to verify | Notes                                          |
|------------------|--------|--------------------|------------------------------------------------|
| binary_search    | DONE   | 1                  | Clean first-try; architect's design predicted every invariant. |
| bounded_log      | DONE   | 1 (post re-freeze) | Surfaced a Verus version mismatch in the operator-authored frozen spec; the reviewer's REJECT was the methodology working as intended. |
| quorum_count     | DONE   | 2                  | Concrete-to-abstract proof bridge; implementer grepped local `vstd` and wrote a recursive cardinality lemma. |

The full narrative lives in `writeup/blog-post.md` (publication-ready)
and `writeup/writeup.md` (raw run record).

## Quickstart

```bash
# 1. one-time setup (installs Verus + Rust toolchain if missing,
#    creates frozen-spec git tags, installs the pre-commit hook)
bash scripts/setup-check.sh
bash scripts/install-hooks.sh

# 2. cheap smoke tests, no claude calls
./ralph/test-state-machine.sh
./ralph/run-exercise.sh binary_search --dry-run

# 3. live run, one exercise
./ralph/run-exercise.sh binary_search

# 4. live run, all three in sequence
./ralph/run-all.sh
```

Each iteration uses fresh context (`--no-session-persistence`). The loop
is **resumable** — `Ctrl-C` any time, re-run, it picks up at the next
state by reading filesystem artifacts.

## What's in the box

```
verus-calibration/
├── README.md
├── AGENTS.md              shared rules every agent reads
├── ORCHESTRATION.md       state machine, prompts, cost discipline
├── exercises/
│   ├── binary_search.rs   exercise 1: frozen spec, unimplemented body
│   ├── bounded_log.rs     exercise 2: frame property on append
│   └── quorum_count.rs    exercise 3: distinct-count vs Set::len()
├── .claude/agents/
│   ├── architect.md       Opus 4.7 — design, no Edit, no Bash(verus)
│   ├── implementer.md     Sonnet 4.6 — full toolset incl. verus
│   └── reviewer.md        Opus 4.7 — audit only, no Edit
├── ralph/
│   ├── run-exercise.sh    main Ralph driver (bash state machine)
│   ├── run-all.sh         sweep all three exercises
│   └── test-state-machine.sh   10/10 unit tests on state classification
├── scripts/
│   ├── setup-check.sh     37-point pre-flight verification
│   ├── install-hooks.sh   symlinks git hooks into .git/hooks/
│   ├── git-hooks/pre-commit   path whitelist + cheat detection
│   └── verify.sh          run `verus` on all exercises
├── logs/<exercise>/       attempt logs, raw verifier output, reviews
└── writeup/
    ├── blog-post.md      publication-ready post in the author's voice
    ├── writeup.md        raw run record, full detail (5k+ words)
    ├── outline.md        original two-post split plan
    └── results_template.md   data table + failure taxonomy skeleton
```

## Architecture

Three roles run as **separate Claude Code subagents on different models**.
The bash outer loop dispatches them in a state machine driven by what
files exist on disk.

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

| Role        | Model              | Job                                                                                            |
|-------------|--------------------|------------------------------------------------------------------------------------------------|
| Architect   | claude-opus-4-7    | Reads frozen spec, writes a design note. No verifier output on first pass. Re-runs on escalation. |
| Implementer | claude-sonnet-4-6  | One verus attempt per call: edit, run verus, log, commit. Stops at the iteration cap or on escalation. |
| Reviewer    | claude-opus-4-7    | After verus passes, audits the diff against `spec-frozen-<ex>`. APPROVE or REJECT. Does not check correctness — verus already did. |

The state is **inferred from the filesystem each iteration**: presence of
`exercises/<ex>.design.md`, contents of `logs/<ex>/attempts.md` and
`logs/<ex>/status`, presence of `logs/<ex>/escalation.md`, the conclusion
in `logs/<ex>/review.md`. No state file, no orchestration daemon — the
state machine is reconstructible by `ls`.

## The two boundaries

The loop is autonomous (no human in the iteration). Two boundaries keep
it honest:

**1. Content boundary — git pre-commit hook**

Every commit goes through `scripts/git-hooks/pre-commit`, symlinked into
`.git/hooks/`. The hook rejects:

- **Path violations:** files outside `exercises/`, `logs/`, `writeup/`,
  `ralph/`, `scripts/`, `.claude/`, or the top-level docs.
- **Cheat tokens in `exercises/*.rs`:** new occurrences of `assume(`,
  `#[verifier::external_body]`, `unreachable!()`, `panic!(`,
  `assume_specification`.
- **Spec drift:** every `requires` / `ensures` line that existed in
  `spec-frozen-<exercise>` must still appear verbatim in the staged
  version. New helper `spec fn`s are fine; modifying frozen ones is not.

The agent cannot bypass the hook with `git commit --no-verify` because
that flag is on the always-deny tool list (next section).

**2. Capability boundary — Claude Code tool whitelist**

Every `claude -p` call in `ralph/run-exercise.sh` passes a strict
`--allowedTools` list scoped to the current role. Highlights:

- **Universal deny:** `WebFetch`, `WebSearch`, `Task` (no sub-agent
  fan-out), `NotebookEdit`, plus bash patterns for `rm`, `git
  push/reset/rebase/checkout/restore/config`, `*--no-verify*`,
  `curl/wget/ssh`, `brew/npm/pip`, `sudo`.
- **Architect:** Read/Write/Glob/Grep + narrow git bash (no verus, no Edit).
- **Implementer:** Read/Edit/Write/Glob/Grep + `Bash(verus *)` + narrow git bash.
- **Reviewer:** Read/Write/Glob/Grep + git inspection bash (no Edit, no verus).

Honest limitations: this is **not** process isolation. The `Write` tool
isn't path-restricted the way `Bash` patterns are, so a file written
outside the repo wouldn't be caught until commit. There's no filesystem
namespace, no network namespace. A motivated adversarial agent could find
paths around all of this. Real isolation would require Docker; for a
weekend experiment with eyes on the first iterations, layer 1+2 is the
deliberate trade-off. The writeup will say so plainly.

## Setup from scratch

If you're starting from an empty Mac:

```bash
git clone <this repo>
cd verus-calibration

# Installs rustup, the pinned Rust toolchain, the Verus binary release,
# and prints the failures clearly if anything's still missing.
bash scripts/setup-check.sh
```

The script is idempotent. The first time it'll likely report some
toolchain failures; it tells you exactly what to install. On a clean Mac
that's three things:

1. **rustup** — `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain none`
2. **Verus** — download the latest arm64-macos release from
   <https://github.com/verus-lang/verus/releases/latest>, unzip into
   `~/Documents/projects/verus`, add that dir to `PATH` in `~/.zshrc`.
3. **The Rust toolchain Verus pins** — `rustup install <version>`
   (Verus's first run will tell you which version).

When `setup-check.sh` exits 0 with 37/37 passes, you're ready.

## Outputs

After a successful run, each exercise produces:

```
exercises/<ex>.rs          verified
exercises/<ex>.design.md   architect's strategy (+ revisions on escalation)
logs/<ex>/
├── attempts.md            one entry per implementer attempt
├── status                 last-iteration signal (verus_passed | _failed | escalated)
├── raw/attempt-N.txt      verbatim verifier output, per attempt
├── escalation.md          present iff the implementer asked for re-think
├── review.md              reviewer's audit (Conclusion: APPROVE | REJECT)
└── done.flag              sentinel created on APPROVE
```

The git history is part of the artifact — every implementer attempt is
its own commit (`<exercise> attempt-N: <one-line>`), so the writeup's
failure taxonomy can quote actual diffs.

## Reproducing the experiment

- Verus version: pinned by the release zip; printed by `verus --version`.
- Claude Code version: `claude --version`.
- Model versions: hard-coded in `ralph/run-exercise.sh` (`MODEL_ARCHITECT`,
  `MODEL_IMPLEMENTER`, `MODEL_REVIEWER`). Change them there if you want
  to swap roles or test a different generation.
- All raw verifier output is committed in `logs/<ex>/raw/`. All prompts
  are inlined in `ralph/run-exercise.sh` (no separate prompt files).
- AGENTS.md is the rule book all roles read. The pre-commit hook in
  `scripts/git-hooks/pre-commit` is the enforcement mechanism. If your
  results don't match mine, those three places are where to look first.

## Why this matters

The blog series so far has tightened the feedback signal an autonomous
coding loop runs against: first plain tests, then mutation testing,
then splitting the auditor from the writer, then catching the gap
between green tests and integration contracts. Each step closes a hole
through which a wrong loop could still pass.

A formal verifier is the limit of that progression. It is a feedback
signal the agent cannot satisfy except by either weakening the spec or
actually being correct. The whole experiment is the empirical test of
whether the first path can be ruled out by rules and tooling alone.

## Related work and next steps

Microsoft Research's [verus-proof-synthesis][msft] is the closest prior
work: AutoVerus ([arXiv:2409.13082][autoverus], OOPSLA 2025) for
single-function tasks, and VeruSAGE ([arXiv:2512.18436][verusage]) for
multi-module code. Both are LLM-based on OpenAI / Azure OpenAI, with a
single-agent pipeline. The differentiator of this project is the
separate audit role on a different model, the commit-time
spec-preservation hook running alongside the LLM audit, and per-attempt
commits as the unit of evaluation. The combination is what the
bounded_log `REJECT` result depended on.

The concrete next step is to draw tasks from VeruSAGE-Bench (849 tasks
across real distributed systems, OS kernels, storage), run them
through this loop, and use AutoVerus and VeruSAGE as baselines on the
same tasks. That gets us comparable-to-published numbers on multi-module
Verus code without inventing exercises.

[msft]: https://github.com/microsoft/verus-proof-synthesis
[autoverus]: https://arxiv.org/abs/2409.13082
[verusage]: https://arxiv.org/abs/2512.18436

## License

Code: MIT.
Writing in `writeup/`, including the outline and any blog drafts: CC BY 4.0.

## Acknowledgments

- The Ralph pattern: Geoffrey Huntley.
- The autoresearch framing: Andrej Karpathy.
- Verus: the verus-lang team.
- Prior work on LLM-based Verus proof synthesis: Microsoft Research's
  AutoVerus / VeruSAGE teams.
