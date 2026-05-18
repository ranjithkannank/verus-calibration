// Entry point for the sensor_poll_honest exercise.
//
// Extends `exercises/sensor_poll_signed/main.rs` by strengthening
// poll's Some-branch ensures with an HONEST-VOTER clause: in addition
// to "some point p is supported by >= n - f reports", the result must
// also witness "some CORRECT (non-Byzantine) sensor k whose interval
// contains p". This is the BFT-meaningful step — the signature trust
// boundary is now load-bearing in the proof, not just threaded
// through the contract.
//
// The spec (the two open spec fns + `poll`'s signature with
// requires/ensures) is FROZEN. The implementer fills in `poll`'s
// body and any helper lemmas. The existing supporters clause is
// preserved verbatim; the new honest-voter clause is the addition.

use vstd::prelude::*;
use vstd::set_lib::*;

mod fusion;
mod auth;

use fusion::{Interval, Reading, well_formed, point_in_interval, intervals_containing,
             correct_at, correct_indices, correct_intervals_overlap, marzullo};
use auth::{SensorReport, distinct_sensors, all_signatures_valid, valid_report_bundle,
           check_distinct};

verus! {

pub open spec fn project_intervals(reports: Seq<SensorReport>) -> Seq<Interval> {
    Seq::new(reports.len(), |i: int| reports[i].interval)
}

pub open spec fn reports_containing(reports: Seq<SensorReport>, p: Reading) -> Set<int> {
    Set::new(|i: int|
        0 <= i < reports.len() && point_in_interval(p, reports[i].interval))
}

// --- Projection lemma -------------------------------------------------------
//
// The two sets are extensionally equal because
// `project_intervals(reports)[i] == reports[i].interval` so the
// membership predicate is identical up to substitution.

proof fn lemma_reports_eq_intervals_containing(reports: Seq<SensorReport>, p: Reading)
    ensures
        reports_containing(reports, p)
            =~= intervals_containing(project_intervals(reports), p),
{
}

// --- Subset / finiteness lemmas for the two index sets ----------------------

proof fn lemma_reports_containing_in_range(reports: Seq<SensorReport>, p: Reading)
    ensures
        reports_containing(reports, p).subset_of(set_int_range(0, reports.len() as int)),
        reports_containing(reports, p).finite(),
        reports_containing(reports, p).len() <= reports.len() as nat,
{
    lemma_int_range(0, reports.len() as int);
    assert(reports_containing(reports, p).subset_of(set_int_range(0, reports.len() as int)));
    lemma_len_subset(reports_containing(reports, p), set_int_range(0, reports.len() as int));
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

// --- Honest-supporter existence ---------------------------------------------
//
// If the supporter set for `p` has size >= n - f, and the correct
// index set has size >= n - f, then with n >= 2f + 1 both subsets of
// [0, n) must intersect — the intersection has size >= n - 2f >= 1.
// The intersection yields an honest sensor whose interval contains p.

proof fn lemma_honest_supporter_exists(reports: Seq<SensorReport>, p: Reading, f: nat)
    requires
        reports.len() >= 2 * f + 1,
        correct_indices(reports.len()).len() >= reports.len() - f,
        reports_containing(reports, p).len() >= reports.len() - f,
    ensures
        exists|k: int|
            0 <= k < reports.len()
            && correct_at(k)
            && point_in_interval(p, reports[k].interval),
{
    let n: int = reports.len() as int;
    let s: Set<int> = reports_containing(reports, p);
    let c: Set<int> = correct_indices(reports.len());

    lemma_reports_containing_in_range(reports, p);
    lemma_correct_indices_in_range(reports.len());

    // Both sets are subsets of [0, n). Their union is too.
    lemma_int_range(0, n);
    assert(s.union(c).subset_of(set_int_range(0, n))) by {
        assert forall|x: int| s.union(c).contains(x)
            implies set_int_range(0, n).contains(x) by {
            if s.contains(x) {
                assert(set_int_range(0, n).contains(x));
            } else {
                assert(c.contains(x));
                assert(set_int_range(0, n).contains(x));
            }
        }
    }
    lemma_len_subset(s.union(c), set_int_range(0, n));
    assert(s.union(c).len() <= n as nat);

    // Inclusion–exclusion: |s ∪ c| + |s ∩ c| == |s| + |c|.
    lemma_set_intersect_union_lens(s, c);
    assert((s + c).len() + s.intersect(c).len() == s.len() + c.len());
    assert(s + c =~= s.union(c));
    assert(s.intersect(c).len() + s.union(c).len() == s.len() + c.len());
    assert(s.intersect(c).len() >= s.len() + c.len() - (n as nat));
    assert(s.len() >= reports.len() - f);
    assert(c.len() >= reports.len() - f);
    assert(reports.len() >= 2 * f + 1);
    assert(s.intersect(c).len() >= 1);

    // Pull a witness out of the non-empty intersection.
    axiom_is_empty_len0(s.intersect(c));
    axiom_is_empty(s.intersect(c));
    let k = choose|x: int| s.intersect(c).contains(x);
    assert(s.intersect(c).contains(k));
    assert(s.contains(k));
    assert(c.contains(k));
    // s.contains(k) gives: 0 <= k < reports.len() && point_in_interval(p, reports[k].interval)
    // c.contains(k) gives: 0 <= k < reports.len() && correct_at(k)
    assert(0 <= k < reports.len());
    assert(correct_at(k));
    assert(point_in_interval(p, reports[k].interval));
}

pub fn poll(reports: &Vec<SensorReport>, n: u32, f: u32) -> (result: Option<Interval>)
    requires
        reports.len() <= u32::MAX as nat,
        reports.len() == n as nat,
        n as nat >= 2 * (f as nat) + 1,
        forall|i: int| 0 <= i < reports.len() ==> reports[i].sensor_id < n,
        well_formed(project_intervals(reports@)),
        correct_indices(reports.len() as nat).len() >= reports.len() as nat - f as nat,
        correct_intervals_overlap(project_intervals(reports@)),
        all_signatures_valid(reports@),
    ensures
        result.is_some() ==> {
            let interval = result.unwrap();
            &&& interval.lo <= interval.hi
            &&& valid_report_bundle(reports@)
            &&& exists|p: Reading|
                interval.lo <= p && p <= interval.hi
                && reports_containing(reports@, p).len()
                    >= reports.len() as nat - f as nat
            &&& exists|p: Reading, k: int|
                interval.lo <= p && p <= interval.hi
                && 0 <= k < reports.len()
                && correct_at(k)
                && point_in_interval(p, reports[k].interval)
        },
        result.is_none() ==> !distinct_sensors(reports@),
{
    // 1. Structural authentication.
    if !check_distinct(reports, n) {
        return None;
    }

    // 2. Combine `distinct_sensors` (from check_distinct) with the
    //    precondition `all_signatures_valid` to get `valid_report_bundle`.
    proof {
        assert(valid_report_bundle(reports@));
    }

    // 3. Project: build a Vec<Interval> whose view equals
    //    project_intervals(reports@).
    let n_usize: usize = reports.len();
    let mut intervals: Vec<Interval> = Vec::with_capacity(n_usize);
    let mut i: usize = 0;
    while i < n_usize
        invariant
            i <= n_usize,
            n_usize == reports.len(),
            intervals@.len() == i as nat,
            forall|k: int| 0 <= k < i as int ==> intervals@[k] == reports@[k].interval,
        decreases n_usize - i,
    {
        let iv = reports[i].interval;
        intervals.push(iv);
        i = i + 1;
    }

    proof {
        assert(intervals@ =~= project_intervals(reports@));
    }

    // 4. Call marzullo.
    let result = marzullo(&intervals, f);

    // 5. Bridge from intervals-frame to reports-frame, and discharge
    //    the honest-voter clause via pigeonhole on the supporter and
    //    correct-index sets.
    proof {
        let p_witness = choose|p: Reading|
            result.lo <= p && p <= result.hi
            && intervals_containing(intervals@, p).len() >= intervals.len() as nat - f as nat;
        assert(result.lo <= p_witness && p_witness <= result.hi);
        assert(intervals_containing(intervals@, p_witness).len()
            >= intervals.len() as nat - f as nat);
        // Substitute intervals@ with project_intervals(reports@) extensionally.
        assert(intervals_containing(intervals@, p_witness)
            =~= intervals_containing(project_intervals(reports@), p_witness));
        lemma_reports_eq_intervals_containing(reports@, p_witness);
        assert(reports_containing(reports@, p_witness)
            =~= intervals_containing(project_intervals(reports@), p_witness));
        assert(reports_containing(reports@, p_witness).len()
            >= reports.len() as nat - f as nat);

        // Honest-supporter clause: pigeonhole on supporters ∩ correct.
        lemma_honest_supporter_exists(reports@, p_witness, f as nat);
        let k_witness = choose|k: int|
            0 <= k < reports@.len()
            && correct_at(k)
            && point_in_interval(p_witness, reports@[k].interval);
        assert(0 <= k_witness < reports@.len());
        assert(correct_at(k_witness));
        assert(point_in_interval(p_witness, reports@[k_witness].interval));
        assert(result.lo <= p_witness && p_witness <= result.hi
            && 0 <= k_witness < reports@.len()
            && correct_at(k_witness)
            && point_in_interval(p_witness, reports@[k_witness].interval));
    }
    Some(result)
}

} // verus!
