// Witness for counter_producer/producer.rs.
// Operator-authored reference implementation.

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
    let start = c.get();
    let mut i: u32 = 0;
    while i < n
        invariant
            c.invariant(),
            c.value() == start + i,
            c.bound() == old(c).bound(),
            i <= n,
            start + n <= c.bound(),
            start == old(c).value(),
        decreases n - i,
    {
        c.incr();
        i = i + 1;
    }
}

} // verus!
