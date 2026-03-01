import { defineConfig } from "vite";

export default defineConfig({
  // 开发服务器配置
  server: {
    port: 3000,
    open: true,
  },
  // 构建配置
  build: {
    outDir: "dist",
    target: "esnext",
  },
  // 优化依赖
  optimizeDeps: {
    exclude: ["@wasmer/sdk"],
  },
});
