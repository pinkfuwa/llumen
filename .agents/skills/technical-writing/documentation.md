# Documentation

## When the documentation is needed

- there is no obvious naming.
- the design is un-intuitive.
- the implementation is complex.

## Levels

- Markdown: complex concept
- Module: the concept, and the umbrella term for it
- Class: that class's responsibility
- Method: that method's responsibility and its contract
- Inline: a detail about the line below it

A fact belongs at exactly one level.

Pick the level by the scope of what the fact governs:
- a concept for entire codebase is markdown level.
- a concept spanning several classes is module level.
- a constraint that only holds for one statement is inline.

## Single source of truth

A definition appears once, other places reference it. Both of these are correct:

```pseudo
// article categorized by XXX column(timestamp):
// 1. stale: see stale
// ...
class TimeController{
    // stale: behind for >5days
    method stale(): bool{
    }
}
```

```pseudo
// article categorized by XXX column(timestamp):
// 1. stale: behind for >5days
// ...
class TimeController{
    method stale(): bool{
    }
}
```

## Example

### Inline

The reasoning behind a isolation level proceed immediately before the statement that opens the transaction.

```pseudo
// because <the reason this isolation level>, <why not REPEATABLE READ>
DB::beginTransaction();
```

### Intuitive

For intuitive naming, you should leave no comment. Best documentation is no documentation!

Don't do this:

```pseudo
class ArticleModel{
    // the version used in OCC validation phase.
    timestamp content_version;
}
```

Reason:
- The type already say it's a timestamp.
- The name already say it's a version.
  - every backend developers know optimistic concurrency control.

