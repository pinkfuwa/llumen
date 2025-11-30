# Llumen Documentation Summary for New Contributors

## Welcome to Llumen! 🎉

This document serves as a quick reference guide for new contributors to understand what documentation has been added and where to find it.

---

## What is Llumen?

**Llumen** is a lightweight LLM (Large Language Model) chat application designed to run efficiently on resource-constrained systems (~1GB memory). It provides users with three distinct chat modes:

1. **Normal Chat** - Direct conversation with an LLM
2. **Search Mode** - Chat augmented with web search and crawling
3. **Deep Research** - Multi-agent research with planning, execution, and reporting (web search, crawling, Lua code execution)

Built with Rust (backend), Svelte 5 (frontend), and integrated with OpenRouter API for LLM access.

---

## Documentation Structure

### 1. **docs/design.md**

**Purpose:** Comprehensive architectural and design documentation

**What's Inside:**
- Executive summary and key characteristics
- High-level system architecture with ASCII diagrams
- Detailed core components explanation (AppState, Context, CompletionContext, Middleware, Routes)
- Data model and database schema
- Complete request flow diagrams
- Explanation of three chat modes with Pipeline trait pattern
- Key systems (prompts, tokens, tools, channels, tracing)
- Development guidelines

**When to Read:**
- Getting familiar with the system architecture
- Understanding how components interact
- Learning about the Pipeline trait pattern and ChatInner trait
- Understanding the three chat modes (Normal, Search, Deep Research)
- Learning about tool integration and OpenRouter API

**Key Sections:**
- Architecture: High-level system design with ASCII diagrams
- Core Components: AppState, Context, CompletionContext, Middleware
- Chat Modes: Pipeline trait pattern and agent implementations
- Key Systems: Token streaming, prompts, tools, channels
- Development Guidelines: Code organization, error handling, testing

---

### 2. **docs/refenence.md** (Quick Reference)

**Purpose:** Quick lookup guide for developers

**What's Inside:**
- Quick start guide
- System overview (60 seconds)
- Project structure overview
- Key concepts summary
- Common development tasks with code examples
- FAQ section

**When to Read:**
- Need quick reference while coding
- Looking for code examples
- Want to understand flow quickly
- Checking common patterns

---

### 3. **docs/tracing.md**

**Purpose:** Tracing feature documentation

**What's Inside:**
- How to enable tracing feature
- Configuration with RUST_LOG
- Integration with tokio-console
- What gets traced (HTTP requests, auth, chat operations, startup)
- Performance considerations

**When to Read:**
- Debugging application issues
- Need observability into async operations
- Analyzing performance bottlenecks
- Setting up development environment

---

### 4. **Key Source Files**

The following core files contain important implementation details:

#### **backend/src/main.rs**
- Application entry point
- AppState structure definition
- Server initialization sequence
- Router configuration
- Middleware stack setup
- Memory optimization notes

**What It Teaches:**
- How the server starts up
- How all components are initialized
- Middleware layering and ordering
- How static assets are served
- Environment variable requirements

#### **backend/src/errors.rs**
- Error types and standardized responses
- WithKind trait for error conversion
- Error kinds and when to use each
- Error propagation strategy

**What It Teaches:**
- How errors flow through the system
- Standard error response format (`{ error, reason }`)
- Error handling best practices
- Client-side error handling expectations

#### **backend/src/chat/context.rs**
- Global Context (singleton pattern)
- CompletionContext (per-request pattern)
- Token streaming flow
- Message persistence flow

**What It Teaches:**
- How chat processing works
- Per-request state management
- Token publishing and subscription
- Database persistence flow
- Tool integration points

#### **backend/src/chat/agent/chat.rs**
- Pipeline trait definition
- ChatInner trait for mode-specific behavior
- ChatPipeline<T> generic implementation
- Tool handling pattern

**What It Teaches:**
- How the Pipeline pattern works
- How different chat modes are implemented
- Tool call execution flow
- Recursive processing for multi-turn tool usage

---

## Key Architectural Concepts

### 1. **Two-Level Context Pattern**

**Global Context (Singleton)**
- Created once at server startup
- Lives for application lifetime
- Shared across all requests via Arc<Context>
- Contains: database, LLM client, tools, prompts

**CompletionContext (Per-Request)**
- Created for each user message
- Lives for duration of completion
- Tracks: user, chat, model, message history
- Publishes tokens to subscribers
- Persists to database when complete

**Why:** Separates global shared state from request-scoped state, enabling clean initialization and lifecycle management.

### 2. **Token Streaming Architecture**

Instead of returning entire response after completion:
1. Client opens WebSocket/SSE subscription to chat
2. Server publishes tokens as they're generated by LLM
3. Client receives tokens in real-time and renders incrementally
4. Stream ends with Complete token containing cost/stats

**Benefits:**
- Instant user feedback (see response appearing)
- Efficient memory usage (no buffering)
- Better perceived performance
- Multiple clients can watch same completion

### 3. **Error Propagation Strategy**

All errors follow this pattern:
```json
{
  "error": "error_kind",
  "reason": "Human-readable description"
}
```

Errors are caught at API boundaries and converted to standard responses using the `WithKind` trait:
```rust
operation().kind(ErrorKind::Internal)?
```

**Why:** Ensures frontend can programmatically handle error categories while having descriptive messages for logging.

### 4. **Three Chat Modes**

| Mode | Purpose | Process | Tools Used |
|------|---------|---------|-----------|
| **Normal** | Direct chat | User message → LLM response | None |
| **Search** | Web-augmented | Web search → LLM with results | Web search |
| **Deep Research** | Multi-step research | Plan → Execute search/crawl/code → Synthesize | All tools |

### 5. **Tool Integration**

Three main tools available:
- **WebSearchTool** - Query web search APIs
- **CrawlTool** - Fetch and parse full page content
- **LuaReplTool** - Execute code in sandboxed Lua environment

Each tool is accessed through the Context and can be called by LLM via tool calling mechanism.

---

## Quick Navigation Guide

### I Want to Understand...

**...how the server starts up**
→ Read `backend/src/main.rs` (annotated)

**...how errors are handled**
→ Read `backend/src/errors.rs` (annotated)

**...how chat completions work**
→ Read `backend/src/chat/context.rs` (annotated)

**...the overall system architecture**
→ Read `DESIGN_DOCUMENT.md` sections: Architecture, Core Components

**...how the three chat modes differ**
→ Read `DESIGN_DOCUMENT.md` section: Chat Modes

**...the database schema**
→ Read `DESIGN_DOCUMENT.md` section: Data Model

**...how to add a new feature**
→ Read `DESIGN_DOCUMENT.md` section: Development Guidelines

**...how authentication works**
→ Read `DESIGN_DOCUMENT.md` section: Key Systems (Authentication)

**...token streaming details**
→ Read `DESIGN_DOCUMENT.md` section: Key Systems (Token System)

---

## Code Reading Order for New Contributors

**Recommended reading path to understand Llumen:**

1. **Start with architecture:**
   - Read: `DESIGN_DOCUMENT.md` - Overview & Architecture sections
   - Time: 10-15 minutes

2. **Understand the entry point:**
   - Read: `backend/src/main.rs` (annotated)
   - Time: 15-20 minutes

3. **Learn error handling:**
   - Read: `backend/src/errors.rs` (annotated)
   - Time: 5-10 minutes

4. **Deep dive into chat processing:**
   - Read: `backend/src/chat/context.rs` (annotated)
   - Time: 20-30 minutes

5. **Explore specific areas of interest:**
   - Routes: `backend/src/routes/*/` for API endpoints
   - Modes: `backend/src/chat/processes/` for processing logic
   - Tools: `backend/src/chat/tools/` for tool implementations
   - Middleware: `backend/src/middlewares/` for cross-cutting concerns

**Total onboarding time:** 1-2 hours for solid understanding

---

## Key Design Principles

These principles guide all code in Llumen:

1. **Correctness Over Performance**
   - Code clarity is paramount
   - Prefer explicit error handling over panics
   - Use type system to prevent bugs

2. **Explicit Error Handling**
   - Never silently discard errors with `let _ =`
   - Use `?` operator to propagate errors
   - Use `WithKind` trait at API boundaries
   - Ensure errors reach frontend users

3. **Memory Efficiency**
   - Target 1GB memory systems
   - Use streaming for responses (no buffering)
   - Prefer Arc for shared ownership
   - Lazy initialize expensive resources

4. **Clear Ownership**
   - Per-request contexts owned by save operation
   - Global resources wrapped in Arc
   - Middleware passes state through type system

5. **Real-time User Experience**
   - Token streaming for instant feedback
   - WebSocket/SSE for push notifications
   - Graceful handling of network issues

---

## Important Files Reference

| Path | Purpose | Key Concept |
|------|---------|-------------|
| `backend/src/main.rs` | Server entry point | AppState setup, router config |
| `backend/src/errors.rs` | Error handling | Standardized error responses |
| `backend/src/chat/context.rs` | Chat processing | Global & per-request contexts |
| `backend/src/chat/channel.rs` | Token streaming | Pub/sub system for tokens |
| `backend/src/chat/processes/` | Mode processors | Normal, Search, Deep Research |
| `backend/src/routes/chat/` | Chat API | HTTP endpoints for chat operations |
| `backend/src/middlewares/auth.rs` | Authentication | PASETO token validation |
| `backend/src/openrouter/` | LLM API | OpenRouter client integration |
| `entity/` | Database schema | SeaORM entity definitions |
| `frontend/src/lib/api/` | API client | TypeScript API integration |

---

## Common Development Tasks

### Adding a New Chat Mode

1. Create processor in `backend/src/chat/processes/my_mode.rs`
2. Implement Pipeline trait with process method
3. Add to process.rs match statement
4. Add prompt template in `prompts/`
5. Update Mode enum in protocol
6. Document in DESIGN_DOCUMENT.md

### Adding a New API Endpoint

1. Create handler in appropriate `routes/` module
2. Use `JsonResult<T>` return type
3. Extract user_id from auth middleware
4. Convert internal errors with `.kind()` trait
5. Update API documentation
6. Test with actual database

### Adding Error Handling

1. Add new ErrorKind variant if needed
2. Use `.kind(ErrorKind::Appropriate)` at boundaries
3. Propagate with `?` operator internally
4. Update error documentation
5. Test frontend error handling

### Adding a Database Migration

1. Create file in `backend/migration/src/`
2. Use SeaORM migration syntax
3. Test with fresh database
4. Update entity definitions in `backend/entity/`
5. Ensure backward compatibility

---

## Performance Characteristics

**Memory Budget (1GB target):**
- SQLite cache: 128MB
- Backend threads: 16MB (4 threads × 4MB each)
- Heap: 256MB
- Lua runtimes: 512MB (8 instances × 64MB each)

**Concurrency Limits:**
- Max 8 concurrent Lua executions
- 4 Tokio worker threads
- One completion per chat at a time

**Streaming:**
- Tokens sent in real-time (no buffering)
- Multiple clients can subscribe to same chat
- Channel-based pub/sub prevents memory buildup

---

## Testing & Quality

**Code Quality Standards:**
- Comments explain "why", not "what"
- All public APIs documented
- Error handling explicit and tested
- No panics on user input
- Memory usage appropriate

**Before Contributing:**
- Test your changes with actual data
- Run the backend and frontend together
- Check error handling paths
- Consider memory impact
- Update documentation

---

## Getting Help

**If you need to understand:**
- **Architecture**: Read docs/design.md
- **Quick reference**: Check docs/refenence.md
- **Error handling**: Read backend/src/errors.rs
- **Chat processing**: Study backend/src/chat/context.rs
- **Pipeline pattern**: Read backend/src/chat/agent/chat.rs
- **API details**: Look in backend/src/routes/
- **Data model**: Check entity/ and docs/design.md
- **Tracing/debugging**: Read docs/tracing.md

**Common Questions:**

Q: Why are there two contexts (global and per-request)?
A: Separates long-lived shared resources (database, tools) from request-scoped state (user, chat, tokens).

Q: How do clients get real-time tokens?
A: Token streaming via pub/sub channel - Pipeline publishes tokens through CompletionContext, clients subscribe via SSE endpoint.

Q: What is the Pipeline trait pattern?
A: A trait-based architecture where each chat mode implements ChatInner trait, and ChatPipeline<T> handles the generic completion flow.

Q: Why use PASETO instead of JWT?
A: PASETO v4 prevents common JWT attacks by design and uses authenticated encryption.

Q: How is memory constrained to 1GB?
A: Careful budgeting of all components plus streaming responses instead of buffering.

Q: Can multiple completions run simultaneously?
A: No, only one per chat (enforced by channel). Different chats can run in parallel.

---

## Next Steps

1. **Read docs/design.md** to understand the big picture and Pipeline pattern
2. **Check docs/refenence.md** for quick reference and common tasks
3. **Review key source files** (main.rs, errors.rs, context.rs, agent/chat.rs)
4. **Clone and run locally** to see the system in action
5. **Pick a small task** and make your first contribution
6. **Follow .rules guidelines** for coding standards

---

## Documentation Structure

```
llumen/
├── docs/
│   ├── design.md                ← Comprehensive architecture doc
│   ├── overview.md              ← This file (high-level guide)
│   ├── refenence.md             ← Quick reference guide
│   ├── tracing.md               ← Tracing feature documentation
│   └── user.md                  ← User-facing documentation
├── backend/src/
│   ├── main.rs                  ← Application entry point
│   ├── errors.rs                ← Error handling system
│   ├── chat/
│   │   ├── context.rs           ← Global & per-request context
│   │   ├── agent/
│   │   │   └── chat.rs          ← Pipeline trait & ChatPipeline
│   │   └── agents/              ← Mode implementations
│   └── routes/                  ← API endpoints
└── .rules                       ← Project coding guidelines
```

---

## Thank You!

Welcome to the Llumen project! The documentation has been organized to help you understand the architecture quickly:

- **Start with this file** (overview.md) for the big picture
- **Read design.md** for detailed architecture and Pipeline pattern
- **Use refenence.md** as a quick lookup while coding
- **Check tracing.md** when you need debugging tools

Feel free to ask questions in issues or discussions if anything is unclear.

Happy coding! 🚀
