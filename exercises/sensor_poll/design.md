# Design — `sensor_poll`

First *composition* exercise. Two prior BFT primitives are composed
into a single end-to-end function whose correctness statement spans
the seam between them.

- `fusion` module — port of `marzullo`: given `n >= 2f+1` sensor
  intervals, return an interval supporting a point covered by at
  least `n - f` inputs.
- `auth` module — distinct-sensor structural check: given a vector
  of `SensorReport { sensor_id, interval }`, return true iff all
  `sensor_id`s are distinct. Shape is the bitmap-backed single-pass
  pattern from `quorum_cert::verify_qc_structure`, simplified to
  one vector and one threshold.
- `main` module — `poll(reports, n, f)`: returns `Some(interval)`
  if the input passes the distinct check and the BFT preconditions
  hold, with a postcondition stated in terms of *report* intervals
  (not the projected intervals); returns `None` if `check_distinct`
  fails.

The new regime this exercise stresses: a composition theorem.
`poll`'s ensures clause references `reports_containing` — a
`Set<int>` of report indices whose interval contains a point. The
proof uses `marzullo`'s postcondition (which is stated in terms of
`intervals_containing` on the *projected* `Seq<Interval>`) and a
small projection lemma showing that
`reports_containing(reports, p) =~= intervals_containing(project(reports), p)`.
The seam is two lines of `assert` in `poll`'s body plus one
extensional-equality lemma.

If the agent solves this in one attempt, the playbook generalises to
system-level composition. If it gets stuck, the sticking points are
data about what the architect playbook is missing for composition
exercises.

---

## 1. Layout

```text
exercises/sensor_poll/
    main.rs     # mod fusion; mod auth; poll(reports, n, f)
    fusion.rs   # marzullo (ported from exercises/marzullo.rs)
    auth.rs     # SensorReport, distinct_sensors, check_distinct
    design.md   # this file
```

Witness mirrors:

```text
exercises/sensor_poll_witness/
    main.rs
    fusion.rs
    auth.rs
```

`main.rs` is the verus entry point. The two siblings `fusion.rs` and
`auth.rs` are declared via `mod fusion;` and `mod auth;`. `auth.rs`
imports `Interval` from `fusion` via `use crate::fusion::Interval;`.

Tested on `/tmp/sensor_poll/` — 18 verified, 0 errors.

---

## 2. Module contracts

### `fusion`

Direct port of the `marzullo` exercise. The frozen spec contains:

- `pub type Reading = i64;`
- `pub struct Interval { pub lo: Reading, pub hi: Reading }` with
  `#[derive(Copy, Clone)]` (needed so `main` can project from
  `SensorReport` to `Interval` without moves).
- `pub uninterp spec fn correct_at(i: int) -> bool;`
- `pub open spec fn` definitions for `well_formed`,
  `point_in_interval`, `intervals_containing`, `correct_indices`,
  `correct_intervals_overlap` (byte-identical to `marzullo.rs`).
- `pub fn marzullo(intervals: &Vec<Interval>, f: u32) -> (result: Interval)`
  with the same requires/ensures as in `marzullo.rs`.

The implementer ports the verified marzullo body from
`exercises/marzullo.rs`. All proof helpers (`count_containing`,
`lemma_max_lo_in_set`, `lemma_exists_supported_lo`, the
subset/finiteness lemmas, the prefix-extend lemma) are agent-authored
— they are not in the frozen spec.

### `auth`

```rust
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
```

The body is a bitmap-backed pass: allocate `Vec<bool>` of length `n`
all false, walk the input; for each `reports[i].sensor_id`, return
false if `seen[id]` is already true, otherwise set `seen[id] = true`
and continue. After the loop, return true.

Pattern source: `quorum_cert::verify_qc_structure` in
`exercises/quorum_cert.rs`. The loop invariant has four conjuncts:
- `seen.len() == n as nat`
- in-range prefix (`reports[k].sensor_id < n` for `k < i`)
- pairwise-distinct prefix (the partial result that becomes the
  postcondition at `i == reports.len()`)
- bitmap abstraction (`seen[s]` is true iff `s` appears among
  `reports[0..i]`'s sensor IDs)

### `main`

```rust
pub open spec fn project_intervals(reports: Seq<SensorReport>) -> Seq<Interval> {
    Seq::new(reports.len(), |i: int| reports[i].interval)
}

pub open spec fn reports_containing(reports: Seq<SensorReport>, p: Reading) -> Set<int> {
    Set::new(|i: int| 0 <= i < reports.len() && point_in_interval(p, reports[i].interval))
}

pub fn poll(reports: &Vec<SensorReport>, n: u32, f: u32) -> (result: Option<Interval>)
    requires
        reports.len() <= u32::MAX as nat,
        reports.len() == n as nat,
        n as nat >= 2 * (f as nat) + 1,
        forall|i: int| 0 <= i < reports.len() ==> reports[i].sensor_id < n,
        well_formed(project_intervals(reports@)),
        correct_indices(reports.len() as nat).len() >= reports.len() as nat - f as nat,
        correct_intervals_overlap(project_intervals(reports@)),
    ensures
        result.is_some() ==> {
            let interval = result.unwrap();
            &&& interval.lo <= interval.hi
            &&& exists|p: Reading|
                interval.lo <= p && p <= interval.hi
                && reports_containing(reports@, p).len()
                    >= reports.len() as nat - f as nat
        },
        result.is_none() ==> !distinct_sensors(reports@),
```

Body:
1. Call `check_distinct(reports, n)`. If false, return `None`.
2. Project: walk `reports`, push each `.interval` into a new
   `Vec<Interval>` whose `@` view equals `project_intervals(reports@)`.
3. Call `marzullo(&intervals, f)`. Get back an `Interval`.
4. Use `marzullo`'s postcondition (existential `p` with
   `intervals_containing(intervals@, p).len() >= n - f`) plus a small
   projection lemma to conclude the same fact for
   `reports_containing(reports@, p)`. Return `Some(interval)`.

The projection lemma is one proof function with empty body:

```rust
proof fn lemma_reports_eq_intervals_containing(reports: Seq<SensorReport>, p: Reading)
    ensures
        reports_containing(reports, p) =~= intervals_containing(project_intervals(reports), p),
{
}
```

The bodies are extensionally equal because `project_intervals(reports)[i] == reports[i].interval`
and both sets are built from the same membership predicate. Verus
closes it with `=~=` alone.

---

## 3. Sub-tasks

1. **Port `marzullo` into `fusion.rs`.** Copy the verified body
   from `exercises/marzullo.rs` (the implementer may read it
   directly). All proof helpers carry over verbatim. End-to-end
   `verus exercises/sensor_poll/main.rs --crate-type=lib` should
   then pass for `fusion`'s contribution alone; the failures
   should be only on `auth::check_distinct` and `main::poll`.
2. **Implement `check_distinct` in `auth.rs`.** Bitmap-backed
   pass, four-conjunct loop invariant. Pattern source: the bitmap
   half of `verify_qc_structure` in `exercises/quorum_cert.rs`.
3. **Implement `poll` in `main.rs`.** Three function calls plus
   the projection lemma. The lemma has an empty body; Verus closes
   it with `=~=`. The body of `poll` calls `check_distinct`, then
   walks the input projecting `interval` field into a fresh
   `Vec<Interval>`, then calls `marzullo`. After `marzullo`, use
   `choose` to extract the witness `p` and instantiate the lemma
   to bridge `intervals_containing` to `reports_containing`.
4. **End-to-end verify.** 18 verified, 0 errors expected. The
   witness gives exactly this number.

---

## 4. Patterns from the playbook that should apply

- **Bitmap-backed structural check** (from `quorum_cert`). Four
  invariant conjuncts; re-establishing the seen-vs-prefix
  abstraction after `seen.set` needs a defensive `assert forall`
  block.
- **`=~=` extensional set equality** (from `ft_midpoint`,
  `marzullo`). The projection lemma is one line plus the empty
  body. Verus closes it from `=~=` alone because both sets share
  a membership predicate up to `reports[i].interval` substitution.
- **`choose` to extract an existential witness, then lemma to
  rewrite** (from `ft_midpoint`'s existential-by-contradiction
  pattern, simplified here — we don't need contradiction, just
  the rewrite). After `marzullo` returns, `choose|p| ...` extracts
  the witness; the lemma converts it from intervals-frame to
  reports-frame.

---

## 5. Anti-patterns

- **Do not weaken `poll`'s precondition** to make verification
  easier (e.g. drop the Helly-1D `correct_intervals_overlap`).
  Marzullo needs it and so does the composition.
- **Do not expose `marzullo`'s helper lemmas as `pub` outside
  `fusion`.** They are proof scaffolding for `marzullo`'s body;
  the composition only needs `marzullo`'s postcondition.
- **Do not state `poll`'s postcondition in terms of
  `project_intervals`** even though it is briefly tempting. The
  caller wants a fact about *their* `Vec<SensorReport>`, not about
  a derived `Seq<Interval>` they did not construct.
