import type {SidebarsConfig} from '@docusaurus/plugin-content-docs';

const sidebars: SidebarsConfig = {
  docs: [
    {
      type: 'category',
      label: 'Overview',
      items: ['index', 'features/overview'],
    },
    {
      type: 'category',
      label: 'Features',
      items: [
        'features/chat-modes',
        'features/rich-media',
        'features/themes',
        'features/performance',
      ],
    },
  ],
  userGuide: [
    {
      type: 'category',
      label: 'Getting Started',
      items: [
        'user-guide/installation',
        'user-guide/configuration',
        'user-guide/model-config',
        'user-guide/first-steps',
      ],
    },
    {
      type: 'category',
      label: 'Usage',
      items: [
        'user-guide/chat-basics',
        'user-guide/search-mode',
        'user-guide/research-mode',
        'user-guide/media-upload',
      ],
    },
    {
      type: 'category',
      label: 'Advanced',
      items: [
        'user-guide/api-providers',
        'user-guide/docker-compose',
        'user-guide/troubleshooting',
      ],
    },
  ],
  developer: [
    {
      type: 'category',
      label: 'Development',
      items: [
        'developer/architecture',
        'developer/building',
        'developer/contributing',
      ],
    },
    {
      type: 'category',
      label: 'Technical Details',
      items: [
        'developer/backend',
        'developer/frontend',
        'developer/deployment',
      ],
    },
  ],
};

export default sidebars;
