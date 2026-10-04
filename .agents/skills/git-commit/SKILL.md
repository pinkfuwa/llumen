---
name: git-commit
description: "Plan, split, and write git commits that are small, self-contained, and use semantic commit messages (type(scope): subject — types feat/fix/docs/style/refactor/test/chore, plus build/ci/perf/revert). Use this skill whenever the user asks to commit, stage, write or fix a commit message, review a diff before committing, split a large change into several commits, prepare a PR or changelist, clean up history with rebase/squash, or asks \"what should my commit message be\" — even if they don't say \"semantic\" or \"conventional commits\". Also use it proactively whenever you are about to run `git commit` yourself while coding."
---

# Git commit

Load `technical-writing` too; every body sentence follows it.

## 1. Read before writing

```bash
git status -sb             # "ahead N" = N commits not pushed yet
git diff --stat; git diff --cached --stat
git diff; git diff --cached
git log --oneline -15
```

If `git log`, `commitlint.config.*`, `.gitmessage`, `CONTRIBUTING.md` or a `commit-msg` hook sets a format, use that format. Otherwise use this skill's.

Never amend, rebase or squash a pushed commit without the user's approval.

## 2. Decide the commit boundary

A commit passes every check:

1. The subject fits without "and" or "also".
2. A refactor (rename, move, extract) and a behaviour change are in separate commits, the refactor first.
3. A behaviour change and its tests are in the same commit.
4. The test suite passes at this commit. Run it.
5. A new function or endpoint lands with at least one caller.
6. `scripts/diff_size.sh` reports under 400 changed lines, not counting whole-file deletions. Over 400: split, or tell the user why the change cannot be split.
7. Formatting of unrelated files and dependency bumps are in commits of their own.

To split, read `references/splitting.md`.

## 3. Write the header

`<type>(<scope>): <subject>`. Types are in `references/types.md`.

- `<scope>`: a scope `git log` already uses, or none.
- `<subject>`: imperative ("add", not "added"/"adds"), lowercase, no trailing period.
- Whole header 50 characters or fewer; 72 at most.

## 4. Write the body

An empty body is correct when the diff, the subject and the linked issue say everything.

For each body sentence, write down the question it answers. If the diff, the subject or the linked issue already answers that question, delete the sentence. The body answers only these questions:

| Question | Example |
|---|---|
| What result does the diff not show? | "Unit line coverage goes from 69.82% to 72.13%." |
| What behaviour did the work find and leave unchanged, and does a test assert it? | "An update without `public_at` resets `public_at` to the time of the update. No test asserts the timestamp." |
| What changes outside the diff when this lands? | "Every logged-in user is logged out at deploy." |
| Which approach was rejected, and why? | "A trigger would hide the write from Eloquent events, so the listener does it." |
| Which commit does this one depend on? | "Needs a1b2c3d, which adds the `rank` column." |

The reason for the change is the goal it serves. If the linked issue states the goal, the footer carries it.

Failures from real commits:

| Wrote | Failure | Instead |
|---|---|---|
| "Feature\AdminLanguageControllerTest covers LanguageController through the routes, which the Unit coverage report does not count." | Explains how CI computes coverage, dressed as the reason. The reason is workitem #23, in the footer. | Delete. |
| "The be-unit-test job measures line coverage with the Unit suite only, so the job does not count Feature\AdminLanguageControllerTest." | Explains how a CI job computes a metric. The team knows its own CI. | Delete. |
| "including an update without `public_at`, which passes the stored value back through the setter" | Gives the mechanism and leaves out the result. | "An update without `public_at` resets `public_at` to the time of the update." |
| "because Language::setPublicAtAttribute() reads the stored timestamp as true" | Mechanism after the result is stated. The reader can open the setter. | Delete. |
| "The new tests call each action directly" / "on a LanguageController instance, without routes or middleware" | Describes how the tests are written; the diff shows it. Replacing "directly" with a precise phrase keeps a sentence that should go. | Delete. |
| "including the `sync` option that hands the service to the group members and the `token` option that issues a new token" | Lists test cases the diff shows. | Delete. |

Wrap body lines at 72 columns.

## 5. Write the footer

- Issue: `Refs <project>#<number>` or `Fixes <project>#<number>`.
- Breaking change: `!` before the colon in the header, and `BREAKING CHANGE: <what callers must change>`.
- Attribution lines the session requires, last.

## 6. Check the draft, then commit

Write the message to a file and check it:

- [ ] `python3 scripts/check_commit_msg.py <file>` exits 0.
- [ ] Each body sentence answers a question from the table in section 4.
- [ ] No sentence explains how a tool, CI job, metric or framework works.
- [ ] No sentence describes how the tests are written.
- [ ] Every behaviour the work found and left unchanged is in the body.

Then `git commit -F <file>`, and show the user `git log --oneline -n <count>`. Do not push.

## Example

```
test(admin): add unit tests for LanguageController

Unit line coverage goes from 69.82% to 72.13%.

An update without `public_at` resets `public_at` to the time of the
update. No test asserts the timestamp.

Refs it_devops/syno_net-mgmt#23
```
