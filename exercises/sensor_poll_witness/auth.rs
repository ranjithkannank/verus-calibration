// Authentication module: distinct-sensor structural check.

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
    let mut seen: Vec<bool> = Vec::new();
    let mut k: u32 = 0;
    while k < n
        invariant
            seen.len() == k as nat,
            k <= n,
            forall|q: int| 0 <= q < seen.len() ==> seen[q] == false,
        decreases n - k,
    {
        seen.push(false);
        k = k + 1;
    }
    assert(k == n);
    assert(seen.len() == n as nat);

    let mut i: usize = 0;
    while i < reports.len()
        invariant
            seen.len() == n as nat,
            forall|q: int| 0 <= q < reports.len() ==> reports[q].sensor_id < n,
            forall|p: int, q: int|
                0 <= p < q < i as int ==> reports[p].sensor_id != reports[q].sensor_id,
            forall|s: int| 0 <= s < n as int ==>
                seen[s] == (exists|p: int| 0 <= p < i as int && reports[p].sensor_id as int == s),
        decreases reports.len() - i,
    {
        let v = reports[i].sensor_id;
        if seen[v as usize] {
            proof {
                assert(seen[v as int] == true);
                assert(exists|p: int| 0 <= p < i as int && reports[p].sensor_id as int == v as int);
                let p = choose|p: int| 0 <= p < i as int && reports[p].sensor_id as int == v as int;
                assert(reports[p].sensor_id == v);
                assert(reports[i as int].sensor_id == v);
                assert(reports[p].sensor_id == reports[i as int].sensor_id);
                assert(p < i as int);
                assert(!distinct_sensors(reports@));
            }
            return false;
        }
        seen.set(v as usize, true);
        proof {
            assert forall|s: int| 0 <= s < n as int implies
                seen[s] == (exists|p: int| 0 <= p < (i as int) + 1 && reports[p].sensor_id as int == s) by {
                if s == v as int {
                    assert(seen[s] == true);
                    assert(reports[i as int].sensor_id as int == s);
                } else {
                    // unchanged
                }
            }
            assert forall|p: int, q: int|
                0 <= p < q < (i as int) + 1 implies reports[p].sensor_id != reports[q].sensor_id by {
                if q < i as int {
                    // prior invariant
                } else {
                    if reports[p].sensor_id == v {
                        assert(false);
                    }
                }
            }
        }
        i = i + 1;
    }
    proof {
        assert(distinct_sensors(reports@));
    }
    true
}

} // verus!
