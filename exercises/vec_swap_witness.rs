// Witness for vec_swap.rs.
//
// Operator-authored reference implementation. Spec block byte-aligned
// to the exercise file. Discharges multiset-preservation under
// in-place swap.

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
    let tmp: u32 = v[i];
    let v_j: u32 = v[j];
    v.set(i, v_j);
    v.set(j, tmp);
    proof {
        broadcast use group_to_multiset_ensures, group_multiset_axioms,
            group_multiset_properties;
        let m = old(v)@.to_multiset();
        let s1 = old(v)@.update(i as int, old(v)@[j as int]);
        let s2 = s1.update(j as int, old(v)@[i as int]);
        assert(v@ =~= s2);

        if i == j {
            assert(s1 =~= old(v)@);
            assert(s2 =~= old(v)@);
        } else {
            // s1 = old(v)@.update(i, old[j]); s1.to_multiset() ==
            //     m.insert(old[j]).remove(old[i])   (to_multiset_update)
            // s1[j] == old[j] since update was at i != j (axiom_seq_update_different)
            assert(s1[j as int] == old(v)@[j as int]);
            // s2 = s1.update(j, old[i]); s2.to_multiset() ==
            //     s1.to_multiset().insert(old[i]).remove(s1[j])
            //   = m.insert(old[j]).remove(old[i]).insert(old[i]).remove(old[j])
            let a = old(v)@[i as int];
            let b = old(v)@[j as int];
            // Count-by-count argument over an arbitrary x.
            assert forall|x: u32|
                #[trigger] s2.to_multiset().count(x) == m.count(x)
            by {
                // count after .insert(b) is count_m(x) + (1 if x == b else 0)
                // count after .remove(a) is that minus (1 if x == a and count >= 1 else 0)
                // count after .insert(a) is that plus (1 if x == a else 0)
                // count after .remove(b) is that minus (1 if x == b and count >= 1 else 0)
                // The lemma_insert_increases_count_by_1 +
                // lemma_insert_other_elements_unchanged broadcasts
                // handle the case split.
                //
                // The "and count >= 1" wrinkle on remove is handled
                // because m.count(a) >= 1 (a == old[i] appears at
                // position i) and m.count(b) >= 1 (b == old[j] appears
                // at position j); the intermediate counts only ever go
                // up before going down.
                assert(old(v)@.contains(a)) by {
                    assert(old(v)@[i as int] == a);
                }
                assert(old(v)@.contains(b)) by {
                    assert(old(v)@[j as int] == b);
                }
                assert(m.count(a) > 0);
                assert(m.count(b) > 0);
            }
            assert(s2.to_multiset() =~= m);
        }
    }
}

} // verus!
