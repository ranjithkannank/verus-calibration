# Review: swap_multiset

**Conclusion:** APPROVE

## Checklist

1. Spec clauses unchanged: YES — the only diff hunk in `exercises/swap_multiset.rs` is inside the function body (lines 37–72 in HEAD, replacing the single `unimplemented!()` at line 37 of the frozen version). The `requires` block (lines 25–27) and `ensures` block (lines 28–35) are untouched by the diff.
2. Pre-existing spec fn bodies unchanged: YES — the frozen baseline contains no `spec fn` declarations at all (only the `swap` exec function), so there is nothing to alter.
3. No bypass tokens introduced: YES — searched the diff for `assume(`, `#[verifier::external_body]`, `unimplemented!()`, `unreachable!()`, `panic!(`. The diff *removes* `unimplemented!()` (frozen line 37) and replaces it with real exec code plus a `proof { ... }` block containing only `let` bindings, `broadcast use group_to_multiset_ensures;`, and four `assert(...)` statements (HEAD lines 54, 58, 71). `assert` and `broadcast use` are permitted.
4. No trivializing requires: YES — the diff adds no `requires` clauses anywhere; the only `requires` block (lines 25–27) is byte-identical to the frozen version.
5. No closed/open toggles: YES — there are no `spec fn` declarations (closed or open) in either version.

## Justification

I diffed `spec-frozen-swap_multiset..HEAD -- exercises/swap_multiset.rs`; the single hunk replaces the `unimplemented!()` body with four exec lines (read, read, write, write) and a `proof { ... }` block. The five `requires`/`ensures` lines (frozen 25–35, HEAD 25–35) are outside the hunk and unchanged. The proof block uses only legitimate constructs — `broadcast use` of a vstd-provided ensures group, ghost `let`s, and extensional-equality `assert`s — with no `assume`, `external_body`, or panicking stub. The implementer added no spec fns and no new requires clauses, so checks 2, 4, and 5 are vacuously satisfied.

## Reviewer notes (optional)

- New proof family worth recording in the playbook: `to_multiset()` preservation via `s.update(i, a).update(j, b)` decomposition + `group_to_multiset_ensures` broadcast + extensional `=~=` closure. The current AGENTS.md "Discovered patterns" section has no `to_multiset` entry; consider asking the implementer to append one.
- The `broadcast use group_to_multiset_ensures;` form is a clean alternative to inlining `lemma_*` calls; future reviewers should not flag `broadcast use` as a cheat token (it does not appear on the rejection list).
