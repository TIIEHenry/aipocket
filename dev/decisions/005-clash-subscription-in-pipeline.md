---
title: "Clash 订阅纳入现有扫描流水线"
type: decision
status: accepted
updated: 2026-08-31
created: 2026-08-31
summary: "完整订阅 URL；Credential.credential_kind；打码面；GitHub P0；装配追加预算；finalize 保留失败链"
---

# ADR-005: Clash 订阅纳入现有扫描流水线

## 状态

accepted（2026-08-31，吸收 Grok 二轮审查 +「完整订阅链接」需求）

## 背景

产品需收集**完整 Clash 订阅链接**（可导入客户端的 URL），覆盖国内 V2Board/Xboard、SSPanel-UIM 等面板。与 AI Key 共用扫描流水线；**`credential.apiurl` 为真源资产**；L0 GET 仅 enrich。

Grok 审查指出：无 `Credential.credential_kind`、无 `apiurl`/`raw_context` 打码、FOFA 无 body、GitHub 绕过 pipeline、`finalize` 丢弃失败链、balance/GPT 未跳过、proxy pack 默认随 `all` 启用等问题会导致**提不出链、泄密或丢链**。本 ADR 将修复项升为硬决策。

## 决策

### 交付与存储

1. **`Credential.apiurl`** = 规范化完整订阅 URL（含 token）。**`Credential.apikey`** = token 字符串（不用 URL hash 充当 apikey）。
2. **`Credential.credential_kind = "proxy_sub"`**（`#[serde(default)]` 空=AI key）。贯穿 extract → validate → finalize → API。**禁止**仅用 `source` 前缀代替。
3. 共享 **`normalize_subscription_url()`**（`aipocket-core`），供 pipeline 与 `github_artifacts` 共用。

### 发现与预算

4. Pack 分 **`PACKS`（AI）** 与 **`PROXY_PACKS`（`proxy_*`）**。`github_pack_ids` 空/`all` **仅选 AI**；proxy 由 **`PROXY_SUB_ENABLED`**（默认 `false`）启用。
5. Proxy FOFA/Shodan 查询 **追加**到源查询列表，条数上限 **`PROXY_SUB_QUERY_BUDGET`**（默认 8），**不从** `FOFA_QUERY_BUDGET` 切片。
6. Proxy 查询须含 **token 形态**；禁止无 token 的面板路径普查。
7. FOFA 当前无 `body` 字段；完整 token 依赖 banner/header、Shodan `http.html`、**GitHub artifacts（P0 范围）**。

### 验证

8. **`ProtocolFamily::ClashSubscription`**；validator **穷尽 match**，禁止 proxy 落入 OpenAI `/v1/models`。
9. `resolve` **path-first**（`/client/subscribe`、`/link/`、`/sub/`）。
10. L0 GET：Clash UA、body ≤2MB、拒绝私网跳转目标；解析顺序 **base64 → YAML → URI 列表**。
11. **`PROXY_SUB_VALIDATE_ENABLED`**（默认 true when enabled）关闭时跳过 GET。
12. `provider_evidence` 仅聚合 enrich；**不**存完整 YAML、节点 password、完整 tagged_nodes。

### 展示与泄密面

13. 列表 **`mask_record`**：`apikey` + `apiurl` token 段 + **清空/打码 `raw_context`、`response_snippet`**。
14. **`reveal`** 以 **`result_id`** 为主键返回完整 `apiurl`；审计 detail 不含明文 URL。
15. 导出 **`subscription-url`** 给完整链接；**`sub2api` 排除** proxy_sub；balance/chat/models → **400**。

### Finalize 与 Scanner

16. 验证失败的 proxy_sub **仍写入 `results`**（`kind=unavailable` 等），**不**被 `finalize_results` 丢弃；保留完整 apiurl 供 reveal。
17. Scanner **跳过** proxy_sub 的 balance、GPT recheck、`high_value_keys`。

### 入库

18. 复用 `results`；不进 `high_value_keys`。

## 后果

### 正面

- 与「完整订阅链接」及 AI Key 产品体验一致。
- Grok 指出的提取/泄密/丢链路径有明确契约。

### 负面

- `Credential` 语义变宽；需 UI 区分。
- FOFA-only 场景可能只有 host 线索、无 token（文档与运维须知晓）。
- L0 GET 对第三方面板出站；须显式 `PROXY_SUB_ENABLED`。

## 未采纳

| 方案 | 原因 |
|------|------|
| 只存标签不存 URL | 不符合产品目标 |
| 列表返回完整 token URL | 泄密；改为打码 + reveal |
| Grok 建议不存任何可用配置 | 用户要完整链接；节点级 password 仍不入库 |
| proxy 查询挤占 AI budget 前 48 条 | 改为追加切片 |
| P2 才做 GitHub 订阅提取 | GitHub 是完整 URL 主源之一 |

## 相关

- [clash-subscription-discovery](../plans/clash-subscription-discovery.md)
- [ADR-004](004-readiness-and-audit.md)
