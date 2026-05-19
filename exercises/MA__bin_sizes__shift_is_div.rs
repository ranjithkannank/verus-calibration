use vstd::prelude::*;


fn main() {}

verus! {

pub open spec fn pow2(i: int) -> nat
    decreases i
{
    if i <= 0 {
        1
    } else {
        pow2(i - 1) * 2
    }
}

proof fn lemma_pow2_eq_vstd(n: nat)
    ensures pow2(n as int) == vstd::arithmetic::power2::pow2(n),
    decreases n,
{
    if n == 0 {
        vstd::arithmetic::power2::lemma2_to64();
        assert(pow2(0int) == 1nat);
    } else {
        let m: nat = (n - 1) as nat;
        lemma_pow2_eq_vstd(m);
        vstd::arithmetic::power2::lemma_pow2_unfold(n);
        assert(n as int > 0);
        assert(pow2(n as int) == pow2(n as int - 1) * 2);
        assert(n as int - 1 == m as int);
    }
}

proof fn shift_is_div(x:u64, shift:u64)
    requires 0 <= shift < 64,
    ensures x >> shift == x as nat / pow2(shift as int),
{
    vstd::bits::lemma_u64_shr_is_div(x, shift);
    lemma_pow2_eq_vstd(shift as nat);
    assert((shift as nat) as int == shift as int);
}

}