---
title: "贡献指南"
type: guide
status: current
updated: 2026-08-18
summary: "开发循环、测试与 lint；agent 入口仍是 AGENTS.md"
---

# 贡献指南

Agent 操作手册见 [AGENTS.md](../../AGENTS.md)。文档门禁见 [DOCUMENTATION.md](../DOCUMENTATION.md)。

## 后端

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo llvm-cov --workspace --fail-under-lines 86
```

PostgreSQL / Redis 兼容用例在 CI 里以 ignored 集成测试跑。提交后端前应跑 workspace 测试。

新增环境变量：在 `Settings` 给向后兼容默认值，并写入 `.env.example`。

Schema：只加不减，语句幂等。不要重建生产表。

## 前端

```bash
cd frontend
pnpm lint
pnpm test -- --run
pnpm build
```

UI 用 shadcn/ui；服务端状态用 TanStack Query；路由 `react-router-dom` v6；后端调用只走 `src/lib/api.ts`。

## 常见任务

| 任务 | 文档 |
|------|------|
| 加平台 Prober | [add-prober.md](add-prober.md) |
| 加 API | [add-endpoint.md](add-endpoint.md) |
| 加页面 | [frontend.md](frontend.md) |

## 文档

用户可见行为或契约变了，和代码同一 commit 更新 1–2 篇 `docs/`。然后：

```bash
python3 scripts/check-docs-health.py
```
