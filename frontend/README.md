---
title: "frontend"
type: concept
status: current
updated: 2026-08-18
summary: "React 19 + Vite + Tailwind v4 + shadcn/ui；后端调用只走 lib/api.ts"
---

# frontend

开发：`pnpm install && pnpm dev`。Vite 把 `/api` 代理到 `http://localhost:8000`（`dev` 与 `preview` 都配了）。后端需另开 `aipocket serve`。生产：`pnpm build`，由 Nginx 或 `WEB_STATIC_DIR` 托管。

## 约定

- 页面：`src/pages/`，路由：`src/App.tsx`，侧栏：`src/lib/navigation.ts`
- 服务端状态：`@tanstack/react-query`
- 组件：`src/components/ui/`（shadcn），不要另装组件库
- 后端：`src/lib/api.ts`

```bash
pnpm lint
pnpm test -- --run
pnpm build
```

## 相关文档

- [Web UI](../docs/guides/web-ui.md)
- [前端约定](../docs/guides/frontend.md)
