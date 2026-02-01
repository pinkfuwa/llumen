# Llumen Documentation

This directory contains the Mintlify documentation for llumen.

## Structure

- **Documentation Tab** - Product overview and features
  - Features overview
  - Chat modes (Normal, Search, Research)
  - Rich media support (PDFs, LaTeX, images)
  - Themes and UI customization
  - Performance details

- **User Guide Tab** - Installation and usage
  - Installation (Docker, native binaries)
  - Configuration (environment variables, API providers)
  - First steps and chat basics
  - Search mode and research mode guides
  - Media upload
  - API provider configuration
  - Docker Compose examples
  - Troubleshooting

- **Developer Docs Tab** - Technical documentation
  - Architecture overview
  - Building from source
  - Contributing guidelines
  - Backend development (Rust/Axum)
  - Frontend development (Svelte 5)
  - Deployment strategies

## Preview Locally

Install Mintlify CLI and run:

```bash
cd docs
npx mintlify dev
```

Visit http://localhost:3000 to see the docs.

## Deployment

These docs are designed to be deployed with Mintlify. Push to your Git repository and connect it in the [Mintlify dashboard](https://dashboard.mintlify.com).

## Features

- ✅ Linden theme (clean, modern)
- ✅ Custom llumen logo (icon + text)
- ✅ Mobile-optimized screenshots
- ✅ Three-tab navigation
- ✅ Comprehensive coverage of all features
- ✅ Step-by-step user guides
- ✅ Developer documentation
- ✅ Dark/light mode support
- ✅ Search and contextual menu

## Theme Configuration

The docs use llumen's brand colors:
- Primary: `#EF7722` (llumen orange)
- Light: `#FF8A3D`
- Dark: `#D96815`

## Images

All images are stored in `/images/` including:
- Mobile theme screenshots
- Settings screenshots  
- UI examples

## Contributing

To improve the docs:
1. Edit the `.mdx` files
2. Test locally with `mintlify dev`
3. Commit and push changes
4. Docs auto-deploy via Mintlify

## License

Same as llumen - Mozilla Public License 2.0
