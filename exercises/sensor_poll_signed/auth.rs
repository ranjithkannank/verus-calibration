// `auth` module for the sensor_poll_signed exercise.
//
// Extends `exercises/sensor_poll/auth.rs` with the cryptographic
// trust boundary: SensorReport gains a `sig: Signature` field, and
// the module declares `pk_of` / `signature_valid` / `report_msg` as
// uninterp spec fns plus `all_signatures_valid` and
// `valid_report_bundle` open spec fns. The exec function
// `check_distinct` is unchanged in contract — it still only checks
// the structural (distinct sensor IDs) half. The `sig` field is
// invisible to its body.
//
// The spec (the struct, the uninterp predicates, the open spec fns,
// and `check_distinct`'s signature with requires/ensures) is FROZEN.
// The implementer fills in `check_distinct`'s body.
//
// Trust-boundary note (mirrors `exercises/quorum_cert.rs`): `pk_of`,
// `signature_valid`, and `report_msg` are deliberately opaque. See
// AGENTS.md (the "Uninterpreted spec functions and trust boundaries"
// section) for the rules — bodies are forbidden and trust-bypass
// constructs are not in the agent's vocabulary. A real deployment
// connects these to a vetted crypto library outside this repo; here
// that connection is out of scope.
//
// Pattern source for `check_distinct`: the bitmap-backed structural
// check in `exercises/quorum_cert.rs::verify_qc_structure`, the
// same pattern used in `exercises/sensor_poll/auth.rs`.

use vstd::prelude::*;
use vstd::set_lib::*;
use crate::fusion::Interval;

verus! {

pub type Hash = u64;
pub type PubKey = u64;
pub type Signature = u64;

pub struct SensorReport {
    pub sensor_id: u32,
    pub interval: Interval,
    pub sig: Signature,
}

pub open spec fn distinct_sensors(reports: Seq<SensorReport>) -> bool {
    forall|i: int, j: int|
        0 <= i < j < reports.len() ==> reports[i].sensor_id != reports[j].sensor_id
}

// --- Cryptographic trust boundary (uninterpreted) --------------------------

pub uninterp spec fn pk_of(sensor_id: u32) -> PubKey;

pub uninterp spec fn signature_valid(pk: PubKey, msg: Hash, sig: Signature) -> bool;

pub uninterp spec fn report_msg(report: SensorReport) -> Hash;

pub open spec fn all_signatures_valid(reports: Seq<SensorReport>) -> bool {
    forall|i: int|
        0 <= i < reports.len() ==>
            signature_valid(
                pk_of(reports[i].sensor_id),
                report_msg(reports[i]),
                reports[i].sig,
            )
}

pub open spec fn valid_report_bundle(reports: Seq<SensorReport>) -> bool {
    distinct_sensors(reports) && all_signatures_valid(reports)
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
