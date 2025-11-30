# Build from Scratch

## Prerequisites

> [!IMPORTANT]
> Only Rust and Node.js are required. But having all make it easy to `create migration`/`trace performance`

- **Rust** (1.89+)
- **Node.js** 21+ and **pnpm**
- [**Just**](https://github.com/casey/just) (command runner)
- [**Nushell**](https://github.com/nushell/nushell) (shell for command runner)
- [**sea-orm-cli**](https://www.sea-ql.org/SeaORM/docs/generate-entity/sea-orm-cli/) (generate migration)
- [**mock-openai**](https://github.com/pinkfuwa/mock-openai) (for performance tracing)

## Backend Setup

```bash
cd backend

# Install dependencies and build
cargo build --release

# Run migrations
cargo run --bin migration

# Start the server
cargo run --release
```

The backend will start on `http://localhost:8001` by default.

### Configuration

Create a `.env` file in `backend/` (optional):

```env
DATABASE_URL=sqlite:db.sqlite
PORT=3000
```

## Frontend Setup

```bash
cd frontend

# Install dependencies
pnpm install

# Build for production
pnpm build

# The built files will be in frontend/build/
```

The backend serves the frontend static files from `frontend/build/` when running in production mode.

## Development Mode

### Backend

> [!IMPORTANT]
> `dev` feature enable CORS and other feature useful for development.

```bash
cd backend
cargo run --features dev
```

### Frontend (dev server)
```bash
cd frontend
pnpm dev
```

Frontend dev server runs on `http://localhost:5173` and proxies API calls to the backend.

## Type Generation

To sync types between Rust and TypeScript:

```bash
cd backend
cargo build  # Generates protocol/bindings.ts

cd ../frontend
pnpm run format-ffi  # Formats generated types
```
