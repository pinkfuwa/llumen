# File Lifecycle Management

This document describes how file uploads are managed in Llumen, including temporary file handling, chat association, and automatic cleanup.

## Overview

Files in Llumen follow a two-stage lifecycle:
1. **Temporary stage**: Files are uploaded without being associated with a chat room
2. **Permanent stage**: Files are associated with a chat room when messages are created

This design allows users to upload files before they're ready to send a message, while ensuring unused files don't accumulate indefinitely.

## Database Schema

The `file` table contains the following columns:

```sql
CREATE TABLE file (
    id INTEGER PRIMARY KEY,
    chat_id INTEGER,           -- NULL for temporary files, set when associated
    owner_id INTEGER,          -- User who uploaded the file (for query optimization)
    mime_type TEXT,           -- MIME type of the file
    valid_until INTEGER,      -- Unix timestamp when temporary file expires
    FOREIGN KEY (chat_id) REFERENCES chat(id) ON DELETE SET NULL ON UPDATE SET NULL,
    FOREIGN KEY (owner_id) REFERENCES user(id) ON DELETE SET NULL ON UPDATE SET NULL
);

CREATE INDEX idx_file_chat_id_valid_until ON file(chat_id, valid_until);
```

The composite index on `(chat_id, valid_until)` optimizes both:
- Querying files by chat_id (for message display)
- Finding expired temporary files (for cleanup)

## File Upload Flow

### 1. Initial Upload

When a user uploads a file via `POST /api/file/upload`:

- Request body (multipart/form-data):
  - `size`: File size in bytes
  - `file`: The file content

- The backend:
  1. Validates file size against `MAX_FILE_SIZE` (128MB)
  2. Creates a database record with:
     - `chat_id = NULL` (temporary file)
     - `owner_id = <current_user_id>`
     - `valid_until = now + 3600` (1 hour)
  3. Stores file content in redb blob storage
  4. Returns the file ID

**Code location**: `backend/src/routes/file/upload.rs`

### 2. Refreshing Expiration

Users can extend the expiration time via `POST /api/file/refresh`:

- Request body:
  ```json
  {
    "id": 123
  }
  ```

- The backend:
  1. Verifies the user owns the file
  2. Updates `valid_until` to `now + 3600`
  3. Returns the new expiration timestamp

**Code location**: `backend/src/routes/file/refresh.rs`

### 3. Chat Association

When a message is created via `POST /api/message/create`:

- The message includes file references:
  ```json
  {
    "chat_id": 456,
    "text": "Check out this file",
    "files": [
      { "id": 123, "name": "document.pdf" }
    ]
  }
  ```

- The backend:
  1. Verifies the user owns all referenced files
  2. Updates file records:
     - `chat_id = <chat_id>`
     - `valid_until = NULL` (no longer temporary)
  3. Creates the message with file references

This makes the files permanent by associating them with the chat room.

**Code location**: `backend/src/routes/message/create.rs`

## Automatic Cleanup

A background service runs every 5 minutes to clean up expired temporary files.

### Cleanup Process

1. Query for expired files:
   ```sql
   SELECT * FROM file 
   WHERE chat_id IS NULL 
     AND valid_until IS NOT NULL 
     AND valid_until <= <current_timestamp>
   ```

2. For each expired file:
   - Delete the blob from redb storage (ignoring "not found" errors)
   - Delete the database record

### Error Handling

- If blob deletion fails (except "not found"), log a warning but continue
- If database deletion fails, log an error and skip to next file
- The service continues running even if individual deletions fail

**Code location**: `backend/src/utils/file_cleanup.rs`

### Service Initialization

The cleanup service is initialized in `main.rs`:

```rust
let cleanup_service = Arc::new(utils::file_cleanup::FileCleanupService::new(
    conn.clone(),
    blob.clone(),
));
cleanup_service.clone().start();
```

## File Access Control

All file operations verify ownership:

- **Upload**: Files are automatically owned by the uploading user
- **Refresh**: Only the owner can refresh expiration
- **Association**: Only the owner can associate files with chats
- **Download**: Access control is based on chat membership (not shown in this document)

## Migration

The `valid_until` field was added via migration `m20251227_085232_add_valid_until_to_file`:

1. Add `valid_until` column as nullable integer
2. Drop old index `idx-file-chat_id`
3. Create new composite index `idx-file-chat_id-valid_until`

Existing files (created before this migration) will have `valid_until = NULL` and won't be cleaned up automatically unless they're temporary files without a chat_id.

## Design Rationale

### Why temporary files?

- **User experience**: Users can upload files before composing their message
- **Reliability**: If message creation fails, files don't get orphaned
- **Performance**: File uploads can happen in parallel with other UI interactions

### Why 1-hour expiration?

- Long enough for normal message composition
- Short enough to prevent storage waste
- Users can refresh if they need more time

### Why composite index?

The `(chat_id, valid_until)` index optimizes two critical queries:

1. **Finding chat files**: `WHERE chat_id = ?` (uses index prefix)
2. **Finding expired files**: `WHERE chat_id IS NULL AND valid_until <= ?`

This avoids needing two separate indexes.

### Why set valid_until to NULL instead of deleting it?

This allows the database schema to clearly distinguish:
- Permanent files: `chat_id IS NOT NULL` (regardless of valid_until)
- Temporary files: `chat_id IS NULL AND valid_until IS NOT NULL`
- Legacy files: `valid_until IS NULL` (from before this feature)

## Testing Considerations

When testing file uploads:

1. Upload a file and verify `valid_until` is set
2. Wait 1 hour (or mock time) and verify cleanup runs
3. Associate file with chat and verify `valid_until` becomes NULL
4. Verify cleanup doesn't delete associated files
5. Test refresh endpoint extends expiration
6. Verify ownership checks prevent unauthorized access

## Future Improvements

Potential enhancements:

- Configurable expiration time (per user or system-wide)
- Admin API to list/clean orphaned files
- Metrics on file storage usage and cleanup efficiency
- Soft-delete with grace period before actual deletion
- File preview generation for images/PDFs