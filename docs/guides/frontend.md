---
title: "前端约定"
type: guide
status: current
updated: 2026-08-18
summary: "新页面走 pages + App.tsx 路由 + sidebar；后端调用只经 lib/api.ts"
---

# 前端约定

## 加页面

1. 在 `frontend/src/pages/` 建页面组件。
2. 在 `frontend/src/App.tsx` 的 `ProtectedRoute` 下加路由。
3. 在 `frontend/src/lib/navigation.ts` 的 `NAV_ITEMS` 加侧栏项（若需要出现在导航里）。
4. 数据用 `@tanstack/react-query`；不要引入 Redux 或其它全局 store。
5. UI 用 `components/ui/` 的 shadcn 组件，不要另装组件库。

## API

所有后端调用放 `frontend/src/lib/api.ts`。类型与后端 DTO 对齐。鉴权 token 走现有 `auth-storage`。扫描日志用 `openScanLogStream`（`EventSource`，JWT 走 `token` query），不要自己 `fetch` SSE。

## 验证

```bash
cd frontend
pnpm lint
pnpm test -- --run
pnpm build
```

## 相关文档

- [`frontend/README.md`](../../frontend/README.md)
- [Web UI](web-ui.md)
- [Web API](../reference/api.md)
