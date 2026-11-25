#!/usr/bin/env bash
# Test of post-merge-skip.sh's docs-only rule (#4054): builds a throwaway repo,
# commits one change set per case, and checks skip (exit 0) or run (exit 1).
# Usage: bash .github/scripts/test-post-merge-skip.sh   (SCRIPT overrides the script)
# Provenance: our own AGENTS.md "The post-merge gate".
set -uo pipefail
here=$(cd "$(dirname "$0")" && pwd)
script="${SCRIPT:-$here/post-merge-skip.sh}"
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

# A fake gh that finds no cut run and no release PR, so only the docs rule decides.
printf '#!/usr/bin/env bash\nexit 0\n' >"$tmp/gh"
chmod +x "$tmp/gh"

repo="$tmp/repo"
git init -q "$repo"
cd "$repo" || exit 1
git config user.email t@example.invalid
git config user.name t
git config commit.gpgsign false
echo base >base.txt
git add . && git commit -qm base

fail=0
# run_case <name> <event> <expect: skip|run> <file>...
run_case() {
  local name=$1 event=$2 expect=$3 f out rc got
  shift 3
  for f in "$@"; do
    mkdir -p "$(dirname "$f")"
    echo "$name" >>"$f"
  done
  git add -A && git commit -qm "$name"
  out=$(EVENT_NAME=$event GH="$tmp/gh" REPO=o/r bash "$script")
  rc=$?
  if [ "$rc" -eq 0 ]; then got=skip; else got=run; fi
  if [ "$got" = "$expect" ]; then
    printf 'ok   %-22s %-17s -> %s %s\n' "$name" "$event" "$got" "$out"
  else
    printf 'FAIL %-22s %-17s -> %s, want %s\n' "$name" "$event" "$got" "$expect"
    fail=1
  fi
}

run_case docs-only push skip docs/a.txt site/index.html README.md
run_case claude-only push skip .claude/skills/x/SKILL.md AGENTS.md
run_case mixed-docs-code push run docs/b.md crates/app/src/lib.rs
run_case github-change push run .github/workflows/post-merge.yml
run_case cargo-lock push run Cargo.lock
run_case xtask-md push run xtask/README.md
run_case crates-md push run crates/app/src/cli_skill.md
run_case assets push run assets/ui/icon.png
run_case scripts push run scripts/gate.sh
run_case build-rs push run crates/app/build.rs
run_case unknown-root-file push run Makefile
run_case docs-only-dispatch workflow_dispatch run docs/c.md
exit "$fail"
