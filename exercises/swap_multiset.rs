// Exercise: in-place swap on a Vec<u32> with multiset preservation.
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
    let vi: u32 = v[i];
    let vj: u32 = v[j];
    v[i] = vj;
    v[j] = vi;

    proof {
        broadcast use group_to_multiset_ensures;

        let s0 = old(v)@;
        let i_ = i as int;
        let j_ = j as int;
        let a = s0[j_];  // value being placed at position i
        let b = s0[i_];  // value being placed at position j

        // After the two assignments, the view satisfies:
        let s1 = s0.update(i_, a);
        // v@ == s1.update(j_, b)
        assert(v@ == s1.update(j_, b));

        // s1[j_] == s0[j_] == a (update at i_ doesn't change j_ — even if i_ == j_,
        // we updated to s0[j_] = a, so s1[j_] = a = s0[j_]).
        assert(s1[j_] == s0[j_]);

        // From the broadcast group `to_multiset_update`:
        //   s1.to_multiset() == s0.to_multiset().insert(a).remove(s0[i_])
        //                    == s0.to_multiset().insert(a).remove(b)
        //   v@.to_multiset() == s1.to_multiset().insert(b).remove(s1[j_])
        //                    == s1.to_multiset().insert(b).remove(a)
        // Composing:
        //   v@.to_multiset() == s0.to_multiset().insert(a).remove(b).insert(b).remove(a)
        //
        // The multiset is invariant under this composition because
        // s0.to_multiset().count(b) >= 1 (since b = s0[i_] and i_ is a valid index).
        // Close via extensional equality.
        assert(v@.to_multiset() =~= s0.to_multiset());
    }
}

} // verus!
