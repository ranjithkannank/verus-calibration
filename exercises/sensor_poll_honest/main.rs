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

    // 2. Combine `distinct_sensors` with the precondition
    //    `all_signatures_valid` to get `valid_report_bundle`.
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

    // 5. Bridge from intervals-frame to reports-frame + honest-voter.
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

        // ---- Honest-voter clause via inclusion-exclusion -------------------
        // Let A = supporters of p_witness in `reports@`,
        //     B = correct (honest) sensor indices.
        // Both subsets of universe U = [0, n).
        // |A| >= n - f, |B| >= n - f, |U| = n.
        // |A ∪ B| <= n  and  |A ∪ B| + |A ∩ B| = |A| + |B|
        // ⇒ |A ∩ B| >= 2(n - f) - n = n - 2f >= 1 (since n >= 2f + 1).
        let a: Set<int> = reports_containing(reports@, p_witness);
        let b: Set<int> = correct_indices(reports.len() as nat);
        let nn: int = reports.len() as int;

        lemma_int_range(0, nn);
        let u: Set<int> = set_int_range(0, nn);

        // A ⊆ U
        assert forall|x: int| a.contains(x) implies u.contains(x) by {};
        // B ⊆ U
        assert forall|x: int| b.contains(x) implies u.contains(x) by {};
        // A ∪ B ⊆ U
        assert((a + b).subset_of(u)) by {
            assert forall|x: int| (a + b).contains(x) implies u.contains(x) by {
                assert(a.contains(x) || b.contains(x));
            }
        }

        // Finiteness + bounds.
        lemma_len_subset(a, u);
        lemma_len_subset(b, u);
        lemma_len_subset(a + b, u);

        // Inclusion-exclusion.
        lemma_set_intersect_union_lens(a, b);

        // Arithmetic: |A ∩ B| >= 1.
        assert(a.intersect(b).len() >= 1);
        assert(a.intersect(b).finite()) by {
            assert(a.intersect(b).subset_of(a));
            lemma_len_subset(a.intersect(b), a);
        }
        assert(!a.intersect(b).is_empty()) by {
            axiom_is_empty_len0(a.intersect(b));
        }
        axiom_is_empty(a.intersect(b));

        // Extract a witness k from A ∩ B.
        let k = choose|x: int| a.intersect(b).contains(x);
        assert(a.intersect(b).contains(k));
        assert(a.contains(k));
        assert(b.contains(k));
        // a.contains(k) ⇒ 0 <= k < reports.len() && point_in_interval(p_witness, reports[k].interval)
        // b.contains(k) ⇒ 0 <= k < n && correct_at(k)
        assert(0 <= k < reports.len());
        assert(correct_at(k));
        assert(point_in_interval(p_witness, reports@[k].interval));
        // Existential discharged for both supporter and honest-voter clauses.
    }
    Some(result)
}

} // verus!
