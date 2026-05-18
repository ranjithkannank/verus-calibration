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
    unimplemented!()
}

} // verus!
