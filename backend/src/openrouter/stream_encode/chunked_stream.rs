use base64::Engine;
use bytes::Bytes;
use std::io::Write;
use tokio::sync::mpsc;

use crate::openrouter::SyncStream;
use struson::writer::{JsonWriter, StringValueWriter};

const CHUNK_SIZE: usize = 256 * 1024; // 256KB

/// Stream text content in chunks while respecting UTF-8 boundaries
#[allow(dead_code)]
pub fn stream_text_to_channel<S: SyncStream + 'static>(
    mut source: S,
    tx: mpsc::Sender<Result<Bytes, std::io::Error>>,
) -> Result<(), std::io::Error> {
    let mut buffer = vec![0u8; CHUNK_SIZE];
    let mut leftover = Vec::new();

    loop {
        let read = source.read_chunk(&mut buffer);
        if read == 0 {
            // Stream finished, send any leftover data
            if !leftover.is_empty() {
                tx.blocking_send(Ok(Bytes::from(leftover))).map_err(|_| {
                    std::io::Error::new(std::io::ErrorKind::BrokenPipe, "Channel closed")
                })?;
            }
            break;
        }

        // Combine leftover from previous iteration with new data
        let mut data = if leftover.is_empty() {
            buffer[..read].to_vec()
        } else {
            let mut combined = std::mem::take(&mut leftover);
            combined.extend_from_slice(&buffer[..read]);
            combined
        };

        // Find the last valid UTF-8 boundary
        let valid_len = find_valid_utf8_boundary(&data);

        if valid_len < data.len() {
            // Save incomplete UTF-8 sequence for next iteration
            leftover = data[valid_len..].to_vec();
            data.truncate(valid_len);
        }

        if !data.is_empty() {
            tx.blocking_send(Ok(Bytes::from(data))).map_err(|_| {
                std::io::Error::new(std::io::ErrorKind::BrokenPipe, "Channel closed")
            })?;
        }
    }

    Ok(())
}

/// Find the last valid UTF-8 character boundary in a byte slice
pub fn find_valid_utf8_boundary(data: &[u8]) -> usize {
    if data.is_empty() {
        return 0;
    }

    // Check if the entire slice is valid UTF-8
    if std::str::from_utf8(data).is_ok() {
        return data.len();
    }

    // Walk backwards to find the last valid UTF-8 boundary
    // UTF-8 continuation bytes start with 10xxxxxx
    let mut pos = data.len();
    while pos > 0 {
        pos -= 1;
        let byte = data[pos];

        // Check if this is not a continuation byte (not 10xxxxxx)
        if (byte & 0b1100_0000) != 0b1000_0000 {
            // This is a potential start of a character
            // Validate from this position
            if std::str::from_utf8(&data[..pos]).is_ok() {
                return pos;
            }
        }
    }

    // If we couldn't find any valid boundary, return 0
    0
}

/// Stream base64-encoded content in chunks
#[allow(dead_code)]
pub fn stream_base64_to_channel<S: SyncStream + 'static>(
    mut source: S,
    tx: mpsc::Sender<Result<Bytes, std::io::Error>>,
    prefix: Option<&str>,
) -> Result<(), std::io::Error> {
    // Write prefix if provided
    if let Some(prefix) = prefix {
        tx.blocking_send(Ok(Bytes::from(prefix.to_string())))
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::BrokenPipe, "Channel closed"))?;
    }

    let mut buffer = vec![0u8; CHUNK_SIZE];
    let mut leftover = Vec::new();

    loop {
        let read = source.read_chunk(&mut buffer);
        if read == 0 {
            // Stream finished, encode any leftover data
            if !leftover.is_empty() {
                let encoded = base64::engine::general_purpose::STANDARD.encode(&leftover);
                tx.blocking_send(Ok(Bytes::from(encoded))).map_err(|_| {
                    std::io::Error::new(std::io::ErrorKind::BrokenPipe, "Channel closed")
                })?;
            }
            break;
        }

        // Combine leftover from previous iteration with new data
        let data = if leftover.is_empty() {
            &buffer[..read]
        } else {
            leftover.extend_from_slice(&buffer[..read]);
            &leftover[..]
        };

        // Base64 encodes 3 bytes to 4 characters
        // Process complete 3-byte chunks, keep remainder for next iteration
        let processable_len = (data.len() / 3) * 3;

        if processable_len > 0 {
            let encoded =
                base64::engine::general_purpose::STANDARD.encode(&data[..processable_len]);
            tx.blocking_send(Ok(Bytes::from(encoded))).map_err(|_| {
                std::io::Error::new(std::io::ErrorKind::BrokenPipe, "Channel closed")
            })?;

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

/// Write text string value using struson's string_value_writer
#[allow(dead_code)]
pub fn write_text_stream<W: Write, S: SyncStream + 'static>(
    json_writer: &mut struson::writer::JsonStreamWriter<W>,
    stream: &mut S,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut string_writer = json_writer.string_value_writer()?;
    let mut buffer = vec![0u8; CHUNK_SIZE];
    let mut leftover = Vec::new();

    loop {
        let read = stream.read_chunk(&mut buffer);
        if read == 0 {
            // Stream finished, write any leftover data
            if !leftover.is_empty() {
                string_writer.write_all(&leftover)?;
            }
            break;
        }

        // Combine leftover from previous iteration with new data
        let mut data = if leftover.is_empty() {
            buffer[..read].to_vec()
        } else {
            let mut combined = std::mem::take(&mut leftover);
            combined.extend_from_slice(&buffer[..read]);
            combined
        };

        // Find the last valid UTF-8 boundary
        let valid_len = find_valid_utf8_boundary(&data);

        if valid_len < data.len() {
            // Save incomplete UTF-8 sequence for next iteration
            leftover = data[valid_len..].to_vec();
            data.truncate(valid_len);
        }

        if !data.is_empty() {
            string_writer.write_all(&data)?;
        }
    }

    string_writer.finish_value()?;
    Ok(())
}

/// Write base64-encoded stream using struson's string_value_writer
pub fn write_base64_stream<W: Write, S: SyncStream + 'static>(
    json_writer: &mut struson::writer::JsonStreamWriter<W>,
    stream: &mut S,
    prefix: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut string_writer = json_writer.string_value_writer()?;

    // Write prefix if provided
    if let Some(prefix) = prefix {
        string_writer.write_all(prefix.as_bytes())?;
    }

    let mut buffer = vec![0u8; CHUNK_SIZE];
    let mut leftover = Vec::new();

    loop {
        let read = stream.read_chunk(&mut buffer);
        if read == 0 {
            // Stream finished, encode any leftover data
            if !leftover.is_empty() {
                let encoded = base64::engine::general_purpose::STANDARD.encode(&leftover);
                string_writer.write_all(encoded.as_bytes())?;
            }
            break;
        }

        // Combine leftover from previous iteration with new data
        let data = if leftover.is_empty() {
            &buffer[..read]
        } else {
            leftover.extend_from_slice(&buffer[..read]);
            &leftover[..]
        };

        // Base64 encodes 3 bytes to 4 characters
        // Process complete 3-byte chunks, keep remainder for next iteration
        let processable_len = (data.len() / 3) * 3;

        if processable_len > 0 {
            let encoded =
                base64::engine::general_purpose::STANDARD.encode(&data[..processable_len]);
            string_writer.write_all(encoded.as_bytes())?;

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

    string_writer.finish_value()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::openrouter::VecStream;

    #[test]
    fn test_find_valid_utf8_boundary() {
        // Valid ASCII
        let data = b"hello";
        assert_eq!(find_valid_utf8_boundary(data), 5);

        // Valid UTF-8 with multi-byte characters
        let data = "hello 世界".as_bytes();
        assert_eq!(find_valid_utf8_boundary(data), data.len());

        // Incomplete UTF-8 sequence (世 = E4 B8 96, cut after E4 B8)
        let mut data = Vec::from("hello 世界".as_bytes());
        let original_len = data.len();
        data.truncate(original_len - 4); // Cut in the middle of 界
        let boundary = find_valid_utf8_boundary(&data);
        assert!(boundary < data.len());
        assert!(std::str::from_utf8(&data[..boundary]).is_ok());
    }

    #[test]
    fn test_stream_text_respects_utf8_boundaries() {
        // Create a text with multi-byte UTF-8 characters
        let text = "Hello 世界! ".repeat(1000); // Repeat to make it larger
        let data = text.as_bytes().to_vec();

        let stream = VecStream::new(data.clone());
        let (tx, mut rx) = mpsc::channel(10);

        std::thread::spawn(move || {
            stream_text_to_channel(stream, tx).unwrap();
        });

        let mut result = Vec::new();
        while let Some(Ok(chunk)) = rx.blocking_recv() {
            // Each chunk should be valid UTF-8
            assert!(std::str::from_utf8(&chunk).is_ok());
            result.extend_from_slice(&chunk);
        }

        assert_eq!(result, data);
    }

    #[test]
    fn test_stream_base64_encoding() {
        let data = vec![0u8, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        let expected = base64::engine::general_purpose::STANDARD.encode(&data);

        let stream = VecStream::new(data);
        let (tx, mut rx) = mpsc::channel(10);

        std::thread::spawn(move || {
            stream_base64_to_channel(stream, tx, None).unwrap();
        });

        let mut result = String::new();
        while let Some(Ok(chunk)) = rx.blocking_recv() {
            result.push_str(&String::from_utf8(chunk.to_vec()).unwrap());
        }

        assert_eq!(result, expected);
    }

    #[test]
    fn test_stream_base64_with_prefix() {
        let data = vec![0u8, 1, 2, 3, 4, 5];
        let prefix = "data:image/png;base64,";
        let expected_data = base64::engine::general_purpose::STANDARD.encode(&data);
        let expected = format!("{}{}", prefix, expected_data);

        let stream = VecStream::new(data);
        let (tx, mut rx) = mpsc::channel(10);

        let prefix_owned = prefix.to_string();
        std::thread::spawn(move || {
            stream_base64_to_channel(stream, tx, Some(&prefix_owned)).unwrap();
        });

        let mut result = String::new();
        while let Some(Ok(chunk)) = rx.blocking_recv() {
            result.push_str(&String::from_utf8(chunk.to_vec()).unwrap());
        }

        assert_eq!(result, expected);
    }
}
