// Entry point for the counter_multifile exercise.
//
// Declares `mod counter;` referencing the sibling counter.rs file.
// The function `count_up_to(target)` creates a fresh counter of bound
// `target` and increments it to `target`, returning the final value
// via `Counter::get`. The proof must rely only on `Counter`'s
// `closed spec fn` API (`value`, `bound`, `invariant`) and the
// function postconditions — the underlying private fields are not
// visible across the module boundary.
//
// The spec below is FROZEN as part of `spec-frozen-counter_multifile`.
// The implementer fills in the `count_up_to` body; the function
// signature, requires/ensures, and the `mod counter;` declaration
// must not be touched.

use vstd::prelude::*;

mod counter;

verus! {

pub fn count_up_to(target: u32) -> (final_count: u32)
    ensures
        final_count == target,
{
    let mut c = counter::Counter::new(target);
    let mut i: u32 = 0;
    while i < target
        invariant
            c.invariant(),
            c.value() == i,
            c.bound() == target,
            i <= target,
        decreases target - i,
    {
        c.incr();
        i = i + 1;
    }
    c.get()
}

} // verus!
