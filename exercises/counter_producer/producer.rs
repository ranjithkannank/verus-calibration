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
    unimplemented!()
}

} // verus!
