import { defineConfig } from 'vitepress'

export default defineConfig({
  title: 'Chub',
  description: 'Fast curated docs for AI coding agents. Team-first. Git-tracked. Built in Rust.',
  head: [
    ['link', { rel: 'icon', type: 'image/svg+xml', href: '/logo.svg' }],
  ],

  themeConfig: {
    logo: '/logo.svg',

    nav: [
      { text: 'Guide', link: '/guide/getting-started' },
      { text: 'Reference', link: '/reference/cli' },
      {
        text: 'Links',
        items: [
          { text: 'GitHub', link: 'https://github.com/nrl-ai/chub' },
          { text: 'npm', link: 'https://www.npmjs.com/package/@nrl-ai/chub' },
          { text: 'PyPI', link: 'https://pypi.org/project/chub/' },
        ]
      }
    ],

    sidebar: {
      '/guide/': [
        {
          text: 'Introduction',
          items: [
            { text: 'Getting Started', link: '/guide/getting-started' },
            { text: 'Why Chub', link: '/guide/why-chub' },
          ]
        },
        {
          text: 'Team Features',
          items: [
            { text: 'Doc Pinning', link: '/guide/pinning' },
            { text: 'Context Profiles', link: '/guide/profiles' },
            { text: 'Team Annotations', link: '/guide/annotations' },
            { text: 'Project Context', link: '/guide/project-context' },
            { text: 'Dep Auto-Detection', link: '/guide/detect' },
            { text: 'Agent Config Sync', link: '/guide/agent-config' },
            { text: 'Snapshots & Freshness', link: '/guide/snapshots' },
          ]
        },
      ],
      '/reference/': [
        {
          text: 'Reference',
          items: [
            { text: 'CLI Commands', link: '/reference/cli' },
            { text: 'Configuration', link: '/reference/configuration' },
            { text: 'MCP Server', link: '/reference/mcp-server' },
          ]
        },
      ],
    },

    socialLinks: [
      { icon: 'github', link: 'https://github.com/nrl-ai/chub' },
    ],

    footer: {
      message: 'Built on <a href="https://github.com/andrewyng/context-hub">Context Hub</a> by Andrew Ng',
      copyright: 'MIT License · NRL AI',
    },

    search: {
      provider: 'local',
    },
  },
})
