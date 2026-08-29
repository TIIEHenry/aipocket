---
title: "aipocket"
type: concept
status: current
updated: 2026-08-18
summary: "二进制 crate：Clap CLI 与 Axum serve 进程装配"
---

# aipocket

## 职责

- 解析 CLI（`scan` / `serve` / `watch` / …）
- 加载 `Settings`、接 PostgreSQL schema、装配 HTTP 客户端
- `serve` 时调用 `aipocket_api::create_app`

## 依赖

依赖 workspace 内其余 crate，本身不放新业务逻辑。

## 不做什么

- 不实现扫描编排（`aipocket-services`）
- 不实现路由 handler（`aipocket-api`）

## 相关文档

- [CLI](../../docs/reference/cli.md)
- [架构](../../docs/architecture.md)
