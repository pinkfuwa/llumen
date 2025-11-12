# Tracing Feature Guide

## Overview

Llumen includes an optional `tracing` feature that provides detailed observability into application behavior using the Rust `tracing` crate and `tokio-console` integration. The integration is powered by `console_subscriber`, which enables seamless connection to the `tokio-console` tool for async task and span inspection. This feature is **disabled by default** to keep the binary size minimal and avoid runtime overhead in production builds.

## Building with Tracing

### Enable the Feature

```bash
# Build with tracing feature enabled
cargo build --features tracing

# Release build with tracing
cargo build --release --features tracing
```

### Default Build (without tracing)

```bash
# Standard build - minimal size, no tracing overhead
cargo build
cargo build --release
```

## Configuration

### Environment Variables

The tracing system respects the `RUST_LOG` environment variable for controlling log levels:

```bash
# Show all backend logs at info level
RUST_LOG=backend=info ./backend

# Show debug-level logs
RUST_LOG=backend=debug ./backend

# Show trace-level logs (very verbose)
RUST_LOG=backend=trace ./backend

# Show logs from specific modules
RUST_LOG=backend::routes=debug,backend::middlewares=info ./backend
```

## What Gets Traced

### 1. HTTP Request Middleware

**File:** `src/middlewares/logger.rs`

- **Incoming Requests:** Logs HTTP method and URI for all incoming requests
- **Responses:** Records HTTP status codes and paths for POST requests
- **Format:** `incoming http_request method=GET uri=/api/chat/read`

Example trace event:
```
17:45:23.123| INFO | incoming http_request method=POST uri=/api/chat/sse
17:45:23.245| INFO | api request completed status=200 path=/api/chat/sse
```

### 2. Authentication

**File:** `src/middlewares/auth.rs`

- **Token Validation:** Traces token parsing and validation
- **Authentication Success:** Records successful auth with user ID
- **Format:** `user_id=42 authentication successful`

Example trace event:
```
17:45:23.125| INFO | authentication successful user_id=42
```

### 3. Chat Operations

**Files:** `src/routes/chat/*.rs`

#### Create Chat
- Records user_id and chat mode
- Format: `user_id=42 mode=Normal creating chat`

#### Read Chat
- Records user_id and chat_id
- Format: `user_id=42 chat_id=5 reading chat`

#### Update Chat
- Records user_id and chat_id
- Format: `user_id=42 chat_id=5 updating chat`

#### SSE Subscription
- Records user_id and chat_id for streaming subscriptions
- Format: `user_id=42 chat_id=5 subscribing to chat events`

### 4. Startup Phases

**File:** `src/main.rs`

The application startup is instrumented with multiple phases:

1. **Backend Startup** (`llumen_backend_startup`)
   - Main startup span encompassing all initialization

2. **Database Initialization** (`database_initialization`)
   - Migration execution
   - Database connection setup
   - PRAGMA configuration

3. **Router Setup** (`router_setup`)
   - Route registration
   - Middleware configuration
   - State initialization

4. **Server Startup** (`server_startup`)
   - TCP listener binding
   - Server listening on address

Example startup sequence:
```
17:45:20.001| INFO | incoming http_request method=GET uri=/
17:45:20.050| INFO | Listening on http://0.0.0.0:8001
17:45:20.051| INFO | tracing initialized
```

## Trace Output Format

Tracing output follows this format when enabled:

```
HH:MM:SS.sssZ LEVEL| message field1=value1 field2=value2
```

Example with multiple fields:
```
17:45:23.123| INFO | authentication successful user_id=42
17:45:23.125| INFO | creating chat user_id=42 mode=Normal
17:45:23.245| INFO | api request completed status=200 path=/api/chat/create
```

## Using with tokio-console

The tracing feature integrates with `tokio-console` for advanced async task visualization.

### Installation

```bash
cargo install tokio-console
```

### Usage

1. Build your application with the tracing feature:
```bash
cargo build --features tracing
```

2. Run the application:
```bash
./backend
```

3. In another terminal, run tokio-console:
```bash
tokio-console
```

Note: tokio-console will be available once you run the application with the tracing feature. The console shows real-time task information from your running Llumen instance.

## Performance Considerations

### With Tracing Disabled (Default)

- No runtime overhead from tracing infrastructure
- Minimal binary size increase
- Standard `log` crate provides basic logging
- Recommended for production

### With Tracing Enabled

- Small runtime overhead from span creation and event recording
- Additional dependencies add ~2-3MB to binary size
- More detailed observability for debugging
- Recommended for development and troubleshooting

## Examples

### Debugging a Chat Creation Issue

```bash
# Build with tracing
cargo build --features tracing

# Run with debug logging
RUST_LOG=backend::routes::chat=debug ./backend

# Output shows:
# - User authentication
# - Chat creation request details
# - Database operations
# - Response status
```

### Monitoring Performance

```bash
# Run with trace-level logging to see all events
RUST_LOG=backend=trace ./backend

# This will show:
# - All HTTP requests and responses
# - All authentication attempts
# - All database operations
# - All timer/span events
```

### Debugging Authentication

```bash
# Focus on auth middleware
RUST_LOG=backend::middlewares::auth=debug ./backend

# Output:
# - Token validation attempts
# - Successful authentications with user IDs
# - Authorization failures
```

## Integration Points

### Middleware Stack

The tracing feature is integrated into the core middleware:
- **LoggerLayer:** HTTP request/response tracing
- **AuthMiddleware:** Token validation and user identification
- All configured to work seamlessly with both `tracing` crate and standard `log` crate

### Route Handlers

Key route handlers emit trace events:
- Chat creation/reading/updating
- Message handling
- SSE subscriptions
- All emit contextual information (user_id, chat_id, etc.)

### Initialization

The logger module (`src/utils/logger.rs`) automatically:
- Initializes console logging when `tracing` feature is enabled
- Sets up structured logging with timestamps
- Configures formatters for clear output

## Development Tips

1. **Use Specific Modules:** Filter logs to specific modules while debugging:
   ```bash
   RUST_LOG=backend::routes::chat=debug ./backend
   ```

2. **Combine with Other Tools:** Use with `grep` for filtering:
   ```bash
   RUST_LOG=backend=info ./backend | grep "chat"
   ```

3. **Capture to File:** Redirect output for analysis:
   ```bash
   RUST_LOG=backend=debug ./backend > trace.log 2>&1
   ```

4. **Real-time Monitoring:** Use `tail -f` to watch logs:
   ```bash
   RUST_LOG=backend=info ./backend 2>&1 | tail -f
   ```

## Troubleshooting

### Tracing Not Appearing

1. **Ensure feature is enabled:**
   ```bash
   cargo build --features tracing
   ```

2. **Check RUST_LOG is set:**
   ```bash
   export RUST_LOG=backend=info
   ```

3. **Verify output goes to stderr:**
   ```bash
   RUST_LOG=backend=info ./backend 2>&1 | head -20
   ```

### Too Much Output

Limit to specific modules:
```bash
RUST_LOG=backend::middlewares=info ./backend
```

### Performance Concerns

The tracing overhead is minimal but if needed:
- Use `info` level instead of `debug`/`trace` in production
- Disable feature entirely for minimal overhead: `cargo build`
- Use log level filters to reduce event volume

## Architecture

### Conditional Compilation

All tracing code is gated behind `#[cfg(feature = "tracing")]`:
- Zero overhead when disabled
- Clean separation of concerns
- Easy to maintain and extend

### Layers

```
Application Code
    ↓
Route Handlers (emit trace events)
    ↓
Middleware (HTTP, Auth, Logger)
    ↓
Tracing Crate (with feature)
    ↓
Logger Output
```

When tracing feature is disabled, route/middleware trace events become no-ops.

## Future Enhancements

Potential improvements to the tracing system:
1. Structured logging to JSON format
2. Integration with external observability platforms (Jaeger, Datadog)
3. Metrics collection alongside traces
4. Custom span creation utilities for domain-specific operations
5. Performance profiling integration

## Related Documentation

- [Design Document](./design.md) - Overall architecture
- [Logger System](../backend/src/utils/logger.rs) - Initialization code
- [Middleware Stack](../backend/src/middlewares/logger.rs) - HTTP tracing
- [Rust Tracing Crate](https://docs.rs/tracing/) - External documentation
- [tokio-console](https://github.com/tokio-rs/console) - Task visualization tool