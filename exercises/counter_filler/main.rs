// Entry point for the counter_filler exercise.
//
// Declares the two sibling modules (`counter` and `filler`) and
// defines `pipeline(target)`, which composes them: create a fresh
// counter of bound `target`, fill it to `target` via the filler,
// return the final reading.
//
// The implementer fills in `pipeline`'s body. The function signature,
// ensures clause, and `mod` declarations are FROZEN.

use vstd::prelude::*;

mod counter;
mod filler;

verus! {

pub fn pipeline(target: u32) -> (r: u32)
    ensures
        r == target,
{
    unimplemented!()
}

} // verus!
