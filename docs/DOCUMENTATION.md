---
title: "文档维护规则"
type: concept
status: accepted
updated: 2026-08-18
summary: "AIPocket 文档维护规则：知识层与行动层分离、frontmatter、提交门禁、归档不删除"
---

# 文档维护规则

> **受众**：参与本仓库的人与 AI 助手。  
> **结构与模板**：[`DOCS-SPEC.md`](DOCS-SPEC.md)。本文件只写 **维护行为**，不重复目录树。

---

## 1. 分层

| 区域 | 路径 | 职责 |
|------|------|------|
| 知识层 | `docs/`、`crates/*/docs/` | 相对稳定的设计、指南、参考 |
| 行动层 | `dev/` | 方案、ADR、当前进度 |
| 归档 | `dev/archive/` | 过时文档只搬入、不删除 |
| 根指令 | `AGENTS.md`、`CLAUDE.md` | Agent 入口（豁免 frontmatter） |
| 配置真源 | `.env.example` + `Settings` | 环境变量名与默认值 |
| CLI 真源 | `crates/aipocket` 的 Clap 定义 | 参数以 `--help` 为准 |
| HTTP 真源 | `crates/aipocket-api/src/routes.rs` | 路径以代码为准（OpenAPI 第二期） |

**Agent 新会话必读**：[`AGENTS.md`](../AGENTS.md) → 本文件 → [`dev/progress/status.md`](../dev/progress/status.md)

根指令冲突时以 `AGENTS.md` 为准。修改 `AGENTS.md` 的 Do Not / 依赖方向前须先向用户确认。

---

## 2. Frontmatter

除豁免文件外，每个新建 `.md` 必须以 YAML frontmatter 开头：

```yaml
---
title: "人类可读标题"
type: architecture | concept | decision | reference | guide | plan | progress | index
status: draft | current | accepted | implemented | archived | in_progress
updated: YYYY-MM-DD
summary: "一行描述"
---
```

可选：`created: YYYY-MM-DD`（plan / decision 建议填写）。

豁免：`AGENTS.md`、`CLAUDE.md`、根 `README.md`。归档目录不强制补 frontmatter。

每次编辑后更新 `updated`；状态变了就改 `status`。

---

## 3. 必须遵守的规则

### 规则 1：编码前先查文档

动手前确认：

- `docs/` 或对应 crate README 是否已有设计
- `dev/progress/status.md` 当前焦点
- `dev/decisions/` 是否已有相关 ADR

### 规则 2：单一真源，禁止复制

每个事实只活在一份文件里，其它地方用链接。尤其不要把 `.env.example`、Clap 参数、HTTP 路径再抄成第二张表。

### 规则 3a：提交前文档门禁

禁止「代码先合、文档下次补」。

| 层级 | 何时更新 |
|------|----------|
| **行动层** `dev/` | 非 trivial 的实施 commit 必做 |
| **知识层** `docs/`、`crates/*/docs/` | 用户可见行为 / 对外接口 / 降级策略变了 |

```
本次 commit 含代码？
├── 否 → 更新改过的 `.md` 的 `updated`；结构变了则更新 INDEX 并跑健康检查
└── 是
    ├── trivial（单行 typo / 纯测试 / 无行为变化）→ 可只交代码
    └── 否 → 更新行动层；再判断知识层
        ├── 用户可见行为 / HTTP/CLI 契约 / 降级策略变了 → 改 1–2 篇 docs
        ├── 新增 crate / 跨 crate 契约 → 更新 INDEX 与 crate README；必要时 ADR
        ├── 仅内部重构 / 命名 / 测试 → 行动层通常足够
        └── 文档目录新增/移动/删除 → 更新 INDEX + `python3 scripts/check-docs-health.py`
```

实施 commit 最小清单：

- [ ] `dev/progress/status.md` 已写 Completed 或 Current Session
- [ ] `status.md` 总行数 ≤ **200**（超出迁入 `status-history-archive.md`）
- [ ] 改过的 plan / spec 的 `updated` 与 `status` 已对齐实现态
- [ ] 结构变更后已运行健康检查

### 规则 3b：提交时保留其他 WIP

只 `git add <本次主题路径>`。禁止为「干净工作区」而 stash / restore / `git add .` 卷进无关改动。详见用户 git 安全协议。

### 规则 4：ADR 只记录分叉

写入 `dev/decisions/` 的条件（任一）：引入新架构概念、在 ≥2 个方案中做选择、定义跨 crate 契约、推翻已有 ADR。

实现过程、踩坑、测试策略 **不是** ADR，可写在 plan 或 status。

`accepted` 的 ADR 不改正文；要推翻就写新 ADR 并引用旧编号。

### 规则 5：过时只归档

移入 `dev/archive/`，改 `status: archived`，更新相关 INDEX。永远不删除文档文件。

### 规则 6：索引随结构走

新增 / 移动 / 删除文档后，更新 `docs/INDEX.md`（以及 `dev/decisions/INDEX.md` 等受影响索引），然后：

```bash
python3 scripts/check-docs-health.py
```

INDEX 人工维护，禁止脚本覆写。

### 规则 7：方案与实施分开提交

- 方案 commit：`dev/plans/`、`dev/decisions/`，不含实现代码
- 实施 commit：代码 + 本次主题的行动层 + 按需知识层，**同一主题一次提交**

trivial bugfix 可跳过方案。非 trivial 功能应先有 plan 或在现有 spec 上改。

---

## 4. 本机私有笔记

机器绝对路径、登录口令、实例口令 **不入库**。放到 `docs/local/`（已 gitignore）。共享部署说明只写可复现步骤，见 [部署](guides/deploy.md)。
