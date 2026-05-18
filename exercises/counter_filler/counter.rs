// Counter module for the counter_filler exercise.
//
// Byte-identical spec to counter_producer/counter.rs. The implementer
// fills in the three exec method bodies; closed spec functions and
// requires/ensures clauses must not be touched.

use vstd::prelude::*;

verus! {

pub struct Counter {
    value: u32,
    bound: u32,
}

impl Counter {
    pub closed spec fn value(&self) -> u32 {
        self.value
    }

    pub closed spec fn bound(&self) -> u32 {
        self.bound
    }

    pub closed spec fn invariant(&self) -> bool {
        self.value <= self.bound
    }

    pub fn new(bound: u32) -> (c: Counter)
        ensures
            c.invariant(),
            c.value() == 0,
            c.bound() == bound,
    {
        unimplemented!()
    }

    pub fn incr(&mut self)
        requires
            old(self).invariant(),
            old(self).value() < old(self).bound(),
        ensures
            final(self).invariant(),
            final(self).value() == old(self).value() + 1,
            final(self).bound() == old(self).bound(),
    {
        unimplemented!()
    }

    pub fn get(&self) -> (v: u32)
        requires
            self.invariant(),
        ensures
            v == self.value(),
    {
        unimplemented!()
    }
}

} // verus!
