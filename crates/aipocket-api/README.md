---
title: "aipocket-api"
type: concept
status: current
updated: 2026-08-18
summary: "Axum 路由、JWT、SSE、ScanManager；handler 不写扫描业务"
---

# aipocket-api

## 职责

- HTTP 路由与 DTO
- JWT 登录、设置读写、扫描启停与 SSE
- `ScanManager`：扫描互斥、日志窗、SSE

## 依赖

services、discovery、clients、db、core。不依赖 prober。

## 不做什么

- handler 内不实现验证/探测算法
- 不改 schema

## 相关文档

- [Web API](../../docs/reference/api.md)
- [加端点](../../docs/guides/add-endpoint.md)
- [Web 鉴权](../../docs/systems/auth.md)
