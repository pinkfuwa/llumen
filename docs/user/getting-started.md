# Getting Started with Llumen

This guide will help you get Llumen up and running in minutes.

## What is Llumen?

Llumen is a lightweight, self-hostable LLM chat application that works with any OpenAI-compatible API endpoint. It features three chat modes:

- **Normal Chat** - Direct conversation with your chosen LLM
- **Search Mode** - Chat enhanced with real-time web search results
- **Deep Research** - Multi-step research with tools for web search, crawling, and code execution

## Prerequisites

- An API key from [OpenRouter](https://openrouter.ai) or another OpenAI-compatible provider
- Docker (recommended) or a system that can run the prebuilt binaries

## Quick Start with Docker

1. **Run the container**:

```bash
docker run -it --rm \
  -e API_KEY="<YOUR_API_KEY>" \
  -p 80:80 \
  -v "$(pwd)/data:/data" \
  ghcr.io/pinkfuwa/llumen:nightly
```

2. **Open your browser** and navigate to `http://localhost`

3. **Log in** with the default credentials:
   - Username: `admin`
   - Password: `P@88w0rd`

4. **Start chatting!** Select a model and choose your preferred chat mode.

## Quick Start with Prebuilt Binaries

1. Download the latest binary from the [releases page](https://github.com/pinkfuwa/llumen/releases)

2. Extract the archive

3. Set your API key:
```bash
export API_KEY="<YOUR_API_KEY>"
```

4. Run the application:
```bash
./llumen
```

5. Open `http://localhost:8001` in your browser

## First Steps After Login

### 1. Explore Chat Modes

- **Normal**: Best for general conversations and quick questions
- **Search**: Use when you need current information from the web
- **Deep Research**: For complex research tasks requiring multiple sources

### 2. Select a Model

Click the model selector to choose from available LLMs. Different models have different capabilities and costs.

### 3. Start a Conversation

Type your message and press Enter or click Send. Responses stream in real-time.

### 4. Upload Files (Optional)

You can attach files to your messages:
- PDFs for document analysis
- Images for visual questions
- Code files for programming assistance

## What's Next?

- Learn about [Installation](./installation.md) options for production deployments
- Explore [Configuration](./configuration.md) to customize your setup
- Discover all [Features](./features.md) including advanced chat modes
