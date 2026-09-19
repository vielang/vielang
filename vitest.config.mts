import { defineConfig } from "vitest/config";
import path from "node:path";

export default defineConfig({
  // Không cài @vitejs/plugin-react: gói đó kéo theo xung đột @babel/core 7 vs
  // 8, mà test thì không cần Fast Refresh — trình dịch sẵn có của Vite đã lo
  // được JSX.
  test: {
    environment: "jsdom",
    include: ["src/**/*.test.{ts,tsx}"],
    // jsdom thiếu vài API mà component vẽ cần (PointerEvent, ResizeObserver…)
    // — dựng sẵn ở đây thay vì rắc vào từng file test.
    setupFiles: ["./src/test-setup.ts"],
  },
  resolve: {
    alias: {
      "@": path.resolve(import.meta.dirname, "./src"),
    },
  },
});
