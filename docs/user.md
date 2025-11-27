# Llumen User Guide

For comprehensive user documentation, see the **[User Guide](./user/README.md)**.

## Quick Links

- **[Getting Started](./user/getting-started.md)** - New to Llumen? Start here!
- **[Installation](./user/installation.md)** - Docker, binaries, and building from source
- **[Configuration](./user/configuration.md)** - Environment variables and model settings
- **[Features](./user/features.md)** - Chat modes and capabilities
- **[Troubleshooting](./user/troubleshooting.md)** - Common issues and solutions
- **[FAQ](./user/faq.md)** - Frequently asked questions

## Quick Start

```bash
docker run -it --rm \
  -e API_KEY="<YOUR_API_KEY>" \
  -p 80:80 \
  -v "$(pwd)/data:/data" \
  ghcr.io/pinkfuwa/llumen:nightly
```

Default credentials: `admin` / `P@88w0rd`

---

**Need help?** Check the [Troubleshooting Guide](./user/troubleshooting.md) or [open an issue](https://github.com/pinkfuwa/llumen/issues).
