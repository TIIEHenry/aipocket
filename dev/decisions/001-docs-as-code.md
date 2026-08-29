---
title: "采用缩小版 docs-as-code"
type: decision
status: accepted
updated: 2026-08-18
created: 2026-08-18
summary: "知识层/行动层分离 + 提交门禁；不引入 VitePress/mdBook，不复制 UniverseAgent 的模块 INDEX"
---

# 001. 采用缩小版 docs-as-code

## Context

仓库几乎没有可导航的设计文档：根 README 同时承担产品页和手册，`CLAUDE.md` 与 `AGENTS.md` 重复，`docs/` 只有截图和本机笔记。UniverseAgent 已有一套可运行的 Git + Markdown 文档系统，但体量对应 20+ 模块与数千篇文档。

需要在「没有系统」和「搬全套仪式」之间做选择。

## Decision

1. 知识层 `docs/` 与行动层 `dev/` 分离；INDEX 人工维护。
2. 学 UniverseAgent 的单一真源、frontmatter、提交门禁 3a、只读健康检查、过时只归档。
3. **不**建 `docs/modules/<crate>/INDEX.md`；crate README 由全局 INDEX 直链。
4. **不**第一期上 VitePress / Docusaurus / mdBook。
5. 配置 / CLI / HTTP 仍以代码与 `.env.example` 为真源，Markdown 只写场景与后果。
6. `CLAUDE.md` 降为指向 `AGENTS.md` 的短指针，避免三份百科。

规范：[`docs/DOCS-SPEC.md`](../../docs/DOCS-SPEC.md)、[`docs/DOCUMENTATION.md`](../../docs/DOCUMENTATION.md)。

## Consequences

- 贡献者与 agent 有固定入口和门禁，文档不会再只活在 README。
- 维护成本是每篇 frontmatter + 结构变更时跑健康检查，而不是文档站 CI。
- crate 增多到难以在一张 INDEX 里扫视时，再考虑模块 INDEX 层。
- 文档超过约 20 篇且需要站内搜索时，再评估 mdBook（Rust 生态）或 VitePress（与前端同栈）。

## Alternatives considered

- **原样复制 UniverseAgent**（模块 INDEX、17 种 type、status.md 流水账、loop）：仪式超过本仓库规模，已在 UA 的超长 `status.md` 上见过反噬。
- **只写几篇 Markdown、无规范**：会回到 README / AGENTS / 本机笔记抢真源。
- **立刻上文档站**：在真源未稳定前增加构建与主题负担，不能解决漂移。
