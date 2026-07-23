#!/usr/bin/env bash
# Enforce the project's positioning and style rules on published copy.
#
# Used by both `just check-copy` and the CI workflow, so the two cannot drift.
#
# A line may legitimately mention a banned phrase in order to prohibit it (the
# README's hard-rules section does exactly that). Such lines carry the marker
# `copy-rules:allow` and are skipped. The marker is an HTML comment in markdown,
# so it stays invisible in the rendered page.

set -uo pipefail

MARKER="copy-rules:allow"

files=$(git ls-files 'docs/*.md' 'site/src/**' 'README.md' \
  | grep -v 'KICKOFF-v1-superseded' \
  | grep -v '^docs/research/' || true)

if [ -z "$files" ]; then
  echo "check-copy: no files to check"
  exit 0
fi

fail=0

# Strip allow-marked lines before checking, so a prohibition is not a violation.
scan() {
  # shellcheck disable=SC2086
  grep -n "$@" $files 2>/dev/null | grep -v "$MARKER" || true
}

hits=$(scan -P '\x{2014}')
if [ -n "$hits" ]; then
  echo "FAIL: em dash found. House style is hyphens only."
  echo "$hits"
  fail=1
fi

# Phrases that must never appear as assertions.
for phrase in "world's first" "first ever" "quantum-proof" "quantum proof" "unbreakable"; do
  hits=$(scan -i -- "$phrase")
  if [ -n "$hits" ]; then
    echo "FAIL: banned phrase '$phrase'."
    echo "$hits"
    fail=1
  fi
done

# The specific false claim the literature check refuted. See docs/LITERATURE.md.
for claim in \
  "first post-quantum signature verification in a STARK" \
  "first PQ signature verification in a STARK"
do
  hits=$(scan -i -- "$claim")
  if [ -n "$hits" ]; then
    echo "FAIL: that claim is false, ePrint 2021/1048 predates it."
    echo "$hits"
    fail=1
  fi
done

if [ "$fail" -eq 0 ]; then
  echo "check-copy: ok ($(echo "$files" | wc -l | tr -d ' ') files)"
fi

exit $fail
