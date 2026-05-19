## Attempt 1 — 2026-05-18

**Sub-task:** Sub-task 2 — smallest possible body. Tried extensional set-equality `=~=` between `seq![x].to_set()` and `set![x]`.

**Approach:** Added a single `assert(seq![x].to_set() =~= set![x]);` to the body of `singleton_seq_to_set_is_singleton_set`. Rationale: `=~=` should trigger ext_equal element-wise membership reasoning, and for both singletons the SMT solver ought to see `y == x` from the RHS and `seq![x][0] == x` from the LHS.

**Verifier output:**
```
error: assertion failed
 --> .../IR__seq_is_unique__singleton_seq_to_set_is_singleton_set.rs:9:12
  |
9 |     assert(seq![x].to_set() =~= set![x]);
  |            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ assertion failed

verification results:: 0 verified, 1 errors
```

**Next idea:** The `=~=` macro alone doesn't trigger the `to_set` axiom needed to expand `seq![x].to_set().contains(y)`. Try invoking the vstd lemma family for `Seq::to_set` — likely `lemma_seq_to_set_contains` or `seq_to_set_equal_iff` — or fall back to `assert_sets_equal!` macro from vstd which often unfolds membership both ways. Also could try the explicit element-membership pattern:
```
assert(seq![x].to_set().contains(x));
assert(forall|y: T| seq![x].to_set().contains(y) ==> y == x);
```
before the `=~=` to give the solver the bridge.
