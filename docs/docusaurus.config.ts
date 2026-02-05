import { themes as prismThemes } from "prism-react-renderer";
import type { Config } from "@docusaurus/types";
import type * as Preset from "@docusaurus/preset-classic";

const config: Config = {
  title: "Llumen",
  tagline: "A powerful LLM chat application",
  favicon: "img/favicon.ico",

  url: "https://pinkfuwa.github.io",
  baseUrl: "/llumen",

  organizationName: "pinkfuwa",
  projectName: "llumen",

  onBrokenLinks: "throw",
  onBrokenMarkdownLinks: "warn",

  i18n: {
    defaultLocale: "en",
    locales: ["en"],
  },

  presets: [
    [
      "classic",
      {
        docs: {
          path: ".",
          routeBasePath: "/",
          sidebarPath: "./sidebars.ts",
          editUrl: "https://github.com/pinkfuwa/llumen/tree/main/docs/",
          exclude: [
            "**/_*.{js,jsx,ts,tsx,md,mdx}",
            "**/_*/**",
            "**/*.test.{js,jsx,ts,tsx}",
            "**/__tests__/**",
            "**/node_modules/**",
            "**/docs.json",
            "**/LICENSE",
            "**/README.md",
            "**/package.json",
            "**/tsconfig.json",
            "**/docusaurus.config.ts",
            "**/sidebars.ts",
            "**/src/**",
            "**/static/**",
          ],
        },
        blog: false,
        theme: {
          customCss: "./src/css/custom.css",
        },
      } satisfies Preset.Options,
    ],
  ],

  themeConfig: {
    navbar: {
      title: "Llumen",
      logo: {
        alt: "Llumen Logo",
        src: "img/logo.svg",
      },
      items: [
        {
          type: "docSidebar",
          sidebarId: "intro",
          position: "left",
          label: "Introduction",
        },
        {
          type: "docSidebar",
          sidebarId: "user",
          position: "left",
          label: "User Guide",
        },
        {
          label: "Download",
          href: "https://github.com/pinkfuwa/llumen/releases",
          position: "right",
        },
        {
          href: "https://github.com/pinkfuwa/llumen",
          position: "right",
          className: "header-github-link",
          "aria-label": "GitHub repository",
        },
      ],
    },
    footer: {
      style: "dark",
      links: [
        {
          title: "Docs",
          items: [
            {
              label: "Documentation",
              to: "/",
            },
            {
              label: "User Guide",
              to: "/user",
            },
          ],
        },
        {
          title: "Community",
          items: [
            {
              label: "GitHub Discussions",
              href: "https://github.com/pinkfuwa/llumen/discussions",
            },
            {
              label: "Issues",
              href: "https://github.com/pinkfuwa/llumen/issues",
            },
          ],
        },
        {
          title: "More",
          items: [
            {
              label: "GitHub",
              href: "https://github.com/pinkfuwa/llumen",
            },
            {
              label: "Releases",
              href: "https://github.com/pinkfuwa/llumen/releases",
            },
          ],
        },
      ],
      copyright: `Copyright © ${new Date().getFullYear()} Llumen. Built with Docusaurus.`,
    },
    prism: {
      theme: prismThemes.github,
      darkTheme: prismThemes.dracula,
    },
    colorMode: {
      defaultMode: "light",
      disableSwitch: false,
      respectPrefersColorScheme: true,
    },
  } satisfies Preset.ThemeConfig,
};

export default config;
