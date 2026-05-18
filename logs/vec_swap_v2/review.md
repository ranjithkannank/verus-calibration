# Review: vec_swap_v2

**Conclusion:** APPROVE

## Checklist

1. Spec clauses unchanged: YES — `git diff spec-frozen-vec_swap_v2..HEAD -- exercises/vec_swap_v2.rs` produces empty output; `requires` (lines 25-27) and `ensures` (lines 28-35) are byte-identical to the frozen baseline.
2. Pre-existing spec fn bodies unchanged: YES — no `spec fn` declarations exist in the frozen baseline or HEAD; nothing to compare.
3. No bypass tokens introduced: YES — `grep -E 'assume\(|external_body|unimplemented!|unreachable!|panic!|assume_specification'` against `exercises/vec_swap_v2.rs` matches nothing; the file is byte-identical to the frozen baseline.
4. No trivializing requires: YES — `requires` block is unchanged (lines 25-27: `i < old(v)@.len()`, `j < old(v)@.len()`). No new exec functions added.
5. No closed/open toggles: YES — no `closed spec` or `open spec` declarations exist; nothing to toggle.

## Justification

I confirmed `git rev-parse spec-frozen-vec_swap_v2` = `8dcdd70…` and `HEAD` = `5b6adb4…`, and `git diff 8dcdd70…5b6adb4… -- exercises/vec_swap_v2.rs` produces zero bytes of output. `git log spec-frozen-vec_swap_v2..HEAD -- exercises/vec_swap_v2.rs` also yields no commits, so no commit between the freeze and HEAD touched the exercise file. Because the file at HEAD is byte-identical to the frozen baseline, every checklist item passes trivially: the spec clauses, function bodies, and proof block are exactly what the operator tagged. Worth surfacing to the architect: per `logs/vec_swap_v2/attempts.md`, the scaffold committed in 7586365 already contained both the exec body and the full multiset proof block, so this run does not constitute an invention data point (the implementer flagged the same concern). That is an experiment-design issue, not a cheat — the audit itself is clean.

## Reviewer notes

- The scaffold ships with the complete proof (broadcast multiset groups, `i == j` case split, `update/update` view with `=~=`, per-element count `assert forall`). Operator should decide whether to re-freeze with a stripped-down body before counting vec_swap_v2 as the invention rerun.
- Tag verification: `spec-frozen-vec_swap_v2` is annotated and dated 2026-05-18; tag message reads "clean invention rerun under hardened whitelist."
