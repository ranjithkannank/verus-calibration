// `auth` module for the sensor_poll exercise.
//
// Defines SensorReport, a `distinct_sensors` spec predicate, and a
// `check_distinct` exec function that decides it. The spec (struct,
// open spec fn, and check_distinct's signature) is FROZEN; the
// implementer fills in check_distinct's body.
//
// Pattern source: the bitmap-backed structural check in
// exercises/quorum_cert.rs (verify_qc_structure).

use vstd::prelude::*;
use vstd::set_lib::*;
use crate::fusion::Interval;

verus! {

pub struct SensorReport {
    pub sensor_id: u32,
    pub interval: Interval,
}

pub open spec fn distinct_sensors(reports: Seq<SensorReport>) -> bool {
    forall|i: int, j: int|
        0 <= i < j < reports.len() ==> reports[i].sensor_id != reports[j].sensor_id
}

pub fn check_distinct(reports: &Vec<SensorReport>, n: u32) -> (b: bool)
    requires
        reports.len() <= u32::MAX as nat,
        forall|i: int| 0 <= i < reports.len() ==> reports[i].sensor_id < n,
    ensures
        b == distinct_sensors(reports@),
{
    unimplemented!()
}

} // verus!
