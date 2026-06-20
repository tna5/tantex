import { defineConfig } from 'vitepress'

export default defineConfig({
    base: '/tantex/',
    title: 'tantex',
    description: 'High-performance full-text search server built on tantivy',
    lang: 'en-US',

    head: [
        ['link', { rel: 'icon', type: 'image/svg+xml', href: '/tantex/logo.svg' }],
    ],

    ignoreDeadLinks: [/^http:\/\/localhost/],

    themeConfig: {
        logo: '/logo.svg',
        siteTitle: 'tantex',

        nav: [
            { text: 'Docs', link: '/docs/tutorial/getting-started' },
            { text: 'Reference', link: '/docs/reference/http-api' },
            { text: 'GitHub', link: 'https://github.com/tna5/tantex' },
        ],

        sidebar: {
            '/docs/': [
                {
                    text: 'Tutorial',
                    items: [
                        { text: 'Getting started', link: '/docs/tutorial/getting-started' },
                    ],
                },
                {
                    text: 'How-to guides',
                    items: [
                        { text: 'Configure tantex', link: '/docs/how-to/configure' },
                        { text: 'Design a schema', link: '/docs/how-to/design-schema' },
                        { text: 'Ingest documents', link: '/docs/how-to/ingest-documents' },
                        { text: 'Search an index', link: '/docs/how-to/search' },
                        { text: 'Use the dashboard', link: '/docs/how-to/dashboard' },
                    ],
                },
                {
                    text: 'Reference',
                    items: [
                        { text: 'Configuration', link: '/docs/reference/configuration' },
                        { text: 'Field types', link: '/docs/reference/field-types' },
                        { text: 'HTTP API', link: '/docs/reference/http-api' },
                        { text: 'Binary protocol', link: '/docs/reference/protocol' },
                        { text: 'Query syntax', link: '/docs/reference/query-syntax' },
                    ],
                },
                {
                    text: 'Explanation',
                    items: [
                        { text: 'Indexing pipeline', link: '/docs/explanation/indexing' },
                        { text: 'Search and scoring', link: '/docs/explanation/searching' },
                        { text: 'Compression', link: '/docs/explanation/compression' },
                        { text: 'SHM ingestion', link: '/docs/explanation/shm-ingestion' },
                        { text: 'Commit & merge policy', link: '/docs/explanation/commit-merge-policy' },
                    ],
                },
                {
                    text: 'SDK',
                    items: [
                        { text: 'JavaScript / TypeScript', link: '/docs/how-to/javascript-client' },
                    ],
                },
            ],
        },

        search: { provider: 'local' },
        editLink: false,

        footer: {
            message: 'Released under the MIT License.',
            copyright: 'Built on tantivy'
        }
    }
});