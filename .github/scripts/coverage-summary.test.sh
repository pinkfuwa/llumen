#!/usr/bin/env bash
set -euo pipefail

script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
fixtures=$(mktemp -d)
trap 'rm -rf "$fixtures"' EXIT

cat > "$fixtures/frontend.json" <<'JSON'
{"total": {
  "lines": {"covered": 5, "total": 6, "pct": 83.33},
  "functions": {"covered": 0, "total": 0, "pct": 100},
  "branchesTrue": {"covered": 0, "total": 0, "pct": 100}
}}
JSON
cat > "$fixtures/backend.json" <<'JSON'
{"data": [{"totals": {
  "lines": {"covered": 5, "count": 6, "percent": 83.333333333},
  "functions": {"covered": 0, "count": 0, "percent": 0},
  "regions": {"covered": 1, "count": 4, "percent": 25},
  "instantiations": {"covered": 1, "count": 4, "percent": 25}
}}]}
JSON

frontend=$(bash "$script_dir/coverage-summary.sh" frontend "$fixtures/frontend.json")
[[ "$frontend" == *'## Frontend unit test coverage'* ]]
[[ "$frontend" == *'| lines | 83.33% | 5 / 6 |'* ]]
[[ "$frontend" == *'| functions | N/A | 0 / 0 |'* ]]
[[ "$frontend" != *'branchesTrue'* ]]

backend=$(bash "$script_dir/coverage-summary.sh" backend "$fixtures/backend.json")
[[ "$backend" == *'## Backend unit test coverage'* ]]
[[ "$backend" == *'| lines | 83.33% | 5 / 6 |'* ]]
[[ "$backend" == *'| functions | N/A | 0 / 0 |'* ]]
[[ "$backend" == *'| regions | 25.00% | 1 / 4 |'* ]]
[[ "$backend" != *'instantiations'* ]]

printf '{}\n' > "$fixtures/empty.json"
printf 'invalid json\n' > "$fixtures/invalid.json"
for format in frontend backend; do
  for report in empty invalid missing; do
    if bash "$script_dir/coverage-summary.sh" "$format" "$fixtures/$report.json" > /dev/null 2>&1; then
      echo "Expected failure for $format $report report" >&2
      exit 1
    fi
  done
done

if bash "$script_dir/coverage-summary.sh" unknown "$fixtures/frontend.json" > /dev/null 2>&1; then
  echo 'Expected failure for unknown report format' >&2
  exit 1
fi

echo 'Coverage summary tests passed.'
