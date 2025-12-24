# Prompt Caching Improvements

## Overview

Llumen now supports optimized prompt caching for faster responses and reduced costs when using compatible LLM providers (like OpenRouter with Claude models).

## What is Prompt Caching?

Prompt caching allows LLM providers to cache parts of your conversation that don't change, resulting in:
- **Faster response times**: Cached portions are processed instantly
- **Lower costs**: Cached tokens are typically 90% cheaper than new tokens
- **Better performance**: Reduced latency for subsequent requests

## How It Works in Llumen

### Previous Behavior

Previously, the system prompt included variable information (current time, chat title) that changed with every request, preventing effective caching.

### New Behavior

Llumen now separates static system prompts from variable context:

1. **Static System Prompt**: Contains all the core instructions (cacheable)
2. **Context Message**: Injected as a user message with variable information:
   - Current date and time (with minute precision)
   - Chat title
   - Llumen-specific documentation (only when relevant)

### Affected Modes

- **Normal Mode**: ✅ Optimized for caching
- **Search Mode**: ✅ Optimized for caching
- **Deep Research Mode**: Uses different caching strategy

## Smart Context Detection

Llumen intelligently detects when to include detailed documentation about itself. The system automatically includes Llumen context when your message contains:
- "llumen" (any capitalization)
- "流明" (Traditional Chinese)
- "app" (when asking about the application)

This means you get relevant information only when you need it, without cluttering every conversation.

## Enhanced Time Precision

Time information is now more precise for better context awareness:
- **Normal/Search**: Includes hour and minute (e.g., "Monday, 14:30, 15 January 2024")
- **Deep Research**: Includes seconds for detailed tracking

## Expected Benefits

When using providers that support prompt caching (OpenRouter with Claude, etc.):

1. **First message in a chat**: Normal processing (no cache)
2. **Subsequent messages**: 
   - System prompt hits cache (faster, cheaper)
   - Only new messages and context are processed fresh
   - You may see 50-90% cost reduction on cached tokens

## Provider Support

Prompt caching works best with:
- **Claude models via OpenRouter**: Full support
- **Other providers**: May have varying levels of support

Check your provider's documentation for specific caching capabilities.

## Tips for Maximum Benefit

1. **Keep conversations in the same chat**: Cache hits work within a chat session
2. **Use consistent models**: Switching models may invalidate caches
3. **Monitor your usage**: Check OpenRouter dashboard for cache statistics
4. **Long conversations benefit most**: More messages = more cache hits

## Transparency

You won't see any visual indication of cache hits in the UI, but you can verify the benefits by:
- Monitoring response times (should be faster)
- Checking your provider's usage dashboard for cache statistics
- Reviewing your billing to see reduced costs on cached tokens