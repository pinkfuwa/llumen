## Branching rule

- `main`: latest release; documentation is deployed from this branch to GitHub Pages.
- `release-candidate`: candidate for the next release.
- `dev`: ongoing development (untested nightly); create contribution branches from here and target pull requests here, including documentation-only changes.

See [CONTRIBUTING.md](../../CONTRIBUTING.md#pull-requests) for contributor guidance.

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
