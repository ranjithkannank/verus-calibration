# Results

Fill in after each exercise completes (or hits the iteration cap).

## Results table

| Exercise        | Spec LOC | Impl LOC | Proof LOC | First-try | Attempts | Wall clock | Tokens | Weakened? | Notes |
|-----------------|----------|----------|-----------|-----------|----------|------------|--------|-----------|-------|
| binary_search   |          |          |           |           |          |            |        |           |       |
| bounded_log     |          |          |           |           |          |            |        |           |       |
| quorum_count    |          |          |           |           |          |            |        |           |       |

- **First-try** = Y if the first verification attempt exited 0; N otherwise.
- **Attempts** = total iterations including the successful one, or "blocked@N" if cap hit.
- **Wall clock** = total elapsed across all attempts on that exercise.
- **Tokens** = approximate from Claude Code session.
- **Weakened?** = Y if any `// SPEC WEAKENED:` marker appears anywhere.

## Failure taxonomy

Categorize each failed attempt across the three exercises. Aim for 3-6 categories.

Suggested starters; rename freely:

- **Loop invariant insufficient.** The body was correct but the inductive invariant the loop guessed didn't carry enough information through the iteration. Example: ...
- **Frame reasoning.** The solver couldn't conclude "this field didn't change" without an explicit assert. Example: ...
- **Concrete-to-abstract gap.** The implementation used `Vec`-level operations while the spec talked about `Seq` or `Set`. Example: ...
- **SMT timeout / encoding.** The solver gave up on an obligation that's mathematically simple. Example: ...
- **Phantom proof.** The loop wrote an assert that was itself the thing being proven. Example: ...

For each, note: how many attempts it consumed, whether the loop recovered, what would have unblocked it faster.

## Interpretation

Two-to-three paragraphs. Answer:

1. Given these numbers, is a full Byzantine-agreement-on-real-hardware project feasible with this tooling?
2. Where will proof engineering dominate human time?
3. Did the no-spec-weakening rule hold? If yes, that's the load-bearing part of the methodology.

## Limitations

State plainly:
- Three exercises is not a benchmark.
- Single model version (`claude-opus-4-7`, weekend of [date]).
- Single operator (me); my prompt-engineering reflexes are confounded with the loop's capability.
- Exercises were chosen to be relevant to Byzantine fault tolerance, which biases away from the easier verification regimes Verus already handles well.

## Reproducibility

- Repo: <commit hash>
- Verus version: `verus --version` output
- Claude Code version: `claude --version` output
- Commands to reproduce: see `README.md`
- All raw verifier output in `logs/<exercise>/raw/`.
