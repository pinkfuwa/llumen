# Configuration Guide

This guide explains how to configure Llumen through environment variables and model settings.

## Environment Variables

### Required

| Variable | Description | Example |
|----------|-------------|---------|
| `API_KEY` | Your OpenRouter or compatible API key | `sk-or-v1-...` |

### Optional

| Variable | Description | Default |
|----------|-------------|---------|
| `OPENAI_API_BASE` | OpenAI-compatible API base URL | `https://openrouter.ai/api` |
| `DATA_PATH` | Directory path for data storage (stores `db.sqlite` and `blobs.redb`) | `.` (current directory) |
| `BIND_ADDR` | Address and port to bind to | `0.0.0.0:80` (Docker) |
| `TRUSTED_HEADER` | HTTP header name for header-based authentication | Not set (disabled) |

### Setting Environment Variables

**Docker:**
```bash
docker run -e API_KEY="your-key" -e OPENAI_API_BASE="https://api.example.com/v1" ...
```

**Docker Compose:**
```yaml
environment:
  - "API_KEY=your-key"
  - "OPENAI_API_BASE=https://api.example.com/v1"
```

**Shell:**
```bash
export API_KEY="your-key"
export OPENAI_API_BASE="https://api.example.com/v1"
./llumen
```

## API Endpoint Configuration

### OpenRouter (Default)

No additional configuration needed. Just set your `API_KEY`:
```bash
API_KEY="sk-or-v1-your-openrouter-key"
```

### Custom OpenAI-Compatible Endpoints

To use a different provider (local models, other cloud providers):

```bash
API_KEY="your-provider-key"
OPENAI_API_BASE="https://your-provider.com/v1"
```

**Examples:**

Local Ollama:
```bash
OPENAI_API_BASE="http://localhost:11434/v1"
API_KEY="ollama"  # Ollama doesn't require a real key
```

Azure OpenAI:
```bash
OPENAI_API_BASE="https://your-resource.openai.azure.com/openai/deployments/your-deployment"
API_KEY="your-azure-key"
```

## Authentication Configuration

### Standard Username/Password Authentication

By default, Llumen uses username and password authentication. Users can log in with their credentials set by administrators.

### Header-Based Authentication

Header-based authentication is useful when Llumen is behind a reverse proxy or SSO middleware (like Authelia, OAuth2-Proxy, etc.) that handles authentication and injects the authenticated username into HTTP headers.

#### Setup

1. **Configure the header name** via the `TRUSTED_HEADER` environment variable:

```bash
# Example: Using X-Remote-User header injected by Authelia
export TRUSTED_HEADER="X-Remote-User"
```

2. **Ensure users exist** in Llumen's database with matching usernames

#### How it Works

When `TRUSTED_HEADER` is configured:

1. When a user's token expires and renewal is attempted
2. The frontend automatically tries the header-based authentication endpoint
3. The backend reads the configured header from the request
4. If the header value matches the username, a new token is issued
5. If the header doesn't match or is missing, normal login is required

#### Example: Authelia Setup

With Authelia as your SSO middleware, configure your reverse proxy to inject the authenticated username:

```yaml
# Authelia configuration
session:
  remember_me: 1y
  
server:
  headers:
    X-Remote-User: "{{ .Username }}"
```

Then set in Llumen:
```bash
TRUSTED_HEADER="X-Remote-User"
```

#### Example: OAuth2-Proxy Setup

With OAuth2-Proxy, configure the header injection:

```bash
# OAuth2-Proxy command line
oauth2-proxy \
  --set-xauthrequest \
  --cookie-name=_oauth2_proxy \
  ...
```

Then set in Llumen:
```bash
TRUSTED_HEADER="X-Auth-Request-User"
```

#### Example: Docker Compose with Authelia

```yaml
services:
  authelia:
    image: authelia/authelia:latest
    environment:
      - AUTHELIA_JWT_SECRET=your-secret
    volumes:
      - ./authelia.yml:/config/configuration.yml

  llumen:
    image: llumen:latest
    environment:
      - API_KEY=your-key
      - TRUSTED_HEADER=X-Remote-User
    depends_on:
      - authelia

  reverse-proxy:
    image: nginx:latest
    # Configure nginx to proxy requests through Authelia
    # and inject X-Remote-User header
```

#### Security Considerations

- **Only enable when behind trusted middleware:** Header-based auth relies on the middleware correctly setting headers. Never expose Llumen directly to untrusted networks without proper proxy configuration.
- **Header must be non-spoofable:** Ensure your reverse proxy only allows the configured header to be set by the authentication middleware, not by clients.
- **Username must exist:** Users must have matching accounts in Llumen with the same username.
- **Not suitable for untrusted networks:** This is designed for enterprise/organizational deployments with controlled infrastructure.

#### Troubleshooting Header Auth

If header authentication isn't working:

1. **Check the TRUSTED_HEADER is set:** Verify `echo $TRUSTED_HEADER` shows the correct header name
2. **Verify header is being sent:** Check reverse proxy logs to confirm the header is injected
3. **Confirm username matches:** Ensure the username in the header matches exactly with the Llumen user account (case-sensitive)
4. **Check case sensitivity:** Header names are case-insensitive in HTTP but values are case-sensitive
5. **Fall back to password login:** If header auth fails, the login page will still work with username/password

## Model Configuration

Llumen allows you to customize model behavior through TOML configuration in the settings UI.

### Accessing Model Settings

1. Click the settings icon in the UI
2. Navigate to the Models section
3. Click on a model to edit its configuration

### Configuration Schema

```toml
display_name = "My Custom Model"  # Name shown in the UI
model_id = "provider/model-name"  # Model identifier

[capability]
# All capability fields are optional and will be auto-detected from OpenRouter
# if not specified and using OpenRouter as the API endpoint
image = true       # Supports image input (optional, auto-detected)
tool = true        # Supports tool/function calling (optional, auto-detected)
audio = false      # Supports audio input/output (optional, auto-detected)
json = true        # Supports structured JSON output (optional, auto-detected)
ocr = "native"     # File/document handling: "native", "text", "mistral", or "disabled" (optional, auto-detected)

[parameter]
temperature = 0.7     # Creativity (0.0-2.0)
top_p = 0.9          # Nucleus sampling threshold
top_k = 40           # Top-k sampling (if supported)
repeat_penalty = 1.1  # Repetition penalty
```

### Capability Auto-Detection

When using OpenRouter as your API endpoint, Llumen automatically detects model capabilities from OpenRouter's model metadata. The detection logic follows this priority:

1. **User explicit setting** - If you explicitly set a capability, your setting is always respected
2. **OpenRouter metadata** - If OpenRouter is used and the capability is not set, Llumen uses OpenRouter's reported capabilities
3. **Not found/not OpenRouter** - If the model isn't found on OpenRouter or a custom endpoint is used, capabilities default to `true` (conservative default)

This means you typically don't need to configure capabilities manually when using OpenRouter. The system will automatically:
- Enable vision features for models like GPT-4 Vision
- Enable tool calling for models that support it
- Enable audio input for models like Gemini with audio support
- Enable structured JSON output for compatible models
- Choose appropriate OCR mode based on model's native file support

#### Message Filtering Based on Capabilities

Llumen automatically filters message content based on detected capabilities:
- **Images**: If a model doesn't support image input, image attachments are filtered out before sending
- **Audio**: If a model doesn't support audio input, audio attachments are filtered out before sending
- **Files/Documents**: If OCR is disabled, document attachments are filtered out before sending

This ensures requests are always compatible with the target model's capabilities.

**Example: Minimal configuration with auto-detection**
```toml
display_name = "GPT-4 Turbo"
model_id = "openai/gpt-4-turbo"
# No [capability] section needed - all capabilities auto-detected from OpenRouter
```

**Example: Override specific capabilities**
```toml
display_name = "Custom GPT-4"
model_id = "openai/gpt-4-turbo"

[capability]
tool = false  # Disable tool calling even though the model supports it
# Other capabilities (image, audio, json) are still auto-detected
```

### Capability Options

All capability fields are **optional** when using OpenRouter (auto-detected). Set them explicitly only when you need to override the auto-detected values or when using custom API endpoints.

| Capability | Description | Values | Auto-Detection |
|------------|-------------|--------|----------------|
| `image` | Vision/image understanding | `true` / `false` | Based on OpenRouter's image input modality |
| `tool` | Tool/function calling support | `true` / `false` | Based on OpenRouter's tools parameter support |
| `audio` | Audio input/output | `true` / `false` | Based on OpenRouter's audio input modality |
| `json` | Structured JSON responses | `true` / `false` | Based on OpenRouter's structured output support |
| `ocr` | File/document handling mode | `"native"`, `"text"`, `"mistral"`, `"disabled"` | `"native"` if model supports File modality, otherwise `"text"` |

#### OCR Mode Details

- **`native`**: Model natively supports file attachments (PDF, documents). Files are sent directly to the model.
- **`text`**: Files are processed with basic text extraction before sending to the model.
- **`mistral`**: Files are processed with Mistral's OCR service before sending to the model.
- **`disabled`**: File attachments are not supported and will be filtered out.

When auto-detecting, if OpenRouter reports the model supports the File modality, `native` is used. Otherwise, `text` is used as a conservative default that works with most models.

### Parameter Options

| Parameter | Description | Range |
|-----------|-------------|-------|
| `temperature` | Controls randomness. Lower = more focused, higher = more creative | 0.0 - 2.0 |
| `top_p` | Nucleus sampling. Consider tokens with cumulative probability | 0.0 - 1.0 |
| `top_k` | Consider only top k tokens | 1 - 100 |
| `repeat_penalty` | Penalty for repeating tokens | 0.0 - 2.0 |

### Enabling Tool Calling for Modes

If Search or Deep Research modes are grayed out, the model may not support tool calling:

1. **Using OpenRouter**: Capabilities are auto-detected. If a mode is grayed out, the model genuinely doesn't support tools according to OpenRouter's metadata.

2. **Using custom endpoints**: Tool support defaults to `true`, but you may need to explicitly enable it:
   ```toml
   [capability]
   tool = true  # Force enable tool support for custom endpoints
   ```

3. **Override for specific models**: If you know a model supports tools but it's not detected:
   ```toml
   [capability]
   tool = true  # Override auto-detection
   ```

## Default Credentials

The default admin account credentials are:
- **Username:** `admin`
- **Password:** `P@88w0rd`

> **Security Note:** Change the default password after first login, especially for public-facing deployments.

## Database Configuration

### SQLite Options

The `DATA_PATH` specifies the directory where both `db.sqlite` and `blobs.redb` will be stored:

```
DATA_PATH="/data"
```

Modes:
- `rwc` - Read/write, create if not exists (recommended)
- `rw` - Read/write, must exist
- `ro` - Read-only

### Data Directory

All persistent data is stored in the data directory:
- `db.sqlite` - Main database (chats, messages, users)
- `blobs.redb` - Binary storage (file uploads, cached content)

## Memory Tuning

For systems with limited memory, you can restrict resources in Docker:

```yaml
deploy:
  resources:
    limits:
      memory: 512M
memswap_limit: 512M
```

## Next Steps

- Learn about [Features](./features.md) and chat modes
- See [Troubleshooting](./troubleshooting.md) if you encounter issues
