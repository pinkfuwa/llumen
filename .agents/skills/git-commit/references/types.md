# Commit types

| Type | Use for |
|---|---|
| `feat` | New behaviour a user of the product sees |
| `fix` | Bug fix a user of the product sees |
| `docs` | Documentation only |
| `style` | Formatting only; no behaviour change |
| `refactor` | Production code restructured; no behaviour change |
| `test` | Tests added or changed; no production code change |
| `chore` | Tooling, dead-code removal, dependency bumps; no production behaviour change |
| `build` | Build system or external dependencies |
| `ci` | CI configuration and scripts |
| `perf` | Performance improvement |
| `revert` | Reverts an earlier commit |

A `feat` or `fix` commit contains its tests; `test` is for commits that change tests only.
