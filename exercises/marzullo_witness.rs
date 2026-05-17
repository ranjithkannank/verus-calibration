// Pre-spec witness for exercises/marzullo.rs.
//
// A *witness* is an operator-authored reference implementation that
// verifies under the same spec block as the exercise file. Its job is
// to falsify the frozen spec before the agent ever sees it: if no
// witness can be made to verify, the spec is wrong (unprovable, or
// under-constrained, or missing a precondition) and the operator
// must fix it before the agent loop starts.
//
// Workflow:
//   1. Operator writes exercises/<name>.rs with `unimplemented!()` body.
//   2. Operator writes exercises/<name>_witness.rs with the SAME spec
//      block and a real reference implementation. (Underscore, not dot —
//      Verus rejects dots in derived crate names.)
//   3. Operator runs `ralph/check-spec.sh <name>`. If it passes
//      (verus verifies the witness, no cheat tokens), the spec
//      provably admits a model.
//   4. Operator tags `spec-frozen-<name>` and starts the agent loop.
//
// This file is operator territory. The pre-commit hook still enforces
// cheat-token prohibition on it, but the agent's tool whitelist does
// not name it — the agent should never touch witnesses.
//
// --- Retroactive backfill note (marzullo, 2026-05-16) -----------------
//
// This particular witness was written *after* the agent completed the
// exercise, because the tool itself is being introduced now. The
// original frozen spec (which omitted `correct_intervals_overlap`)
// was logically unprovable; the agent surfaced that across attempts
// 5-7 and the operator re-froze with the missing precondition. If
// this witness file had existed before the first freeze, the bug
// would have surfaced at operator time instead — a single Verus run
// against the witness, no agent cycles spent. That is the test the
// tool would have caught: see scripts/test-witness-catches-bad-spec.sh
// for the demonstration.

use vstd::prelude::*;
use vstd::set_lib::*;

verus! {

// --- SPEC BLOCK (byte-identical to exercises/marzullo.rs) --------------------

pub type Reading = i64;

pub struct Interval {
    pub lo: Reading,
    pub hi: Reading,
}

pub uninterp spec fn correct_at(i: int) -> bool;

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

pub open spec fn correct_intervals_overlap(intervals: Seq<Interval>) -> bool {
    forall|i: int, j: int|
        0 <= i < intervals.len() && 0 <= j < intervals.len()
        && correct_at(i) && correct_at(j)
            ==> intervals[i].lo <= intervals[j].hi
}

// --- END SPEC BLOCK ----------------------------------------------------------
//
// Below is the witness implementation. The proof helpers and lemmas
// are the same shape as the agent's solution because the proof
// structure is dictated by the spec, not by the implementer. The
// equality of the witness to the agent's output is incidental — for
// future exercises the operator should write the simplest correct
// reference impl they can and let the agent find a different
// (perhaps better) one.

spec fn containing_upto(intervals: Seq<Interval>, p: Reading, m: int) -> Set<int> {
    Set::new(|i: int| 0 <= i < m && i < intervals.len() && point_in_interval(p, intervals[i]))
}

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
    let s: Set<int> = correct_indices(intervals.len());

    lemma_correct_indices_in_range(intervals.len());
    assert(s.len() >= intervals.len() - f);
    assert(intervals.len() >= 2 * f + 1);
    assert(s.len() >= 1);

    assert forall|j: int| s.contains(j) implies 0 <= j < intervals.len() by {}
    let jm = lemma_max_lo_in_set(s, intervals);
    assert(s.contains(jm));

    let p: Reading = intervals[jm].lo;

    assert(s.subset_of(intervals_containing(intervals, p))) by {
        assert forall|k: int| s.contains(k)
               implies intervals_containing(intervals, p).contains(k) by {
            assert(correct_at(k));
            assert(0 <= k < intervals.len());
            assert(correct_at(jm));
            assert(0 <= jm < intervals.len());
            assert(intervals[jm].lo <= intervals[k].hi);
            assert(intervals[k].lo <= intervals[jm].lo);
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

// --- SPEC BLOCK: function signature (byte-identical) -------------------------

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
    // --- END SPEC BLOCK ----------------------------------------------------
    //
    // Operator-authored reference impl: scan each interval's `lo` as a
    // candidate point. Whichever first witnesses a count >= n - f
    // becomes a degenerate [p, p] output interval. Existence is
    // guaranteed by lemma_exists_supported_lo (which depends on
    // correct_intervals_overlap — strip that precondition and this
    // proof breaks at the argmax/Helly step).
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
