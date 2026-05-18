// Filler module for the counter_filler exercise.
//
// `fill_to(c, target)` bulk-increments a Counter until its value
// equals `target`. The implementer fills in the function body. The
// signature, requires, and ensures clauses are FROZEN.

use vstd::prelude::*;
use crate::counter::Counter;

verus! {

pub fn fill_to(c: &mut Counter, target: u32)
    requires
        old(c).invariant(),
        old(c).value() <= target,
        target <= old(c).bound(),
    ensures
        final(c).invariant(),
        final(c).value() == target,
        final(c).bound() == old(c).bound(),
{
    unimplemented!()
}

} // verus!
