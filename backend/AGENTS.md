# Backend AGENTS

Backend is built with Rust, Axum, SeaORM, and mLua with luau sandboxing.

## Rust Coding Guidelines

- Prioritize code correctness and clarity. Speed and efficiency are secondary priorities unless otherwise specified.
- Do not write organizational or comments that summarize the code. Comments should only be written in order to explain "why" the code is written in some way in the case there is a reason that is tricky / non-obvious.
- Prefer implementing functionality in existing files unless it is a new logical component. Avoid creating many small files.
- Avoid using functions that panic like `unwrap()`, instead use mechanisms like `?` to propagate errors.
- Be careful with operations like indexing which may panic if the indexes are out of bounds.
- Never silently discard errors with `let _ =` on fallible operations. Always handle errors appropriately:
  - Propagate errors with `?` when the calling function should handle them
  - Use `.log_err()` or similar when you need to ignore errors but want visibility
  - Use explicit error handling with `match` or `if let Err(...)` when you need custom logic
  - For error caused by user's bad request body, send the error by return `Json<Error>`
  - For error caused by false output of LLM, place insert chunk to top-level message.
- Never create files with `mod.rs` paths - prefer `src/some_module.rs` instead of `src/some_module/mod.rs`.
- Use full words for variable names (no abbreviations like "q" for "queue")
- Always use `cargo add` to add new dependency, which should automatically choose latest version
- Use variable shadowing to scope clones in async contexts for clarity, minimizing the lifetime of borrowed references.
- Always follow separation of concerns principle

Example for async variable shadowing:
```rust
executor.spawn({
    let task_ran = task_ran.clone();
    async move {
        *task_ran.borrow_mut() = true;
    }
});
```

## Database

We use sea-orm without entity-codegen. When modifying migration files, ensure they are compatible with the current database schema. Don't generate entity-codegen.

## Agent and Subagent

There are three agents: normal/search/deep-research

The **subagent pattern** is the superset of simple tool for handling complex, multi-step tasks in LLM applications.

To create new subagent:
1. Define a tool in `get_tools` with a unique name and schema.
2. On tool call, invoke `handoff_tool`, passing the pipeline and tool arguments.
3. Use mutable `CompletionContext` for all state updates.
4. On error, append an **error chunk** to the assistant output.
5. When sub-agent finished its job, return true if agent takes the final output, or return false to trigger next completion (common pattern for simple tool call).
6. Create/modify agent to wire subagent to agent

## Using cargo xtask

Run commands from the `backend` directory:

```bash
cd backend
cargo xtask <command>
```

Available commands:
- `build-frontend` - Build frontend via pnpm
- `build` - Build frontend + cargo release
- `run` - Run backend with dev features
- `run-with-build` - Build frontend then run
- `fresh` - Run migration fresh
- `refresh` - Run migration refresh
- `gen-ts` - Run typeshare codegen
- `gen-entity` - Generate SeaORM entities
- `gen-license` - Generate THIRDPARTY.toml
