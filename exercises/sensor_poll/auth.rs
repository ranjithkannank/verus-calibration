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
    let mut seen: Vec<bool> = vec![false; n as usize];

    assert(seen@.len() == n as nat);
    assert(forall|k: int| 0 <= k < n as int ==> seen@[k] == false);

    let mut i: usize = 0;
    while i < reports.len()
        invariant
            i <= reports@.len(),
            seen@.len() == n as nat,
            reports.len() <= u32::MAX as nat,
            forall|j: int| 0 <= j < reports.len() ==>
                (#[trigger] reports@[j].sensor_id) < n,
            // pairwise-distinct prefix
            forall|j: int, k: int| 0 <= j < k < i as int ==>
                reports@[j].sensor_id != reports@[k].sensor_id,
            // bitmap abstraction: seen[k] iff some prefix report has sensor_id == k
            forall|k: int| 0 <= k < n as int ==>
                (#[trigger] seen@[k]) == (exists|j: int|
                    0 <= j < i as int && (#[trigger] reports@[j].sensor_id as int) == k),
        decreases reports@.len() - i,
    {
        let s_id: u32 = reports[i].sensor_id;
        assert(s_id < n);
        let s: usize = s_id as usize;
        if seen[s] {
            // witness: some earlier index has the same sensor_id
            proof {
                // from invariant (d) at k = s_id as int: seen[s] is true ⇒ ∃ j < i with that voter.
                assert(seen@[s_id as int] == true);
                let j0 = choose|j: int|
                    0 <= j < i as int && reports@[j].sensor_id as int == s_id as int;
                assert(0 <= j0 < i as int);
                assert(reports@[j0].sensor_id == s_id);
                assert(reports@[i as int].sensor_id == s_id);
                assert(reports@[j0].sensor_id == reports@[i as int].sensor_id);
                assert(j0 < i as int);
                assert(!distinct_sensors(reports@));
            }
            return false;
        }
        let ghost s_ghost: u32 = reports@[i as int].sensor_id;
        let ghost old_i: int = i as int;
        assert(s_ghost == s_id);
        assert(s_ghost as int == s as int);

        assert(seen@[s_ghost as int] == false);
        assert(forall|j: int|
            0 <= j < old_i ==> reports@[j].sensor_id as int != s_ghost as int)
        by {
            assert(!(exists|j: int|
                0 <= j < old_i && reports@[j].sensor_id as int == s_ghost as int));
        };

        seen.set(s, true);
        i = i + 1;

        // Re-establish (c) at the new i = old_i + 1.
        assert forall|j: int, k: int| 0 <= j < k < i as int implies
            reports@[j].sensor_id != reports@[k].sensor_id
        by {
            if k < old_i {
                // covered by old (c)
            } else {
                // k == old_i
                assert(reports@[old_i].sensor_id == s_ghost);
                assert(reports@[j].sensor_id as int != s_ghost as int);
            }
        };

        // Re-establish (d) at the new i = old_i + 1.
        assert forall|k: int| 0 <= k < n as int implies
            (#[trigger] seen@[k]) == (exists|j: int|
                0 <= j < i as int && (#[trigger] reports@[j].sensor_id as int) == k)
        by {
            if k == s_ghost as int {
                assert(seen@[s_ghost as int] == true);
                assert(0 <= old_i < i as int);
                assert(reports@[old_i].sensor_id as int == k);
            } else {
                assert(reports@[old_i].sensor_id as int == s_ghost as int);
                assert(s_ghost as int != k);
                if exists|j: int| 0 <= j < i as int && reports@[j].sensor_id as int == k {
                    let j0 = choose|j: int|
                        0 <= j < i as int && reports@[j].sensor_id as int == k;
                    if j0 == old_i {
                        assert(reports@[j0].sensor_id as int == s_ghost as int);
                        assert(false);
                    }
                    assert(0 <= j0 < old_i);
                }
            }
        };
    }
    // At loop exit: (c) gives distinct_sensors.
    assert(distinct_sensors(reports@));
    true
}

} // verus!
