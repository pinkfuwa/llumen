# Splitting Strategies — Worked Examples

Read this when a change is too large or mixes concerns and you need to plan a series of commits. Plan the split *before* coding when possible — it's much cheaper than untangling afterward.

## Stacked commits

Commit a small piece, keep working on top of it. Each commit must build and pass tests on its own.

```
1. refactor(orders): extract price calculation into PriceCalculator
2. test(orders): cover PriceCalculator edge cases
3. feat(orders): apply regional tax in PriceCalculator
```

## By files / reviewers

Group files that different people review, or that ship on different schedules.

```
1. feat(proto): add Discount message to order schema
2. feat(orders): apply discounts from order payload
```
The schema commit must land first, but both can be reviewed in parallel. Mention the sibling commit in each body.

Code vs. config:
```
1. feat(search): add fuzzy matching behind flag
2. chore(config): enable fuzzy_search flag in staging
```
Config often deploys faster than code and rolling back just the flag is easy.

## Horizontal (by layer)

Introduce a shared interface or stub so layers can land independently. Bottom-up usually works best:

```
1. feat(model): add Subscription entity
2. feat(service): add SubscriptionService with stubbed billing
3. feat(api): expose POST /subscriptions
4. feat(web): add subscribe button
```

## Vertical (by sub-feature)

Independent full-stack slices, each shippable alone:

```
1. feat(calc): support multiplication
2. feat(calc): support division
```
Shared bits (common validation, button style) go with whichever slice lands first; the second reuses them.

## Horizontal × vertical

Combine both: each cell is its own commit.

| Layer   | Multiplication        | Division                      |
|---------|-----------------------|-------------------------------|
| Client  | add button            | add button                    |
| API     | add endpoint          | add endpoint                  |
| Service | implement operation   | reuse shared operation logic  |
| Model   | add schema definition | add schema definition         |

## Tests-first commits

These independent test changes can land before the main work:

- New tests for existing, untested code — especially before refactoring it, so the refactor provably preserves behavior.
- Refactoring test code (helpers, fixtures).
- New test infrastructure (e.g. an integration-test harness).

```
1. test(parser): add coverage for nested quotes
2. refactor(parser): replace regex tokenizer with state machine
```

## Untangling an existing messy working tree

```bash
git add -p                      # stage hunk by hunk; 's' to split a hunk, 'e' to edit
git diff --cached               # confirm the staged set is exactly one thing
git commit -F- <<'EOF' ... EOF
git stash --keep-index -u       # optional: test the staged state in isolation
# run tests, then: git stash pop
```

Splitting the last commit:
```bash
git reset HEAD~                 # keeps changes in working tree
# re-stage and commit in pieces
```

Splitting an older, unpushed commit: `git rebase -i <base>`, mark it `edit`, then `git reset HEAD~`, commit pieces, `git rebase --continue`.

Never rewrite already-pushed shared history without the user's explicit OK.

## "It can't be split"

Almost always it can. Try, in order:
1. A preparatory refactor-only commit that makes the real change smaller.
2. Landing new code behind a flag, disabled, in several pieces.
3. Asking teammates for a decomposition idea.

If all fail, get reviewers' agreement before sending a large change, and be extra thorough with tests.
