## Attempt 1 — 2026-05-17

**Sub-task:** All sub-tasks 1–6 (full implementation in one attempt). Exercise is simple enough that the design's algorithmic sketch maps directly to working code without proof hints.

**Approach:** Filled in all four exec bodies following the design:
- `Counter::new` returns `Counter { value: 0, bound: bound }`.
- `Counter::incr` does `self.value = self.value + 1` (no overflow: `value < bound <= u32::MAX`).
- `Counter::get` returns `self.value`.
- `client::count_up_to` creates `Counter::new(target)`, runs a `while i < target` loop with the four-conjunct invariant from the design (`c.invariant()`, `c.value() == i`, `c.bound() == target`, `i <= target`) and `decreases target - i`, then returns `c.get()`.

**Verifier output:**
```
verification results:: 5 verified, 0 errors
```

**Next idea:** Done — verus exited 0. Hand to reviewer.
