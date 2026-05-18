// Witness for sensor_poll_signed/main.rs.
//
// Operator-authored reference implementation. Spec block byte-aligned
// to the exercise file (uses `Reading` type alias rather than the
// underlying `i64`).

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

// The projection preserves the indices-containing-p set. Extensional
// set equality; Verus closes it from =~= alone. The new `sig` field
// on SensorReport is irrelevant — membership depends only on
// `reports[i].interval`.
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
        },
        result.is_none() ==> !distinct_sensors(reports@),
{
    if !check_distinct(reports, n) {
        return None;
    }
    // distinct_sensors(reports@) ∧ all_signatures_valid(reports@) (precondition)
    // ==> valid_report_bundle(reports@)
    assert(valid_report_bundle(reports@));

    let intervals: Vec<Interval> = extract_intervals(reports);
    let agreed = marzullo(&intervals, f);
    proof {
        let p = choose|p: Reading|
            agreed.lo <= p && p <= agreed.hi
            && intervals_containing(intervals@, p).len()
                >= intervals.len() as nat - f as nat;
        lemma_reports_eq_intervals_containing(reports@, p);
        assert(reports_containing(reports@, p).len()
            == intervals_containing(project_intervals(reports@), p).len());
        assert(intervals@ =~= project_intervals(reports@));
    }
    Some(agreed)
}

} // verus!
