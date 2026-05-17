// Exercise 7: cross-module verified bounded counter.
//
// The first multi-module exercise in the calibration. A `counter`
// module exports a bounded counter abstraction (`new`, `incr`, `get`)
// stated in terms of three *closed* spec functions: `value()`,
// `bound()`, `invariant()`. The bodies of those spec functions are
// visible inside `counter` but opaque to other modules.
//
// A `client` module imports `Counter` and implements `count_up_to`,
// which creates a fresh counter of bound `target` and increments it
// to `target`. The proof of `count_up_to` must rely on the `Counter`
// methods' postconditions alone — `client` cannot see the underlying
// `value: u32` and `bound: u32` fields.
//
// The trust boundary in this exercise is the module boundary itself.
// No `uninterp spec fn` here. The implementer's job: fill in the four
// exec bodies (Counter::new, Counter::incr, Counter::get,
// client::count_up_to) so that verus verifies the file end-to-end.
//
// Design note: exercises/cross_module_counter.design.md.
//
// The spec below is FROZEN. Iteration cap: 15. See AGENTS.md.

use vstd::prelude::*;

verus! {

mod counter {
    use vstd::prelude::*;

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
}

mod client {
    use super::counter::Counter;
    use vstd::prelude::*;

    pub fn count_up_to(target: u32) -> (final_count: u32)
        ensures
            final_count == target,
    {
        let mut c = Counter::new(target);
        let mut i: u32 = 0;
        while i < target
            invariant
                c.invariant(),
                c.value() == i,
                c.bound() == target,
                i <= target,
            decreases target - i,
        {
            c.incr();
            i = i + 1;
        }
        c.get()
    }
}

} // verus!
