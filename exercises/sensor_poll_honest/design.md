# Design — `sensor_poll_honest`

Third composition exercise — and the first one set up as a
**discovery test** for the methodology. The contract strengthens
`sensor_poll_signed`'s postcondition with a clause asserting the
existence of a *correct, non-Byzantine* sensor whose interval contains
the agreed point. This makes the signature trust boundary load-bearing:
the strengthening is what justifies trusting the output as derived
from at least one honest sensor.

**The methodology question this exercise tests.** Every 1-attempt
exercise since `marzullo` has pre-named its load-bearing proof
construct in the design note. That makes the wins ambiguous — the
agent executed a designed proof rather than discovering one. This
design note deliberately states only the proof obligation and the
informal mathematical content. It does not name the supporting
lemmas, the helper-set constructions, the trigger annotations, or
the sub-proof structure. If the methodology supports discovery, the
agent should reach success without those props. If not, the
iteration count and any escalation cycles are the measurement.

The agent has access (via the architect's playbook accumulated in
`AGENTS.md` and prior exercises' inline comments) to patterns it
has *executed before*. The test is whether it can recognise that
those patterns apply here.

---

## 1. Layout

```text
exercises/sensor_poll_honest/
    main.rs     # mod fusion; mod auth; poll(reports, n, f)
    fusion.rs   # marzullo (verbatim from exercises/sensor_poll_signed/fusion.rs)
    auth.rs    # SensorReport + sig, crypto uninterps, check_distinct
    design.md   # this file
```

Witness mirrors:

```text
exercises/sensor_poll_honest_witness/
    main.rs
    fusion.rs
    auth.rs
```

`main.rs` is the verus entry point. `fusion.rs` and `auth.rs` are
byte-identical to their counterparts in `exercises/sensor_poll_signed/`.

Witness tested with `ralph/check-spec.sh sensor_poll_honest` —
verus verifies, no cheat tokens.

---

## 2. Module contracts

### `fusion` and `auth`

Byte-identical to `exercises/sensor_poll_signed/`. Same frozen specs,
same exec function bodies the implementer must port. The
cryptographic trust boundary (`pk_of` / `signature_valid` /
`report_msg` uninterps, `all_signatures_valid` /
`valid_report_bundle` open spec fns, the `sig: Signature` field on
`SensorReport`) is unchanged.

### `main` — the strengthening

`poll`'s precondition is unchanged from `sensor_poll_signed`. The
`Some`-branch ensures clause gains one new conjunct:

```rust
&&& exists|p: Reading, k: int|
    interval.lo <= p && p <= interval.hi
    && 0 <= k < reports.len()
    && correct_at(k)
    && point_in_interval(p, reports[k].interval)
```

In words: there exists a point `p` in the returned interval AND an
index `k` such that sensor `k` is honest (not Byzantine) AND its
reported interval contains `p`. The existing supporter clause
(`exists p: ... reports_containing >= n - f`) is preserved.

**Why this holds, informally.** `marzullo`'s output is an interval
containing a point `p` supported by at least `n - f` input
intervals. The precondition guarantees that at least `n - f`
sensors are correct. Both sets live inside the universe of `n`
sensor indices. Two large-enough subsets of an `n`-element universe
must overlap — that overlap is a correct sensor whose interval
contains `p`. With `n >= 2f + 1`, the overlap has at least
`n - 2f >= 1` element.

---

## 3. Sub-tasks

1. **Port `marzullo` into `fusion.rs`.** Verbatim copy from
   `exercises/sensor_poll_signed/fusion.rs` (or its parents).
2. **Implement `check_distinct` in `auth.rs`.** Verbatim copy from
   `exercises/sensor_poll_signed/auth.rs`. The new `sig` field on
   `SensorReport` is invisible to the bitmap-backed distinct check.
3. **Implement `poll` in `main.rs`.** The skeleton (`check_distinct`
   → project → `marzullo` → bridge to reports-frame) is byte-equivalent
   to `exercises/sensor_poll_signed/main.rs::poll`. The new work is
   discharging the honest-voter clause: after the existing proof
   establishes the supporter set's size, the implementer must argue
   that this set overlaps with the correct-sensor set. The design
   note deliberately leaves the proof structure unstated — figure it
   out from the contract and the patterns already in the playbook.
4. **End-to-end verify.** Expected: a single `verus exercises/sensor_poll_honest/main.rs --crate-type=lib`
   call exits 0 with the strengthened postcondition discharged.

---

## 4. Anti-patterns

- **Do not weaken the new ensures clause.** The honest-voter
  guarantee is the whole point.
- **Do not introduce an uninterp axiom asserting "there is always
  an honest supporter."** That is the property being proven, not a
  precondition to assume.
- **Do not state `correct_at` as part of an exec function's
  postcondition.** `correct_at` is opaque trust-boundary
  vocabulary; reasoning about it happens at the spec layer.
- **Do not expose `marzullo`'s helper lemmas as `pub` outside
  `fusion`.** Proof scaffolding for `marzullo`'s body.
