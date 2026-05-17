// Producer module for the counter_producer exercise.
//
// `produce(c, n)` advances a bounded counter by `n` increments. The
// implementer fills in the function body. The signature, requires,
// and ensures clauses are FROZEN.

use vstd::prelude::*;
use crate::counter::Counter;

verus! {

pub fn produce(c: &mut Counter, n: u32)
    requires
        old(c).invariant(),
        old(c).value() + n <= old(c).bound(),
    ensures
        final(c).invariant(),
        final(c).value() == old(c).value() + n,
        final(c).bound() == old(c).bound(),
{
    let start: u32 = c.get();
    let mut i: u32 = 0;
    while i < n
        invariant
            c.invariant(),
            c.value() == start + i,
            c.bound() == old(c).bound(),
            i <= n,
            start == old(c).value(),
            start + n <= c.bound(),
        decreases n - i,
    {
        c.incr();
        i = i + 1;
    }
}

} // verus!
