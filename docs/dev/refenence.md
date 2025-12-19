# Llumen Quick Reference Guide

**For:** New contributors getting started with the codebase
**Read time:** 5 minutes
**Last updated:** Oct 2025

---

## 🚀 Quick Start

### First Time Setup
1. Read: `overview.md` (5 min)
2. Read: `design.md` Architecture section (10 min)
3. Skim: `backend/src/main.rs` comments (10 min)

### Finding Code
- **Server setup:** `backend/src/main.rs`
- **Error handling:** `backend/src/errors.rs`
- **Chat logic:** `backend/src/chat/context.rs`
- **API endpoints:** `backend/src/routes/*/`
- **Database:** `backend/entity/` and migrations
- **Frontend:** `frontend/src/`

---

## 📋 System Overview (60 seconds)

**Llumen** is a lightweight LLM chat app with 3 modes:
1. **Normal** - Direct LLM chat
2. **Search** - Chat + web search
3. **Deep Research** - Multi-step research with tools

**Tech Stack:**
- Backend: Rust (Axum, SeaORM, Tokio)
- Frontend: Svelte 5 + TypeScript + TailwindCSS
- Database: SQLite + ReDB (blobs)
- LLM API: OpenRouter

**Key Architecture:**
- `AppState` - Shared resources container
- `Context` - Global singleton with tools/DB/LLM client
- `CompletionContext` - Per-request state
- `Pipeline` trait - Processing pattern for chat modes
- Channel-based token streaming for real-time responses

---

## 🏗️ Project Structure

```
backend/
├── src/
│   ├── main.rs              ← Entry point (read first!)
│   ├── errors.rs            ← Error types & conversion
│   ├── chat/
│   │   ├── context.rs       ← Core logic (read this!)
│   │   ├── channel.rs       ← Token pub/sub
│   │   ├── agent/           ← Pipeline trait & ChatPipeline
│   │   ├── agents/          ← Mode implementations (Normal, Search, Deep)
│   │   └── tools.rs         ← Web search, crawl, Lua REPL
│   ├── routes/              ← API endpoints
│   ├── middlewares/         ← Auth, compression, logging
│   └── openrouter/          ← LLM API client
├── entity/                  ← Database schemas
└── migration/               ← Database migrations

frontend/
├── src/
│   ├── lib/
│   │   ├── api/            ← API client
│   │   ├── components/     ← Reusable UI components
│   │   └── store.ts        ← State management
│   └── routes/             ← Page components

Documentation/
├── design.md               ← Full architecture guide
├── overview.md             ← High-level overview
├── refenence.md            ← This file
└── tracing.md              ← Tracing feature guide
```

---

## 💡 Key Concepts (5 min read)

### AppState (Global Container)
```
AppState (Arc<AppState>) contains:
├── conn: Database connection
├── key: Encryption key (PASETO)
├── hasher: Password hashing
├── processor: Chat Context (the main logic)
└── blob: File storage (ReDB)
```
Passed to all route handlers via Axum's `with_state()`.

### Two-Level Context Pattern

**Context (Global/Singleton)**
- Created once at startup
- Lives entire app lifetime
- Shared across all requests
- Contains: database, LLM client, tools, prompts

**CompletionContext (Per-Request)**
- Created for each message via `get_completion_context()`
- Lives until completion saved (ownership passed to Pipeline)
- Tracks: user, chat, model, history, cost, tokens
- Publishes tokens to subscribers via internal publisher

### Token Streaming & Pipeline Pattern
```
Client sends message to /api/message/create
    ↓
Server creates CompletionContext
    ↓
Pipeline::process() called in background task
    ↓
ChatPipeline<T: ChatInner> generates tokens
    ↓
Tokens published to channel via CompletionContext
    ↓
Client subscribes via /api/chat/sse
    ↓
All subscribers receive tokens in real-time
    ↓
Client renders incrementally
```

**Pipeline Trait Pattern:**
- Each mode implements via `ChatInner` trait
- `ChatPipeline<T>` handles LLM streaming & tool execution
- Returns `BoxFuture<'static, anyhow::Result<()>>`

### Error Handling Strategy
```json
{
  "error": "error_kind",
  "reason": "Human description"
}
```

Use trait for conversion:
```rust
operation().kind(ErrorKind::Internal)?
```

---

## 🔍 Understanding Chat Flow

### User Sends Message
1. HTTP POST `/api/message/create`
2. Auth middleware validates token → extracts user_id
3. Handler:
   - Creates user message in database
   - Creates CompletionContext (loads chat, history, model)
   - Returns message IDs immediately
4. Pipeline starts in background task:
   - Set chat mode on completion context
   - Select Pipeline (Normal/Search/Deep)
   - ChatPipeline::new() constructs message history
   - Stream from OpenRouter LLM
   - Publish tokens to channel via CompletionContext
   - Handle tool calls via ChatInner::handoff_tool()
   - Recursively process if tool returns false
   - Save message and chat to database
5. Client subscribes to `/api/chat/sse` for real-time tokens

### Request Headers
```
Authorization: Bearer v4.local.<token_data>
Content-Type: application/json
```

### Response Tokens
```rust
enum Token {
    Start { id, user_msg_id },    // Beginning
    Text(String),                 // LLM output
    ToolCall { tool, input, id }, // Function call
    ToolResult { call_id, result }, // Tool response
    Title(String),                // Chat title
    Error(String),                // Error occurred
    Complete { message_id, cost, token }, // Done
}
```

---

## 🛠️ Common Tasks

### Add New Route Handler
1. Create in `backend/src/routes/domain/handler_name.rs`
2. Add route to `domain/mod.rs`
3. Return `JsonResult<T>` type
4. Extract user_id: Use `Extension(UserId(user_id)): Extension<UserId>`
5. Handle errors: Use `.kind(ErrorKind::Appropriate)?` or `.raw_kind()`
6. Return `Ok(Json(response))`

**Example:**
```rust
pub async fn route(
    State(state): State<Arc<AppState>>,
    Extension(UserId(user_id)): Extension<UserId>,
    Json(req): Json<MyRequest>,
) -> JsonResult<MyResponse> {
    let result = state.conn
        .find_by_id(req.id)
        .one()
        .await
        .raw_kind(ErrorKind::Internal)?
        .ok_or_else(|| Json(Error::not_found()))?;

    Ok(Json(result))
}
```

### Handle Errors Correctly
```rust
// ✅ GOOD: Propagate with context
operation().kind(ErrorKind::Internal)?

// ✅ GOOD: Check None explicitly
item.ok_or_else(|| Json(Error { ... }))?

// ❌ BAD: Unwrap (panics on user input!)
item.unwrap()

// ❌ BAD: Silent discard (loses error info)
let _ = operation()?;
```

### Write Comments
```rust
// ✅ GOOD: Explains WHY
// Arc<Context> enables sharing across async tasks without cloning

// ❌ BAD: Explains WHAT (obvious from code)
// Create Arc<Context>

// ❌ BAD: Every line
let x = 5; // Set x to 5
```

### Add Chat Mode
1. Create `backend/src/chat/agents/mymode.rs`
2. Define `pub struct Inner;`
3. Implement `ChatInner` trait with:
   - `get_system_prompt()` - render prompt
   - `get_model()` - return model config
   - `get_tools()` - return available tools
   - `handoff_tool()` - handle tool execution (if needed)
4. Add type alias in `agents/mod.rs`: `pub type MyMode = ChatPipeline<mymode::Inner>;`
5. Add prompt template `prompts/mymode.md`
6. Update `routes/message/create.rs` to handle new mode
7. Update protocol `ChatMode` enum
8. Document in design.md

### Modify Database
1. Create migration: `backend/migration/src/m[timestamp]_description.rs`
2. Update entity: `backend/entity/src/[table].rs`
3. Test migrations run correctly
4. Update routes to handle new fields
5. Update frontend types (TypeScript)

---

## 🔐 Authentication Flow

**PASETO v4 Tokens** (better than JWT)

```
Login: username + password
    ↓
Hash password with Argon2, compare with stored
    ↓
Generate PASETO token with user_id claim
    ↓
Return token to client
    ↓
Client stores in localStorage
    ↓
Client sends: Authorization: Bearer <token>
    ↓
Middleware validates & extracts user_id
    ↓
Handler gets UserId(user_id) extractor
```

---

## 📊 Database Schema (Quick View)

| Table | Purpose | Key Fields |
|-------|---------|-----------|
| users | User accounts | id, username, password_hash |
| chats | Chat sessions | id, user_id, title, mode |
| messages | Individual messages | id, chat_id, inner (JSON) |
| models | LLM models available | id, name, config (TOML) |
| files | Uploaded files | id, chat_id, blob_id |

---

## ⚙️ Configuration

### Environment Variables
```
API_KEY=your_openrouter_key          # Required
DATA_PATH=.                          # Optional
BIND_ADDR=0.0.0.0:8001              # Optional
STATIC_DIR=../frontend/build         # Optional
```

### Memory Budget (1GB Target)
- SQLite cache: 128MB
- Backend threads: 16MB (4×4MB)
- Heap: 256MB
- Lua runtimes: 512MB (8×64MB)

---

## 📚 Documentation Map

| File | Purpose | Read When |
|------|---------|-----------|
| README.md | Project intro | First time |
| docs/overview.md | High-level overview | Getting started |
| docs/design.md | Full architecture | Need deep understanding |
| docs/refenence.md | This file | Quick lookup |
| docs/tracing.md | Tracing feature | Debugging/observability |
| backend/src/main.rs | Entry point | Understand startup |
| backend/src/errors.rs | Error handling | Learning error flow |
| backend/src/chat/context.rs | Core logic | Understanding chat |
| backend/src/chat/agent/chat.rs | Pipeline pattern | Understanding modes |
| .rules | Project standards | Coding guidelines |

---

## 🧪 Testing

**Unit Tests:**
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_something() {
        let result = function();
        assert_eq!(result, expected);
    }
}
```

**Run:**
```bash
cargo test
```

---

## 🚀 Performance Tips

1. **Memory:** Watch Arc usage, prefer references
2. **Database:** Use indexed queries, check PRAGMA settings
3. **Async:** Use tokio::join! for parallel operations
4. **Streaming:** Tokens aren't buffered (memory safe)
5. **Compression:** Zstd applied to API responses

---

## ❓ FAQ

**Q: Why two contexts?**
A: Separates long-lived shared resources from per-request state.

**Q: How do I add a new field to a message?**
A: Update MessageInner enum in protocol crate, create migration, update routes and converters.

**Q: Where's the LLM API integration?**
A: `backend/src/openrouter/` handles all OpenRouter communication.

**Q: Can two users chat simultaneously?**
A: Yes, different chats are independent. Only one completion per chat at a time.

**Q: How is the 1GB memory enforced?**
A: Careful budgeting + streaming responses instead of buffering.

**Q: Do I need to know Lua?**
A: No, unless working on LuaReplTool. Luau code is executed in sandboxed mlua runtime.

---

## 🎯 First Contribution Checklist

- [ ] Read docs/overview.md for high-level understanding
- [ ] Skim docs/design.md Architecture and Chat Modes sections
- [ ] Review key files (main.rs, errors.rs, context.rs, agent/chat.rs)
- [ ] Set up local environment
- [ ] Find a `good-first-issue` or small feature
- [ ] Create feature branch
- [ ] Follow error handling patterns from errors.rs
- [ ] Add comments explaining "why", not "what"
- [ ] Test your changes
- [ ] Update docs/user.md for user-facing changes
- [ ] Update docs/design.md for architecture changes
- [ ] Submit PR

---

## 🔗 Important Links

- **Repository:** https://github.com/pinkfuwa/llumen
- **OpenRouter:** https://openrouter.ai
- **Axum Docs:** https://docs.rs/axum
- **SeaORM Docs:** https://www.sea-orm.io
- **Svelte 5:** https://svelte.dev/blog/svelte-5-is-here

---

## 💬 Need Help?

1. **Architecture questions:** Check docs/design.md
2. **Pipeline pattern:** Read backend/src/chat/agent/chat.rs
3. **Code understanding:** Read key source files (main.rs, context.rs)
4. **Coding standards:** Check .rules file in project root
5. **Still stuck:** Open an issue or discussion

---

**Welcome to Llumen! Happy coding! 🚀**
