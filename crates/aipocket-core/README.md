---
title: "aipocket-core"
type: concept
status: current
updated: 2026-08-18
summary: "Settings、领域模型、URL 规范化；无 HTTP、无 SQL"
---

# aipocket-core

## 职责

- `Settings`：从环境 / `.env` 加载，字段名与 `.env.example` 兼容
- 领域类型：`Credential`、`ScanMode`、`ValidationState`、`TargetIdentity` 等
- URL / endpoint 规范化

## 依赖

无其它 aipocket crate。

## 不做什么

- 不发起 HTTP
- 不访问 PostgreSQL / Redis

## 相关文档

- [配置](../../docs/guides/configuration.md)
- [架构](../../docs/architecture.md)
