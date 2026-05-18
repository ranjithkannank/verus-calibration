// Witness for counter_filler/filler.rs.
//
// `fill_to(c, target)` bulk-increments a Counter until its value
// equals `target`. Different loop shape from counter_producer's
// `produce(c, n)`: the loop is target-bounded (`while c.get() < target`)
// rather than counter-bounded (`while i < n`), and the decreases
// clause uses the gap to the target rather than a separate counter
// variable.

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
    while c.get() < target
        invariant
            c.invariant(),
            c.value() <= target,
            target <= c.bound(),
            c.bound() == old(c).bound(),
        decreases (target - c.value()) as int,
    {
        c.incr();
    }
}

} // verus!
