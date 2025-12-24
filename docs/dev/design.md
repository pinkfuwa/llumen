# Llumen Design Document

> [!IMPORTANT]
> The document was most written by LLM(supervised by human), so it's lengthy.

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
- OpenRouter API client with capability auto-detection
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

## Model Capability Auto-Detection

Llumen automatically detects model capabilities when using OpenRouter, eliminating the need for manual configuration in most cases.

### Architecture

The auto-detection system follows a three-tier priority hierarchy:

1. **User Explicit Setting**: If a user explicitly sets a capability in the model configuration, that value is always respected.
2. **OpenRouter Metadata**: When using OpenRouter and no explicit setting is provided, Llumen queries OpenRouter's model metadata for capabilities.
3. **Default Fallback**: If the model isn't found on OpenRouter or a custom endpoint is used, capabilities default to `true`.

### Implementation

#### Backend Components

**`openrouter::Openrouter`** (`backend/src/openrouter/openrouter.rs`):
- On initialization, spawns a background task to fetch all available models from OpenRouter
- Stores models in a `HashMap<String, raw::Model>` wrapped in `Arc<RwLock<>>`
- Provides `get_model_capabilities(&self, model_id: &str) -> Option<Capabilities>` to query capabilities
- Provides `supports_tools(&self, model_id: &str) -> Option<bool>` for tool calling detection

**`raw::Model`** (`backend/src/openrouter/raw.rs`):
- Contains `supported_parameters: Vec<SupportedParams>` - indicates support for `Tools`, `ResponseFormat`, `StructuredOutput`
- Contains `architecture: Architecture` with `input_modalities` and `output_modalities` - indicates support for `Image`, `Audio`, `File`, `Text`

**`ModelCapabilityWithOpenRouter` trait** (`backend/src/utils/model.rs`):
- Provides unified `get_capability(&self, openrouter: &Openrouter) -> DetectedCapabilities` method
- Returns a `DetectedCapabilities` struct containing all detected capabilities at once
- Implements the three-tier priority logic for all capability fields
- Used by the model list endpoint to determine UI capabilities

**`DetectedCapabilities` struct** (`backend/src/utils/model.rs`):
- Contains all capability fields: `image`, `audio`, `ocr`, `tool`, `json`
- Returned by the unified `get_capability()` method
- Can be converted to `protocol::ModelCapability` via `From` trait for API responses

#### Capability Mapping

| ModelConfig Field | OpenRouter Source | Detection Logic |
|-------------------|-------------------|-----------------|
| `image` | `architecture.input_modalities` contains `Image` | User setting → OpenRouter → default true |
| `audio` | `architecture.input_modalities` contains `Audio` | User setting → OpenRouter → default true |
| `ocr` (documents) | `architecture.input_modalities` contains `File` | User setting → OpenRouter (Native if File supported, else Text) → default Text |
| `tool` | `supported_parameters` contains `Tools` | User setting → OpenRouter → default true |
| `json` | `supported_parameters` contains `StructuredOutput` | User setting → OpenRouter → default true |

#### OCR Mode Detection

The `ocr` field is an enum (`OcrEngine`) rather than a boolean:
- **`Native`**: Model natively supports File modality. Documents are sent directly to the model.
- **`Text`**: Basic text extraction is performed before sending to the model.
- **`Mistral`**: Mistral's OCR service is used to process documents.
- **`Disabled`**: Documents are filtered out and not sent.

When auto-detecting:
- If OpenRouter reports File modality support → `Native`
- Otherwise → `Text` (conservative default that works with most models)

#### Message Filtering

The `Message::to_raw_message()` method (`backend/src/openrouter/openrouter.rs`) now accepts a `capabilities` parameter and filters message content based on detected capabilities:

1. **Image filtering**: If `capabilities.image_input` is `false`, `ImageUrl` message parts are filtered out
2. **Audio filtering**: If `capabilities.audio` is `false`, `InputAudio` message parts are filtered out
3. **Document filtering**: If `capabilities.ocr` is `Disabled`, `File` message parts are filtered out

This ensures requests are always compatible with the target model's capabilities, preventing API errors.

#### Usage in List Endpoint

The `/api/model/list` endpoint (`backend/src/routes/model/list.rs`):
1. Retrieves the OpenRouter instance via `app.processor.get_openrouter()`
2. For each model, calls `config.get_capability(openrouter)` once to get all capabilities
3. Returns capability flags to the frontend for UI state (e.g., graying out unavailable modes)

### User Configuration

Users can override auto-detection by explicitly setting capabilities in model TOML configuration:

```toml
display_name = "Custom Model"
model_id = "provider/model-name"

[capability]
tool = false  # Override: disable tools even if model supports it
# Other capabilities auto-detected from OpenRouter
```

### Compatibility Mode

When using custom API endpoints (non-OpenRouter), the system enters "compatibility mode":
- Model metadata fetching is disabled
- All capabilities default to `true`
- OpenRouter-specific features (plugins, web search) are disabled
- Users must manually configure capabilities if needed

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
│  │         │ - Generated images           │           │     │
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

**Usage:** Injected into route handlers via `State(app): State<Arc<AppState>>` extractor.

### 2. Context (Global Chat Context)

Located in `src/chat/context.rs`

Singleton context created at server startup, manages:
- Database connection (`db`)
- LLM API client (`openrouter`)
- Token streaming channels (`channel`)
- Tool instances (`web_search_tool`, `crawl_tool`, `lua_repl_tool`)
- Prompt templates (`prompt`, `deep_prompt`)
- Blob storage (`blob`)

**Key methods:**
- `new()` - Initialize context with all resources
- `get_completion_context(user_id, chat_id, model_id)` - Create per-request context
- `halt_completion(chat_id)` - Stop an active completion
- `subscribe(chat_id, cursor)` - Listen to token stream for a chat
- `is_streaming(chat_id)` - Check if completion in progress
- `get_model_ids()` - Get available model IDs from OpenRouter

**Lifetime:** Created once at server startup, lives for entire application duration, shared via `Arc<Context>`.

### 3. CompletionContext (Per-Request Context)

Located in `src/chat/context.rs`

Created for each LLM completion request. Tracks:
- The user making the request (`user`)
- The chat and message history (`chat`, `message`, `messages`)
- The model being used (`model`)
- Accumulated tokens and cost (`token_count`, `cost`)
- Token publisher for streaming (internal)

**Key methods:**
- `new(ctx, user_id, chat_id, model_id)` - Create new completion context
- `add_token(token)` - Publish token to stream (checks halt)
- `add_token_force(token)` - Publish token bypassing halt check
- `add_error(msg)` - Add error chunk to assistant message
- `put_stream(stream)` - Consume token stream and publish to channel
- `update_usage(cost, tokens)` - Update cost and token count
- `save()` - Persist message and chat to database
- `get_message_id()` - Get the assistant message ID
- `latest_user_message()` - Get the most recent user message text

**Lifecycle:**
1. Created when user submits message via `get_completion_context()`
2. Passed to Pipeline trait's `process()` method
3. Accumulates tokens as LLM responds
4. Pipeline calls `save()` to persist results to database
5. Ownership transferred to save operation, destroyed after completion

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
- `POST /api/chat/read` - Get chat details
- `POST /api/chat/delete` - Delete chat
- `POST /api/chat/paginate` - Paginated chat list
- `POST /api/chat/write` - Update chat title
- `GET /api/chat/sse` - Subscribe to chat token stream (SSE)
- `POST /api/chat/halt` - Stop active completion

**Message** (`src/routes/message/`)
- `POST /api/message/create` - Send user message and start completion
- `POST /api/message/delete` - Delete specific message
- `POST /api/message/paginate` - List messages in chat (paginated)

**User** (`src/routes/user/`)
- `POST /api/user/read` - Get user profile
- `POST /api/user/write` - Update user settings
- `POST /api/user/delete` - Delete user account

**Model** (`src/routes/model/`)
- `POST /api/model/paginate` - List available LLM models

**File** (`src/routes/file/`)
- `POST /api/file/upload` - Upload file
- `POST /api/file/download` - Download file

**Auth** (`src/routes/auth/`)
- `POST /api/auth/login` - User login with username/password
- `POST /api/auth/renew` - Renew expired authentication token
- `POST /api/auth/header` - Header-based authentication (SSO/proxy integration)

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
- `resource_not_found` - Resource not found
- `unauthorized` - Action not permitted
- `resource_not_found` - 404
- `api_fail` - External API error
- `tool_call_fail` - Tool execution error

The `WithKind` trait enables ergonomic conversion:

```rust
some_operation()
    .kind(ErrorKind::Internal)?
```

---

## Authentication System

Llumen supports two authentication mechanisms:

### 1. Standard Authentication (Username/Password)

**Flow:**
1. User submits login request: `POST /api/auth/login` with username and password
2. Backend validates credentials against the User database
3. Backend generates a PASETO v4 symmetric token containing user_id
4. Frontend stores token and expiry time in localStorage
5. Token is included in Authorization header for subsequent requests

**Token Management:**
- Tokens expire after 7 days
- Frontend automatically renews tokens halfway through their lifetime
- Token renewal: `POST /api/auth/renew` with existing token
- Middleware validates token signature and claims for all protected routes

**Files:**
- Authentication logic: `src/routes/auth/login.rs`, `src/routes/auth/renew.rs`
- Middleware: `src/middlewares/auth.rs`
- Password hashing: Argon2 via `utils/password_hash.rs`

### 2. Header-Based Authentication (SSO/Proxy Integration)

**Purpose:** Allows Llumen to integrate with reverse proxies and SSO systems (Authelia, OAuth2-Proxy, etc.) that handle authentication and inject user information via HTTP headers.

**Flow:**
1. Reverse proxy authenticates the user (external to Llumen)
2. Proxy injects authenticated username into a configured HTTP header
3. When token expires, frontend calls `POST /api/auth/header` with username
4. Backend reads configured header from request
5. If header value matches the requested username, a new token is issued
6. Otherwise, normal login is required

**Configuration:**
- Environment variable: `TRUSTED_HEADER` - HTTP header name containing username (e.g., "X-Remote-User")
- When not set, header auth is disabled
- Users must exist in Llumen's database with matching usernames
- Header name is parsed and cached in AppState during startup (non-blocking to requests)

**Implementation Files:**
- Header auth endpoint: `src/routes/auth/header_auth.rs`
- AppState enhancement: `src/main.rs` - Parses TRUSTED_HEADER env var at startup
- Configuration: Stored as `Option<HeaderName>` in AppState.user_header

**Performance Optimization:**
- TRUSTED_HEADER environment variable is read and parsed exactly once at server startup
- Parsed HeaderName is stored in AppState (non-blocking shared memory)
- Per-request handlers simply reference pre-parsed header name from AppState
- No blocking I/O or parsing happens during request handling

**Security Model:**
- Relies on reverse proxy to correctly authenticate and inject headers
- Should only be used when Llumen is behind a trusted proxy
- Proxy must prevent external clients from spoofing the header
- Falls back to standard login if header auth fails or header not present

**Use Cases:**
- Enterprise deployments with centralized SSO
- Kubernetes clusters with identity provider integration
- Shared hosting with per-tenant authentication middleware
- Reducing password management burden in team environments

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
Client sends POST /api/message/create
    │
    ├─ Verify auth token (middleware)
    │
    ├─ Extract user_id from token
    │
    ├─ Validate request (chat_id, model_id exist)
    │
    ├─ Create user message in database
    │
    ├─ Create CompletionContext
    │    ├─ Load user record
    │    ├─ Load chat + message history
    │    ├─ Load model config
    │    └─ Create new assistant message record
    │
    ├─ Return message IDs immediately
    │
    ├─ (in background task) Pipeline processing:
    │    ├─ Set chat mode on completion context
    │    ├─ Select Pipeline based on mode:
    │    │  └─ Normal: ChatPipeline<normal::Inner>
    │    │  └─ Search: ChatPipeline<search::Inner>
    │    │  └─ Deep: ChatPipeline<deep::Inner>
    │    │
    │    ├─ Pipeline::process() called:
    │    │  ├─ Create ChatPipeline instance
    │    │  ├─ Get system prompt (ChatInner trait)
    │    │  ├─ Get model config (ChatInner trait)
    │    │  ├─ Get available tools (ChatInner trait)
    │    │  ├─ Convert message history to OpenRouter format
    │    │  ├─ Call OpenRouter streaming API
    │    │  ├─ For each token/chunk:
    │    │  │  ├─ Publish to channel
    │    │  │  ├─ Add to assistant message
    │    │  ├─ If tool calls:
    │    │  │  ├─ handoff_tool() (ChatInner trait)
    │    │  │  ├─ Execute tools and add results
    │    │  │  ├─ Recursively call process() if not finalized
    │    │  ├─ On error: add error chunk to message
    │    │  └─ Save message and chat to database
    │    │
    │    └─ Publish Complete token with cost/tokens
    │
    └─ Client subscribes to GET /api/chat/sse for tokens

Client can POST /api/chat/halt to stop
    └─ Sets halt signal on channel
    └─ Pipeline stops publishing tokens
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

Llumen implements three chat modes using a **Pipeline trait pattern** with specialized agent implementations.

### Architecture: Pipeline Trait System

All chat modes implement the `Pipeline` trait defined in `src/chat/agent/mod.rs`:

```rust
pub trait Pipeline {
    fn process(
        ctx: Arc<Context>,
        completion_ctx: CompletionContext,
    ) -> BoxFuture<'static, anyhow::Result<()>>;
}
```

The modes are type aliases over `ChatPipeline<T: ChatInner>` where `ChatInner` defines mode-specific behavior:

```rust
pub type Normal = ChatPipeline<normal::Inner>;
pub type Search = ChatPipeline<search::Inner>;
pub type Deep = ChatPipeline<deep::Inner>;
```

**ChatInner Trait** (`src/chat/agent/chat.rs`):
```rust
pub trait ChatInner {
    fn get_system_prompt(ctx: &Context, completion_ctx: &CompletionContext) -> Result<String>;
    fn get_model(ctx: &Context, completion_ctx: &CompletionContext) -> Result<openrouter::Model>;
    fn get_tools(ctx: &Context, completion_ctx: &CompletionContext) -> Result<Vec<openrouter::Tool>>;
    fn handoff_tool<'a>(
        pipeline: &'a mut ChatPipeline<Self>,
        toolcall: Vec<openrouter::ToolCall>,
    ) -> BoxFuture<'a, Result<bool, anyhow::Error>>;
}
```

**ChatPipeline** handles the core completion loop:
1. Constructs OpenRouter message history
2. Streams tokens from LLM
3. Publishes tokens to channel
4. Handles tool calls via `ChatInner::handoff_tool()`
5. Recursively processes if tool returns `false` (not finalized)
6. Saves message on completion or error

---

### Normal Mode

**Implementation:** `src/chat/agents/normal.rs`

Direct conversation with the LLM without tools.

**ChatInner Implementation:**
- `get_system_prompt()`: Renders `prompts/normal.md`
- `get_model()`: Returns model config from database
- `get_tools()`: Returns empty vec (no tools)
- `handoff_tool()`: Not called (no tools available)

**Flow:**
1. Load chat history
2. Render system prompt
3. Call LLM with messages
4. Stream tokens to client
5. Save message

**System Prompt:** `prompts/normal.md`

---

### Search Mode

**Implementation:** `src/chat/agents/search.rs`

Augments responses with web search and content crawling capabilities.

**ChatInner Implementation:**
- `get_system_prompt()`: Renders `prompts/search.md`
- `get_model()`: Returns model with `online: true` flag
- `get_tools()`: Returns `[web_search_tool, crawl_tool]`
- `handoff_tool()`: Executes tools, adds results to messages, returns `false` to continue

**Flow:**
1. Load chat history
2. Render search system prompt
3. Call LLM with available tools
4. LLM generates response or requests tool use
5. If tool calls:
   - Execute web_search_tool or crawl_tool
   - Add tool results to message history
   - Recursively call LLM with tool results
   - Continue until LLM produces final response
6. Stream tokens to client
7. Save message

**System Prompt:** `prompts/search.md`

**Tools:**
- `web_search_tool`: Search the web using configured search API
- `crawl_tool`: Fetch and parse web page content to markdown

**Features:**
- Online mode enables real-time information retrieval
- Tool results added as both ToolCall and ToolResult chunks
- Multiple tool calls in a single turn supported
- LLM decides when and how to use tools based on user queries

---

### Deep Research Mode

**Implementation:** `src/chat/agents/deep/`

Multi-agent research system with prompt enhancement, planning, step execution, and reporting.

**ChatInner Implementation:**
- `get_system_prompt()`: Renders `prompts/coordinator.md`
- `get_model()`: Returns model config from database
- `get_tools()`: Returns `[handoff_to_planner]` tool
- `handoff_tool()`: Delegates to `DeepAgent::handoff_tool()`, returns `true` (finalizes)

**DeepAgent Structure** (`src/chat/agents/deep/agent.rs`):

The coordinator hands off to `DeepAgent` which orchestrates a multi-phase research process:

**Phase 1: Enhance**
- Takes user's original prompt
- Uses LLM to enhance clarity and add context
- System prompt: `deep_prompt.render_prompt_enhancer()`

**Phase 2: Plan**
- Planner agent creates structured research plan
- Breaks task into actionable steps with descriptions and tools
- Uses JSON schema output if model supports it
- System prompt: `deep_prompt.render_planner()`

**Phase 3: Execute Steps**
- For each step in the plan:
  - Step executor agent processes the step
  - Decides which tools to use (web search, crawl, Lua code)
  - Executes tools and reasons over results
  - Accumulates completed steps with outcomes
- System prompt: `deep_prompt.render_step_executor()`

**Phase 4: Report** (if configured)
- Reporter agent synthesizes all findings
- Generates comprehensive final response
- System prompt: `deep_prompt.render_reporter()`

**Tools Available to Step Executor:**
- `web_search_tool`: Web search
- `crawl_tool`: Fetch and parse URLs
- `lua_repl_tool`: Execute Lua code for data processing

**System Prompts:**
- `prompts/coordinator.md` - Initial coordinator
- Deep prompts managed by `DeepPrompt` (`src/chat/deep_prompt.rs`)

**Key Features:**
- Multi-phase agentic workflow
- Structured planning with step-by-step execution
- Tool use guided by step requirements
- Accumulated context from previous steps
- JSON schema support for structured outputs (when model supports it)
- Graceful error handling with error chunks

**Benefits:**
- Modularizes complex research into phases
- Each phase has specialized prompting and purpose
- Maintains context across multiple LLM calls
- Extensible for additional tools or phases

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

Three main tools are available through the Context:

**WebSearchTool** (`src/chat/tools.rs`)
- Queries web search API (configurable)
- Returns ranked results with titles, URLs, descriptions
- Used by Search and Deep Research modes
- Function: `web_search_tool.search(query) -> Vec<SearchResult>`

**CrawlTool** (`src/chat/tools.rs`)
- Fetches full page content from URL
- Parses and converts HTML to markdown
- Caches results in blob storage
- Used by Search and Deep Research modes
- Function: `crawl_tool.crawl(url) -> String`

**LuaReplTool** (`src/chat/tools.rs`)
- Executes Lua (Luau dialect) code in sandboxed environment
- Limited to 64MB memory per instance
- Max 8 concurrent instances via semaphore
- Used by Deep Research mode
- Function: `lua_repl_tool.execute(code) -> String`

**Tool Definition Functions:**
- `get_web_search_tool_def()` - Returns OpenRouter tool schema for web search
- `get_crawl_tool_def()` - Returns OpenRouter tool schema for crawling
- `get_lua_repl_def()` - Returns OpenRouter tool schema for Lua execution

**Integration Pattern:**
Each mode's `ChatInner::handoff_tool()` implementation receives tool calls from the LLM and:
1. Parses tool arguments (JSON)
2. Executes the appropriate tool via Context
3. Formats results as strings
4. Adds ToolCall and ToolResult chunks to assistant message
5. Publishes tokens for real-time display
6. Adds tool messages to OpenRouter conversation history
7. Returns `false` to continue processing, or `true` to finalize

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

### 6. Blob Storage System

Located in `src/utils/blob.rs`

ReDB-based key-value store for binary data:

**Use cases:**
- User-uploaded files (images, documents)
- Generated images from LLMs
- Cached web crawl results
- Processing artifacts

**Key operations:**
```rust
// Insert blob
blob.insert(file_id, size, byte_stream).await?;

// Retrieve blob
let data = blob.get_vectored(file_id).await?;

// Delete blob
blob.delete(file_id)?;
```

**Image Generation Flow:**
1. LLM returns image in completion result
2. File record created in SQLite `file` table
3. Image data stored in blob storage with file_id
4. `AssistantChunk::Image(file_id)` added to message
5. Frontend fetches via `GET /api/file/image/{id}`

**Parallelization:**
- File loads use `tokio::spawn` for concurrent reads
- Multiple images load in parallel during message conversion
- Reduces latency when re-sending messages with images

### 7. Tracing and Observability

**Non-default Feature: `tracing`**

Enable with: `cargo build --features tracing`

The application supports distributed tracing using the `tracing` crate with `tokio-console` integration. This feature provides detailed observability of async operations throughout the application.

**Key Components:**

1. **Logger Middleware** (`src/middlewares/logger.rs`)
   - Logs incoming HTTP requests with method and URI
   - Logs HTTP response status codes and paths
   - Records timing information when tracing is enabled

2. **Authentication Middleware** (`src/middlewares/auth.rs`)
   - Traces token validation and parsing
   - Records successful authentication with user IDs
   - Helps debug authorization issues

3. **Route Handlers** (`src/routes/chat/*.rs`)
   - Create, read, update operations emit trace events with context
   - SSE subscriptions traced for real-time monitoring
   - Chat operations include user_id and chat_id in spans

4. **Startup Process** (`src/main.rs`)
   - Database initialization phase traced separately
   - Router setup phase tracked for startup debugging
   - Server startup and binding events recorded

**Usage:**

```bash
# Build with tracing feature
cargo build --features tracing

# Run with tracing enabled
RUST_LOG=backend=debug ./backend

# Connect with tokio-console (requires tokio-console CLI tool)
tokio-console
```

**Output:**

Tracing events are emitted to stderr with thread IDs and target module information:
- HTTP requests logged at INFO level
- Authentication events logged at INFO level
- Chat operations logged at INFO level
- Can be filtered using RUST_LOG environment variable

When disabled (default), the application uses standard `log` crate only, keeping the binary size minimal and runtime overhead low.

---

## Development Guidelines

### Code Organization

**Project Structure:**
- `src/chat/agent/` - Core Pipeline trait and ChatPipeline generic
- `src/chat/agents/` - Mode implementations (Normal, Search, Deep)
- `src/chat/agents/deep/` - Deep research multi-agent system
- `src/routes/` - API endpoints organized by resource
- `src/middlewares/` - Cross-cutting concerns
- `src/openrouter/` - LLM API client
- `src/utils/` - Shared utilities
- `entity/` - Database entity definitions (SeaORM)
- `migration/` - Database migrations

**Do:**
- Put related functionality in the same file (prefer fewer, larger files)
- Use module structure for logical grouping
- Document public APIs with doc comments
- Handle errors explicitly with Result types
- Never create `mod.rs` files - use `src/module.rs` instead of `src/module/mod.rs`

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
- `DATA_PATH` - Directory path for data storage (optional, defaults to `.`). Stores `db.sqlite` and `blobs.redb`
- `BIND_ADDR` - Server address (optional, defaults to `0.0.0.0:8001`)

**Optional:**
- `OPENAI_API_BASE` - Custom API base URL (defaults to OpenRouter)
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
