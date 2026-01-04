use bytes::Bytes;
use std::io::Write;
use tokio::sync::mpsc;

use super::encode_base64;
use crate::openrouter::{SyncStream, raw};

const BUFFER_SIZE: usize = 64 * 1024; // 64KB buffer for channel

/// Serialize CompletionReq to a streaming body using channels
pub async fn serialize_to_body<S: SyncStream + Send + 'static>(
    req: raw::CompletionReq,
    messages_with_streams: Vec<MessageWithStreams<S>>,
) -> (
    mpsc::Receiver<Result<Bytes, std::io::Error>>,
    tokio::task::JoinHandle<()>,
) {
    let (tx, rx) = mpsc::channel::<Result<Bytes, std::io::Error>>(4);

    let handle = tokio::task::spawn_blocking(move || {
        let result = write_json_to_channel(tx.clone(), req, messages_with_streams);
        if let Err(e) = result {
            let _ = tx.blocking_send(Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("JSON serialization error: {}", e),
            )));
        }
    });

    (rx, handle)
}

pub struct MessageWithStreams<S: SyncStream + 'static> {
    pub message: raw::Message,
    pub stream_files: Vec<StreamFileData<S>>,
}

pub struct StreamFileData<S: SyncStream + 'static> {
    pub part_index: usize,
    pub stream: S,
    pub filename: String,
}

fn write_json_to_channel<S: SyncStream + 'static>(
    tx: mpsc::Sender<Result<Bytes, std::io::Error>>,
    req: raw::CompletionReq,
    mut messages_with_streams: Vec<MessageWithStreams<S>>,
) -> Result<(), Box<dyn std::error::Error>> {
    use struson::writer::{JsonStreamWriter, JsonWriter};

    let writer = ChannelWriter::new(tx);
    let mut json_writer = JsonStreamWriter::new(writer);

    json_writer.begin_object()?;

    // model
    json_writer.name("model")?;
    json_writer.string_value(&req.model)?;

    // messages
    json_writer.name("messages")?;
    json_writer.begin_array()?;
    for msg_with_streams in &mut messages_with_streams {
        write_message(&mut json_writer, msg_with_streams)?;
    }
    json_writer.end_array()?;

    // stream
    json_writer.name("stream")?;
    json_writer.bool_value(req.stream)?;

    // Optional fields
    if let Some(temp) = req.temperature {
        json_writer.name("temperature")?;
        json_writer.fp_number_value(temp as f64)?;
    }

    if let Some(penalty) = req.repeat_penalty {
        json_writer.name("repeat_penalty")?;
        json_writer.fp_number_value(penalty as f64)?;
    }

    if let Some(top_k) = req.top_k {
        json_writer.name("top_k")?;
        json_writer.number_value(top_k as i64)?;
    }

    if let Some(top_p) = req.top_p {
        json_writer.name("top_p")?;
        json_writer.fp_number_value(top_p as f64)?;
    }

    if let Some(max_tokens) = req.max_tokens {
        json_writer.name("max_tokens")?;
        json_writer.number_value(max_tokens as i64)?;
    }

    // tools
    if !req.tools.is_empty() {
        json_writer.name("tools")?;
        write_serde_value(&mut json_writer, &req.tools)?;
    }

    // plugins
    if !req.plugins.is_empty() {
        json_writer.name("plugins")?;
        write_serde_value(&mut json_writer, &req.plugins)?;
    }

    // usage
    if let Some(usage) = &req.usage {
        json_writer.name("usage")?;
        write_serde_value(&mut json_writer, usage)?;
    }

    // response_format
    if let Some(response_format) = &req.response_format {
        json_writer.name("response_format")?;
        write_serde_value(&mut json_writer, response_format)?;
    }

    // reasoning
    if !req.reasoning.is_empty() {
        json_writer.name("reasoning")?;
        json_writer.begin_object()?;
        if let Some(effort) = &req.reasoning.effort {
            json_writer.name("effort")?;
            json_writer.string_value(effort)?;
        }
        if let Some(enabled) = req.reasoning.enabled {
            json_writer.name("enabled")?;
            json_writer.bool_value(enabled)?;
        }
        json_writer.end_object()?;
    }

    // modalities
    if !req.modalities.is_empty() {
        json_writer.name("modalities")?;
        json_writer.begin_array()?;
        for modality in &req.modalities {
            json_writer.string_value(modality)?;
        }
        json_writer.end_array()?;
    }

    json_writer.end_object()?;
    json_writer.finish_document()?;

    Ok(())
}

fn write_message<W: Write, S: SyncStream + 'static>(
    json_writer: &mut struson::writer::JsonStreamWriter<W>,
    msg_with_streams: &mut MessageWithStreams<S>,
) -> Result<(), Box<dyn std::error::Error>> {
    use struson::writer::JsonWriter;

    let message = &msg_with_streams.message;

    json_writer.begin_object()?;

    // role
    json_writer.name("role")?;
    let role_str = match message.role {
        raw::Role::User => "user",
        raw::Role::Assistant => "assistant",
        raw::Role::Tool => "tool",
        raw::Role::System => "system",
    };
    json_writer.string_value(role_str)?;

    // content (either content or contents, not both)
    if let Some(content) = &message.content {
        json_writer.name("content")?;
        json_writer.string_value(content)?;
    } else if let Some(contents) = &message.contents {
        json_writer.name("content")?;
        json_writer.begin_array()?;
        for (idx, part) in contents.iter().enumerate() {
            write_message_part(json_writer, part, &mut msg_with_streams.stream_files, idx)?;
        }
        json_writer.end_array()?;
    }

    // tool_calls
    if let Some(tool_calls) = &message.tool_calls {
        json_writer.name("tool_calls")?;
        write_serde_value(json_writer, tool_calls)?;
    }

    // tool_call_id
    if let Some(tool_call_id) = &message.tool_call_id {
        json_writer.name("tool_call_id")?;
        json_writer.string_value(tool_call_id)?;
    }

    // annotations
    if let Some(annotations) = &message.annotations {
        json_writer.name("annotations")?;
        write_serde_value(json_writer, annotations)?;
    }

    // reasoning_details
    if !message.reasoning_details.is_empty() {
        json_writer.name("reasoning_details")?;
        write_serde_value(json_writer, &message.reasoning_details)?;
    }

    json_writer.end_object()?;
    Ok(())
}

fn write_message_part<W: Write, S: SyncStream + 'static>(
    json_writer: &mut struson::writer::JsonStreamWriter<W>,
    part: &raw::MessagePart,
    stream_files: &mut [StreamFileData<S>],
    part_index: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    use struson::writer::JsonWriter;

    json_writer.begin_object()?;

    // type
    json_writer.name("type")?;
    let type_str = match part.r#type {
        raw::MultiPartMessageType::Text => "text",
        raw::MultiPartMessageType::ImageUrl => "image_url",
        raw::MultiPartMessageType::File => "file",
        raw::MultiPartMessageType::InputAudio => "input_audio",
    };
    json_writer.string_value(type_str)?;

    // text
    if let Some(text) = &part.text {
        json_writer.name("text")?;
        json_writer.string_value(text)?;
    }

    // Check if we need to stream a file for this part
    let mut stream_file_opt = stream_files
        .iter_mut()
        .find(|sf| sf.part_index == part_index);

    // input_audio
    if let Some(input_audio) = &part.input_audio {
        if let Some(ref mut stream_data) = stream_file_opt {
            json_writer.name("input_audio")?;
            json_writer.begin_object()?;
            json_writer.name("format")?;

            // Extract format from filename
            let format = stream_data
                .filename
                .rsplit('.')
                .next()
                .unwrap_or("wav")
                .to_string();
            json_writer.string_value(&format)?;

            json_writer.name("data")?;
            write_base64_stream_inline(json_writer, &mut stream_data.stream, None)?;
            json_writer.end_object()?;
        } else {
            write_serde_value(json_writer, input_audio)?;
        }
    }

    // file
    if let Some(file) = &part.file {
        if let Some(ref mut stream_data) = stream_file_opt.as_mut() {
            json_writer.name("file")?;
            json_writer.begin_object()?;
            json_writer.name("filename")?;
            json_writer.string_value(&stream_data.filename)?;
            json_writer.name("file_data")?;

            // Detect mime type for file
            let mime_prefix = if stream_data.filename.ends_with(".pdf") {
                Some("data:application/pdf;base64,")
            } else {
                None
            };

            write_base64_stream_inline(json_writer, &mut stream_data.stream, mime_prefix)?;
            json_writer.end_object()?;
        } else {
            write_serde_value(json_writer, file)?;
        }
    }

    // image_url
    if let Some(image_url) = &part.image_url {
        if let Some(ref mut stream_data) = stream_file_opt.as_mut() {
            json_writer.name("image_url")?;
            json_writer.begin_object()?;
            json_writer.name("url")?;

            // Extract image format from filename
            let ext = stream_data.filename.rsplit('.').next().unwrap_or("png");
            let mime_prefix = format!("data:image/{};base64,", ext);

            write_base64_stream_inline(json_writer, &mut stream_data.stream, Some(&mime_prefix))?;
            json_writer.end_object()?;
        } else {
            write_serde_value(json_writer, image_url)?;
        }
    }

    json_writer.end_object()?;
    Ok(())
}

fn write_base64_stream_inline<W: Write, S: SyncStream + 'static>(
    json_writer: &mut struson::writer::JsonStreamWriter<W>,
    stream: &mut S,
    prefix: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    use struson::writer::JsonWriter;

    // We accumulate each file's base64-encoded content in memory here because:
    // 1. struson doesn't provide access to underlying writer during JSON construction
    // 2. This is still a MAJOR improvement over the previous approach:
    //    - BEFORE: Entire CompletionReq with ALL files serialized via .json(&req) - ALL files in memory at once
    //    - NOW: Only ONE file's base64 at a time, streamed through channel in 64KB chunks
    // 3. The Reader uses mmap, so the source file isn't copied into memory
    // 4. Base64 encoding happens in 256KB chunks (see base64_encoder.rs)
    // 5. The ChannelWriter flushes at 64KB boundaries, so the base64 string flows to HTTP immediately
    //
    // For a 100MB file:
    // - Base64 encoded: ~133MB string in memory temporarily
    // - Then written through channel and freed
    // - vs OLD approach: 100MB + 133MB + rest of request ALL in memory simultaneously

    let mut buffer = Vec::new();
    encode_base64(stream, &mut buffer, prefix)?;

    let base64_str = String::from_utf8(buffer)?;
    json_writer.string_value(&base64_str)?;

    Ok(())
}

fn write_serde_value<W: Write, T: serde::Serialize>(
    json_writer: &mut struson::writer::JsonStreamWriter<W>,
    value: &T,
) -> Result<(), Box<dyn std::error::Error>> {
    let json_value = serde_json::to_value(value)?;
    write_json_value(json_writer, &json_value)?;
    Ok(())
}

fn write_json_value<W: Write>(
    json_writer: &mut struson::writer::JsonStreamWriter<W>,
    value: &serde_json::Value,
) -> Result<(), Box<dyn std::error::Error>> {
    use struson::writer::JsonWriter;

    match value {
        serde_json::Value::Null => json_writer.null_value()?,
        serde_json::Value::Bool(b) => json_writer.bool_value(*b)?,
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                json_writer.number_value(i)?;
            } else if let Some(f) = n.as_f64() {
                json_writer.fp_number_value(f)?;
            }
        }
        serde_json::Value::String(s) => json_writer.string_value(s)?,
        serde_json::Value::Array(arr) => {
            json_writer.begin_array()?;
            for item in arr {
                write_json_value(json_writer, item)?;
            }
            json_writer.end_array()?;
        }
        serde_json::Value::Object(obj) => {
            json_writer.begin_object()?;
            for (key, val) in obj {
                json_writer.name(key)?;
                write_json_value(json_writer, val)?;
            }
            json_writer.end_object()?;
        }
    }
    Ok(())
}

/// A writer that sends data through a channel as Bytes
struct ChannelWriter {
    tx: mpsc::Sender<Result<Bytes, std::io::Error>>,
    buffer: Vec<u8>,
}

impl ChannelWriter {
    fn new(tx: mpsc::Sender<Result<Bytes, std::io::Error>>) -> Self {
        Self {
            tx,
            buffer: Vec::with_capacity(BUFFER_SIZE),
        }
    }

    fn flush_buffer(&mut self) -> std::io::Result<()> {
        if !self.buffer.is_empty() {
            let data = std::mem::replace(&mut self.buffer, Vec::with_capacity(BUFFER_SIZE));
            self.tx.blocking_send(Ok(Bytes::from(data))).map_err(|_| {
                std::io::Error::new(std::io::ErrorKind::BrokenPipe, "Channel closed")
            })?;
        }
        Ok(())
    }
}

impl Write for ChannelWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buffer.extend_from_slice(buf);

        if self.buffer.len() >= BUFFER_SIZE {
            self.flush_buffer()?;
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.flush_buffer()
    }
}

impl Drop for ChannelWriter {
    fn drop(&mut self) {
        let _ = self.flush_buffer();
    }
}
