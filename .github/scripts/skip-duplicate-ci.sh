#!/usr/bin/env bash
set -euo pipefail

api_headers=(
  --header 'Accept: application/vnd.github+json'
  --header 'X-GitHub-Api-Version: 2022-11-28'
  --header "Authorization: Bearer $GH_TOKEN"
)

pulls=$(curl --fail --silent --show-error --retry 3 "${api_headers[@]}" \
  "$GITHUB_API_URL/repos/$GITHUB_REPOSITORY/commits/$GITHUB_SHA/pulls?per_page=100")

if jq -e --arg branch "$GITHUB_REF_NAME" --arg repo "$GITHUB_REPOSITORY" \
  --arg sha "$GITHUB_SHA" '
    any(.[]; .state == "open" and .head.sha == $sha
      and .head.ref == $branch and .head.repo.full_name == $repo)
  ' <<< "$pulls" >/dev/null; then
  echo 'skip=true' >> "$GITHUB_OUTPUT"
  echo 'An open pull request tests this push.'
  exit 0
fi

if jq -e --arg branch "$GITHUB_REF_NAME" --arg sha "$GITHUB_SHA" '
    any(.[]; .merged_at != null and .base.ref == $branch
      and .merge_commit_sha == $sha)
  ' <<< "$pulls" >/dev/null; then
  commit=$(curl --fail --silent --show-error --retry 3 "${api_headers[@]}" \
    "$GITHUB_API_URL/repos/$GITHUB_REPOSITORY/commits/$GITHUB_SHA")

  if jq -e --arg before "$PUSH_BEFORE" '.parents[0].sha == $before' \
    <<< "$commit" >/dev/null; then
    echo 'skip=true' >> "$GITHUB_OUTPUT"
    echo 'This push contains only a pull request merge.'
  fi
fi
