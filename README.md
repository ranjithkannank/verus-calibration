# verus-calibration

Started as a weekend experiment: how reliably can an autonomous coding
loop produce **formally verified** Verus code without cheating? Now an
ongoing track toward making verified Byzantine-tolerant systems
cheaper to build for safety-critical applications.

Verus exercises run through a Ralph-style autonomous loop with three
boundaries that forbid the agent from weakening specs, bypassing the
verifier, or reading operator-authored reference implementations: a
git pre-commit hook, a Claude Code tool whitelist, and an
operator-authored witness file gated behind explicit deny patterns.

## Status

16 exercises DONE on `main` (verus passed + reviewer `APPROVE`).
The original three calibration tasks landed first; the track has
since extended into Byzantine-tolerant primitives, multi-module
composition, deliberate discovery tests on the methodology itself,
and a first invention test on a proof family the playbook did not
document.

**Blog posts (in draft, not yet shared externally):**

- [Wiring a Formal Verifier into an Autonomous Coding Loop](https://ranjithkannan.com/2026/05/10/verus-calibration-formal-verifier-loop/)
  — methodology baseline, calibration-era. Currently being revised
  with the methodology refinements landed since publication (see
  `writeup/methodology-updates.md`).
- [Verified Byzantine-Tolerant Sensor Fusion](https://ranjithkannan.com/2026/05/17/verified-byzantine-tolerant-sensor-fusion/)
  — BFT-track primitives (ft_midpoint, marzullo). Currently being
  revised to absorb the composition track (see
  `writeup/composition-post.md`).

Exercise inventory and per-exercise notes live in `AGENTS.md`
(authoritative exercise order + the discovered-patterns playbook).
Snapshot of the verified set:

| Track                 | Exercises                                                                                              | Total |
|-----------------------|---------------------------------------------------------------------------------------------------------|-------|
| Calibration           | `binary_search`, `bounded_log`, `quorum_count`                                                          | 3     |
| BFT primitives        | `quorum_cert`, `ft_midpoint`, `marzullo`                                                                | 3     |
| Multi-module          | `cross_module_counter`, `counter_multifile`, `counter_producer`                                         | 3     |
| Composition           | `sensor_poll`, `sensor_poll_signed`, `sensor_poll_honest`                                               | 3     |
| Discovery tests       | `sensor_poll_honest` (also above), `counter_filler`                                                     | 2     |
| Invention test        | `swap_multiset`                                                                                         | 1     |
| Invalidated (evidence)| `vec_swap`, `vec_swap_v2` — kept on disk + tagged, see methodology-refinements section                  | 2     |

The discovery and invention tests were re-audited on 2026-05-18
under a hardened tool whitelist that explicitly denies the agent
reading the operator-authored witness file; both discovery claims
verified in one attempt again with the agent's own prior playbook
summary stripped from `AGENTS.md` (re-added with an
`audit-confirmed` tag after the re-run passed).

Forward-looking items live in `BACKLOG.md`. The original calibration
post and raw run record sit in `writeup/blog-post.md` and
`writeup/writeup.md`.

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
├── AGENTS.md              shared rules + exercise order + discovered-patterns playbook
├── ORCHESTRATION.md       state machine, prompts, cost discipline
├── BACKLOG.md             deferred work + "what's next" decision record
├── exercises/
│   ├── binary_search.rs           calibration exercise 1
│   ├── bounded_log.rs             calibration exercise 2
│   ├── quorum_count.rs            calibration exercise 3
│   ├── quorum_cert.rs             BFT quorum certificate + safety lemma
│   ├── ft_midpoint.rs             sensor-fusion fault-tolerant midpoint
│   ├── marzullo.rs                Marzullo's algorithm (interval form)
│   ├── cross_module_counter.rs    single-file multi-module exercise
│   ├── counter_multifile/         first multi-file exercise (sibling .rs files)
│   ├── counter_producer/          cross-module composition; producer's loop invariant
│   ├── counter_filler/            second deliberate discovery test (target-bounded loop)
│   ├── sensor_poll/               composition demonstration (port marzullo + auth)
│   ├── sensor_poll_signed/        signature trust boundary at the spec layer
│   ├── sensor_poll_honest/        first deliberate discovery test (honest-voter clause)
│   ├── swap_multiset.rs           first invention test (multiset preservation)
│   ├── vec_swap.rs                INVALIDATED invention test #1 (witness was readable)
│   ├── vec_swap_v2.rs             INVALIDATED invention test #2 (operator copy-paste error)
│   └── *_witness.rs / *_witness/  operator-authored reference impls (denied to the agent)
├── .claude/agents/
│   ├── architect.md       Opus 4.7 — design, no Edit, no Bash(verus)
│   ├── implementer.md     Opus 4.7 — full toolset incl. verus
│   └── reviewer.md        Opus 4.7 — audit only, no Edit
├── ralph/
│   ├── run-exercise.sh    main Ralph driver (bash state machine)
│   ├── run-all.sh         sweep multiple exercises
│   ├── check-spec.sh      pre-spec verification: verus the operator witness
│   └── test-*.sh          unit tests on state classification + failure classification
├── scripts/
│   ├── setup-check.sh                pre-flight verification
│   ├── install-hooks.sh              symlinks git hooks into .git/hooks/
│   ├── git-hooks/pre-commit          path whitelist + cheat detection + spec preservation
│   ├── test-witness-catches-bad-spec.sh   empirical proof the witness check catches Helly-1D drop
│   ├── probe-witness-deny.sh         empirical proof the witness-deny ACL fires (six attack vectors)
│   └── verify.sh                     run `verus` on all exercises
├── logs/
│   ├── <exercise>/        per-exercise attempt logs, raw verifier output, reviews
│   └── _probe/            output of scripts/probe-witness-deny.sh
└── writeup/
    ├── blog-post.md          publication-ready calibration-era post
    ├── writeup.md            raw run record, full detail
    ├── outline.md            original two-post split plan
    ├── results_template.md   data table + failure taxonomy skeleton
    ├── methodology-updates.md   revision input for the May 10 post
    ├── composition-post.md      revision input for the May 17 post
    ├── multi-module-post.md     source draft on the multi-module track
    ├── sensor-fusion-post.md    source draft on the sensor-fusion primitives
    └── quorum-cert-post.md      source draft on the BFT quorum certificate
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
| Architect   | claude-opus-4-7    | Reads frozen spec, writes a design note with an ordered sub-task list. No verifier output on first pass. Re-runs on escalation. |
| Implementer | claude-opus-4-7    | One verus attempt per call, scoped to the smallest unfinished sub-task: edit, run verus, log, commit. Stops at the iteration cap or on escalation. |
| Reviewer    | claude-opus-4-7    | After verus passes, audits the diff against `spec-frozen-<ex>`. APPROVE or REJECT. Does not check correctness — verus already did. |

The implementer was originally `claude-sonnet-4-6` and handled the three calibration exercises (binary_search, bounded_log, quorum_count) competently. Switched to `claude-opus-4-7` for quorum_cert onward because the BFT-path proof obligations need deeper reasoning. The bash script in `ralph/run-exercise.sh` holds the source of truth for model choices.

The state is **inferred from the filesystem each iteration**: presence of
`exercises/<ex>.design.md`, contents of `logs/<ex>/attempts.md` and
`logs/<ex>/status`, presence of `logs/<ex>/escalation.md`, the conclusion
in `logs/<ex>/review.md`. No state file, no orchestration daemon — the
state machine is reconstructible by `ls`.

## The three boundaries

The loop is autonomous (no human in the iteration). Three boundaries
keep it honest:

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
namespace, no network namespace. A motivated adversarial agent could
find paths around all of this. Real isolation would require Docker; for
a weekend experiment with eyes on the first iterations, the three
layers documented here are the deliberate trade-off. The writeup says
so plainly.

**3. Operator-territory boundary — witness-file deny patterns**

`ralph/check-spec.sh` requires the operator to write a reference
implementation in `exercises/<name>_witness.rs` (or
`exercises/<name>_witness/`) that satisfies the same frozen spec as
the exercise, so verus can confirm the spec admits a model before the
loop ever runs. The witness contains a working proof; the agent must
not see it, or discovery-test and invention-test results lose meaning.

The `DISALLOWED_TOOLS` array in `ralph/run-exercise.sh` denies the
agent reading these files via every common path:

- `Read(**/*_witness*)`, `Read(**/*_witness/**)`
- `Glob(**/*_witness*)`, `Grep(**/*_witness.rs)`
- `Bash(cat|head|tail|ls|grep|rg *_witness*)`

Empirically verified: `scripts/probe-witness-deny.sh` invokes
`claude -p` with the production deny list and asks the agent to read
a witness file via six different paths. Every one is blocked.

Historical context: the first invention test (`vec_swap`) ran under
a permissive whitelist that allowed witness reads. The agent's own
attempt-1 commit message was "port witness proof"; its self-authored
playbook entry read "the architect-owned witness file was visible to
the implementer and gave the proof skeleton verbatim. For pure
invention measurement, future exercises should hide the witness." The
deny patterns landed in response; the clean re-test
(`swap_multiset`) verified in one attempt with a proof structurally
different from the witness.

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

## Methodology refinements since the original blog post

The original post (linked above) captured the methodology as it stood after the three calibration exercises. The harness has since been refined based on lessons from the BFT-path, multi-module, composition, and methodology-integrity work:

- **Signal-aware orchestrator.** `fire_claude` now classifies non-zero claude exit codes by grepping the iteration log: rate-limit, budget cap, network blip, invocation error all surface as a distinct infrastructure-failure state. The orchestrator exits cleanly with an `infra_failure.md` marker rather than burning iterations against a transient problem. Tests in `ralph/test-classify-failure.sh`.
- **Hook spec-preservation extended.** The pre-commit hook now extracts complete `requires` / `ensures` clause bodies via indentation tracking, not just the keyword lines. Closes the body-content blind spot the reviewer used to be the only layer catching. Tests in `scripts/test-hook-spec-preservation.sh`.
- **Implementer scoped per iteration.** `prompt_work()` now directs the implementer to either pick the next unfinished sub-task from the design's order list, or to scope edits to the specific failing function from the latest verifier output. The orchestrator iterates; the implementer no longer tries to land everything in one attempt.
- **Architect requires a Sub-tasks section.** Every design note ends with a numbered list of sub-tasks, ordered easiest to hardest, each small enough to land in one edit-verus-iterate cycle.
- **Architect playbook grew.** Recurring proof patterns are accumulated in `AGENTS.md`'s "Discovered patterns" section, one entry per exercise. The agent reads these on every iteration; cross-exercise pattern transfer is what makes the methodology compound.
- **Pre-spec verification via witness files.** Two early exercises required operator intervention to re-freeze the spec (bounded_log: Verus syntax migration; marzullo: missing Helly-1D precondition). Both bugs surfaced *after* the agent had burned several attempts. `ralph/check-spec.sh <name>` verifies an operator-authored reference implementation in `exercises/<name>_witness.rs` (or `exercises/<name>_witness/`) against the same frozen spec. If the witness verifies, the spec admits a model. If verus rejects it, the spec is wrong and the operator fixes it before the agent ever sees it. The empirical negative test in `scripts/test-witness-catches-bad-spec.sh` confirms the tool would have caught the marzullo bug at operator time.
- **Witness-deny ACL + empirical probe.** The original implementer tool whitelist granted generic `Read` and `Glob` with no path qualifier, so the witness file was readable. A first invention test (`vec_swap`) made this concrete: the agent's iter-1 commit message read "port witness proof" and its own playbook entry flagged the leak. The `DISALLOWED_TOOLS` array in `ralph/run-exercise.sh` was extended with `Read(**/*_witness*)`, `Glob`/`Grep` equivalents, and `Bash(cat|grep|rg|head|tail|ls *_witness*)` patterns. `scripts/probe-witness-deny.sh` empirically demonstrates that all six common attack vectors get blocked. The clean re-test (`swap_multiset`) verified in one attempt with a proof structurally different from the witness.
- **Deliberate discovery and invention tests.** Beyond standard exercises, the methodology track now includes exercises designed to test specific methodology claims: discovery tests (`sensor_poll_honest`, `counter_filler`) measure whether the agent can adapt a proof family the playbook names to a new obligation; the invention test (`swap_multiset`) measures whether the agent can assemble a proof in a family the playbook does *not* document. The discovery tests were re-audited on 2026-05-18 under the hardened whitelist with each exercise's own prior playbook summary stripped from AGENTS.md; both verified in one attempt again.

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
