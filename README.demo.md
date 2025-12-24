# Llumen Demo

This is the **demo branch** of Llumen - a modern LLM chat web application with restricted operations for public demonstration.

## 🚀 Quick Start

### Using Docker

```bash
docker run -d \
  --name llumen-demo \
  -e API_KEY="<YOUR_OPENROUTER_API_KEY>" \
  -p 80:80 \
  -v "$(pwd)/data:/data" \
  ghcr.io/pinkfuwa/llumen:demo
```

### Setup Demo Content

After the container starts, apply the tutorial content:

```bash
# Copy demo SQL to container
docker cp demo.sql llumen-demo:/tmp/demo.sql

# Execute the script
docker exec llumen-demo sqlite3 /data/db.sqlite < /tmp/demo.sql
```

### Access the Demo

Open your browser to `http://localhost` and log in with:

- **Username**: `admin`
- **Password**: `P@88w0rd`

You'll be automatically redirected to a pre-configured tutorial chat that showcases Llumen's features.

## ✨ Demo Features

The demo includes an interactive tutorial that demonstrates:

- 📝 **Markdown Support** - Rich text editing with tables, code blocks, and formatting
- 📎 **File Upload** - Attach images and documents to conversations
- 🔍 **Search Mode** - Toggle between normal chat and search-enhanced mode
- ⚙️ **Model Configuration** - TOML-based model setup
- 🎨 **Theme Switching** - Dark/light mode support
- 🌐 **Localization** - Multi-language interface

## 🔒 Demo Restrictions

To protect the demo environment, the following operations are **disabled**:

### Backend Restrictions
- ❌ Cannot delete chat ID 1 (tutorial chat)
- ❌ Cannot create/delete messages in chat ID 1
- ❌ Cannot create, update, or delete models
- ❌ Cannot create, update, read, or delete users

### What You CAN Do
- ✅ Create new chats (chat ID != 1)
- ✅ Send messages in new chats
- ✅ Upload files to new chats
- ✅ Use existing models configured in the system
- ✅ View the tutorial content in chat ID 1
- ✅ Change UI preferences (theme, locale) - stored in browser

## 📚 Documentation

For complete deployment and configuration details, see:

- [Demo Deployment Guide](docs/DEMO_DEPLOYMENT.md) - Full setup instructions
- [Main README](README.md) - General Llumen documentation
- [User Documentation](docs/user/) - User guides and tutorials
- [Developer Documentation](docs/dev/) - Architecture and development guides

## 🛠️ Configuration

### Required Environment Variables

- `API_KEY` - Your OpenRouter API key ([Get one here](https://openrouter.ai/keys))

### Optional Environment Variables

- `DATABASE_URL` - Custom database connection (default: SQLite at `/data/db.sqlite`)
- `RUST_LOG` - Logging level (default: `info`)
- `PORT` - Internal port (default: `3000`)

## 🔄 Resetting Demo Data

To restore the tutorial content:

```bash
docker exec llumen-demo sqlite3 /data/db.sqlite < /tmp/demo.sql
```

## 📦 Building from Source

```bash
# Clone the repository
git clone https://github.com/pinkfuwa/llumen.git
cd llumen

# Checkout demo branch
git checkout demo

# Build with Docker
docker build -t llumen:demo .

# Run
docker run -d \
  -e API_KEY="your_key" \
  -p 80:80 \
  -v "$(pwd)/data:/data" \
  llumen:demo
```

## 🌐 Production Deployment

For production deployments, consider:

1. **Use HTTPS** - Set up a reverse proxy (nginx, Caddy, Traefik)
2. **Rate Limiting** - Protect against abuse
3. **Monitoring** - Set up logging and metrics
4. **Backups** - Regular database backups
5. **Update Strategy** - Plan for updates without data loss

Example nginx configuration:

```nginx
server {
    listen 443 ssl http2;
    server_name demo.example.com;

    ssl_certificate /etc/ssl/certs/cert.pem;
    ssl_certificate_key /etc/ssl/private/key.pem;

    location / {
        proxy_pass http://localhost:80;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

## 🐛 Troubleshooting

### Demo content not showing
- Ensure `demo.sql` was executed after first run
- Verify database file exists at `/data/db.sqlite`
- Check container logs: `docker logs llumen-demo`

### Cannot send messages
- Verify `API_KEY` environment variable is set
- Check OpenRouter API key is valid and has credits
- Review logs for specific error messages

### File uploads not working
- Ensure `/data/files` directory has proper permissions
- Check available disk space
- Verify file size limits

### Container won't start
- Check port 80 is not already in use
- Verify Docker daemon is running
- Review container logs for errors

## 🤝 Contributing

This is a demo branch. For contributing to the main project, please see:

- [CONTRIBUTING.md](CONTRIBUTING.md)
- [Development Documentation](docs/dev/)

## 📄 License

Llumen is licensed under the MIT License. See [LICENSE](LICENSE) for details.

## 🔗 Links

- **Main Repository**: https://github.com/pinkfuwa/llumen
- **Docker Hub**: https://ghcr.io/pinkfuwa/llumen
- **Documentation**: [docs/](docs/)
- **Issues**: https://github.com/pinkfuwa/llumen/issues

## 💬 Support

For questions and support:

1. Check the [documentation](docs/)
2. Search [existing issues](https://github.com/pinkfuwa/llumen/issues)
3. Open a new issue with details about your problem

---

**Note**: This demo branch is specifically configured for public demonstrations with restricted operations. For a full-featured deployment, use the `main` branch instead.