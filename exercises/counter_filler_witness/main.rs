// Witness for counter_filler/main.rs.
//
// `pipeline(target)` creates a fresh counter of bound `target`, fills
// it to `target` via the filler module, and returns the final reading.

use vstd::prelude::*;

mod counter;
mod filler;

verus! {

pub fn pipeline(target: u32) -> (r: u32)
    ensures
        r == target,
{
    let mut c = counter::Counter::new(target);
    filler::fill_to(&mut c, target);
    c.get()
}

} // verus!
