// Entry point for the sensor_poll_signed exercise.
//
// Extends `exercises/sensor_poll/main.rs` with the cryptographic
// trust boundary: poll gains an `all_signatures_valid(reports@)`
// precondition and a strengthened `Some`-branch postcondition
// requiring `valid_report_bundle(reports@)` (== distinct + signed).
// The structural composition (project to intervals + marzullo +
// projection lemma) is identical to `sensor_poll`.
//
// The spec (the two open spec fns + `poll`'s signature with
// requires/ensures) is FROZEN. The implementer fills in `poll`'s
// body and any helper lemmas (in particular, a projection lemma
// relating `reports_containing` to `intervals_containing`).

use vstd::prelude::*;
use vstd::set_lib::*;

mod fusion;
mod auth;

use fusion::{Interval, Reading, well_formed, point_in_interval, intervals_containing,
             correct_indices, correct_intervals_overlap, marzullo};
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

    // 5. Bridge from intervals-frame to reports-frame.
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
    }
    Some(result)
}

} // verus!
