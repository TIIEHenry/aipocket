---
title: "添加 API 端点"
type: guide
status: current
updated: 2026-08-18
summary: "路由与 DTO 放 aipocket-api，业务放 services，并补契约测试"
---

# 添加 API 端点

## 步骤

1. 在 `crates/aipocket-api/src/routes.rs` 注册路由；DTO 用 Serde。
2. Handler 只做鉴权、取参、调 `aipocket-services` / `Repository`。**不要**在 handler 里写扫描或验证逻辑。
3. 扫描启停不要绕过 `ScanManager`：它负责 running 互斥、日志环形缓冲、SSE broadcast 与把 `ScanEvent` 写入 run 日志。
4. 需要登录的接口走现有 JWT 中间件（`auth::verify`）。
5. 在 `crates/aipocket-api/tests/` 加契约测试（请求/响应形状，而不是实现细节）。
6. 前端若要调用：只加 `frontend/src/lib/api.ts`，页面不要裸 `fetch`。
7. 更新 [Web API](../reference/api.md) 对应资源小节（路径以 `routes.rs` 为准）。

## 不变量

- 列表接口打码 apikey；明文只走显式 reveal。
- 会消耗目标额度的接口（chat / 部分 balance）必须在文档里标明。
- 依赖方向：`api → services → …`，api 不直接依赖 prober。

## 相关文档

- [`crates/aipocket-api/README.md`](../../crates/aipocket-api/README.md)
- [Web 鉴权](../systems/auth.md)
- [前端约定](frontend.md)
