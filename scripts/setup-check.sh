#!/usr/bin/env bash
# Friday-evening environment check for the verus-calibration weekend.
#
# Runs every prerequisite the Saturday loop depends on. Fails fast if any
# piece is missing. Safe to run repeatedly — no side effects on the repo.

set -u

cd "$(dirname "$0")/.."

pass=0
fail=0
tmp=$(mktemp)
trap 'rm -f "$tmp"' EXIT

check() {
  local name="$1"
  local cmd="$2"
  printf '  [%-38s] ' "$name"
  if eval "$cmd" >"$tmp" 2>&1; then
    printf 'PASS\n'
    pass=$((pass + 1))
  else
    printf 'FAIL\n'
    sed 's/^/      /' "$tmp"
    fail=$((fail + 1))
  fi
}

echo
echo "verus-calibration setup check"
echo "============================="

echo
echo "Toolchain:"
check "verus on PATH"                "command -v verus"
check "verus --version runs"         "verus --version"
check "claude on PATH"               "command -v claude"
check "claude --version runs"        "claude --version"
check "git on PATH"                  "command -v git"
check "git --version runs"           "git --version"

echo
echo "Repo state:"
check "inside git repo"              "git rev-parse --git-dir"
check "AGENTS.md present"            "test -f AGENTS.md"
check "ORCHESTRATION.md present"     "test -f ORCHESTRATION.md"
check "README.md present"            "test -f README.md"
check "subagent: architect"          "test -f .claude/agents/architect.md"
check "subagent: implementer"        "test -f .claude/agents/implementer.md"
check "subagent: reviewer"           "test -f .claude/agents/reviewer.md"
check "exercise: binary_search"      "test -f exercises/binary_search.rs"
check "exercise: bounded_log"        "test -f exercises/bounded_log.rs"
check "exercise: quorum_count"       "test -f exercises/quorum_count.rs"
check "verify.sh executable"         "test -x scripts/verify.sh"
check "logs dir: binary_search"      "test -d logs/binary_search"
check "logs dir: bounded_log"        "test -d logs/bounded_log"
check "logs dir: quorum_count"       "test -d logs/quorum_count"
check "writeup/outline.md present"   "test -f writeup/outline.md"
check "writeup/results_template.md"  "test -f writeup/results_template.md"

echo
echo "Subagent frontmatter sanity:"
check "architect has model field"    "grep -q '^model:' .claude/agents/architect.md"
check "implementer has model field"  "grep -q '^model:' .claude/agents/implementer.md"
check "reviewer has model field"     "grep -q '^model:' .claude/agents/reviewer.md"
check "architect model is Opus 4.7"  "grep -q '^model: claude-opus-4-7' .claude/agents/architect.md"
check "implementer is Sonnet 4.6"    "grep -q '^model: claude-sonnet-4-6' .claude/agents/implementer.md"
check "reviewer model is Opus 4.7"   "grep -q '^model: claude-opus-4-7' .claude/agents/reviewer.md"

echo
echo "Frozen baseline tags:"
check "tag: spec-frozen-binary_search" "git rev-parse --verify --quiet spec-frozen-binary_search"
check "tag: spec-frozen-bounded_log"   "git rev-parse --verify --quiet spec-frozen-bounded_log"
check "tag: spec-frozen-quorum_count"  "git rev-parse --verify --quiet spec-frozen-quorum_count"

echo
echo "Verus end-to-end (verifies a tiny example):"
sanity_dir=$(mktemp -d)
trap 'rm -rf "$sanity_dir"; rm -f "$tmp"' EXIT
cat >"$sanity_dir/sanity.rs" <<'EOF'
use vstd::prelude::*;

verus! {

fn add_one(x: u32) -> (result: u32)
    requires x < u32::MAX,
    ensures result == x + 1,
{
    x + 1
}

} // verus!
EOF
check "verus verifies tiny example"  "verus --crate-type=lib '$sanity_dir/sanity.rs'"

echo
echo "Working tree:"
check "no uncommitted changes"       "git diff --quiet && git diff --cached --quiet"

echo
echo "Summary"
echo "-------"
echo "  passed: $pass"
echo "  failed: $fail"
echo

if [ "$fail" -gt 0 ]; then
  cat <<'HINTS'
Not ready. Common fixes:

  - verus missing: build from source per https://github.com/verus-lang/verus
    (needs the specific Rust nightly pinned in the Verus README)
  - claude missing: npm install -g @anthropic-ai/claude-code
  - tags missing: from the repo root,
        git add -A && git commit -m "initial scaffold"
        git tag spec-frozen-binary_search HEAD
        git tag spec-frozen-bounded_log HEAD
        git tag spec-frozen-quorum_count HEAD
  - uncommitted changes: commit the scaffold first so the tags pin a known state
  - tiny example fails: Verus toolchain is broken — re-check the Rust nightly
    and that vstd is on the verus search path
HINTS
  exit 1
fi

cat <<'NEXT'
Ready. Saturday morning is go.

Suggested first command from the repo root:

  claude "Begin work on exercises/binary_search.rs following ORCHESTRATION.md.
          Drive the state machine: architect, then implementer, then reviewer.
          Stop after binary_search is either DONE or BLOCKED."

After that finishes, run the same command with bounded_log, then quorum_count.
NEXT
