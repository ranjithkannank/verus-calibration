# verus-calibration

A weekend experiment: how reliably can an autonomous coding loop produce verified Verus code?

Three exercises of increasing difficulty, run through Claude Code with the loop forbidden from weakening specs or bypassing the verifier. The goal is four numbers: first-try success rate, iterations-to-convergence on the rest, tokens per verified function, and a taxonomy of recurring failures.

## Layout

- `exercises/` — three single-file Verus exercises with spec/signature only, bodies left unimplemented
- `logs/<exercise>/` — per-exercise attempt logs the loop writes into
- `scripts/verify.sh` — runs `verus` on every exercise, exits non-zero on any failure
- `writeup/` — outline and final two-page results writeup
- `AGENTS.md` — rules the loop must follow (no spec weakening, iteration caps, logging)

## Running an exercise

```bash
# verify a single exercise
verus exercises/binary_search.rs --crate-type=lib

# verify all
./scripts/verify.sh
```

## The experiment, briefly

For each of three exercises:

1. The spec is fixed. The loop must not change `requires`, `ensures`, or any `spec fn`.
2. The loop fills in `exec` bodies and adds `proof fn` / `assert` / loop invariants as needed.
3. Each attempt is logged. Stop after the iteration cap and record what's blocking.
4. Measure: attempt count, wall clock, tokens, spec-weakening incidents (target: zero).

Results table and failure taxonomy land in `writeup/results.md`.

## Why

The blog series so far has tightened the feedback signal the loop runs against — tests, then mutation testing, then audit/decision separation, then integration contracts. Each step closes a hole through which a wrong loop could still pass. A formal verifier is the limit of that progression: an obligation the loop can't satisfy except by either weakening the spec or actually being correct.

This calibration is the cheapest test of whether that limit is reachable in practice.
