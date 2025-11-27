# Frequently Asked Questions

## Getting Started

### What is Llumen?

Llumen is a lightweight, self-hostable LLM chat application that works with OpenAI-compatible API endpoints. It features three chat modes: Normal chat, Search (with web integration), and Deep Research (multi-step research with tools).

### How do I get an API key?

1. Sign up at [OpenRouter](https://openrouter.ai)
2. Create an API key in your account settings
3. Set it as the `API_KEY` environment variable

### What are the default login credentials?

- **Username:** `admin`
- **Password:** `P@88w0rd`

> Change this password after first login for security.

## Configuration

### How do I change the API endpoint?

Set the `OPENAI_API_BASE` environment variable:

```bash
export OPENAI_API_BASE="https://your-custom-endpoint.com/v1"
```

**Note:** The variable name has evolved across versions:
- v0.2.0 and earlier: `API_BASE`
- v0.3.0+: `OPENAI_API_BASE` (preferred)

Both names work in v0.3.0+.

### How do I configure a model?

Use TOML configuration in the settings UI. Press **Tab** for auto-completion.

```toml
display_name = "My Model"         # UI display name
model_id = "provider/model-name"  # API model identifier

[capability]
image = true       # Vision support
tool = true        # Function calling
audio = false      # Audio support
json = true        # Structured output
ocr = "native"     # OCR mode

[parameter]
temperature = 0.7
top_p = 0.9
```

See the [Configuration Guide](./configuration.md) for full details.

### Where is my data stored?

All data is stored in the `/data` directory (Docker) or `./data` (binary):
- `db.sqlite` - Database with chats, messages, users
- `blobs.redb` - File uploads and cached content

## Chat Modes

### What's the difference between the chat modes?

| Mode | Purpose | Speed | Depth |
|------|---------|-------|-------|
| **Normal** | Direct LLM conversation | Fast | Basic |
| **Search** | Web-enhanced responses | Medium | Current info |
| **Deep Research** | Multi-step investigation | Slow | Comprehensive |

### Why are Search/Deep Research modes grayed out?

These modes require tool calling support. Your selected model may not support it.

**Fix:** Add to the model's configuration:
```toml
[capability]
tool = true
```

### Why do I see "parameter tool not supported" errors?

Earlier versions relied on API auto-detection which was sometimes inaccurate. Now you must explicitly enable tool support in the model config.

## Features

### What file types can I upload?

- **Documents:** PDF
- **Images:** PNG, JPEG, GIF, WebP
- **Code:** Various text formats

### Does Llumen support image/vision analysis?

Yes, if you select a model with vision capability. Enable it in config:
```toml
[capability]
image = true
```

### Can I edit my messages?

Yes! Hover over your message and click the edit icon. The conversation will regenerate from that point.

### How does real-time streaming work?

Responses stream token-by-token as the LLM generates them. You see the response appearing in real-time rather than waiting for the complete response.

## Technical

### What ports does Llumen use?

- **Docker:** Port 80 by default
- **Binary:** Port 8001 by default

Configure with `BIND_ADDR` environment variable.

### Is Llumen secure for public deployment?

Basic security is included:
- PASETO v4 authentication tokens
- Password hashing with Argon2
- Standard web security headers

For public deployment, consider:
- Changing default credentials
- Running behind HTTPS (reverse proxy)
- Restricting network access

### Can multiple users share one instance?

Yes, Llumen supports multiple user accounts. Each user has their own chats and settings.

## Troubleshooting

### I can't connect to Llumen

1. Check if the container/process is running
2. Verify port binding (`-p 80:80` for Docker)
3. Check firewall settings

### API calls are failing

1. Verify `API_KEY` is set correctly
2. Check network connectivity to the API endpoint
3. Ensure your API key has sufficient credits

### Responses are slow

- Try a smaller/faster model
- Check your network connection
- Start a new chat if conversation is very long

See the full [Troubleshooting Guide](./troubleshooting.md) for more solutions.

## Contributing

### How can I contribute?

See [CONTRIBUTING.md](https://github.com/pinkfuwa/llumen/blob/main/CONTRIBUTING.md) for guidelines on:
- Submitting bug reports
- Making feature requests
- Contributing code

### Where can I get help?

1. Check this FAQ and [Troubleshooting Guide](./troubleshooting.md)
2. Search [GitHub Issues](https://github.com/pinkfuwa/llumen/issues)
3. Open a new issue if needed
4. Contact the maintainer directly (it's a small project!)

---

**Still stuck?** Let me know what's unclear—I'm happy to help! 😊

> [Eason0729](https://github.com/Eason0729/)
