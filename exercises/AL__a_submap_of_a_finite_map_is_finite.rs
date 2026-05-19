use vstd::prelude::*;

fn main() {}
verus! {

pub proof fn a_submap_of_a_finite_map_is_finite<K, V>(m1: Map<K, V>, m2: Map<K, V>)
    requires
        m1.submap_of(m2),
        m2.dom().finite(),
    ensures
        m1.dom().finite(),
{
    assert(m1.dom().subset_of(m2.dom()));
    vstd::set_lib::lemma_len_subset(m1.dom(), m2.dom());
}

}
