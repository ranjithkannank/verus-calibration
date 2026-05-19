# Design: OS__array__impl4__init2none

External-validity task from VeruSAGE-Bench (upstream prefix `OS` =
atmosphere, an OS kernel). Operator-authored design note — neutral
by construction; this batch is a methodology probe and the design
note deliberately withholds Verus tooling hints, lemma names, and
proof-structure suggestions.

## Frozen obligation

```
pub fn init2none(&mut self)
    requires
        old(self).wf(),
        N <= usize::MAX,
    ensures
        forall|index:int| 0<= index < N ==> #[trigger] self@[index].is_None(),
        self.wf(),
```

where `Array<A, N>` is a struct over `[A; N]` with a ghost
`Seq<A>` view and `.wf()` means the view's length equals `N`. The
upstream task scaffold contains the loop body already (a `for i in 0..N`
that calls `self.set(i, None)` plus a ghost snapshot binding) but no
invariant. The obligation is to make verus accept the function.

## Forbidden (audited at commit time)

- Adding any verification-bypass tokens beyond what the scaffold
  baseline already carries.
- Modifying the `requires` or `ensures` clauses on `init2none`, the
  baseline `set` declaration, or any of the `view` / `wf` / struct
  definitions.
- Removing `fn main() {}` or the `verus!{}` wrapper.

## Sub-tasks

1. Run `verus exercises/OS__array__impl4__init2none.rs --crate-type=lib`
   on the unmodified scaffold. Capture the exact rejection messages.
2. Attempt the smallest possible body that could close the
   obligation. Run verus.
3. If step 2 fails, read the rejection and iterate. Standard
   per-attempt protocol from AGENTS.md applies (one attempt per
   iteration, log each attempt, escalate after 3 consecutive
   failures on the same obligation).
