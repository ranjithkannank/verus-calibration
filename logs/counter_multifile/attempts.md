## Attempt 1 — 2026-05-17

**Sub-task:** All three sub-tasks (1: counter.rs exec bodies, 2: main.rs client body, 3: end-to-end verify) in one attempt. Direct port from the verified `cross_module_counter.rs`, which the design explicitly identifies as the algorithmic parent.

**Approach:** Filled in all four exec bodies:
- `counter::Counter::new` returns `Counter { value: 0, bound: bound }`.
- `counter::Counter::incr` does `self.value = self.value + 1` (no overflow: precondition gives `value < bound <= u32::MAX`).
- `counter::Counter::get` returns `self.value`.
- `main::count_up_to` creates `counter::Counter::new(target)`, runs a `while i < target` loop with the four-conjunct invariant (`c.invariant()`, `c.value() == i`, `c.bound() == target`, `i <= target`) and `decreases target - i`, returning `c.get()`. References the type via the path `counter::Counter` (no `use`).

**Verifier output:**
```
verification results:: 5 verified, 0 errors
```

**Next idea:** Done — verus exited 0. Hand to reviewer.
