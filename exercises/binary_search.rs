// Exercise 1: verified binary search on a sorted Vec<i64>.
//
// The spec below is FROZEN. The loop's job is to fill `binary_search`
// such that `verus exercises/binary_search.rs --crate-type=lib` exits 0.
//
// Iteration cap: 10. See AGENTS.md.

use vstd::prelude::*;

verus! {

pub open spec fn is_sorted(s: Seq<i64>) -> bool {
    forall|i: int, j: int| 0 <= i <= j < s.len() ==> s[i] <= s[j]
}

pub fn binary_search(v: &Vec<i64>, target: i64) -> (result: Option<usize>)
    requires
        is_sorted(v@),
    ensures
        result.is_some() ==> {
            let i = result.unwrap();
            &&& (i as int) < v@.len()
            &&& v@[i as int] == target
        },
        result.is_none() ==> forall|i: int| 0 <= i < v@.len() ==> v@[i] != target,
{
    // TODO(loop): fill in the body. Do not modify any spec above.
    unimplemented!()
}

} // verus!
