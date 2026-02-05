# Llumen Documentation

This directory contains the documentation for Llumen, built with [Docusaurus](https://docusaurus.io/).

## Development

```bash
# Install dependencies
pnpm install

# Start development server (with hot reload)
pnpm start

# Build for production
pnpm build

# Serve production build
pnpm serve
```

## Structure

- **Documentation** - Product overview and features
  - Features overview
  - Chat modes (Normal, Search, Research)
  - Rich media support (PDFs, LaTeX, images)
  - Themes and UI customization
  - Performance details

- **User Guide** - Installation and usage
  - Installation (Docker, native binaries)
  - Configuration (environment variables, API providers)
  - First steps and chat basics
  - Search mode and research mode guides
  - Media upload
  - API provider configuration
  - Docker Compose examples
  - Troubleshooting

- **Developer Docs** - Technical documentation
  - Architecture overview
  - Building from source
  - Contributing guidelines
  - Backend development (Rust/Axum)
  - Frontend development (Svelte 5)
  - Deployment strategies

## Directory Structure

```
docs/
├── features/           # Feature documentation
├── user-guide/         # User guides and tutorials
├── developer/          # Developer documentation
├── src/                # Custom components and theme
│   ├── components/     # React components (Card, etc.)
│   ├── css/            # Custom styles
│   └── theme/          # Theme overrides
├── static/             # Static assets
│   ├── img/            # Logos and favicons
│   └── images/         # Screenshots and images
├── docusaurus.config.ts  # Docusaurus configuration
└── sidebars.ts          # Sidebar navigation
```

## Theme Configuration

The docs use llumen's brand colors matching the main application:
- Primary: `#ef7722` (llumen orange)
- Light backgrounds and dark mode support
- Custom CSS matching llumen's UI design

## Contributing

To improve the docs:
1. Edit the `.mdx` files in the appropriate directory
2. Test locally with `pnpm start`
3. Build to ensure no errors: `pnpm build`
4. Commit and push changes

## License

Same as llumen - Mozilla Public License 2.0
