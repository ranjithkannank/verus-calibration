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
    unimplemented!()
}

} // verus!
