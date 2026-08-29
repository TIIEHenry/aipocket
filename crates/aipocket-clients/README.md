---
title: "aipocket-clients"
type: concept
status: current
updated: 2026-08-18
summary: "FOFA、Shodan、GitHub、Tavily 的异步 HTTP 客户端"
---

# aipocket-clients

## 职责

对上游 API 发请求、解析 JSON、报告非 2xx。使用调用方传入的共享 `reqwest::Client`。

## 依赖

| crate | 用途 |
|-------|------|
| aipocket-core | Settings |

## 不做什么

- 不决定扫哪些 query（discovery packs）
- 不验证泄露密钥

## 相关文档

- [扫描流水线](../../docs/systems/scan-pipeline.md)
