// Witness for counter_filler/counter.rs. Byte-identical Counter
// module to counter_producer (closed value/bound/invariant + new /
// incr / get exec methods).

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
        Counter { value: 0, bound: bound }
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
        self.value = self.value + 1;
    }

    pub fn get(&self) -> (v: u32)
        requires
            self.invariant(),
        ensures
            v == self.value(),
    {
        self.value
    }
}

} // verus!
