# Chunked Streaming Implementation

## Overview

This document describes the memory-efficient chunked streaming implementation for file uploads in the OpenRouter integration. The implementation significantly reduces memory usage when handling large files by streaming them in chunks rather than loading entire files into memory.

## Motivation

Previously, when uploading files (images, documents, audio) to LLM providers:
- Entire files were read into memory at once
- Base64 encoding created additional memory copies
- For a 100MB file: ~100MB raw + ~133MB base64 = 233MB+ in memory simultaneously
- Multiple files would multiply this memory usage

The new implementation:
- Streams files in 256KB chunks
- Encodes base64 on-the-fly without buffering entire result
- Properly handles UTF-8 boundaries for text files
- Memory usage is now O(chunk_size) instead of O(file_size)

## Architecture

### SyncStream Trait

The core abstraction is the `SyncStream` trait, which provides chunked reading:

```rust
pub trait SyncStream {
    /// Read next chunk into buffer, returns number of bytes read
    /// Returns 0 when no more data available
    fn read_chunk(&mut self, buf: &mut [u8]) -> usize;

    /// Get total size of the stream
    fn len(&self) -> usize;

    /// Check if stream is empty
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
```

Implementations:
- `Reader` (from redb/mmap): Tracks position in memory-mapped data
- `VecStream`: Stateful wrapper around Vec<u8> with position tracking
- `Vec<u8>`: Direct implementation that consumes data as read

### Chunked Processing

#### Base64 Encoding

`stream_encode/chunked_stream.rs:write_base64_stream()`

- Reads source in 256KB chunks
- Encodes only complete 3-byte units (base64 encodes 3 bytes → 4 chars)
- Carries over incomplete units to next iteration
- Uses `struson::string_value_writer()` for direct JSON streaming

```rust
// Base64 padding handling
let processable_len = (data.len() / 3) * 3;
encode(&data[..processable_len]);
leftover = data[processable_len..].to_vec();
```

#### UTF-8 Text Streaming

`stream_encode/chunked_stream.rs:write_text_stream()`

- Reads text files in 256KB chunks
- Detects UTF-8 character boundaries to avoid splitting multi-byte characters
- Carries over incomplete UTF-8 sequences to next iteration

```rust
// UTF-8 boundary detection
let valid_len = find_valid_utf8_boundary(&data);
if valid_len < data.len() {
    leftover = data[valid_len..].to_vec();
    data.truncate(valid_len);
}
```

The `find_valid_utf8_boundary()` function:
1. Checks if entire buffer is valid UTF-8
2. If not, walks backwards to find last complete character
3. UTF-8 continuation bytes have pattern `10xxxxxx`
4. Returns position of last valid character boundary

### JSON Serialization

`stream_encode/json_serializer.rs`

Uses `struson` library's `string_value_writer()` for streaming JSON generation:

```rust
let mut string_writer = json_writer.string_value_writer()?;

// Write data in chunks
loop {
    let read = stream.read_chunk(&mut buffer);
    if read == 0 { break; }
    string_writer.write_all(&buffer[..read])?;
}

string_writer.finish_value()?;
```

This approach:
- Writes directly to underlying HTTP stream
- No intermediate buffering of entire value
- Automatically handles JSON escaping

## Memory Characteristics

### Before (Old Implementation)

For a 100MB file upload:
```
100MB (file in memory) + 133MB (base64 string) = 233MB peak memory
```

With 5 files: ~1.2GB peak memory

### After (New Implementation)

For a 100MB file upload:
```
256KB (read buffer) + ~341KB (base64 chunk) + 64KB (channel buffer) = ~661KB peak memory
```

With 5 files: Still ~661KB peak memory (files processed sequentially)

**Memory reduction: ~350x for single large file**

## Implementation Details

### Chunk Size

`const CHUNK_SIZE: usize = 256 * 1024; // 256KB`

Chosen because:
- Large enough for good throughput
- Small enough to limit memory usage
- Divisible by 3 for base64 (256*1024 / 3 = 87,381 complete units)

### Thread Architecture

The implementation uses two-thread pattern:

1. **Blocking thread** (`tokio::task::spawn_blocking`):
   - Reads from mmap (redb) - may cause page faults
   - Converts to base64 if needed
   - Sends chunks via channel

2. **Async thread**:
   - Receives chunks from channel
   - Streams to HTTP response

This prevents page faults from blocking the async runtime.

### Error Handling

- UTF-8 validation errors: Stream writes valid data up to error point
- Channel closure: Detected and propagated as IO error
- Base64 encoding: Always succeeds (operates on bytes)

## Testing

Key test cases:

1. **Large file streaming** (`test_chunked_streaming_large_file`):
   - Verifies 1MB file streams correctly
   - Validates base64 encoding accuracy
   - Confirms memory-efficient processing

2. **UTF-8 boundary handling** (`test_chunked_text_streaming_utf8_boundaries`):
   - Tests multi-byte characters (Chinese, emoji)
   - Verifies no character splitting
   - Confirms accurate reconstruction

3. **Base64 chunk encoding** (`test_stream_base64_encoding`):
   - Tests padding handling
   - Verifies chunk boundary processing
   - Confirms output matches single-pass encoding

## Future Improvements

1. **Parallel file processing**: Currently files are processed sequentially. Could process multiple files in parallel while maintaining memory bounds.

2. **Adaptive chunk size**: Could adjust chunk size based on file size and available memory.

3. **Compression**: Could add optional streaming compression for large text files.

4. **Progress tracking**: Could add callbacks for upload progress reporting.

## Related Files

- `backend/src/openrouter/mod.rs` - SyncStream trait definition
- `backend/src/openrouter/stream_encode/chunked_stream.rs` - Core streaming utilities
- `backend/src/openrouter/stream_encode/json_serializer.rs` - JSON serialization
- `backend/src/utils/blob.rs` - Reader implementation with mmap