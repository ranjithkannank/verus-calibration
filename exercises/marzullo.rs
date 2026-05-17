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
    // TODO(loop): fill in. Do not modify any spec above.
    unimplemented!()
}

} // verus!
