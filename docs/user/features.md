# Features Guide

This guide explains Llumen's chat modes and features.

## Chat Modes

Llumen offers three distinct chat modes, each designed for different use cases.

### Normal Mode

**Best for:** General conversations, quick questions, creative writing, coding assistance

Normal mode provides direct conversation with the selected LLM without additional tools or web access.

**How it works:**
1. You send a message
2. The LLM processes your message with conversation history
3. Responses stream back in real-time

**Use cases:**
- General Q&A
- Creative writing and brainstorming
- Code review and debugging
- Explanations and tutoring

### Search Mode

**Best for:** Questions requiring current information, fact-checking, research with web sources

Search mode augments the LLM with real-time web search capabilities, URL crawling, and code execution.

**How it works:**
1. You send a message
2. The LLM decides if web search, crawling, or code execution is needed
3. Tools execute and return results to the LLM
4. The LLM synthesizes information into a response

**Available tools:**
- **Web Search** - Queries DuckDuckGo for relevant results
- **URL Crawl** - Fetches and parses full page content
- **Lua REPL** - Executes code for calculations and data processing

**Use cases:**
- Current events and news
- Product comparisons with latest pricing
- Technical documentation lookup
- Fact verification

**Tip:** Search mode is only available for models that support tool calling. If grayed out, enable `tool = true` in the model's configuration.

### Deep Research Mode

**Best for:** Complex research tasks requiring multiple sources, in-depth analysis, comprehensive reports

Deep Research mode uses a multi-step research process with specialized sub-agents.

**How it works:**
1. **Prompt Enhancement** - Your query is refined for better research
2. **Planning** - A research plan with multiple steps is created
3. **Execution** - Each step is executed with appropriate tools
4. **Synthesis** - Findings are compiled into a comprehensive report

**Sub-agents:**
- **Planner** - Creates structured research plans
- **Step Executor** - Executes individual research steps
- **Reporter** - Synthesizes findings into final output

**Use cases:**
- Market research and analysis
- Academic research assistance
- Competitive analysis
- Multi-source investigations

**Note:** Deep Research takes longer but provides more thorough results with citations.

## File Uploads

Llumen supports attaching files to your messages for analysis. Files can be uploaded before you send your message, giving you time to compose your thoughts.

### Uploading Files

There are three ways to attach files:

1. **Click the upload button** - Select files from your device
2. **Drag and drop** - Drag files directly onto the message input area
3. **Paste** - Copy files to clipboard and paste them into the message input

All file types are accepted for upload, giving you maximum flexibility.

### File Lifecycle

- **Temporary storage**: Uploaded files are stored temporarily (1 hour) until you send your message
- **Automatic upload**: Files begin uploading as soon as you add them
- **Permanent storage**: When you send your message, attached files become permanently associated with the chat

This approach ensures you can upload large files without worrying about timing, and unused files are automatically cleaned up.

### Supported File Types by Model

While you can upload any file type, model capabilities determine what can be analyzed:

| Type | Extensions | Use Cases | Model Requirement |
|------|------------|-----------|-------------------|
| Images | PNG, JPEG, GIF, WebP, AVIF, BMP | Visual questions, image description | `image_input = true` |
| Audio | MP3, WAV, OGG, M4A, etc. | Audio transcription, analysis | `audio_input = true` |
| Documents | PDF, DOC, DOCX, XLS, XLSX, PPT, PPTX | Document analysis, summarization | `other_file_input = true` |
| Code/Text | TXT, MD, JSON, CSV, and programming files | Code review, debugging, analysis | Always supported |

**Warning indicators**: If you upload a file type not supported by your selected model, a warning icon (⚠️) will appear. You can still send the message, but the model may not be able to process that file.

### File Size Limits

- Maximum file size: **100 MB**
- Files are uploaded in the background as you add them
- Large files may take time to upload; wait for upload completion before sending

### Image Analysis

When uploading images to a vision-capable model:
- Ask questions about the image content
- Request descriptions or analysis
- Use for OCR (text extraction from images)
- Compare multiple images in one message

**Tip:** Ensure your selected model has `image_input = true` in its capabilities for vision features.

### Best Practices

- Upload files early if they're large, so they're ready when you finish composing your message
- Check the warning icon if you're unsure whether your file type is supported
- Use text/code files for the most reliable analysis across all models
- Keep total file count reasonable for better model performance

## Real-Time Streaming

Responses in Llumen stream in real-time as the LLM generates them.

**Benefits:**
- See responses as they're generated
- Cancel long responses by clicking halt
- Better perceived performance

**Technical details:**
- Uses Server-Sent Events (SSE) for streaming
- Tokens are published as they arrive from the LLM
- Multiple browser tabs can watch the same conversation

## Message Editing

You can edit previous messages to refine your conversation.

**How to edit:**
1. Hover over your message
2. Click the edit icon
3. Modify your message
4. Submit to regenerate the response

## Image Generation

Some models can generate images directly in response to your prompts.

**How it works:**
1. Use a model that supports image generation (e.g., models with DALL-E capabilities)
2. Request an image in your message (e.g., "Generate an image of a sunset over mountains")
3. The generated image will appear inline in the assistant's response
4. Images are stored locally and persist across sessions

**Viewing generated images:**
- Images display directly in the chat
- Click on an image to view it at full size
- Images are saved to your local blob storage

**Use cases:**
- Creating illustrations for concepts
- Visualizing ideas and designs
- Generating artwork or graphics
- Prototyping visual content

**Note:** Not all models support image generation. Check the model's capabilities before requesting images.

## Markdown Support

Llumen renders rich Markdown in responses:

- **Text formatting** - Bold, italic, strikethrough
- **Code blocks** - Syntax highlighting for many languages
- **LaTeX math** - Mathematical equations with KaTeX
- **Lists and tables** - Ordered, unordered, and table formatting
- **Links** - Clickable URLs

## Model Selection

### Choosing a Model

Different models have different strengths:

| Consideration | Guidance |
|---------------|----------|
| Speed | Smaller models respond faster |
| Quality | Larger models often produce better output |
| Cost | Check pricing on OpenRouter |
| Capabilities | Some models support vision, tools, etc. |

### Viewing Model Capabilities

In the model selector, capabilities are indicated:
- 🖼️ Vision/image support
- 🔧 Tool calling support
- 📊 Structured output support

### Configuring Models

See the [Configuration Guide](./configuration.md) for detailed model configuration options.

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Enter` | Send message |
| `Shift + Enter` | New line in message |
| `Escape` | Cancel editing |

## Cost Tracking

Llumen tracks token usage and costs for each message:
- Token counts are displayed per message
- Costs are calculated based on model pricing
- View totals in the chat information

## Next Steps

- Configure your models: [Configuration](./configuration.md)
- Resolve issues: [Troubleshooting](./troubleshooting.md)
- Common questions: [FAQ](./faq.md)
