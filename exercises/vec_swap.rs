// Exercise: verified in-place swap on a Vec<u32>.
//
// `swap(v, i, j)` exchanges the elements at positions i and j. The
// proof obligation has two halves:
//
//   1. Positional: the values at positions i, j are exchanged, all
//      other positions unchanged.
//   2. Multiset-preservation: `final(v)@.to_multiset() ==
//      old(v)@.to_multiset()`.
//
// The second clause is the substance of this exercise. The
// playbook accumulated in AGENTS.md does not currently document
// any proof family touching `to_multiset()`. The agent must find
// the right vstd vocabulary and assemble the argument.
//
// The spec below is FROZEN. Iteration cap: 25. See AGENTS.md.

use vstd::prelude::*;
use vstd::seq_lib::*;
use vstd::multiset::*;

verus! {

pub fn swap(v: &mut Vec<u32>, i: usize, j: usize)
    requires
        i < old(v)@.len(),
        j < old(v)@.len(),
    ensures
        final(v)@.len() == old(v)@.len(),
        final(v)@[i as int] == old(v)@[j as int],
        final(v)@[j as int] == old(v)@[i as int],
        forall|k: int|
            0 <= k < final(v)@.len() && k != i as int && k != j as int ==>
                final(v)@[k] == old(v)@[k],
        final(v)@.to_multiset() == old(v)@.to_multiset(),
{
    unimplemented!()
}

} // verus!
