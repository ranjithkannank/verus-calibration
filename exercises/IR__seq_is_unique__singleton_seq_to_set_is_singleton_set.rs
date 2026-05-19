use vstd::prelude::*;
fn main() {}
verus! {

pub proof fn singleton_seq_to_set_is_singleton_set<T>(x: T)
    ensures
        seq![x].to_set() == set![x],
{
    // `seq![x]` desugars to `Seq::empty().push(x)`; `set![x]` desugars to
    // `Set::empty().insert(x)`. Invoke `lemma_push_to_set_commute` to bridge
    // the push-to-set side, then close empty-set extensional equality.
    Seq::<T>::empty().lemma_push_to_set_commute(x);
    assert(Seq::<T>::empty().to_set() =~= Set::<T>::empty());
    assert(seq![x].to_set() =~= set![x]);
}

} // verus!
