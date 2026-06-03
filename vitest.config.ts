import { defineConfig } from "vitest/config";
import path from "node:path";

// 前端纯逻辑测试（重命名模板渲染、节目档案匹配）。无 DOM 依赖，使用 node 环境。
export default defineConfig({
  resolve: {
    alias: { "@": path.resolve(__dirname, "./src") },
  },
  test: {
    environment: "node",
    include: ["src/**/*.test.ts"],
  },
});
