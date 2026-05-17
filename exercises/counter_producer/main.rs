// Entry point for the counter_producer exercise.
//
// Declares the two sibling modules (`counter` and `producer`) and
// defines `pipeline(target)`, which composes them: create a fresh
// counter of bound `target`, bulk-increment by `target` via the
// producer, return the final reading.
//
// The implementer fills in `pipeline`'s body. The function signature,
// ensures clause, and `mod` declarations are FROZEN.

use vstd::prelude::*;

mod counter;
mod producer;

verus! {

pub fn pipeline(target: u32) -> (r: u32)
    ensures
        r == target,
{
    unimplemented!()
}

} // verus!
