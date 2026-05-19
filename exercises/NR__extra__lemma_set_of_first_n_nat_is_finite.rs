use vstd::prelude::*;

fn main() {}

verus!{

// File: extra.rs
pub proof fn lemma_set_of_first_n_nat_is_finite( n: nat, )
    ensures Set::new(|i: nat| i < n).finite()
    decreases n
{
    if n == 0 {
        assert(Set::new(|i: nat| i < n) =~= Set::<nat>::empty());
    } else {
        let m: nat = (n - 1) as nat;
        lemma_set_of_first_n_nat_is_finite(m);
        assert(Set::new(|i: nat| i < n) =~= Set::new(|i: nat| i < m).insert(m));
    }
}


}
