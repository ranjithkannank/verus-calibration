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
// NOTE (operator intervention 2026-05-16): the original frozen spec
// omitted the Helly-1D precondition `correct_intervals_overlap`. The
// implementer proved (with a constructive counterexample at attempt
// 5-7) that the postcondition was logically unprovable without it:
// three "correct" sensors are allowed to report disjoint singleton
// intervals like [[0,0], [10,10], [20,20]], for which no point lies
// in >= n-f input intervals, so the existential postcondition cannot
// be satisfied. The methodology held — the architect re-confirmed the
// diagnosis through three revisions and the implementer wrote a
// detailed blocker report rather than weakening the spec. The
// operator re-froze the spec to add the missing precondition. The
// prior tag's `logs/marzullo/blocked.md` (preserved in git history)
// has the full constructive counterexample. The
// `spec-frozen-marzullo` tag has been force-moved to this commit.
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
//   - `correct_intervals_overlap(intervals@)`: all correct sensors'
//     intervals share at least one common point (the Helly-1D
//     condition). This is the canonical sensor-fusion assumption
//     that all honest sensors are reporting bounds around some shared
//     true value.
//   - The output is an `Interval` with `result.lo <= result.hi`.
//   - The safety property: there exists a point `p` in `[result.lo,
//     result.hi]` such that at least `n - f` input intervals contain
//     `p`. By pigeonhole over the `f` Byzantine budget, at least one
//     of those is a correct sensor's interval, so the output region
//     is "supported" — any caller can trust that some correct sensor
//     would have reported a value covering this region.
//
// The spec below is FROZEN. Iteration cap: 20. See AGENTS.md.

use vstd::prelude::*;
use vstd::set_lib::*;

verus! {

pub type Reading = i64;

#[derive(Copy, Clone)]
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

// Helly-1D condition: every pair of correct intervals overlaps. Equivalent
// (in one dimension) to "all correct intervals share at least one common
// point." This is the missing precondition the original spec elided; the
// algorithm's safety guarantee depends on it.
pub open spec fn correct_intervals_overlap(intervals: Seq<Interval>) -> bool {
    forall|i: int, j: int|
        0 <= i < intervals.len() && 0 <= j < intervals.len()
        && correct_at(i) && correct_at(j)
            ==> intervals[i].lo <= intervals[j].hi
}

// --- Proof-only spec helpers ------------------------------------------------

spec fn containing_upto(intervals: Seq<Interval>, p: Reading, m: int) -> Set<int> {
    Set::new(|i: int| 0 <= i < m && i < intervals.len() && point_in_interval(p, intervals[i]))
}

// --- Subset / finiteness lemmas ---------------------------------------------

proof fn lemma_containing_in_range(intervals: Seq<Interval>, p: Reading)
    ensures
        intervals_containing(intervals, p).subset_of(set_int_range(0, intervals.len() as int)),
        intervals_containing(intervals, p).finite(),
        intervals_containing(intervals, p).len() <= intervals.len() as nat,
{
    lemma_int_range(0, intervals.len() as int);
    assert(intervals_containing(intervals, p).subset_of(set_int_range(0, intervals.len() as int)));
    lemma_len_subset(intervals_containing(intervals, p), set_int_range(0, intervals.len() as int));
}

proof fn lemma_containing_upto_in_range(intervals: Seq<Interval>, p: Reading, m: int)
    requires 0 <= m <= intervals.len(),
    ensures
        containing_upto(intervals, p, m).subset_of(set_int_range(0, m)),
        containing_upto(intervals, p, m).finite(),
        containing_upto(intervals, p, m).len() <= m as nat,
{
    lemma_int_range(0, m);
    assert(containing_upto(intervals, p, m).subset_of(set_int_range(0, m)));
    lemma_len_subset(containing_upto(intervals, p, m), set_int_range(0, m));
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

proof fn lemma_containing_upto_extend(intervals: Seq<Interval>, p: Reading, i: int)
    requires 0 <= i < intervals.len(),
    ensures
        point_in_interval(p, intervals[i]) ==>
            containing_upto(intervals, p, i + 1)
                =~= containing_upto(intervals, p, i).insert(i),
        !point_in_interval(p, intervals[i]) ==>
            containing_upto(intervals, p, i + 1)
                =~= containing_upto(intervals, p, i),
{
}

// --- Counting helper -------------------------------------------------------

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
        assert(containing_upto(intervals@, p, 0) =~= Set::<int>::empty());
    }
    while i < intervals.len()
        invariant
            0 <= i as int <= intervals@.len() as int,
            intervals.len() <= u32::MAX as nat,
            c as nat == containing_upto(intervals@, p, i as int).len(),
            containing_upto(intervals@, p, i as int).finite(),
            c as nat <= i as nat,
        decreases intervals.len() - i,
    {
        let iv_lo = intervals[i].lo;
        let iv_hi = intervals[i].hi;
        proof {
            lemma_containing_upto_extend(intervals@, p, i as int);
            lemma_containing_upto_in_range(intervals@, p, (i + 1) as int);
        }
        if iv_lo <= p && p <= iv_hi {
            proof {
                assert(!containing_upto(intervals@, p, i as int).contains(i as int));
            }
            c = c + 1;
        }
        i = i + 1;
    }
    proof {
        assert(containing_upto(intervals@, p, intervals@.len() as int)
            =~= intervals_containing(intervals@, p));
        lemma_containing_in_range(intervals@, p);
    }
    c
}

// --- Argmax-`lo` over a finite set of interval indices ---------------------

proof fn lemma_max_lo_in_set(s: Set<int>, intervals: Seq<Interval>) -> (jm: int)
    requires
        s.finite(),
        s.len() >= 1,
        forall|j: int| s.contains(j) ==> 0 <= j < intervals.len(),
    ensures
        s.contains(jm),
        forall|j: int| s.contains(j) ==> intervals[j].lo <= intervals[jm].lo,
    decreases s.len(),
{
    axiom_is_empty_len0(s);
    axiom_is_empty(s);
    let j0 = choose|x: int| s.contains(x);
    assert(s.contains(j0));
    let s2 = s.remove(j0);
    assert(s2.finite());
    assert(s2.len() == s.len() - 1);
    if s2.len() == 0 {
        assert forall|j: int| s.contains(j) implies intervals[j].lo <= intervals[j0].lo by {
            if j != j0 {
                assert(s2.contains(j));
                axiom_is_empty_len0(s2);
                axiom_is_empty(s2);
            }
        }
        j0
    } else {
        let jm2 = lemma_max_lo_in_set(s2, intervals);
        if intervals[j0].lo >= intervals[jm2].lo {
            assert forall|j: int| s.contains(j) implies intervals[j].lo <= intervals[j0].lo by {
                if j != j0 {
                    assert(s2.contains(j));
                }
            }
            j0
        } else {
            assert forall|j: int| s.contains(j) implies intervals[j].lo <= intervals[jm2].lo by {
                if j != j0 {
                    assert(s2.contains(j));
                }
            }
            jm2
        }
    }
}

// --- Existence lemma: some intervals[j].lo is supported by >= n - f --------

proof fn lemma_exists_supported_lo(intervals: Seq<Interval>, f: nat)
    requires
        intervals.len() >= 2 * f + 1,
        well_formed(intervals),
        correct_indices(intervals.len()).len() >= intervals.len() - f,
        correct_intervals_overlap(intervals),
    ensures
        exists|j: int|
            0 <= j < intervals.len()
            && intervals_containing(intervals, intervals[j].lo).len()
                >= intervals.len() - f,
{
    let n: int = intervals.len() as int;
    let s: Set<int> = correct_indices(intervals.len());

    lemma_correct_indices_in_range(intervals.len());
    // s.len() >= n - f >= f + 1 >= 1.
    assert(s.len() >= intervals.len() - f);
    assert(intervals.len() >= 2 * f + 1);
    assert(s.len() >= 1);

    assert forall|j: int| s.contains(j) implies 0 <= j < intervals.len() by {}
    let jm = lemma_max_lo_in_set(s, intervals);
    assert(s.contains(jm));

    let p: Reading = intervals[jm].lo;

    // Claim: s ⊆ intervals_containing(intervals, p).
    assert(s.subset_of(intervals_containing(intervals, p))) by {
        assert forall|k: int| s.contains(k)
               implies intervals_containing(intervals, p).contains(k) by {
            // s.contains(k) ⇒ correct_at(k) ∧ 0 <= k < n
            assert(correct_at(k));
            assert(0 <= k < intervals.len());
            // s.contains(jm) ⇒ correct_at(jm)
            assert(correct_at(jm));
            assert(0 <= jm < intervals.len());
            // Helly-1D at (jm, k): intervals[jm].lo <= intervals[k].hi
            assert(intervals[jm].lo <= intervals[k].hi);
            // Argmax at k: intervals[k].lo <= intervals[jm].lo
            assert(intervals[k].lo <= intervals[jm].lo);
            // So p = intervals[jm].lo lies in intervals[k].
            assert(intervals[k].lo <= p);
            assert(p <= intervals[k].hi);
            assert(point_in_interval(p, intervals[k]));
        }
    }
    lemma_containing_in_range(intervals, p);
    lemma_len_subset(s, intervals_containing(intervals, p));

    assert(intervals_containing(intervals, p).len() >= intervals.len() - f);
    assert(0 <= jm < intervals.len()
        && intervals_containing(intervals, intervals[jm].lo).len()
            >= intervals.len() - f);
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
// return it as the output. Existence is guaranteed by the pigeonhole
// argument combined with the Helly-1D precondition: the n - f correct
// intervals all overlap (by `correct_intervals_overlap`), so any point
// in their common intersection has count >= n - f.
//
// Reusable patterns from ft_midpoint and the prior (blocked) marzullo
// run:
//   - finite-universe bridge via `lemma_int_range` + `lemma_len_subset`
//     for any `Set::new(|i| ...).len()` reasoning
//   - inclusion-exclusion pigeonhole via `lemma_set_intersect_union_lens`
//   - `assert(false)` + concrete return for any provably-unreachable
//     fallback at the end of the search
//   - the Helly-1D precondition closes the missing step in the
//     argmax/argmin-based existence lemma: `intervals[jm].lo <= intervals[k].hi`
//     follows directly from `correct_intervals_overlap(intervals@)` when
//     both jm and k are in `correct_indices`.

pub fn marzullo(intervals: &Vec<Interval>, f: u32) -> (result: Interval)
    requires
        intervals.len() <= u32::MAX as nat,
        intervals.len() as nat >= 2 * (f as nat) + 1,
        well_formed(intervals@),
        correct_indices(intervals.len() as nat).len() >= intervals.len() as nat - f as nat,
        correct_intervals_overlap(intervals@),
    ensures
        result.lo <= result.hi,
        exists|p: Reading|
            result.lo <= p && p <= result.hi
                && intervals_containing(intervals@, p).len() >= intervals.len() as nat - f as nat,
{
    // Overflow-safety: f + 1 <= 2*f + 1 <= n <= u32::MAX, so n - f is safe.
    let n: usize = intervals.len();
    assert(f as nat + 1 <= n as nat);
    let n_u32: u32 = n as u32;
    let n_f: u32 = n_u32 - f;
    assert(n_f as nat == intervals.len() as nat - f as nat);

    let mut i: usize = 0;
    while i < n
        invariant
            0 <= i as int <= n as int,
            n == intervals.len(),
            intervals.len() <= u32::MAX as nat,
            intervals.len() as nat >= 2 * (f as nat) + 1,
            n_f as nat == intervals.len() as nat - f as nat,
            well_formed(intervals@),
            correct_indices(intervals.len() as nat).len() >= intervals.len() as nat - f as nat,
            correct_intervals_overlap(intervals@),
            forall|j2: int| 0 <= j2 < i as int ==>
                intervals_containing(intervals@, #[trigger] intervals@[j2].lo).len()
                    < intervals.len() as nat - f as nat,
        decreases n - i,
    {
        let p: Reading = intervals[i].lo;
        let c: u32 = count_containing(intervals, p);
        if c >= n_f {
            proof {
                assert(intervals_containing(intervals@, p).len() >= intervals.len() as nat - f as nat);
            }
            return Interval { lo: p, hi: p };
        }
        proof {
            assert(c < n_f);
            assert(p == intervals@[i as int].lo);
            assert(c as nat == intervals_containing(intervals@, p).len());
            assert(intervals_containing(intervals@, intervals@[i as int].lo).len()
                < intervals.len() as nat - f as nat);
        }
        i = i + 1;
    }
    // Post-loop: i == n. lemma_exists_supported_lo produces a witness
    // contradicting the loop invariant.
    proof {
        assert(i == n);
        lemma_exists_supported_lo(intervals@, f as nat);
        let jw = choose|jx: int|
            0 <= jx < intervals@.len()
            && intervals_containing(intervals@, intervals@[jx].lo).len()
                >= intervals@.len() - f as nat;
        assert(0 <= jw < intervals@.len());
        assert(intervals_containing(intervals@, intervals@[jw].lo).len()
            >= intervals@.len() - f as nat);
        assert(0 <= jw < i as int);
        assert(intervals_containing(intervals@, intervals@[jw].lo).len()
            < intervals.len() as nat - f as nat);
        assert(false);
    }
    Interval { lo: 0, hi: 0 }
}

} // verus!
