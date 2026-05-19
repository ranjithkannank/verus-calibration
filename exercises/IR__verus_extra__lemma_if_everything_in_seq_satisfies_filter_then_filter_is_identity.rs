use vstd::prelude::*;
fn main() {}
verus! {

pub proof fn lemma_if_everything_in_seq_satisfies_filter_then_filter_is_identity<A>(
    s: Seq<A>,
    pred: spec_fn(A) -> bool,
)
    requires
        forall|i: int| 0 <= i && i < s.len() ==> pred(s[i]),
    ensures
        s.filter(pred) == s,
    decreases s.len(),
{
    reveal(Seq::filter);
    if s.len() == 0 {
        assert(s.filter(pred) =~= s);
    } else {
        let s2 = s.drop_last();
        assert forall|i: int| 0 <= i && i < s2.len() implies pred(s2[i]) by {
            assert(s2[i] == s[i]);
        }
        lemma_if_everything_in_seq_satisfies_filter_then_filter_is_identity(s2, pred);
        assert(pred(s.last())) by {
            assert(s.last() == s[s.len() - 1]);
        }
        assert(s.filter(pred) =~= s2.push(s.last()));
        assert(s2.push(s.last()) =~= s);
    }
}

} // verus!
