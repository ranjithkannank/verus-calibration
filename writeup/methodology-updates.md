# Source material — updates to the existing blog post

This file is raw input for the blog-writing agent. Goal: refresh the
existing post at <https://ranjithkannan.com/2026/05/10/verus-calibration-formal-verifier-loop/>
with the methodology changes made since publication. The post is still
in draft, so direct edits are appropriate.

Tone target: same as the existing post (technical-practical, "we"
voice, minimal em-dashes, no folksy aphorisms, no time-of-day
framing, no cost numbers). Every claim below is backed by a commit in
<https://github.com/ranjithkannank/verus-calibration>.

## What is and isn't changing

**Staying.** The trust-ladder framing, the three-role architecture
overview, the two-sandbox-layer description, the four exercises (now
counting quorum_cert as the fourth), the "what's different from
existing work" section, the reproducibility links.

**Updating.** The model used by the implementer. The
methodology-contributions section adds two new items. The
limitations section has two items that are no longer limitations
(they were fixed). The "what we'd do next" section is reworked: items
already done should move out, and the BFT direction is now an active
artifact, not a hypothetical.

**Adding.** A short paragraph in the architecture section about
prompt-level scoping per iteration, since this is a new methodology
piece that wasn't in the original post. A new exercise row in the
results table.

## Section-by-section delta

### Architecture (section 2 of the existing post)

The model table currently shows:

| Role        | Model              | ... |
| Architect   | claude-opus-4-7    | ... |
| Implementer | claude-sonnet-4-6  | ... |
| Reviewer    | claude-opus-4-7    | ... |

Change the implementer row's model to `claude-opus-4-7`. Add a
sentence after the table:

> The implementer ran on Sonnet 4.6 for the original three
> exercises and handled them cleanly. For BFT-path exercises starting
> with quorum_cert the implementer was switched to Opus 4.7. Those
> proof obligations involve genuine cardinality and pigeonhole
> reasoning where the model needs to plan over many tokens of
> internal thinking before producing structured output. Aligning the
> hardest role to the strongest model, even when that role is the
> most expensive.

The state-machine diagram is unchanged.

The "boring on purpose" `claude -p` invocation snippet is unchanged.
The `--` separator note is still useful.

After the per-attempt-commits paragraph, add:

> Each implementer attempt is also scoped narrowly. The prompt
> directs the implementer to either pick the next unfinished
> sub-task from the architect's design or scope its edits to the
> specific failing function from the latest verifier output. Small,
> surgical edits per iteration, not file-wide rewrites. The
> orchestrator iterates; the implementer does not need to fix
> everything in one call.

### What happened (section 4)

The results-at-a-glance table needs a fourth row:

| Exercise         | Status | Attempts | Notable                                      |
|------------------|--------|----------|----------------------------------------------|
| binary_search    | DONE   | 1        | (unchanged)                                  |
| bounded_log      | DONE   | 1 (post re-freeze) | (unchanged)                        |
| quorum_count     | DONE   | 2        | (unchanged)                                  |
| quorum_cert      | DONE   | 6        | First BFT-shaped exercise. Six narrow iterations through the architect's sub-task list. Pigeonhole-via-contradiction proof of a quorum certificate's honest-voter guarantee. |

A short narrative for quorum_cert (one to two paragraphs) is in the
separate quorum-cert-post.md file. For this update, the existing
post should either summarise it in two sentences and link to the
follow-up post, or add a §4.4 mirroring the §4.1–§4.3 structure.
Author's call.

### Methodology contributions (section 5)

Currently lists three: no-spec-weakening at two layers, per-attempt
commits, model-per-role wiring. Add two more, written in the same
shape as the existing three:

> **Per-iteration scoping in the implementer prompt.** Each
> implementer call is directed at the smallest unfinished sub-task
> from the architect's design, or at the specific failing function
> from the latest verifier output. Without this, an implementer
> attempt could rewrite the whole file and still pass the
> "one-attempt-per-call" rule; with it, attempts are narrow and the
> next iteration picks up where the previous left off. The
> orchestrator does the iterating; the implementer does one thing
> well per call.

> **Architect produces a sub-task list as part of the design.**
> Every design note ends with a numbered list of sub-tasks, ordered
> easiest to hardest, each small enough to land in one
> edit-verus-iterate cycle. The implementer reads this list and
> works through it in order. The list is what makes the per-call
> scoping concrete: "scope to the smallest unfinished sub-task"
> means something specific because the architect listed them.

> **Pre-spec verification via operator-authored witness files.**
> Before tagging `spec-frozen-<name>`, the operator writes a
> reference implementation in `exercises/<name>_witness.rs` carrying
> the same spec block as the exercise file, then runs
> `ralph/check-spec.sh <name>`. If the witness verifies under Verus
> with no cheat tokens, the spec admits a model and the freeze is
> safe. If verus rejects it, the spec is unprovable or the witness
> is wrong, and the operator fixes one before the agent loop ever
> starts. Two of the calibration exercises required the operator to
> re-freeze the spec mid-run. `bounded_log` because the original
> syntax stopped compiling under a newer Verus release;
> `marzullo` because the original spec omitted a precondition (the
> Helly-1D condition that all correct sensors' intervals share at
> least one common point) that the agent took several attempts to
> surface via constructive counterexample. Both held the
> methodology — the agent refused to weaken specs and wrote
> blocked reports cleanly — but both also burned agent cycles on
> bugs that were ultimately in operator-authored input. The
> witness check catches that class at operator time. The empirical
> negative test in
> [`scripts/test-witness-catches-bad-spec.sh`](https://github.com/ranjithkannank/verus-calibration/blob/main/scripts/test-witness-catches-bad-spec.sh)
> strips the Helly-1D precondition from a copy of the marzullo
> witness and confirms verus rejects it. One operator-time Verus
> run would have replaced those agent attempts.

### What the loop got wrong (section 6 / limitations)

Two items in the existing post should be marked as fixed:

> **The hook's spec-preservation check has a known gap.** It looks
> for lines whose first token is `requires` or `ensures`. The body
> content of those clauses, the continuation lines, is not on the
> frozen-line list. [...]

This is now fixed. The hook walks the frozen file by indentation and
extracts the complete clause body, not just keyword lines. Rewrite
this bullet to past-tense and link to the commit:

> **The hook's spec-preservation check had a known gap.** It used
> to look for lines whose first token was `requires` or `ensures`,
> missing the body content underneath. The bounded_log REJECT was
> caught only by the reviewer because the diff touched body lines.
> The hook now walks the frozen file by indentation and extracts the
> complete clause body, not just keyword lines. The fix sits in
> [`scripts/git-hooks/pre-commit`](https://github.com/ranjithkannank/verus-calibration/blob/main/scripts/git-hooks/pre-commit)
> with tests under `scripts/test-hook-spec-preservation.sh`.

Similarly for the signal-aware bullet:

> **The orchestrator treats every non-zero claude exit code as
> verus-failed.** It isn't. [...]

This is also fixed. Rewrite:

> **The orchestrator used to treat every non-zero claude exit code
> as verus-failed.** A rate-limited response, a budget cap firing, a
> network blip, an invocation error — all produced the same exit
> code and the loop would try the next iteration against the same
> transient problem. The first quorum_count run burned 27
> rate-limited iterations before its outer ceiling fired. The
> orchestrator now classifies failures by grepping the iteration log
> against known signatures and exits cleanly on infrastructure
> failures rather than churning. Tests in
> [`ralph/test-classify-failure.sh`](https://github.com/ranjithkannank/verus-calibration/blob/main/ralph/test-classify-failure.sh).

The "three exercises is not a benchmark" bullet remains true and is
now slightly stronger ("four exercises is still not a benchmark").

### What we'd do next (section 9)

Items 1 and 2 ("reproduce and fix the silent wrapper hang" — actually
no, that one we didn't reproduce, leave it) and item 3 ("fix the
hook's spec-preservation gap") need handling.

- Item 1 (wrapper hang) — leave as-is. Not investigated to root
  cause.
- Item 2 (signal-aware orchestrator) — move out, since it's now
  done. Mention in the limitations rewrite above.
- Item 3 (hook gap) — move out, done.
- Item 4 (stream-mode logging) — leave as-is. Not done.
- Item 5 (per-attempt time and token measurement in attempts.md) —
  partly done; the agent self-reports its approach now but not
  costs. Leave.
- Item 6 (fourth exercise needing escalation) — replace with: a
  fourth exercise has now run (quorum_cert), but the architect's
  initial design was correct and no escalation fired. The path that
  needs exercising remains untested.
- Item 7 (no spaces in working directory) — leave as-is. Not done.

Add a new closing item about the BFT direction:

> The next rung — applying the methodology to multi-module Verus
> code with cross-module invariants — has its first artifact now.
> `quorum_cert` is a single-module quorum-certificate library with a
> safety lemma, verified end-to-end through the same loop that
> produced the calibration exercises. The full progression continues
> with a verified Byzantine agreement primitive, then verified
> sensor fusion for safety-critical use, then a hardware-deployed
> demonstration.

### Where this fits (section 8)

No structural changes. The Schubert / Huntley / Karpathy / Microsoft
contrast is still accurate. The closing sentence about the next rung
can now be more concrete:

> The next rung is multi-module Verus code with cross-module
> invariants. The first single-module BFT primitive (a verified
> quorum certificate with its safety lemma) is in the repo as
> `quorum_cert`; the multi-module step is the next concrete artifact
> on the path.

### Reproducing (section 10)

Models list needs to show the implementer is now Opus 4.7. Add a
parenthetical:

> Models: `claude-opus-4-7` for all three roles. The implementer was
> originally `claude-sonnet-4-6` and handled the three calibration
> exercises competently; switched for the BFT-path exercises. The
> bash script in `ralph/run-exercise.sh` holds the model choices.

## Commits supporting these claims

For the writing agent's reference. Each refinement maps to a specific
public commit:

- Hook spec-preservation extension: `e30a65d`
- Signal-aware orchestrator: `3575527`
- Architect playbook (initial six patterns): `1ae18a9`
- `quorum_cert` scaffolded: `4c2b880`
- `MODEL_IMPLEMENTER` switched to Opus, prompt scoping, architect Sub-tasks requirement: `bda7dbd`
- `quorum_cert` DONE: `5a2a87d`
- Pre-spec verification tooling (`ralph/check-spec.sh`, marzullo
  witness, empirical negative test): `17a4fdd`

All on `main` at <https://github.com/ranjithkannank/verus-calibration>.
