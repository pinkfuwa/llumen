#!/usr/bin/env bash
set -euo pipefail
export LC_ALL=C

case "$1" in
  frontend)
    title='Frontend unit test coverage'
    filter='.total | to_entries[] | select(.key == "lines" or .key == "statements" or .key == "functions" or .key == "branches") | [.key, .value.covered, .value.total, .value.pct] | @tsv'
    scope='JavaScript and TypeScript modules; excludes Svelte templates and generated code.'
    ;;
  backend)
    title='Backend unit test coverage'
    filter='.data[0].totals | to_entries[] | select(.key == "lines" or .key == "functions" or .key == "regions") | [.key, .value.covered, .value.count, .value.percent] | @tsv'
    scope='Rust workspace unit tests; excludes xtask.'
    ;;
  *)
    echo 'Usage: coverage-summary.sh frontend|backend REPORT' >&2
    exit 1
    ;;
esac

rows=$(jq -er "$filter" "$2")
printf '## %s\n\n%s\n\n' "$title" "$scope"
printf '| Metric | Coverage | Covered / Total |\n| --- | ---: | ---: |\n'
while IFS=$'\t' read -r metric covered total percent; do
  if [ "$total" -eq 0 ]; then
    percentage='N/A'
  else
    printf -v percentage '%.2f%%' "$percent"
  fi
  printf '| %s | %s | %s / %s |\n' "$metric" "$percentage" "$covered" "$total"
done <<< "$rows"
