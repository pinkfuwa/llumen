import type { SidebarsConfig } from "@docusaurus/plugin-content-docs";

const sidebars: SidebarsConfig = {
  intro: [
    {
      type: "doc",
      label: "Introduction",
      id: "intro/introduction",
    },
    {
      type: "category",
      label: "Features",
      items: [
        "intro/chat-modes",
        "intro/rich-media",
        "intro/themes",
        "intro/performance",
      ],
    },
  ],
  user: [
    {
      type: "doc",
      label: "Installation",
      id: "user/installation",
    },
    {
      type: "doc",
      label: "First Step",
      id: "user/first-steps",
    },
    {
      type: "doc",
      label: "API Provider",
      id: "user/api-provider",
    },
    {
      type: "category",
      label: "Configuration",
      items: ["user/config/model", "user/config/environment"],
    },
    {
      type: "doc",
      label: "Docker sample",
      id: "user/docker",
    },
  ],
  // TODO: finish dev
  // developer: [],
};

export default sidebars;
