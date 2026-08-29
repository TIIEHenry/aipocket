---
title: "文档系统设计规范"
type: concept
status: accepted
updated: 2026-08-18
summary: "AIPocket docs-as-code 结构：知识层/行动层、INDEX 导航、crate README、健康检查"
---

# 文档系统设计规范

> **维护行为**（门禁、frontmatter、ADR）：[`DOCUMENTATION.md`](DOCUMENTATION.md)。本文件只写 **结构与模板**。

原则：文档即代码、单一真源、与仓库同生命周期。体量按 8 个 crate + 一个前端缩放，**不**复制 UniverseAgent 的模块 INDEX 层。

---

## 1. 目录

```
README.md                 # 对外漏斗（产品 + Docker 最短路径）
AGENTS.md                 # 唯一 agent 入口
CLAUDE.md                 # 指向 AGENTS.md

docs/
  README.md               # 按读者说明怎么读
  INDEX.md                # 全局导航（人工维护）
  DOCUMENTATION.md        # 维护规则
  DOCS-SPEC.md            # 本文件
  architecture.md         # 全景、crate 图、流水线
  glossary.md             # 术语
  screenshots.md          # 界面截图
  guides/                 # 人怎么用 / 怎么贡献
  systems/                # 跨 crate 协作
  reference/              # CLI / HTTP 场景说明（细节以代码为准）
  images/

crates/<name>/README.md   # 职责、依赖、不做什么
crates/<name>/docs/       # 仅当该 crate 有非显而易见的内部设计时才建

frontend/README.md        # 前端怎么跑与约定

dev/
  README.md
  progress/status.md      # 当前会话摘要（≤200 行）
  plans/                  # 实施方案
  decisions/              # ADR
  archive/                # 只搬不删
```

不建 `docs/modules/<crate>/INDEX.md`。crate 少，全局 INDEX 直接链到 `crates/*/README.md`。等 crate 明显增多或出现独立子项目再加这一层。

---

## 2. 四层定位

| 层 | 位置 | 写什么 |
|----|------|--------|
| 全局架构 | `docs/architecture.md` | 全景、依赖方向、扫描流水线摘要 |
| 跨 crate 系统 | `docs/systems/` | 存储、鉴权、流水线细节等协作 |
| crate 介绍 | `crates/<name>/README.md` | 职责、依赖、边界 |
| crate 内部设计 | `crates/<name>/docs/` | 仅复杂内部设计（如 prober 风险门控） |

README 负责介绍；INDEX 负责导航，不写大段正文。两者互相链接。

---

## 3. 单一真源

| 事实 | 只写在这 | 其它地方 |
|------|----------|----------|
| 产品定位 / 免责 | 根 `README.md` | 链过去 |
| 环境变量名与默认值 | `.env.example` + `Settings` | `guides/configuration.md` 只解释分组和后果 |
| CLI 参数 | Clap | `reference/cli.md` 只写场景 |
| HTTP 路径与字段 | `aipocket-api` | `reference/api.md` 过渡期可手写，标「以代码为准」 |
| 表结构 | `migrations/schema.sql` | `systems/storage.md` 只写职责 |
| crate 依赖方向 | `AGENTS.md` + `architecture.md` | 不在 README 再抄一份依赖图 |
| 如何加 Prober / 路由 / 页面 | `docs/guides/add-*.md` | AGENTS 任务表只链过去 |
| 部署拓扑 | `docs/guides/deploy.md` | README 只保留 Docker 最短路径 |
| 本机口令 / 绝对路径 | `docs/local/`（不入库） | — |

---

## 4. 模板

### 4.1 crate README

```markdown
---
title: "aipocket-<name>"
type: concept
status: current
updated: YYYY-MM-DD
summary: "一句话职责"
---

# aipocket-<name>

## 职责
- …

## 依赖
| crate | 用途 |
|-------|------|
| aipocket-core | 领域模型 / Settings |

## 不做什么
- …

## 相关文档
- [架构](../../docs/architecture.md)
```

### 4.2 系统文档

```markdown
---
title: "…"
type: architecture
status: current
updated: YYYY-MM-DD
summary: "…"
---

# 主题

## 涉及 crate
- [aipocket-services](../../crates/aipocket-services/README.md)

## 设计目标
…

## 相关文档
- [架构](../architecture.md)
```

### 4.3 ADR：`dev/decisions/NNN-short-name.md`

```markdown
---
title: "决策简述"
type: decision
status: accepted
updated: YYYY-MM-DD
summary: "一句话"
created: YYYY-MM-DD
---

# NNN. 标题

## Context
## Decision
## Consequences
## Alternatives considered
```

---

## 5. 健康检查

```bash
python3 scripts/check-docs-health.py
python3 scripts/check-docs-health.py --strict-frontmatter --strict-links
```

默认检查：必备入口、Cargo crate README、`status.md` ≤ 200 行。frontmatter 与本地 `.md` 断链默认 warning，`--strict-*` 升为 error。

INDEX 人工维护。不要写「本文件由脚本自动生成」。
