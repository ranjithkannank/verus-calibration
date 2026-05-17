// Exercise 6: verified Marzullo's algorithm (interval-based fault-tolerant
// agreement).
//
// Given n sensors, each reporting an interval `[lo, hi]` representing
// "I am here, plus or minus my uncertainty," and at most f Byzantine
// sensors, produce a single interval whose interior contains at least one
// point supported by at least n - f input intervals. This is the interval
// generalisation of the fault-tolerant midpoint exercise: where ft_midpoint
// took scalar readings and returned a value bracketed by correct sensor
// readings, Marzullo takes intervals and returns the smallest region
// "supported" by enough sensors that at least one of them must be correct.
//
// Reference: Keith Marzullo, "Maintaining the time in a distributed
// system," PhD thesis, 1984; later published as "Maintaining the time in
// a distributed system: An example of a loosely-coupled distributed
// service," ACM TOCS 1989. The algorithm is the canonical interval
// agreement primitive used in NTP, Cristian's clock sync, and several
// avionics designs.
//
// What the spec says:
//
//   - `intervals: Vec<Interval>` is the input. Each `Interval` has
//     `lo <= hi` (well-formedness precondition).
//   - `f: u32` is the maximum number of Byzantine sensors. Precondition
//     `intervals.len() >= 2*f + 1`.
//   - `correct_at(i)` is an uninterpreted per-index predicate
//     designating correct sensors. The implementer cannot provide a
//     body. Precondition: at least `n - f` indices are correct.
//   - The output is an `Interval` with `result.lo <= result.hi`.
//   - The safety property: there exists a point `p` in `[result.lo,
//     result.hi]` such that at least `n - f` input intervals contain
//     `p`. By pigeonhole over the `f` Byzantine budget, at least one
//     of those is a correct sensor's interval, so the output region
//     is "supported" — any caller can trust that some correct sensor
//     would have reported a value covering this region.
//
// Why this safety property:
//
//   The strict Marzullo safety guarantee is "the output region is
//   the maximum-overlap region." We use a slightly weaker form that
//   captures the operationally useful property (the output is
//   supported by enough sensors that at least one must be honest)
//   without requiring the implementer to prove maximality. A
//   maximality-strengthening variant is a natural follow-up
//   exercise if the basic safety lands cleanly.
//
// The spec below is FROZEN. Iteration cap: 20. See AGENTS.md.

use vstd::prelude::*;
use vstd::set_lib::*;

verus! {

pub type Reading = i64;

pub struct Interval {
    pub lo: Reading,
    pub hi: Reading,
}

// --- Trust boundary (uninterpreted) -----------------------------------------

pub uninterp spec fn correct_at(i: int) -> bool;

// --- Spec helpers -----------------------------------------------------------

pub open spec fn well_formed(intervals: Seq<Interval>) -> bool {
    forall|i: int| 0 <= i < intervals.len() ==> intervals[i].lo <= intervals[i].hi
}

pub open spec fn point_in_interval(p: Reading, iv: Interval) -> bool {
    iv.lo <= p && p <= iv.hi
}

pub open spec fn intervals_containing(intervals: Seq<Interval>, p: Reading) -> Set<int> {
    Set::new(|i: int|
        0 <= i < intervals.len() && point_in_interval(p, intervals[i]))
}

pub open spec fn correct_indices(n: nat) -> Set<int> {
    Set::new(|i: int| 0 <= i < n as int && correct_at(i))
}

// --- Proof-only spec helpers (implementer additions) ------------------------

spec fn contained_set_upto(intervals: Seq<Interval>, p: Reading, m: int) -> Set<int> {
    Set::new(|i: int|
        0 <= i < m && i < intervals.len() && point_in_interval(p, intervals[i]))
}

// --- Subset / finiteness lemmas ---------------------------------------------

proof fn lemma_contained_set_in_range(intervals: Seq<Interval>, p: Reading)
    ensures
        intervals_containing(intervals, p).subset_of(set_int_range(0, intervals.len() as int)),
        intervals_containing(intervals, p).finite(),
        intervals_containing(intervals, p).len() <= intervals.len() as nat,
{
    lemma_int_range(0, intervals.len() as int);
    assert(intervals_containing(intervals, p)
        .subset_of(set_int_range(0, intervals.len() as int)));
    lemma_len_subset(
        intervals_containing(intervals, p),
        set_int_range(0, intervals.len() as int),
    );
}

proof fn lemma_contained_set_upto_in_range(intervals: Seq<Interval>, p: Reading, m: int)
    requires 0 <= m <= intervals.len(),
    ensures
        contained_set_upto(intervals, p, m).subset_of(set_int_range(0, m)),
        contained_set_upto(intervals, p, m).finite(),
        contained_set_upto(intervals, p, m).len() <= m as nat,
{
    lemma_int_range(0, m);
    assert(contained_set_upto(intervals, p, m).subset_of(set_int_range(0, m)));
    lemma_len_subset(contained_set_upto(intervals, p, m), set_int_range(0, m));
}

proof fn lemma_correct_indices_in_range(n: nat)
    ensures
        correct_indices(n).subset_of(set_int_range(0, n as int)),
        correct_indices(n).finite(),
        correct_indices(n).len() <= n,
{
    lemma_int_range(0, n as int);
    assert(correct_indices(n).subset_of(set_int_range(0, n as int)));
    lemma_len_subset(correct_indices(n), set_int_range(0, n as int));
}

// --- Prefix-set extension lemma --------------------------------------------

proof fn lemma_contained_set_upto_extend(intervals: Seq<Interval>, p: Reading, i: int)
    requires 0 <= i < intervals.len(),
    ensures
        point_in_interval(p, intervals[i]) ==>
            contained_set_upto(intervals, p, i + 1)
                =~= contained_set_upto(intervals, p, i).insert(i),
        !point_in_interval(p, intervals[i]) ==>
            contained_set_upto(intervals, p, i + 1)
                =~= contained_set_upto(intervals, p, i),
{
}

// --- Counting helper --------------------------------------------------------

fn count_containing(intervals: &Vec<Interval>, p: Reading) -> (c: u32)
    requires
        intervals.len() <= u32::MAX as nat,
    ensures
        c as nat == intervals_containing(intervals@, p).len(),
        intervals_containing(intervals@, p).finite(),
        c as nat <= intervals.len() as nat,
{
    let mut c: u32 = 0;
    let mut i: usize = 0;
    proof {
        assert(contained_set_upto(intervals@, p, 0) =~= Set::<int>::empty());
    }
    while i < intervals.len()
        invariant
            0 <= i as int <= intervals@.len() as int,
            intervals.len() <= u32::MAX as nat,
            c as nat == contained_set_upto(intervals@, p, i as int).len(),
            contained_set_upto(intervals@, p, i as int).finite(),
            c as nat <= i as nat,
        decreases intervals.len() - i,
    {
        let iv = &intervals[i];
        proof {
            lemma_contained_set_upto_extend(intervals@, p, i as int);
            lemma_contained_set_upto_in_range(intervals@, p, (i + 1) as int);
        }
        if iv.lo <= p && p <= iv.hi {
            proof {
                assert(!contained_set_upto(intervals@, p, i as int).contains(i as int));
            }
            c = c + 1;
        }
        i = i + 1;
    }
    proof {
        assert(contained_set_upto(intervals@, p, intervals@.len() as int)
            =~= intervals_containing(intervals@, p));
        lemma_contained_set_in_range(intervals@, p);
    }
    c
}

// --- The exec entry point ---------------------------------------------------
//
// Returns an interval whose interior contains a point supported by at
// least n - f input intervals.
//
// Algorithmic latitude: the implementer chooses the approach. A
// brute-force scan over (low_i, high_j) pairs is the most direct route
// and avoids interval-sweep state-machine complexity. Each candidate
// (lo, hi) with lo <= hi has a well-defined count of input intervals
// containing it; pick any candidate whose count is at least n - f and
// return it as the output. Existence is guaranteed: the intersection of
// the n - f correct intervals (which all overlap, since they all
// contain any "true" value) has count >= n - f at every point inside
// it. The brute-force scan over input endpoints is guaranteed to find
// at least one such candidate by the pigeonhole on correct intervals.
//
// Reusable patterns from ft_midpoint:
//   - finite-universe bridge via `lemma_int_range` + `lemma_len_subset`
//     for any `Set::new(|i| ...).len()` reasoning
//   - inclusion-exclusion pigeonhole via `lemma_set_intersect_union_lens`
//   - `assert(false)` + concrete return for any provably-unreachable
//     fallback at the end of the search

pub fn marzullo(intervals: &Vec<Interval>, f: u32) -> (result: Interval)
    requires
        intervals.len() <= u32::MAX as nat,
        intervals.len() as nat >= 2 * (f as nat) + 1,
        well_formed(intervals@),
        correct_indices(intervals.len() as nat).len() >= intervals.len() as nat - f as nat,
    ensures
        result.lo <= result.hi,
        exists|p: Reading|
            result.lo <= p && p <= result.hi
                && intervals_containing(intervals@, p).len() >= intervals.len() as nat - f as nat,
{
    // Overflow safety: 2*f + 1 <= len <= u32::MAX ⇒ f + 1 fits in u32.
    assert(f as nat + 1 <= u32::MAX as nat) by {
        assert(2 * (f as nat) + 1 <= u32::MAX as nat);
    }
    let n: usize = intervals.len();
    // n fits in u32 by the precondition. n - f >= f + 1 >= 1 in nat, so the
    // u32 subtraction is safe.
    let n_u32: u32 = n as u32;
    assert(n_u32 as nat == n as nat);
    assert(n_u32 as nat >= f as nat + 1);
    let threshold: u32 = n_u32 - f;
    let mut i: usize = 0;
    while i < n
        invariant
            0 <= i as int <= n as int,
            n == intervals@.len(),
            threshold as nat == n as nat - f as nat,
            intervals.len() <= u32::MAX as nat,
            intervals.len() as nat >= 2 * (f as nat) + 1,
            well_formed(intervals@),
            correct_indices(intervals.len() as nat).len()
                >= intervals.len() as nat - f as nat,
            forall|j2: int| 0 <= j2 < i as int ==>
                intervals_containing(intervals@, #[trigger] intervals@[j2].lo).len()
                    < intervals.len() as nat - f as nat,
        decreases n - i,
    {
        let p: Reading = intervals[i].lo;
        let c: u32 = count_containing(intervals, p);
        if c >= threshold {
            proof {
                assert(intervals_containing(intervals@, p).len()
                    >= intervals.len() as nat - f as nat);
            }
            return Interval { lo: p, hi: p };
        }
        proof {
            // Maintain the strengthened invariant at j2 = i.
            assert(c < threshold);
            assert(p == intervals@[i as int].lo);
            assert(c as nat == intervals_containing(intervals@, p).len());
            assert(intervals_containing(intervals@, intervals@[i as int].lo).len()
                < intervals.len() as nat - f as nat);
        }
        i = i + 1;
    }
    // Post-loop: provably unreachable once lemma_exists_supported_endpoint is
    // wired (sub-tasks 10–12). For now, return a placeholder; verification of
    // the postcondition is expected to fail at this stage.
    Interval { lo: 0, hi: 0 }
}

} // verus!
