// `fusion` module for the sensor_poll exercise.
//
// Direct port of the marzullo exercise. The spec (the type aliases,
// the uninterp spec fn, the five open spec fns, and `marzullo`'s
// signature with requires/ensures) is FROZEN. The implementer ports
// the verified body from exercises/marzullo.rs and writes any helper
// lemmas needed.
//
// Reference: see exercises/marzullo.rs for the verified body and
// proof helpers. The agent may read that file freely.

use vstd::prelude::*;
use vstd::set_lib::*;

verus! {

pub type Reading = i64;

#[derive(Copy, Clone)]
pub struct Interval {
    pub lo: Reading,
    pub hi: Reading,
}

// --- Trust boundary (uninterpreted) -----------------------------------------

pub uninterp spec fn correct_at(i: int) -> bool;

// --- Spec helpers -----------------------------------------------------------

pub open spec fn well_formed(intervals: Seq<Interval>) -> bool {
    forall|i: int| 0 <= i < intervals.len() ==> intervals[i].lo <= intervals[i].hi
}

pub open spec fn point_in_interval(p: Reading, iv: Interval) -> bool {
    iv.lo <= p && p <= iv.hi
}

pub open spec fn intervals_containing(intervals: Seq<Interval>, p: Reading) -> Set<int> {
    Set::new(|i: int|
        0 <= i < intervals.len() && point_in_interval(p, intervals[i]))
}

pub open spec fn correct_indices(n: nat) -> Set<int> {
    Set::new(|i: int| 0 <= i < n as int && correct_at(i))
}

pub open spec fn correct_intervals_overlap(intervals: Seq<Interval>) -> bool {
    forall|i: int, j: int|
        0 <= i < intervals.len() && 0 <= j < intervals.len()
        && correct_at(i) && correct_at(j)
            ==> intervals[i].lo <= intervals[j].hi
}

// --- The exec entry point ---------------------------------------------------

pub fn marzullo(intervals: &Vec<Interval>, f: u32) -> (result: Interval)
    requires
        intervals.len() <= u32::MAX as nat,
        intervals.len() as nat >= 2 * (f as nat) + 1,
        well_formed(intervals@),
        correct_indices(intervals.len() as nat).len() >= intervals.len() as nat - f as nat,
        correct_intervals_overlap(intervals@),
    ensures
        result.lo <= result.hi,
        exists|p: Reading|
            result.lo <= p && p <= result.hi
                && intervals_containing(intervals@, p).len() >= intervals.len() as nat - f as nat,
{
    unimplemented!()
}

} // verus!
