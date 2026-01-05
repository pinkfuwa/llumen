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

    let mut read_buffer = vec![0u8; CHUNK_SIZE];
    let mut leftover = Vec::new();

    loop {
        let read = source.read_chunk(&mut read_buffer);
        if read == 0 {
            // Stream finished, encode any leftover data
            if !leftover.is_empty() {
                let encoded = base64::engine::general_purpose::STANDARD.encode(&leftover);
                writer.write_all(encoded.as_bytes())?;
            }
            break;
        }

        // Combine leftover from previous iteration with new data
        let data = if leftover.is_empty() {
            &read_buffer[..read]
        } else {
            leftover.extend_from_slice(&read_buffer[..read]);
            &leftover[..]
        };

        // Base64 encodes 3 bytes to 4 characters
        // Process complete 3-byte chunks, keep remainder for next iteration
        let processable_len = (data.len() / 3) * 3;

        if processable_len > 0 {
            let encoded =
                base64::engine::general_purpose::STANDARD.encode(&data[..processable_len]);
            writer.write_all(encoded.as_bytes())?;

            // Keep remainder for next iteration
            if processable_len < data.len() {
                leftover = data[processable_len..].to_vec();
            } else {
                leftover.clear();
            }
        } else {
            // Not enough data to encode yet, keep for next iteration
            if leftover.is_empty() {
                leftover = data.to_vec();
            }
        }
    }

    Ok(())
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
        fn read_chunk(&mut self, buf: &mut [u8]) -> usize {
            let remaining = self.data.len() - self.pos;
            let to_read = std::cmp::min(buf.len(), remaining);
            if to_read > 0 {
                buf[..to_read].copy_from_slice(&self.data[self.pos..self.pos + to_read]);
                self.pos += to_read;
            }
            to_read
        }

        fn len(&self) -> usize {
            self.data.len()
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
            pos: usize,
        }

        impl ChunkedStream {
            fn new(data: Vec<u8>) -> Self {
                Self { data, pos: 0 }
            }
        }

        impl SyncStream for ChunkedStream {
            fn read_chunk(&mut self, buf: &mut [u8]) -> usize {
                let remaining = self.data.len() - self.pos;
                let to_read = std::cmp::min(buf.len(), remaining);
                if to_read > 0 {
                    buf[..to_read].copy_from_slice(&self.data[self.pos..self.pos + to_read]);
                    self.pos += to_read;
                }
                to_read
            }

            fn len(&self) -> usize {
                self.data.len()
            }
        }

        // Test with data that will be written in small chunks
        let data = vec![0x41u8; 10000];
        let mut stream = ChunkedStream::new(data.clone());

        let mut output = Vec::new();
        encode_base64(&mut stream, &mut output, None).unwrap();

        let expected = base64::engine::general_purpose::STANDARD.encode(&data);
        assert_eq!(String::from_utf8(output).unwrap(), expected);
    }
}
