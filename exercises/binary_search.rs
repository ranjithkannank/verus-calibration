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
    let mut lo: usize = 0;
    let mut hi: usize = v.len();
    while lo < hi
        invariant
            is_sorted(v@),
            0 <= lo <= hi <= v@.len(),
            hi <= v.len(),
            forall|k: int| 0 <= k < lo as int ==> v@[k] != target,
            forall|k: int| hi as int <= k < v@.len() ==> v@[k] != target,
        decreases hi - lo,
    {
        let mid: usize = lo + (hi - lo) / 2;
        assert(lo <= mid && mid < hi);
        let x = v[mid];
        if x == target {
            return Some(mid);
        } else if x < target {
            // v@[mid] < target; by sortedness, all k <= mid have v@[k] <= v@[mid] < target
            assert(forall|k: int| 0 <= k <= mid as int ==> v@[k] <= v@[mid as int]) by {
                assert forall|k: int| 0 <= k <= mid as int implies v@[k] <= v@[mid as int] by {
                    assert(is_sorted(v@));
                }
            };
            assert(forall|k: int| 0 <= k < mid as int + 1 ==> v@[k] != target);
            lo = mid + 1;
        } else {
            // x > target; by sortedness, all k >= mid have v@[k] >= v@[mid] > target
            assert(forall|k: int| mid as int <= k < v@.len() ==> v@[k] >= v@[mid as int]) by {
                assert forall|k: int| mid as int <= k < v@.len() implies v@[k] >= v@[mid as int] by {
                    assert(is_sorted(v@));
                }
            };
            assert(forall|k: int| mid as int <= k < v@.len() ==> v@[k] != target);
            hi = mid;
        }
    }
    None
}

} // verus!
