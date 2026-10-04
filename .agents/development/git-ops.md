## Branching rule
- main: manually tested nightly
- release-candidate: candidate for next release
- dev: untested nightly

## How to release
This is a personal project, we run slow release cycle, operate as following:
- Wait for maintainer busy(like final exam), which maintainer would not change as much.
- Create a new branch name `release-candidate`
- Ported any important change back to `release-candidate`

## Conventional Commits

The Conventional Commits specification is a lightweight convention on top of commit messages:
```
<type>: <description>

[optional body]

[optional footer(s)]
[BREAKING CHANGE: `function_name` changed]
```
