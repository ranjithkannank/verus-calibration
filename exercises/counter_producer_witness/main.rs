// Witness for counter_producer/main.rs.
// Operator-authored reference implementation.

use vstd::prelude::*;

mod counter;
mod producer;

verus! {

pub fn pipeline(target: u32) -> (r: u32)
    ensures
        r == target,
{
    let mut c = counter::Counter::new(target);
    producer::produce(&mut c, target);
    c.get()
}

} // verus!
