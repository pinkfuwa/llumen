---
name: technical-writing
description: "Write precise technical prose in English or Traditional Chinese. Load when you are writing: docblock, inline doc, markdown file, MR description, commit message, GitLab note, review finding, status report, README - and whenever the user calls writing imprecise, badly structured, or hard to read."
---

# Technical writing

## Vocabulary

### Precise

- Never use vague vocabulary. Avoid common vocabulary: file/database/bearing.
  - form is a vague vocabulary: Use `XXX is not readable` instead of `XXX has no readable form`
- Apply adjective word: begin phase/postgres DBMS.
- Never omit vocabulary: In context of OCC, use `in the begin phase` instead of `when begin`.

In case there are just existing problematic umbrella term:
- identify the topology of term.
  - For example, attachment may contain image/pdf/web page, and web page is a bad naming for attachment: $\{image, pdf, web page\}\subset attachment\}$ 
- if such rename will be in-scope, rename it, then inform the user.
- (if not renamed), show parentheses after the name when talking to user: web page(attachment).
- (if not renamed), in MR description or anything colleagues may read, use plain word without parentheses: web page.

### Correctness

Use correct vocabulary:
- lookup dictionary/wikipedia when in question
- database is an organized collection of data or a type of data store based on the use of a database management system: database is not DBMS. 
- schedule is a process of dispatching resource: you don't schedule job, you put job in redis(or other queue service)

| Reject | Use |
|---|---|
| 表 | `table` |
| FK | foreign key |
| 組, for a copy | 取好 / 建立 |
| 組列 | the operation, or no gloss |
| 解析, for a lookup | 查出; a parse names both forms |
| 語句 | SQL 陳述式, with the engine |
| 排, 排程, for enqueue | 寫進 `jobs`; 排程 is the cron |
| 排出 | P-007; it also reads as sorting |
| dispatch, 派出 | P-007; nothing is sent |
| business object | the type, or do not name one |
| 往外拋 | the status code the client gets |
| 例外保護 | the status code; no 保護 / 防護 / 保險 / 守門 |
| 接住 | `catch` |
| cache | the store |
| axis, policy | the two settings |
| holder, guard, load-bearing, backstop, trap | the condition and what it decides |
| branching, ternary | the methods, or the quoted condition |
| bearing, document | the question answered; grep the noun |

## Structure

- Avoid passive sentence: Avoid "It is closed by reviewer because review find the bug acceptable"
- Avoid personification: Use `the concern of closure on XXX is ...` instead of `the closure asks`.
- Use active sentence: "The bug was found to be acceptable, so the reviewer close the review comment on gitLab"
- Put `if`/`when` at front: Use "if the service reject the token due to expiry, call once more" instead of "once
  more with a fresh one if the service reject"
- Use full sentence: Use "the service reject the token due to token expired" insead of "the service reject"
- Name it directly
  - Don't use "isPDFBearing", if you means "does it support PDF", just name it "supportPDF"
  - Avoid pronoun unless the pronoun is immediately followed(within 10 word) by what it is.

## Docuementation 

Whenever you are writing docuement/docblock, read `./documentation.md`.

- only write unintuitive thing, naming should got intuitive covered("supportPDF" function does not need a docblock).
- docblock above method should only contain **API contract**.
- implementation detail stay above code, use **inline comment**.

