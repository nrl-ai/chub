import { defineConfig } from 'vitepress'

export default defineConfig({
  title: 'Chub',
  description: 'Curated docs for AI coding agents, with self-learning annotations. Team-first. Git-tracked. Built in Rust.',
  head: [
    ['link', { rel: 'icon', type: 'image/svg+xml', href: '/logo.svg' }],
  ],

  appearance: 'dark',

  themeConfig: {
    logo: '/logo.svg',

    nav: [
      { text: 'Guide', link: '/guide/getting-started' },
      { text: 'Reference', link: '/reference/cli' },
      { text: 'Registry', link: 'https://cdn.chub.nrl.ai' },
      {
        text: 'Links',
        items: [
          { text: 'GitHub', link: 'https://github.com/nrl-ai/chub' },
          { text: 'npm', link: 'https://www.npmjs.com/package/@nrl-ai/chub' },
          { text: 'PyPI', link: 'https://pypi.org/project/chub/' },
          { text: 'crates.io', link: 'https://crates.io/crates/chub' },
        ]
      }
    ],

    sidebar: {
      '/guide/': [
        {
          text: 'Introduction',
          items: [
            { text: 'Getting Started', link: '/guide/getting-started' },
            { text: 'Installation', link: '/guide/installation' },
            { text: 'Why Chub', link: '/guide/why-chub' },
            { text: 'Showcases', link: '/guide/showcases' },
          ]
        },
        {
          text: 'Self-Learning',
          items: [
            { text: 'Annotations', link: '/guide/annotations' },
            { text: 'Agent Config Sync', link: '/guide/agent-config' },
          ]
        },
        {
          text: 'Team Features',
          items: [
            { text: 'Doc Pinning', link: '/guide/pinning' },
            { text: 'Context Profiles', link: '/guide/profiles' },
            { text: 'Project Context', link: '/guide/project-context' },
            { text: 'Dep Auto-Detection', link: '/guide/detect' },
            { text: 'Snapshots & Freshness', link: '/guide/snapshots' },
          ]
        },
        {
          text: 'Going Further',
          items: [
            { text: 'Content Guide', link: '/guide/content-guide' },
            { text: 'Self-Hosting a Registry', link: '/guide/self-hosting' },
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
