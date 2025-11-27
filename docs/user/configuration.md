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
| `DATABASE_URL` | SQLite database connection string | `sqlite://data/db.sqlite?mode=rwc` |
| `BLOB_URL` | Path for blob storage (file uploads) | `/data/blobs.redb` |
| `BIND_ADDR` | Address and port to bind to | `0.0.0.0:80` (Docker) |

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
image = true       # Supports image input
tool = true        # Supports tool/function calling
audio = false      # Supports audio input/output
json = true        # Supports structured JSON output
ocr = "native"     # OCR mode: "native", "text", "mistral", or "disabled"

[parameter]
temperature = 0.7     # Creativity (0.0-2.0)
top_p = 0.9          # Nucleus sampling threshold
top_k = 40           # Top-k sampling (if supported)
repeat_penalty = 1.1  # Repetition penalty
```

### Capability Options

| Capability | Description | Values |
|------------|-------------|--------|
| `image` | Vision/image understanding | `true` / `false` |
| `tool` | Tool/function calling support | `true` / `false` |
| `audio` | Audio input/output | `true` / `false` |
| `json` | Structured JSON responses | `true` / `false` |
| `ocr` | OCR processing mode | `"native"`, `"text"`, `"mistral"`, `"disabled"` |

### Parameter Options

| Parameter | Description | Range |
|-----------|-------------|-------|
| `temperature` | Controls randomness. Lower = more focused, higher = more creative | 0.0 - 2.0 |
| `top_p` | Nucleus sampling. Consider tokens with cumulative probability | 0.0 - 1.0 |
| `top_k` | Consider only top k tokens | 1 - 100 |
| `repeat_penalty` | Penalty for repeating tokens | 0.0 - 2.0 |

### Enabling Tool Calling for Modes

If Search or Deep Research modes are grayed out, the model may not be detected as supporting tool calling. To enable:

```toml
[capability]
tool = true  # Force enable tool support
```

## Default Credentials

The default admin account credentials are:
- **Username:** `admin`
- **Password:** `P@88w0rd`

> **Security Note:** Change the default password after first login, especially for public-facing deployments.

## Database Configuration

### SQLite Options

The `DATABASE_URL` supports SQLite URI parameters:

```bash
DATABASE_URL="sqlite://data/db.sqlite?mode=rwc"
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
