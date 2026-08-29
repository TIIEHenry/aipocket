---
title: "文档索引"
type: index
status: current
updated: 2026-08-29
summary: "AIPocket 全局文档导航；含验证状态机、Redis 契约、crate 内部设计入口"
---

# 文档索引

> 本文件人工维护。结构变更后请同步更新，并运行 `python3 scripts/check-docs-health.py`。[文档介绍 →](README.md)

## 快速入口

| 我想… | 去这里 |
|-------|--------|
| 了解整体架构 | [架构概览](architecture.md) |
| Docker / 本地跑起来 | [快速开始](guides/getting-started.md) |
| 看某个环境变量的后果 | [配置](guides/configuration.md)（真源 [`.env.example`](../.env.example)） |
| 查 CLI 场景 | [CLI](reference/cli.md) |
| 查 HTTP 契约 | [Web API](reference/api.md) |
| 看架构决策 | [ADR](../dev/decisions/INDEX.md) |
| 当前进度 | [status](../dev/progress/status.md) |

## 指南

| 文档 | 说明 |
|------|------|
| [快速开始](guides/getting-started.md) | Docker 与本地开发最短路径 |
| [部署](guides/deploy.md) | 生产 Docker、持久化、本机编译 |
| [配置](guides/configuration.md) | 环境变量分组与后果 |
| [日常运维](guides/operations.md) | 备份、watch、日志、故障排查 |
| [安全边界](guides/security.md) | 授权范围、探测门控、reveal/chat 风险 |
| [Web UI](guides/web-ui.md) | 页面与能力 |
| [贡献](guides/contributing.md) | 开发循环、测试、clippy |
| [加 Prober](guides/add-prober.md) | 平台探测标准路径 |
| [加 API 端点](guides/add-endpoint.md) | 路由 + 服务 + 契约测试 |
| [前端约定](guides/frontend.md) | 页面 / 路由 / `lib/api.ts` |

## 架构与系统

| 文档 | 说明 |
|------|------|
| [架构概览](architecture.md) | 全景、crate 图、流水线摘要 |
| [扫描流水线](systems/scan-pipeline.md) | discovery → extract → probe → validate → balance |
| [验证与高价值](systems/validation.md) | ValidationState、high_value 入库条件 |
| [存储](systems/storage.md) | PostgreSQL 真源、Redis key 契约、spill 表 |
| [Web 鉴权](systems/auth.md) | JWT、登录节流、SSE query token |
| [术语表](glossary.md) | 项目用语 |
| [界面截图](screenshots.md) | Web UI 预览 |

## 参考

| 文档 | 真源 |
|------|------|
| [CLI](reference/cli.md) | `aipocket --help` |
| [Web API](reference/api.md) | `crates/aipocket-api/src/routes.rs` |

## Crate

| crate | 职责 | 介绍 |
|-------|------|------|
| **aipocket** | CLI + `serve` 装配 | [README](../crates/aipocket/README.md) |
| **aipocket-core** | Settings、领域模型、URL 规范化 | [README](../crates/aipocket-core/README.md) |
| **aipocket-db** | SQLx、Redis dedup/lease | [README](../crates/aipocket-db/README.md) |
| **aipocket-clients** | FOFA / Shodan / GitHub / Tavily | [README](../crates/aipocket-clients/README.md) |
| **aipocket-discovery** | Sources + Provider Packs | [README](../crates/aipocket-discovery/README.md) · [packs](../crates/aipocket-discovery/docs/packs.md) |
| **aipocket-prober** | Prober、风险门控、Validator | [README](../crates/aipocket-prober/README.md) · [风险门控](../crates/aipocket-prober/docs/risk-gating.md) |
| **aipocket-services** | Scanner / Balance / Scheduler | [README](../crates/aipocket-services/README.md) |
| **aipocket-api** | Axum 路由、JWT、SSE | [README](../crates/aipocket-api/README.md) |
| **frontend** | React UI | [README](../frontend/README.md) |

## 规范与行动层

| 文档 | 说明 |
|------|------|
| [维护规则](DOCUMENTATION.md) | frontmatter、提交门禁 |
| [结构规范](DOCS-SPEC.md) | 目录与模板 |
| [dev/](../dev/README.md) | 方案、ADR、进度 |
