# Demo Deployment Guide

This guide explains how to deploy and configure the Llumen demo site.

## Overview

The demo branch includes several restrictions to prevent users from modifying critical data:

### Backend Restrictions

- **Chat Operations**: Chat ID 1 (tutorial chat) cannot be deleted
- **Message Operations**: Messages in chat ID 1 cannot be created or deleted
- **Model Operations**: All model CRUD operations are disabled (create, delete, update)
- **User Operations**: All user management operations are disabled (create, delete, read, update)

### Frontend Changes

- **Mock API**: User-related API calls use mock data to avoid unnecessary backend requests
- **Default Route**: Users are redirected to `/chat/1` instead of `/chat/new` after login
- **Preference Handling**: User preferences are mocked on the frontend

### Configuration Changes

- **Search Mode**: Web search is disabled in search configuration
- **PDF Engine**: Changed from `mistral-ocr` to `pdf-text` for document parsing

## Database Setup

After deploying the application, you need to populate the database with tutorial content:

1. Ensure your database is initialized with migrations
2. Run the demo SQL script to populate tutorial content:

```bash
# For SQLite (default)
sqlite3 data/db.sqlite < demo.sql

# For PostgreSQL
psql -h localhost -U your_user -d llumen < demo.sql
```

The `demo.sql` script will:
- Create a welcome message (message ID 1)
- Set up tutorial chunks explaining Llumen features
- Configure a sample file attachment (demo.png)
- Populate chat ID 1 with interactive tutorial content

## Docker Deployment

### Standard Demo Deployment

```bash
docker run -d \
  --name llumen-demo \
  -e API_KEY="<YOUR_OPENROUTER_API_KEY>" \
  -p 80:80 \
  -v "$(pwd)/data:/data" \
  ghcr.io/pinkfuwa/llumen:demo
```

### With Docker Compose

Create a `docker-compose.demo.yml`:

```yaml
version: '3.8'

services:
  llumen-demo:
    image: ghcr.io/pinkfuwa/llumen:demo
    container_name: llumen-demo
    environment:
      - API_KEY=${OPENROUTER_API_KEY}
    ports:
      - "80:80"
    volumes:
      - ./data:/data
    restart: unless-stopped
```

Run with:

```bash
OPENROUTER_API_KEY="your_key_here" docker-compose -f docker-compose.demo.yml up -d
```

## Post-Deployment Setup

### 1. Initialize Database

On first run, the database will be created automatically with migrations.

### 2. Apply Demo Content

```bash
# Copy demo.sql to the container
docker cp demo.sql llumen-demo:/tmp/demo.sql

# Execute the SQL script
docker exec llumen-demo sqlite3 /data/db.sqlite < /tmp/demo.sql
```

### 3. Create Demo Image (Optional)

If you want to include the demo.png referenced in the tutorial:

```bash
# Create a sample image or use your own
docker cp demo.png llumen-demo:/data/files/1.png
```

### 4. Access the Demo

Navigate to `http://localhost` (or your configured domain) and log in with:

- **Username**: `admin`
- **Password**: `P@88w0rd`

## Environment Variables

Required:
- `API_KEY`: Your OpenRouter API key for LLM access

Optional:
- `DATABASE_URL`: Custom database connection string (defaults to SQLite)
- `RUST_LOG`: Logging level (default: `info`)
- `PORT`: Internal port (default: `3000`)

## Maintenance

### Resetting Demo Data

To reset the demo to its initial state:

```bash
# Backup current database
docker exec llumen-demo cp /data/db.sqlite /data/db.sqlite.backup

# Reapply demo content
docker exec llumen-demo sqlite3 /data/db.sqlite < /tmp/demo.sql
```

### Monitoring

Check application logs:

```bash
docker logs -f llumen-demo
```

### Updating

To update to the latest demo version:

```bash
docker pull ghcr.io/pinkfuwa/llumen:demo
docker-compose -f docker-compose.demo.yml down
docker-compose -f docker-compose.demo.yml up -d
```

## Security Considerations

1. **Change Default Password**: After deployment, consider changing the admin password through the UI (though user operations are restricted in demo mode)
2. **API Key Protection**: Never commit your `API_KEY` to version control
3. **Rate Limiting**: Consider adding rate limiting if exposing to public internet
4. **Reverse Proxy**: Use nginx or similar for SSL/TLS termination:

```nginx
server {
    listen 443 ssl http2;
    server_name demo.llumen.example.com;

    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;

    location / {
        proxy_pass http://localhost:80;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

## Troubleshooting

### Demo Content Not Showing

1. Verify the SQL script was applied successfully
2. Check that chat ID 1 exists in the database
3. Ensure file ID 1 is properly configured

### API Errors

1. Verify `API_KEY` environment variable is set correctly
2. Check OpenRouter API key validity
3. Review logs for specific error messages

### Permission Issues

Ensure the data directory has proper permissions:

```bash
chmod -R 755 data/
```

## Development

To build the demo branch locally:

```bash
git checkout demo
docker build -t llumen:demo .
```

## Additional Resources

- [Main README](../README.md)
- [User Documentation](user/)
- [Development Documentation](dev/)