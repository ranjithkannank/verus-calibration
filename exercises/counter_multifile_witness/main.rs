// Witness for counter_multifile/main.rs.
//
// Operator-authored reference implementation. Function signature +
// requires/ensures byte-identical to exercises/counter_multifile/main.rs.

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
