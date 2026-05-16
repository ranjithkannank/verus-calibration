# Review: quorum_count

**Conclusion:** APPROVE

## Checklist

1. Spec clauses unchanged: YES — The diff hunk at `exercises/quorum_count.rs` lines 149-154 shows `is_byzantine_quorum`'s `requires` (`n > 0,`, `all_in_range(voters@, n),`) and `ensures` (`result == (distinct_count(voters@) >= byzantine_threshold(n)),`) appear only as unchanged context lines. No `requires` or `ensures` line is marked `+` or `-` in the diff. The new `requires`/`ensures` clauses appearing in the diff (e.g. lines 33-34, 49, 100, 107-108, 115-116, 126-128) are all on freshly-added `proof fn` helpers, not on pre-existing functions.
2. Pre-existing spec fn bodies unchanged: YES — The three frozen spec fns `all_in_range` (lines 19-21), `distinct_count` (lines 23-25), and `byzantine_threshold` (lines 27-29) appear in the diff only as unchanged context. The diff's first `+` hunk begins at line 31 (the new `lemma_prefix_extend` helper), strictly after the last pre-existing spec fn ends.
3. No bypass tokens introduced: YES — `grep` for `assume(|external_body|unimplemented!|unreachable!|panic!|assume_specification` over `exercises/quorum_count.rs` returns no matches. The baseline's `unimplemented!()` (frozen file line 44) was removed and replaced with a real implementation.
4. No trivializing requires: YES — All new `requires` clauses (diff lines 33, 107, 115) are on `proof fn` helper lemmas, not `exec` functions. They are standard preconditions (e.g. `0 <= i < s.len()`, `s.finite() && !s.contains(x)`) that genuinely scope the lemma rather than make it vacuous. No `requires false` and no new `requires` on `is_byzantine_quorum`.
5. No closed/open toggles: YES — Grep shows the three pre-existing declarations remain `pub open spec fn` (lines 19, 23, 27). No `closed` keyword appears in the file or the diff.

## Justification

I diffed `spec-frozen-quorum_count..HEAD` on `exercises/quorum_count.rs`, then cross-checked by reading the frozen blob and the current file in full. The diff adds (a) six new `proof fn` helper lemmas at lines 31-147 and (b) a real loop-based implementation replacing the single `unimplemented!()` line at body of `is_byzantine_quorum`. The three frozen `pub open spec fn` declarations and `is_byzantine_quorum`'s `requires`/`ensures` appear only as unchanged context. A targeted grep confirms no `assume(`, `external_body`, `unimplemented!`, `unreachable!`, `panic!`, or `assume_specification` is present in HEAD. The new helpers (`lemma_prefix_extend`, `lemma_push_to_set`, `lemma_to_set_finite`, `lemma_set_insert_new_len`, `lemma_set_insert_existing`, `lemma_range_nodeid_len`) have reasonable preconditions and ensure-clauses that look like genuine library-level facts, not trivializations.

## Reviewer notes

- The `lemma_to_set_finite` helper has an empty body relying solely on the vstd axiom firing — clean, but worth noting it's effectively a re-exposure of `axiom_seq_to_set_finite`. Not a cheat, just a pattern to flag.
- The implementer leans heavily on `=~=` extensional equality and `choose` witnesses to push set/seq reasoning through the SMT solver; this pattern recurred in `bounded_log` too and is worth canonizing in the architect's playbook.
- `lemma_range_nodeid_len` uses structural recursion on `u32` with a `decreases n` clause — a clean way to enumerate finite ranges and bound `to_set().len()` by `n`.
