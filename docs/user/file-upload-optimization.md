# Memory-Efficient File Upload

## Overview

llumen now uses a highly memory-efficient streaming approach for file uploads to LLM providers. This improvement significantly reduces memory usage when uploading images, documents, audio files, and other attachments.

## Benefits

### Reduced Memory Usage

Previously, uploading large files would load the entire file into memory multiple times:
- Original file data
- Base64-encoded version
- JSON serialization buffer

**Example:** A 100MB image would use over 230MB of RAM during upload.

Now, files are processed in small 256KB chunks, using only about 660KB of RAM regardless of file size.

### Better Performance

- **Lower memory footprint**: More memory available for other operations
- **Faster start**: Upload begins immediately without waiting to load entire file
- **Scalability**: Can handle very large files without memory issues
- **Multiple files**: Memory usage doesn't multiply with file count

## Technical Details

### Supported File Types

The optimization applies to all file types:
- **Images**: PNG, JPEG, GIF, WebP (uploaded as base64 data URLs)
- **Documents**: PDF, text files (uploaded with OCR support)
- **Audio**: WAV, MP3, other formats (uploaded for audio-capable models)

### Streaming Process

1. **Chunked Reading**: Files are read in 256KB chunks from disk
2. **Incremental Encoding**: Base64 encoding happens on-the-fly for each chunk
3. **Direct Streaming**: Encoded data streams directly to HTTP request
4. **UTF-8 Safety**: Text files respect character boundaries (no split multi-byte characters)

### Memory Guarantees

- **Peak memory per upload**: ~660KB regardless of file size
- **Multiple files**: Processed sequentially to maintain memory bounds
- **mmap support**: Uses memory-mapped I/O when available for even better efficiency

## For Developers

### SyncStream Trait

If you're extending llumen with custom file sources, implement the `SyncStream` trait:

```rust
pub trait SyncStream {
    /// Read next chunk into buffer, returns number of bytes read
    fn read_chunk(&mut self, buf: &mut [u8]) -> usize;
    
    /// Get total size of the stream
    fn len(&self) -> usize;
}
```

### Example Implementation

```rust
use llumen::openrouter::SyncStream;

struct MyCustomStream {
    data: Vec<u8>,
    position: usize,
}

impl SyncStream for MyCustomStream {
    fn read_chunk(&mut self, buf: &mut [u8]) -> usize {
        let remaining = self.data.len() - self.position;
        let to_read = std::cmp::min(buf.len(), remaining);
        
        if to_read > 0 {
            buf[..to_read].copy_from_slice(
                &self.data[self.position..self.position + to_read]
            );
            self.position += to_read;
        }
        
        to_read
    }
    
    fn len(&self) -> usize {
        self.data.len()
    }
}
```

## Limitations

- **Sequential processing**: Files in a single request are processed one at a time
- **No progress callbacks**: Currently no API for upload progress tracking (coming soon)

## See Also

- [Architecture Documentation](../dev/chunked-streaming.md) - Technical implementation details
- [OpenRouter Integration](../dev/openrouter-integration.md) - LLM provider interface