import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

// https://vitejs.dev/config/
export default defineConfig({
    server: {
        proxy: {
            "/api": {
                target: "http://localhost:9394", // 後端 API 進入點
                changeOrigin: true,
                secure: false,
                ws: true,
                // rewrite: (path) => path.replace(/^\/api/, ""),
            },
        },
    },
    plugins: [react()],
})
