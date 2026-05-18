## Attempt 1 — 2026-05-18

**Sub-task:** All three sub-tasks in one pass (counter.rs exec bodies, filler.rs `fill_to` loop, main.rs `pipeline` composition). The methodology question this exercise tests is whether the right invariant for `fill_to` can be derived from the contract without copying `counter_producer`'s loop invariant verbatim.

**Approach:**
- `counter.rs`: identical exec bodies to `counter_producer/counter.rs` (`Counter { value: 0, bound }`, `self.value = self.value + 1`, `self.value`).
- `filler.rs`: target-bounded loop `while c.get() < target { c.incr(); }` with four-conjunct invariant `c.invariant() && c.value() <= target && c.bound() == old(c).bound() && target <= c.bound()` and `decreases target - c.value()`. No separate counter variable — `c.value()` itself is the loop progress measure. This is structurally different from `counter_producer`'s `produce(c, n)` invariant: there's no `start` snapshot, no `i: u32` counter, no `start + i` composed claim. The invariant is anchored entirely to `target` and the persisted bound.
- `main.rs`: three-call pipeline `Counter::new(target); fill_to(&mut c, target); c.get()`. The `fill_to` precondition `0 <= target && target <= target` is immediate from `new`'s ensures.

**Verifier output:** `verification results:: 6 verified, 0 errors`.

**Next idea:** done. Hand to reviewer.
