# Design — `counter_multifile`

First multi-file Verus exercise in the harness. Same algorithm as
`cross_module_counter` (a bounded counter with `closed spec fn`
encapsulation, used by a client function `count_up_to`), but split
across two files in a directory rather than nested `mod` blocks in a
single file.

The exercise is deliberately not algorithmically harder than
`cross_module_counter`. The variable being tested is the *tooling
regime*: directory layout, multi-file `verus` invocation, hook spec
preservation across files, witness file structure across files.

If the agent succeeds on first attempt — i.e. the algorithm
generalises trivially once the tooling is in place — that's the
expected outcome and the data point is "tooling generalises." If
something breaks, the breakage is most likely on the harness side
(verus invocation, hook, witness check), not in the algorithmic
proof. We learn what we need to fix.

---

## 1. Layout

```text
exercises/counter_multifile/
    main.rs           # entry: declares `mod counter;`, defines client::count_up_to
    counter.rs        # the Counter struct + methods
    design.md         # this file
```

The witness mirrors the structure:

```text
exercises/counter_multifile_witness/
    main.rs           # operator's reference impl, declares `mod counter;`
    counter.rs        # operator's reference impl
```

Verus is invoked on `main.rs`; the `mod counter;` declaration
resolves to `counter.rs` in the same directory. No build system
beyond `verus main.rs --crate-type=lib`. Tested on
`/tmp/multi_test/` — 5 verified, 0 errors.

---

## 2. Module contracts

### `counter::Counter`

Identical to the `mod counter` block in `cross_module_counter.rs`:

- Private fields `value: u32`, `bound: u32`.
- Three `closed spec fn`s: `value()`, `bound()`, `invariant()`.
- Three exec methods: `new(bound)`, `incr(&mut self)`, `get(&self)`.
- Same requires / ensures clauses, byte-identical.

### `main::count_up_to`

Identical to the `mod client` block:

```rust
pub fn count_up_to(target: u32) -> (final_count: u32)
    ensures final_count == target,
```

The body builds a fresh counter, increments it `target` times in a
loop with the four-conjunct invariant `c.invariant() && c.value()
== i && c.bound() == target && i <= target`, then returns
`c.get()`.

---

## 3. Sub-tasks

1. **Counter file skeleton.** Fill `exercises/counter_multifile/counter.rs` with the three exec method bodies (`new` constructs the struct, `incr` increments `self.value`, `get` returns `self.value`). The closed spec fns and the requires/ensures clauses are frozen — do not touch them.
2. **Client body.** Fill `exercises/counter_multifile/main.rs`'s `count_up_to` body with the loop sketched above. The `mod counter;` declaration is already in place; reference the type as `counter::Counter` or via a `use` statement at function scope.
3. **End-to-end verify.** `verus exercises/counter_multifile/main.rs --crate-type=lib` exits 0 with no warnings. `verus` walks both files automatically.

---

## 4. Patterns from the playbook

- **`closed spec fn` + postcondition bridge.** Direct port from `cross_module_counter`. The client never sees `Counter`'s internal fields; all facts come from the function postconditions. The four-conjunct loop invariant is stated entirely in the closed spec-fn vocabulary.
- **`final(self)` syntax** for `incr`'s ensures. Direct port from `bounded_log` + `cross_module_counter`.
- **No new pattern.** This exercise's job is to surface tooling issues, not new proof patterns. Anything novel here likely indicates the implementer is taking a wrong turn.

---

## 5. Anti-patterns

- **Do not declare `mod counter` inside main.rs as an inline module.** That collapses the multi-file structure back to single-file and defeats the purpose. The `mod counter;` declaration must reference the sibling file.
- **Do not change `closed` to `open` on the three spec fns.** Same boundary rule as `cross_module_counter`.
- **Do not commit changes to design.md or to any file outside `exercises/counter_multifile/`.** The agent's tool whitelist scopes edits to the exercise directory; the orchestrator enforces this.
