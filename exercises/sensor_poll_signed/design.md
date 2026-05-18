# Design — `sensor_poll_signed`

Second composition exercise. Extends `sensor_poll` by adding the
*signature-verification half* of `quorum_cert` into the composition.
Same three-module layout, same proof seam, same one-attempt
expectation as `sensor_poll` — the new substance is wiring the
cryptographic trust boundary through the contract.

- `fusion` module — verbatim port of `marzullo`, byte-identical to
  `exercises/sensor_poll/fusion.rs`.
- `auth` module — `sensor_poll`'s distinct-sensor check, **plus** the
  cryptographic trust boundary: `Hash` / `PubKey` / `Signature` type
  aliases, `pk_of` / `signature_valid` / `report_msg` uninterp
  predicates, `all_signatures_valid` over a `Seq<SensorReport>`, and a
  `valid_report_bundle` predicate combining `distinct_sensors` with
  `all_signatures_valid`. `SensorReport` gains a `sig: Signature`
  field; `check_distinct`'s contract is unchanged.
- `main` module — `poll(reports, n, f)` with a new precondition
  `all_signatures_valid(reports@)` and a strengthened `Some`-branch
  ensures requiring `valid_report_bundle(reports@)`. The
  `None`-branch ensures is unchanged (returning `None` still witnesses
  `!distinct_sensors`; signature validity does not affect that path).

What this exercise *does not* do — see `BACKLOG.md`:

- Does not push signature verification into the exec layer. The
  trust boundary lives at the *spec* layer: `all_signatures_valid`
  is a precondition, supplied by the caller (the analog of how a
  real BFT protocol receives already-signed messages from the
  network layer). An exec wrapper around `signature_valid` would
  require `#[verifier::external_body]`, which is not in the
  agent's vocabulary. Adding it is option 1(b)-extended; this
  exercise is the minimum coherent step that brings the signature
  abstraction into the composition theorem.
- Does not invoke the honest-voter pigeonhole lemma. The
  postcondition still talks about `>= n - f` supporters, not "at
  least one honest signed supporter". That strengthening would
  require an inclusion-exclusion lemma analogous to
  `lemma_qc_has_honest_voter`; deferring to keep this in the
  one-attempt regime.

What this exercise *does* do: demonstrate that the cryptographic
trust boundary can be threaded through a multi-module composition
without the exec layer touching it. The agent must understand that
`all_signatures_valid` flows pre → post by virtue of being a
precondition that none of the exec functions disturb, and that
`valid_report_bundle` collapses to a one-line conjunction once
`check_distinct` returns true.

---

## 1. Layout

```text
exercises/sensor_poll_signed/
    main.rs     # mod fusion; mod auth; poll(reports, n, f)
    fusion.rs   # marzullo (verbatim from exercises/sensor_poll/fusion.rs)
    auth.rs     # SensorReport + sig, crypto uninterps, check_distinct
    design.md   # this file
```

Witness mirrors:

```text
exercises/sensor_poll_signed_witness/
    main.rs
    fusion.rs
    auth.rs
```

`main.rs` is the verus entry point. The two siblings `fusion.rs` and
`auth.rs` are declared via `mod fusion;` and `mod auth;`. `auth.rs`
imports `Interval` from `fusion` via `use crate::fusion::Interval;`.

Witness tested with `ralph/check-spec.sh sensor_poll_signed` —
expected: verus verifies, no cheat tokens.

---

## 2. Module contracts

### `fusion`

Byte-identical to `exercises/sensor_poll/fusion.rs` (which is a
verbatim port of `exercises/marzullo.rs`). All spec items — the
`Reading` / `Interval` types, `correct_at`, `well_formed`,
`point_in_interval`, `intervals_containing`, `correct_indices`,
`correct_intervals_overlap`, and `marzullo`'s signature with
requires/ensures — are frozen. The implementer ports the verified
body (the same body that lives in `exercises/sensor_poll/fusion.rs`).

### `auth`

```rust
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
```

The body is the same bitmap-backed pass as in `sensor_poll`. The
only change in the source-of-truth file is `SensorReport`'s extra
`sig: Signature` field, which the loop does not touch. Pattern
source: `quorum_cert::verify_qc_structure`. Four-conjunct loop
invariant. The `pk_of` / `signature_valid` / `report_msg` predicates
are opaque trust-boundary abstractions; the implementer must not
provide a body for them. See AGENTS.md ("Uninterpreted spec
functions and trust boundaries") for the rules.

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
        all_signatures_valid(reports@),
    ensures
        result.is_some() ==> {
            let interval = result.unwrap();
            &&& interval.lo <= interval.hi
            &&& valid_report_bundle(reports@)
            &&& exists|p: Reading|
                interval.lo <= p && p <= interval.hi
                && reports_containing(reports@, p).len()
                    >= reports.len() as nat - f as nat
        },
        result.is_none() ==> !distinct_sensors(reports@),
```

Body shape (same skeleton as `sensor_poll` with one extra assert):

1. Call `check_distinct(reports, n)`. If `false`, return `None`.
   The `None` branch's ensures (`!distinct_sensors(reports@)`) is
   discharged directly from `check_distinct`'s ensures.
2. After `check_distinct` returns `true`, we have
   `distinct_sensors(reports@)`. Combined with the precondition
   `all_signatures_valid(reports@)`, this gives
   `valid_report_bundle(reports@)` (one-line assert).
3. Project: walk `reports`, push each `.interval` into a fresh
   `Vec<Interval>` whose `@` view equals `project_intervals(reports@)`.
4. Call `marzullo(&intervals, f)`. Get back an `Interval`.
5. Use `marzullo`'s postcondition (existential `p` with
   `intervals_containing(intervals@, p).len() >= n - f`) plus the
   projection lemma to conclude the same fact for
   `reports_containing(reports@, p)`. Return `Some(interval)`.

The projection lemma is the same one-line empty-body lemma as in
`sensor_poll`:

```rust
proof fn lemma_reports_eq_intervals_containing(reports: Seq<SensorReport>, p: Reading)
    ensures
        reports_containing(reports, p) =~= intervals_containing(project_intervals(reports), p),
{
}
```

Verus closes it from `=~=` alone: both sets are built from the same
membership predicate, and `project_intervals(reports)[i] == reports[i].interval`.
The `sig` field has no effect on the membership predicate — the
extensional equality is unchanged from `sensor_poll`'s lemma.

---

## 3. Sub-tasks

1. **Port `marzullo` into `fusion.rs`.** Verbatim copy from
   `exercises/sensor_poll/fusion.rs` (or `exercises/marzullo.rs`).
   The implementer may read either freely. All proof helpers
   (`count_containing`, `lemma_max_lo_in_set`,
   `lemma_exists_supported_lo`, the subset/finiteness lemmas, the
   prefix-extend lemma) carry over verbatim. End-to-end
   `verus exercises/sensor_poll_signed/main.rs --crate-type=lib`
   should then pass for `fusion`'s contribution alone; the
   failures should be only on `auth::check_distinct` and `main::poll`.
2. **Implement `check_distinct` in `auth.rs`.** Bitmap-backed
   pass, four-conjunct loop invariant. Verbatim port from
   `exercises/sensor_poll/auth.rs` (the `sig` field on
   `SensorReport` is irrelevant to the distinct check; the loop
   only reads `sensor_id`).
3. **Implement `poll` in `main.rs`.** Same skeleton as
   `exercises/sensor_poll/main.rs::poll`, with one additional
   assert at the start of the success path:
   `assert(valid_report_bundle(reports@));`
   This collapses `distinct_sensors(reports@)` (from
   `check_distinct`) and `all_signatures_valid(reports@)` (from
   the precondition) into the predicate the postcondition asks
   for. The projection lemma and the `choose`/lemma bridge are
   verbatim from `sensor_poll`.
4. **End-to-end verify.** Expected: same `<n> verified, 0 errors`
   shape as `sensor_poll` (16 from `sensor_poll`'s success;
   small delta for any extra inline asserts).

---

## 4. Patterns from the playbook that should apply

- **Threading an uninterp precondition through the body** (new
  here; closest playbook entry is `quorum_cert`'s treatment of
  `signature_valid` as a pure spec predicate). The implementer does
  nothing exec-side; the precondition flows to the postcondition
  because no exec function mutates `reports`.
- **Bitmap-backed structural check** (from `quorum_cert` /
  `sensor_poll`). Four invariant conjuncts; re-establishing the
  seen-vs-prefix abstraction after `seen.set` needs a defensive
  `assert forall` block.
- **`=~=` extensional set equality** (from `ft_midpoint`,
  `marzullo`, `sensor_poll`). The projection lemma is one line
  plus the empty body. The new `sig` field on `SensorReport` does
  not change the membership predicate — the lemma is byte-identical
  to `sensor_poll`'s.
- **`choose` to extract an existential witness, then lemma to
  rewrite** (from `sensor_poll`). After `marzullo` returns,
  `choose|p| ...` extracts the witness; the lemma converts it
  from intervals-frame to reports-frame.

---

## 5. Anti-patterns

- **Do not try to write an exec wrapper for `signature_valid`.**
  The trust boundary is intentionally at the precondition; an exec
  wrapper would require `#[verifier::external_body]`, which is
  denied. The composition is sound *because* the caller is trusted
  to verify signatures upstream.
- **Do not weaken `poll`'s precondition.** `all_signatures_valid`
  is what makes the strengthened postcondition true; dropping it
  would make `valid_report_bundle(reports@)` unprovable in the
  `Some` branch.
- **Do not state `valid_report_bundle` in terms of the projected
  intervals.** The caller's facts are about their original
  `Vec<SensorReport>`, not a derived `Seq<Interval>`. The
  predicate lives in `auth.rs` and reads `reports[i].sensor_id`
  and `reports[i].sig` directly.
- **Do not expose `marzullo`'s helper lemmas as `pub` outside
  `fusion`.** Proof scaffolding for `marzullo`'s body; the
  composition only needs `marzullo`'s postcondition.
