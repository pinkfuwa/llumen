#!/usr/bin/env bash
# Summarize the size of a pending change against small-commit guidance.
# Usage: diff_size.sh            # staged changes
#        diff_size.sh --all      # staged + unstaged vs HEAD
#        diff_size.sh <rev-range> # e.g. main..HEAD
set -euo pipefail
case "${1:-}" in
  "")     args=(--cached) ;;
  --all)  args=(HEAD) ;;
  *)      args=("$1") ;;
esac

stats=$(git diff --numstat "${args[@]}")
if [ -z "$stats" ]; then echo "No changes."; exit 0; fi

files=$(echo "$stats" | wc -l | tr -d ' ')
added=$(echo "$stats" | awk '$1!="-"{s+=$1} END{print s+0}')
deleted=$(echo "$stats" | awk '$2!="-"{s+=$2} END{print s+0}')
# whole-file deletions are cheap to review; count them separately
fulldel=$(git diff --diff-filter=D --numstat "${args[@]}" | awk '{s+=$2} END{print s+0}')
effective=$((added + deleted - fulldel))

echo "files: $files  +$added -$deleted  (whole-file deletions: $fulldel lines, discounted)"
echo "effective changed lines: $effective"
if   [ "$effective" -le 150 ] && [ "$files" -le 10 ]; then echo "size: small — good"
elif [ "$effective" -le 400 ] && [ "$files" -le 20 ]; then echo "size: medium — fine if it's one self-contained change"
else echo "size: large — look for a split (refactor first? tests first? by layer or sub-feature?)"
fi
