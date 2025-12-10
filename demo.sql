/* --------------------------------------------------------------
   1️⃣  User message (id = 1) – unchanged apart from kind flag
   -------------------------------------------------------------- */
UPDATE message
SET
    kind        = 1,          -- 1 = user‑message
    price       = 0.0,
    token_count = 0
WHERE id = 1;

/* --------------------------------------------------------------
   2️⃣  First user chunk (id = 1) – simple welcome line
   -------------------------------------------------------------- */
UPDATE chunk
SET
    content = 'Welcome to **Llumen** – a modern LLM‑chat web‑app. '
            || 'Let's walk through its core features.',
    kind    = 0               -- plain‑text/markdown
WHERE id = 1;

/* --------------------------------------------------------------
   3️⃣  Placeholder "reasoning" chunk (id = 3) – fake content
   -------------------------------------------------------------- */
UPDATE chunk
SET
    content = '**Reasoning (placeholder)**' || char(10) ||
              '- This block is intentionally left vague.' || char(10) ||
              '- It is only displayed to keep the UI layout consistent.' || char(10) ||
              '- No substantive information is provided here.',
    kind    = 1               -- markdown‑styled placeholder
WHERE id = 3;

/* --------------------------------------------------------------
   4️⃣  Assistant response (id = 2) – keep price & token info
   -------------------------------------------------------------- */
UPDATE message
SET
    kind        = 2,                     -- 2 = assistant‑response
    price       = 0.00104109395761043,
    token_count = 5616
WHERE id = 2;

/* --------------------------------------------------------------
   5️⃣  Full tutorial now lives in the *answer* chunk (id = 4)
   -------------------------------------------------------------- */
UPDATE chunk
SET
    content = '**Answer – Llumen Tutorial**' || char(10) || char(10) ||
              '### Features on a new chat' || char(10) ||
              '1. **Markdown** support – write rich text, tables, code blocks, …' || char(10) ||
              '2. **File upload** and **search‑mode** toggle located under the message input.' || char(10) || char(10) ||
              '### Features on each message' || char(10) ||
              '1. **Edit** button under the user's own messages (no need to clone the chat).' || char(10) ||
              '2. **Completion cost** badge displayed beneath the assistant's response.' || char(10) || char(10) ||
              '### Settings (bottom‑left gear icon in sidebar)' || char(10) ||
              '1. **Model configuration** via a TOML file.' || char(10) ||
              '2. **Preference configuration** – locale & theme selection.' || char(10) ||
              '3. **User management** panel.' || char(10) || char(10) ||
              '### Run Llumen with Docker' || char(10) ||
              '```bash' || char(10) ||
              'docker run -it --rm -e API_KEY="<YOUR_OPENROUTER_API_KEY>" -p 80:80 -v "$(pwd)/data:/data" ghcr.io/pinkfuwa/llumen:latest' || char(10) ||
              '```' || char(10) || char(10) ||
              'Open **http://localhost** (or port 80) and log in with the default admin credentials:' || char(10) ||
              '- **Username:** `admin`' || char(10) ||
              '- **Password:** `P@88w0rd`' || char(10) || char(10) ||
              'You are now ready to explore Markdown editing, file uploads, model tuning via TOML, theme switching, and user administration.',
    kind    = 0               -- plain‑text/markdown (final answer)
WHERE id = 4;

/* --------------------------------------------------------------
   6️⃣  (Optional) Keep the demo file entry accurate
   -------------------------------------------------------------- */
UPDATE file
SET
    mime_type = 'image/png'   -- demo.png is a PNG image
WHERE id = 1;

UPDATE chunk
SET
    content = '{"name":"demo.png","id":1,"description":"Sample image used in the tutorial"}',
    kind    = 7           -- 7 = JSON‑metadata chunk (kept from the original schema)
WHERE id = 2;
