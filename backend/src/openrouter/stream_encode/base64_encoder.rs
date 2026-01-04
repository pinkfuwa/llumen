use std::io::Write;

use base64::Engine;

use super::super::SyncStream;

const CHUNK_SIZE: usize = 256 * 1024;

/// Encode base64 data from a SyncStream source to a writer
/// This implementation streams data in chunks without loading the entire file into memory
pub fn encode_base64<S: SyncStream, W: Write>(
    source: &mut S,
    writer: &mut W,
    prefix: Option<&str>,
) -> std::io::Result<()> {
    // Write prefix if provided
    if let Some(prefix) = prefix {
        writer.write_all(prefix.as_bytes())?;
    }

    // Create a chunked writer that reads from source and writes base64 to writer
    let mut chunk_buffer = Vec::with_capacity(CHUNK_SIZE);
    let mut chunk_writer = ChunkWriter {
        buffer: &mut chunk_buffer,
        target: writer,
    };

    // Read from source into our chunk writer
    source.read(&mut chunk_writer);

    // Flush any remaining data
    chunk_writer.flush()?;

    Ok(())
}

/// A writer that accumulates data in chunks and encodes each chunk to base64
struct ChunkWriter<'a, W: Write> {
    buffer: &'a mut Vec<u8>,
    target: &'a mut W,
}

impl<'a, W: Write> Write for ChunkWriter<'a, W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buffer.extend_from_slice(buf);

        // When buffer is full, encode and write
        if self.buffer.len() >= CHUNK_SIZE {
            self.flush_buffer()?;
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.flush_buffer()
    }
}

impl<'a, W: Write> ChunkWriter<'a, W> {
    fn flush_buffer(&mut self) -> std::io::Result<()> {
        if self.buffer.is_empty() {
            return Ok(());
        }

        // Encode the buffer to base64
        let encoded = base64::engine::general_purpose::STANDARD.encode(&self.buffer);
        self.target.write_all(encoded.as_bytes())?;

        // Clear buffer for next chunk
        self.buffer.clear();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine;

    struct TestStream {
        data: Vec<u8>,
        pos: usize,
    }

    impl TestStream {
        fn new(data: Vec<u8>) -> Self {
            Self { data, pos: 0 }
        }
    }

    impl SyncStream for TestStream {
        fn read(&mut self, writer: &mut dyn Write) -> usize {
            let len = self.data.len() - self.pos;
            if len > 0 {
                writer.write_all(&self.data[self.pos..]).ok();
                self.pos = self.data.len();
            }
            len
        }
    }

    #[test]
    fn test_basic_encoding() {
        let data = b"Hello, World!".to_vec();
        let mut stream = TestStream::new(data.clone());

        let mut output = Vec::new();
        encode_base64(&mut stream, &mut output, None).unwrap();

        let expected = base64::engine::general_purpose::STANDARD.encode(&data);
        assert_eq!(String::from_utf8(output).unwrap(), expected);
    }

    #[test]
    fn test_with_prefix() {
        let data = b"Test data".to_vec();
        let mut stream = TestStream::new(data.clone());

        let mut output = Vec::new();
        encode_base64(&mut stream, &mut output, Some("data:image/png;base64,")).unwrap();

        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.starts_with("data:image/png;base64,"));

        let encoded_part = output_str.strip_prefix("data:image/png;base64,").unwrap();
        let expected = base64::engine::general_purpose::STANDARD.encode(&data);
        assert_eq!(encoded_part, expected);
    }

    #[test]
    fn test_large_data() {
        // Create data larger than CHUNK_SIZE
        let data = vec![0x42u8; CHUNK_SIZE * 2 + 1000];
        let mut stream = TestStream::new(data.clone());

        let mut output = Vec::new();
        encode_base64(&mut stream, &mut output, None).unwrap();

        let expected = base64::engine::general_purpose::STANDARD.encode(&data);
        assert_eq!(String::from_utf8(output).unwrap(), expected);
    }

    #[test]
    fn test_empty_data() {
        let data = Vec::new();
        let mut stream = TestStream::new(data);

        let mut output = Vec::new();
        encode_base64(&mut stream, &mut output, None).unwrap();

        assert_eq!(output.len(), 0);
    }

    #[test]
    fn test_single_byte() {
        let data = vec![0x42];
        let mut stream = TestStream::new(data.clone());

        let mut output = Vec::new();
        encode_base64(&mut stream, &mut output, None).unwrap();

        let expected = base64::engine::general_purpose::STANDARD.encode(&data);
        assert_eq!(String::from_utf8(output).unwrap(), expected);
    }

    #[test]
    fn test_chunked_streaming() {
        // Test with a stream that writes in multiple small chunks
        struct ChunkedStream {
            data: Vec<u8>,
            chunk_size: usize,
            pos: usize,
        }

        impl ChunkedStream {
            fn new(data: Vec<u8>, chunk_size: usize) -> Self {
                Self {
                    data,
                    chunk_size,
                    pos: 0,
                }
            }
        }

        impl SyncStream for ChunkedStream {
            fn read(&mut self, writer: &mut dyn Write) -> usize {
                let mut written = 0;
                while self.pos < self.data.len() {
                    let end = std::cmp::min(self.pos + self.chunk_size, self.data.len());
                    writer.write_all(&self.data[self.pos..end]).ok();
                    written += end - self.pos;
                    self.pos = end;
                }
                written
            }
        }

        // Test with data that will be written in small chunks
        let data = vec![0x41u8; 10000];
        let mut stream = ChunkedStream::new(data.clone(), 1024);

        let mut output = Vec::new();
        encode_base64(&mut stream, &mut output, None).unwrap();

        let expected = base64::engine::general_purpose::STANDARD.encode(&data);
        assert_eq!(String::from_utf8(output).unwrap(), expected);
    }
}
