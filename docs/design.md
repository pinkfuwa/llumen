# Llumen Design Document

## Table of Contents

1. [Overview](#overview)
2. [Architecture](#architecture)
3. [Core Components](#core-components)
4. [Data Model](#data-model)
5. [Request Flow](#request-flow)
6. [Chat Modes](#chat-modes)
7. [Key Systems](#key-systems)
8. [Development Guidelines](#development-guidelines)

---

## Overview

**Llumen** is a lightweight LLM chat application designed to run efficiently on 1GB memory systems. It provides users with three distinct chat modes for interacting with various LLM models through the OpenRouter API.

### Key Features

- **Normal Chat**: Direct conversation with selected LLM
- **Search Mode**: Chat with web search integration for current information
- **Deep Research**: Advanced multi-step research with tools (web search, web crawling, code execution)
- **Real-time Streaming**: Tokens streamed to clients as they're generated
- **Tool Integration**: Web search, URL crawling, Lua code execution
- **Multi-user Support**: User accounts with authentication
- **Cost Tracking**: Monitor token usage and API costs

### Technology Stack

**Backend:**
- Rust with Axum web framework
- SeaORM for database abstraction
- Tokio async runtime
- OpenRouter API client
- Lua sandbox for code execution

**Frontend:**
- Svelte 5 (SPA)
- TypeScript
- TailwindCSS
- Vite build tool

**Database:**
- SQLite for persistence
- ReDB for blob storage

---

## Architecture

### High-Level Design

```
┌─────────────────────────────────────────────────────────────┐
│                     Frontend (Svelte)                        │
│  ┌────────────────────────────────────────────────────────┐  │
│  │ Chat UI, Message History, Settings                      │  │
│  └────────────────────────────────────────────────────────┘  │
└──────────────────────┬──────────────────────────────────────┘
                       │ HTTP + SSE
                       │
┌──────────────────────┴──────────────────────────────────────┐
│                  Axum Web Server                             │
│  ┌────────────────────────────────────────────────────────┐  │
│  │ Routes: /api/chat, /api/message, /api/user, /api/file  │  │
│  │ Middleware: Auth, Compression, Logging, Cache Control  │  │
│  └────────────────────────────────────────────────────────┘  │
│                       │                                       │
│  ┌────────────────────┴────────────────────────────────┐     │
│  │                                                      │     │
│  │         ┌──────────────────────────────┐           │     │
│  │         │  Chat Processing Pipeline    │           │     │
│  │         │  (Context)                   │           │     │
│  │         │                              │           │     │
│  │         │ ┌────────────────────────┐   │           │     │
│  │         │ │  Token Publisher       │   │           │     │
│  │         │ │  (Async Channels)      │   │           │     │
│  │         │ └────────────────────────┘   │           │     │
│  │         │                              │           │     │
│  │         │ ┌────────────────────────┐   │           │     │
│  │         │ │  Processing Modes      │   │           │     │
│  │         │ │  - Normal              │   │           │     │
│  │         │ │  - Search              │   │           │     │
│  │         │ │  - Deep Research       │   │           │     │
│  │         │ └────────────────────────┘   │           │     │
│  │         │                              │           │     │
│  │         │ ┌────────────────────────┐   │           │     │
│  │         │ │  Tools                 │   │           │     │
│  │         │ │  - Web Search          │   │           │     │
│  │         │ │  - Web Crawl           │   │           │     │
│  │         │ │  - Lua REPL            │   │           │     │
│  │         │ └────────────────────────┘   │           │     │
│  │         └──────────────────────────────┘           │     │
│  │                       │                            │     │
│  │         ┌─────────────┴─────────────┐              │     │
│  │         │                           │              │     │
│  │    ┌────▼────┐            ┌────────▼──┐           │     │
│  │    │ Database│            │ OpenRouter │           │     │
│  │    │(SQLite) │            │  API       │           │     │
│  │    └─────────┘            └────────────┘           │     │
│  │                                                     │     │
│  │         ┌──────────────────────────────┐           │     │
│  │         │ Blob Storage (ReDB)          │           │     │
│  │         │ - File uploads               │           │     │
│  │         │ - Cached web crawls          │           │     │
│  │         │ - Processing results         │           │     │
│  │         └──────────────────────────────┘           │     │
│  │                                                     │     │
│  └─────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
```

### Memory Management

Llumen is designed to run on systems with approximately 1GB of total memory:

| Component | Size |
|-----------|------|
| SQLite page cache | 128MB |
| Backend threads (4 × 4MB) | 16MB |
| Heap memory | 256MB |
| Lua runtimes (8 × 64MB) | 512MB |
| **Total** | **~1GB** |

Key optimizations:
- MiMalloc allocator for efficient memory use
- SQLite WAL mode for better concurrency
- Streaming token responses (no buffering entire completion)
- Limited Lua runtime instances (max 8 concurrent code executions)

---

## Core Components

### 1. AppState

Located in `src/main.rs`

The central state container that holds all shared application resources. Wrapped in `Arc<AppState>` and passed to all route handlers via Axum's state system.

```rust
pub struct AppState {
    pub conn: DbConn,                        // Database connection pool
    pub key: SymmetricKey<V4>,              // Encryption key for tokens
    pub hasher: Hasher,                     // Password hasher
    pub processor: Arc<Context>,            // Chat pipeline context
    pub blob: Arc<BlobDB>,                  // Binary object storage
}
```

### 2. Context (Global Chat Context)

Located in `src/chat/context.rs`

Singleton context created at server startup, manages:
- Database access
- LLM API client (OpenRouter)
- Token streaming channels
- Tool instances
- Prompt templates

Key methods:
- `new()` - Initialize context with all resources
- `get_completion_context()` - Create per-request context
- `halt_completion()` - Stop an active completion
- `subscribe()` - Listen to token stream for a chat
- `is_streaming()` - Check if completion in progress

### 3. CompletionContext (Per-Request Context)

Located in `src/chat/context.rs`

Created for each LLM completion request. Tracks:
- The user making the request
- The chat and message history
- The model being used
- Accumulated tokens and metadata
- Token publisher for streaming

Lifecycle:
1. Created when user submits message
2. Accumulates tokens as LLM responds
3. Persists results to database
4. Destroyed after save

### 4. Middleware Stack

**Authentication** (`src/middlewares/auth.rs`)
- Validates PASETO tokens in Authorization header
- Extracts user_id from token claims
- Applied to all `/api/*` routes except `/api/auth/*`

**Compression** (`src/middlewares/compression.rs`)
- Zstandard compression for response bodies
- Applied to JSON responses (not files)

**Logging** (`src/middlewares/logger.rs`)
- Logs all requests and responses
- Tracks latency and status codes

**Cache Control** (`src/middlewares/cache_control.rs`)
- Adds appropriate cache headers for static assets
- Version-aware caching for SvelteKit builds

### 5. Routes

Organized by feature:

**Chat** (`src/routes/chat/`)
- `POST /api/chat/create` - Start new chat
- `GET /api/chat/read` - Get chat details
- `DELETE /api/chat/delete` - Delete chat
- `GET /api/chat/list` - Paginated chat list
- `POST /api/chat/message` - Send message and get streaming response
- `POST /api/chat/halt` - Stop active completion

**Message** (`src/routes/message/`)
- `GET /api/message/read` - Get specific message
- `GET /api/message/list` - List messages in chat

**User** (`src/routes/user/`)
- `GET /api/user/profile` - Get user info
- `POST /api/user/update` - Update user settings

**Model** (`src/routes/model/`)
- `GET /api/model/list` - Available LLM models

**File** (`src/routes/file/`)
- `POST /api/file/upload` - Upload file
- `GET /api/file/download` - Download file

**Auth** (`src/routes/auth/`)
- `POST /api/auth/login` - User login
- `POST /api/auth/signup` - User registration
- `POST /api/auth/refresh` - Refresh token

### 6. Error Handling

Located in `src/errors.rs`

All errors follow a consistent pattern:

```json
{
  "error": "error_kind",
  "reason": "Human-readable description"
}
```

Error kinds:
- `unauthorized` - Auth required or failed
- `malformed_token` - Invalid token
- `malformed_request` - Bad request body
- `internal` - Server error
- `login_fail` - Wrong credentials
- `resource_not_found` - 404
- `api_fail` - External API error
- `tool_call_fail` - Tool execution error

The `WithKind` trait enables ergonomic conversion:

```rust
some_operation()
    .kind(ErrorKind::Internal)?
```

---

## Data Model

### Database Schema

**users**
- `id`: Primary key
- `username`: Unique username
- `email`: User email
- `password_hash`: Argon2 hash
- `created_at`: Account creation time

**chats**
- `id`: Primary key
- `user_id`: Foreign key to users
- `title`: Chat topic (auto-generated from first message)
- `mode`: ModeKind (Normal, Search, Deep)
- `created_at`: Chat creation time
- `updated_at`: Last message time

**messages**
- `id`: Primary key
- `chat_id`: Foreign key to chats
- `inner`: MessageInner (JSON)
  - User messages: `{ text, files: [] }`
  - Assistant messages: `{ chunks: [AssistantChunk] }`
  - AssistantChunk types: Text, Error, ToolCall, ToolResult
- `token_count`: Tokens used
- `price`: Cost in USD
- `created_at`: Timestamp

**models**
- `id`: Primary key
- `name`: Model display name
- `provider`: API provider (e.g., "openrouter")
- `config`: TOML configuration
  - max_tokens, temperature, top_p, stop sequences
  - capabilities: tools, vision, streaming
- `pricing`: JSON with input/output costs
- `created_at`: When model was added

**files**
- `id`: Primary key
- `message_id`: Which message includes this file
- `blob_id`: Reference to blob storage
- `filename`: Original filename
- `mimetype`: Content type
- `size`: File size in bytes

**config**
- `id`: String key ("paseto_key", etc.)
- `value`: Configuration value
- Singleton entries for server-wide settings

### Message Format (MessageInner)

User messages:
```rust
MessageInner::User {
    text: String,                    // The actual message
    files: Vec<FileMetadata>,        // Attached files
}
```

Assistant messages:
```rust
MessageInner::Assistant(Vec<AssistantChunk>)

enum AssistantChunk {
    Text(String),                    // Generated text
    ToolCall {                       // Tool invocation
        tool: String,                // "web_search", "crawl", "lua_repl"
        input: serde_json::Value,    // Tool parameters
        id: String,                  // Unique call ID
    },
    ToolResult {                     // Tool result
        call_id: String,
        result: serde_json::Value,
    },
    Error(String),                   // Error during generation
}
```

---

## Request Flow

### 1. Chat Message Request Flow

```
Client sends POST /api/chat/message
    │
    ├─ Verify auth token (middleware)
    │
    ├─ Extract user_id from token
    │
    ├─ Validate request (chat_id, model_id exist)
    │
    ├─ Create CompletionContext
    │    ├─ Load user record
    │    ├─ Load chat + message history
    │    ├─ Load model config
    │    └─ Create new message record
    │
    ├─ Determine chat mode (Normal/Search/Deep)
    │
    ├─ Get appropriate processor
    │    └─ Normal: Direct LLM call
    │    └─ Search: Web search + LLM
    │    └─ Deep: Multi-step research
    │
    ├─ Start token stream subscription
    │    └─ Subscribe to chat's token channel
    │    └─ Return SSE/WebSocket stream
    │
    ├─ (in background) Processor pipeline:
    │    ├─ Render system prompt
    │    ├─ Call OpenRouter API (streaming)
    │    ├─ For each token:
    │    │  ├─ Publish to channel
    │    │  ├─ Add to message
    │    ├─ Process tool calls if needed
    │    ├─ Generate title if needed
    │    ├─ Save message to database
    │    └─ Publish Complete token
    │
    └─ Client receives token stream

Client can POST /api/chat/halt to stop
    └─ Sets halt signal on channel
    └─ Processor stops accepting tokens
```

### 2. Token Streaming

The channel system enables real-time streaming:

```rust
// Publisher side (in processor)
let mut publisher = ctx.channel.publish(chat_id)?;
for token in completion_stream {
    publisher.publish(Token::Text(token))?;
}
publisher.publish_force(Token::Complete { ... });

// Subscriber side (client)
let mut stream = ctx.subscribe(chat_id);
while let Some(token) = stream.next().await {
    // Send to client
}
```

### 3. Database Persistence

After completion finishes:

```
CompletionContext::save()
    │
    ├─ Generate title (LLM call if needed)
    │
    ├─ Update message record
    │   └─ INSERT/UPDATE messages table
    │   └─ Set content, tokens, price, timestamp
    │
    ├─ Update chat record
    │   └─ UPDATE chats table
    │   └─ Set title, updated_at
    │
    └─ Publish Complete token
        └─ Notify all subscribers stream is done
```

---

## Chat Modes

### Normal Mode

Direct conversation with the LLM.

**Flow:**
1. Load chat history
2. Render system prompt
3. Call LLM with messages
4. Stream tokens to client
5. Save message

**System Prompt:** `prompts/normal.md`

### Search Mode

Augments responses with web search results.

**Flow:**
1. Load chat history
2. Web search for user's query
3. Render search system prompt with results
4. Call LLM with search context
5. Stream tokens to client
6. Save message

**System Prompt:** `prompts/search.md`

**Tools:**
- WebSearchTool: Queries search API

### Deep Research Mode

Multi-step research with multiple tools and LLM reasoning.

**Flow:**
1. Load chat history
2. Use Deep Research coordinator prompt
3. Multiple LLM calls with tools:
   - Web search for information
   - Crawl URLs for full content
   - Execute code for analysis
   - Reason over results
4. Return comprehensive response

**System Prompts:**
- `prompts/coordinator.md` - Orchestrates multi-step process
- Various tool prompts

**Tools:**
- WebSearchTool: Search
- CrawlTool: Fetch page content
- LuaReplTool: Code execution

---

## Key Systems

### 1. Prompt Templates

Located in `src/chat/prompt.rs`

Uses Jinja2 templating for rendering system prompts with dynamic context.

Templates:
- `normal.md` - Normal chat system prompt
- `search.md` - Search mode system prompt
- `coordinator.md` - Deep research coordinator
- `title.md` - Title generation

Example rendering:
```rust
let prompt = ctx.prompt.render(
    PromptKind::Normal,
    &completion_context
)?;
```

### 2. Token System

Located in `src/chat/token.rs`

Tokens represent discrete events in the completion lifecycle:

```rust
pub enum Token {
    Start {
        id: i32,              // Message ID
        user_msg_id: i32,     // Last user message ID
    },
    Text(String),            // Text chunk
    ToolCall { ... },        // Tool invocation
    ToolResult { ... },      // Tool result
    Title(String),           // Chat title
    Error(String),           // Error message
    Complete {               // Completion finished
        message_id: i32,
        cost: f32,
        token: i32,
    },
}
```

### 3. Tool Integration

Three main tools available:

**WebSearchTool** (`src/chat/tools/web_search.rs`)
- Queries web search API
- Returns ranked results with titles, URLs, snippets
- Used by Search and Deep Research modes

**CrawlTool** (`src/chat/tools/crawl.rs`)
- Fetches full page content from URL
- Parses and extracts text
- Caches results in blob storage
- Used by Deep Research mode

**LuaReplTool** (`src/chat/tools/lua_repl.rs`)
- Executes Lua code in sandboxed environment
- Limited to 64MB memory per instance
- Max 8 concurrent instances
- Used by Deep Research mode

### 4. OpenRouter Integration

Located in `src/openrouter/`

Abstracts all LLM API interactions:

**completion.rs** - Completion requests
**stream.rs** - Streaming completions
**raw.rs** - Raw API types

Features:
- Streaming response support
- Tool calling
- Error handling and retries
- Cost calculation

### 5. Channel System

Located in `src/chat/channel.rs`

Pub/sub system for token streaming:

```rust
// Publisher (one per chat)
let mut pub = ctx.channel.publish(chat_id)?;
pub.publish(Token::Text("hello"))?;

// Subscribers (many per chat)
let sub = ctx.subscribe(chat_id);
while let Some(token) = sub.next().await { ... }
```

Benefits:
- Multiple clients watch same completion
- Non-blocking publishing
- Halting support
- Memory-efficient (doesn't buffer entire response)

---

## Development Guidelines

### Code Organization

**Do:**
- Put related functionality in the same file (prefer fewer, larger files)
- Use module structure for logical grouping
- Document public APIs with doc comments
- Handle errors explicitly with Result types

**Don't:**
- Create many small single-purpose files
- Use `unwrap()` or `panic!()` - propagate errors instead
- Create `mod.rs` files - use `module_name.rs` instead
- Silently discard errors with `let _ =`

### Error Handling

**Use the `WithKind` trait for ergonomic error conversion:**

```rust
// Good
let user = user::Entity::find_by_id(id)
    .one(&db)
    .await
    .kind(ErrorKind::Internal)?
    .ok_or_else(|| Json(Error { ... }))?;

// Bad
let user = user::Entity::find_by_id(id).one(&db).await.unwrap();
```

**Propagate errors to frontend:**

```rust
// Ensure errors reach UI layer
async fn handler() -> JsonResult<SomeType> {
    let data = operation().kind(ErrorKind::ApiFail)?;
    Ok(Json(data))
}
```

### Async Patterns

**Use variable shadowing for lifetime management:**

```rust
executor.spawn({
    let items = items.clone();
    async move {
        // items is scoped to this async block
        process(items).await
    }
});
```

### Comments

**Only comment non-obvious code:**

```rust
// Good: Explains *why*
let timeout = Duration::from_secs(30);
// Prevent timeout from being too aggressive for slow network

// Bad: Explains *what* (already clear from code)
let timeout = Duration::from_secs(30);  // Set timeout to 30 seconds
```

### Testing

- Unit tests in same file as code
- Integration tests for API routes
- Database tests use migrations
- Mock external services

### Frontend Development

**State Management:**

Use the `localState` helper for persistent state:

```ts
import { localState } from '$lib/store';

export const token = localState<TokenInfo | undefined>(
    'token',
    undefined,
    (data) => data?.expireAt != null  // Validation
);
```

**Queries:**

Manual top-level invocation for sharing queries:

```ts
export function useRoom(id: number): QueryResult<ChatReadResp> {
    return CreateQuery<ChatReadReq, ChatReadResp>({
        key: ['chatRead', id.toString()],
        path: 'chat/read',
        body: { id },
        revalidateOnFocus: false,
        staleTime: Infinity
    });
}
```

---

## Deployment

### Environment Variables

**Required:**
- `API_KEY` - OpenRouter API key
- `DATABASE_URL` - SQLite connection string (optional, defaults to `sqlite://db.sqlite`)
- `BIND_ADDR` - Server address (optional, defaults to `0.0.0.0:8001`)
- `STATIC_DIR` - Frontend build directory path

**Optional:**
- `OPENAI_API_BASE` - Custom API base URL (defaults to OpenRouter)
- `BLOB_URL` - Blob storage path (defaults to `./blobs.redb`)

### Docker

```bash
docker build -f package/Dockerfile -t llumen:latest .
docker run -e API_KEY=your_key -p 8001:8001 llumen:latest
```

### Performance Tuning

1. **Memory:**
   - Reduce Lua runtime instances if memory constrained
   - Monitor SQLite cache usage
   - Tune MiMalloc settings

2. **Concurrency:**
   - Adjust Tokio worker threads
   - Tune connection pool size
   - Monitor channel buffer sizes

3. **Storage:**
   - Use WAL mode for SQLite (configured by default)
   - Regular VACUUM for SQLite maintenance
   - Monitor blob storage growth

---

## Future Enhancements

1. **Persistence:**
   - Conversation export (PDF, JSON)
   - Backup/restore functionality

2. **Collaborative:**
   - Share chats with other users
   - Collaborative editing

3. **Models:**
   - Fine-tuning support
   - Custom model providers

4. **Tools:**
   - Additional tool types (image generation, etc.)
   - Tool marketplace

5. **Observability:**
   - Better metrics and monitoring
   - Usage analytics dashboard

---

## Resources

- [Architecture Decision Records](./adr/)
- [API Documentation](./API.md)
- [Running Locally](./DEV.md)
- [Contributing](./CONTRIBUTING.md)
