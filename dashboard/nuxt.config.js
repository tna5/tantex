import tailwindcss from '@tailwindcss/vite'

export default defineNuxtConfig({
    ssr: false,
    css: ['~/assets/css/tailwind.css'],
    compatibilityDate: '2025-01-01',
    vite: {
        plugins: [tailwindcss()],
        server: {
            proxy: {
                '/api': {
                    target: process.env.TANTEX_HTTP_URL || 'http://127.0.0.1:7200',
                    changeOrigin: true
                }
            }
        }
    },
    devServer: {
        port: 7201
    },
    modules: ['shadcn-nuxt'],
    app: {
        head: {
            script: [
                {
                    innerHTML: `
            if (localStorage.getItem('dark-mode') === 'true') {
              document.documentElement.classList.add('dark')
            } else if (localStorage.getItem('dark-mode') === 'false') {
              document.documentElement.classList.remove('dark')
            }
          `,
                    type: 'text/javascript',
                    tagPosition: 'head'
                }
            ]
        }
    },
    shadcn: {
        prefix: '',
        componentDir: '@/components/ui'
    }
    // In dev mode, /api/* is proxied to the tantex Rust server via vite.server.proxy above.
    // Set TANTEX_HTTP_URL to override (default: http://127.0.0.1:7200).
    // Start tantex with: cargo run -- --port 7200
});