## Attempt 1 — 2026-05-18

**Sub-task:** Sub-tasks 1+2 collapsed (skip scaffold-only run; jump straight to smallest body that could close the obligation).

**Approach:** Filled in `lemma_maxphyaddr_facts` body with: (a) call `axiom_max_phyaddr_width_facts()` to gain `32 <= MAX_PHYADDR_WIDTH <= 52`, (b) compute-mode asserts `1usize << 32 == 0x100000000` and `1usize << 52 == 0x10000000000000` to pin the two endpoint values, (c) bit_vector forall asserting monotonicity of left-shift `n <= m < 64 ==> 1usize << n <= 1usize << m`. With those three facts, Verus closes both inequality bounds on `((1usize << MAX_PHYADDR_WIDTH) - 1usize) as usize`.

**Verifier output:** `verification results:: 5 verified, 0 errors`. Exit code 0.

**Next idea:** done — hand to reviewer.
