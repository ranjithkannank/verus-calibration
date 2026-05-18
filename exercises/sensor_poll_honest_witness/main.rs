// Witness for sensor_poll_honest/main.rs.
//
// Operator-authored reference implementation. Spec block byte-aligned
// to the exercise file. Discharges the strengthened poll ensures —
// the `Some` branch now also asserts the existence of a CORRECT
// (non-Byzantine) sensor whose interval contains the agreed point.

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

proof fn lemma_reports_eq_intervals_containing(reports: Seq<SensorReport>, p: Reading)
    ensures
        reports_containing(reports, p)
            =~= intervals_containing(project_intervals(reports), p),
{
}

fn extract_intervals(reports: &Vec<SensorReport>) -> (intervals: Vec<Interval>)
    requires
        reports.len() <= u32::MAX as nat,
    ensures
        intervals@ =~= project_intervals(reports@),
        intervals.len() == reports.len(),
{
    let mut out: Vec<Interval> = Vec::new();
    let mut i: usize = 0;
    while i < reports.len()
        invariant
            out.len() == i as nat,
            i <= reports.len(),
            forall|k: int| 0 <= k < i as int ==> out[k] == reports[k].interval,
        decreases reports.len() - i,
    {
        out.push(reports[i].interval);
        i = i + 1;
    }
    assert(out.len() == reports.len());
    assert(forall|k: int| 0 <= k < reports.len() ==> out[k] == reports[k].interval);
    assert(out@ =~= project_intervals(reports@));
    out
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
    if !check_distinct(reports, n) {
        return None;
    }
    assert(valid_report_bundle(reports@));

    let intervals: Vec<Interval> = extract_intervals(reports);
    let agreed = marzullo(&intervals, f);

    proof {
        // p witness from marzullo's existential.
        let p = choose|p: Reading|
            agreed.lo <= p && p <= agreed.hi
            && intervals_containing(intervals@, p).len()
                >= intervals.len() as nat - f as nat;
        assert(agreed.lo <= p && p <= agreed.hi);
        assert(intervals_containing(intervals@, p).len()
            >= intervals.len() as nat - f as nat);

        // Translate from intervals-frame to reports-frame.
        lemma_reports_eq_intervals_containing(reports@, p);
        assert(intervals@ =~= project_intervals(reports@));
        assert(reports_containing(reports@, p).len()
            >= reports.len() as nat - f as nat);

        // Honest-supporter argument.
        let support: Set<int> = reports_containing(reports@, p);
        let correct: Set<int> = correct_indices(reports.len() as nat);
        let n_int: int = reports.len() as int;

        // Universe is set_int_range(0, n); finite with length n.
        lemma_int_range(0, n_int);

        // support ⊆ [0, n)
        assert(support.subset_of(set_int_range(0, n_int))) by {
            assert forall|i: int| support.contains(i) implies set_int_range(0, n_int).contains(i) by {
                // support's predicate already requires 0 <= i < reports.len()
            };
        };
        lemma_len_subset(support, set_int_range(0, n_int));
        assert(support.finite());
        assert(support.len() >= reports.len() as nat - f as nat);

        // correct ⊆ [0, n)
        assert(correct.subset_of(set_int_range(0, n_int))) by {
            assert forall|i: int| correct.contains(i) implies set_int_range(0, n_int).contains(i) by {
                // correct's predicate already requires 0 <= i < n
            };
        };
        lemma_len_subset(correct, set_int_range(0, n_int));
        assert(correct.finite());
        assert(correct.len() >= reports.len() as nat - f as nat);

        // Inclusion-exclusion.
        lemma_set_intersect_union_lens(support, correct);
        // (support ∪ correct).len() + (support ∩ correct).len()
        //     == support.len() + correct.len()

        // support ∪ correct ⊆ [0, n)
        assert(support.union(correct).subset_of(set_int_range(0, n_int))) by {
            assert forall|i: int| support.union(correct).contains(i)
                implies set_int_range(0, n_int).contains(i) by {};
        };
        lemma_len_subset(support.union(correct), set_int_range(0, n_int));
        assert(support.union(correct).len() <= n_int as nat);

        // Therefore (support ∩ correct).len() >= 2(n - f) - n = n - 2f >= 1.
        let inter = support.intersect(correct);
        assert(inter.len() + support.union(correct).len()
            == support.len() + correct.len());
        assert(inter.len() >= 1);

        // Non-empty ⇒ choose a witness.
        axiom_is_empty_len0(inter);
        axiom_is_empty(inter);
        let k = choose|x: int| inter.contains(x);
        assert(inter.contains(k));
        assert(support.contains(k));
        assert(correct.contains(k));
        assert(0 <= k < reports.len());
        assert(point_in_interval(p, reports[k].interval));
        assert(correct_at(k));
    }
    Some(agreed)
}

} // verus!
