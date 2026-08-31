---
title: "Clash 订阅发现实施方案"
type: plan
status: draft
updated: 2026-08-31
created: 2026-08-31
summary: "完整订阅 URL 为交付物；Credential.credential_kind；P0 含 GitHub artifacts；打码/reveal/finalize/装配器契约"
---

# Clash 订阅发现

**Goal:** 收集**完整 Clash 订阅链接**（`https://…/subscribe?token=…` 或 `/link/{token}` 拼出的可导入 URL），与 AI Key 同级：入库明文、列表打码、reveal/导出给全文。验证 L0 GET 仅 enrich（节点数、家宽/IEPL 标签、`subscription-userinfo`），**不替代**链接存储。

**架构决策：** [ADR-005](../decisions/005-clash-subscription-in-pipeline.md)（Grok 二轮修订后 accept 前须满足其中硬决策）。

---

## 0. 核心契约

| 环节 | 行为 |
|------|------|
| 入库 | `credential.apiurl` = 规范化**完整 URL**；`credential.apikey` = **token 字符串**（禁止 URL hash 当 apikey） |
| 种类 | `credential.credential_kind = "proxy_sub"`（**Credential 字段**，见 §3） |
| 列表 | 打码 `apikey` + `apiurl` token 段 + 清空/打码 `raw_context`、`response_snippet` |
| reveal | 按 **`result_id`**（或 `run_id`+`index`），返回与库内**完全相同**的 `apiurl` |
| 验证失败 | 链接**仍入库**可见（§7 finalize） |
| 导出 | `subscription-url` 每行完整 URL；`sub2api` 排除 proxy_sub |

---

## 1. 数据模型

### 1.1 `Credential` 扩展（P0）

在 [`aipocket-core/src/models.rs`](../../crates/aipocket-core/src/models.rs) 增加：

```rust
pub credential_kind: String,  // "" = AI key； "proxy_sub" = 订阅链接
```

- `#[serde(default)]`，旧记录空字符串按 AI key 处理。
- extract 阶段即写入 `proxy_sub`；validate 拷贝到 `ValidationResult.credential_kind`。
- **禁止**用 `source` 前缀 `proxy_sub:` 代替该字段。

### 1.2 共享规范化（P0）

新建 `aipocket-core/src/subscription_url.rs`（或 `aipocket-discovery` 导出供 services 用）：

```rust
pub fn normalize_subscription_url(raw: &str, base_host: &str) -> Option<NormalizedSubUrl> {
    // scheme 默认 https；保留 flag/sub/query；URL 解码 token；去 fragment；
    // 相对路径与 hit.host 拼接；输出 canonical apiurl + token
}
```

调用方：`pipeline` extract、`github_artifacts` extract、单测、去重键 `(canonical_apiurl)`。

### 1.3 Pack 注册表拆分

[`packs.rs`](../../crates/aipocket-discovery/src/packs.rs)：

- `PACKS` — 现有 AI pack（不变）。
- `PROXY_PACKS` — 仅 `proxy_*` id。
- `registry()` / `proxy_registry()` 分开；`compose_queries` 见 §2。

---

## 2. 发现（Discovery）

### 2.1 启用与装配（P0）

[`scan_assembly.rs`](../../crates/aipocket-services/src/scan_assembly.rs) 伪代码：

```text
ai_packs = registry() 全部或 github_pack_ids 筛选（不含 proxy_*）
proxy_packs = if settings.proxy_sub_enabled { PROXY_PACKS 全部或 proxy_pack_ids }
              else { [] }

q_ai = compose_queries(&ai_packs)
q_proxy = compose_proxy_queries(&proxy_packs)   // 见下

FofaSource.queries = q_ai.fofa
  + take_head(q_proxy.fofa, settings.proxy_sub_query_budget)   // 追加，不占用 AI 切片

Shodan 同理追加 q_proxy.shodan

GitHub: q_ai.github + q_proxy.github（受 github_*_budget 约束，proxy term 单独 prioritize）
```

| env | 默认 | 含义 |
|-----|------|------|
| `PROXY_SUB_ENABLED` | `false` | 挂载 proxy pack + 提取/验证 |
| `PROXY_SUB_QUERY_BUDGET` | `8` | FOFA/Shodan **追加**条数上限 |
| `PROXY_SUB_VALIDATE_ENABLED` | `true` | `PROXY_SUB_ENABLED=true` 时是否 L0 GET |

`github_pack_ids` 空 / `all`：**仅 AI pack**；proxy 靠 `PROXY_SUB_ENABLED` + 可选 `proxy_pack_ids`（P1 UI）。CLI 同理。

### 2.2 查询优先级

在 [`legacy_queries.rs`](../../crates/aipocket-discovery/src/legacy_queries.rs) `query_priority` 增加 proxy 档（建议 **250**）：查询串含 `subscribe?token=`、`/link/`、`SUBSCRIBE_URL` 等。与 AI 800/850 **分开排序**；proxy 列表独立 `prioritize_proxy_queries` 后 **追加**到 FOFA/Shodan，不从 `FOFA_QUERY_BUDGET` 前 48 条里切。

### 2.3 FOFA 字段限制（必读）

当前 `FOFA_FIELDS` **无 `body`**，只有 `header`/`banner`/…。proxy 查询命中主机后，**完整 token 须出现在 banner/header/link 字段**，否则只能记 host 线索、无法拼 URL。

P0 策略：

1. FOFA proxy 查询 targeting `banner`/`header` 中含 `subscribe?token=` 或 32hex 的形态。
2. Shodan `http.html` 作为主补充（等同 body）。
3. **GitHub artifacts 为完整 URL 的首要来源**（§3.4，P0 必做）。
4. P1 可选：`PROXY_SUB_FOFA_BODY=true` 时对 proxy 专用请求追加 `body` 字段（单独文档后果）。

### 2.4 P0 Pack 与查询形态（写入 packs.rs）

| Pack | FOFA/Shodan 意图 | 说明 |
|------|------------------|------|
| `proxy_v2board` | `banner`/`http.html` 含 `subscribe?token=` + hex | token 长度 ≥16，不强制仅 32 |
| `proxy_sspanel` | 含 `/link/` + alnum token；`SUBSCRIBE_URL=` | env 泄露走 GitHub/页面 banner |

禁止裸 `body="/api/v1/client/subscribe"`（无 token）。

**P1：** `proxy_marzban`、`proxy_3xui`、`proxy_clash_env`、`proxy_subconverter`（只提取 hit 内 URL，不跟 `url=` GET）。

**P2 / 待取证：** nezha、wings、mqpanel、`proxy_node_uri`（默认关）。

### 2.5 被动 Prober

P1。Prober **不**提取订阅链接（避免与 extract 重复）。

---

## 3. 提取（Extract）

### 3.1 pipeline

[`pipeline.rs`](../../crates/aipocket-services/src/pipeline.rs)（可拆 `subscription_extract.rs`）：

1. 在 `extract_credentials` 末尾调用 `extract_subscription_urls(hits)`。
2. 每条设 `credential_kind = "proxy_sub"`、`product = proxy_v2board` 等。
3. 调用 `normalize_subscription_url`；失败则丢弃。

### 3.2 AI 规则豁免（仅 `credential_kind == "proxy_sub"`）

| 规则 | 处理 |
|------|------|
| `blocked_key_format`（32hex 等） | 跳过 |
| `is_noise` **长度** < 15 | 跳过；**占位符**（`example`、`your-token`）仍拒绝 |
| `locations.len() <= 5` 跨 host | 按 **canonical apiurl** 去重，不按 token 跨 host 杀 |
| `finalize_results` hex 过滤 | 跳过 proxy_sub |

### 3.3 SUB_PATTERNS（代码真源，示意）

- 绝对 URL：`https?://…/api/v1/client/subscribe?token=…`
- 相对：`/api/v1/client/subscribe?token=…` + host join
- SSPanel：`/link/{token}`、`SUBSCRIBE_URL=https?://…`
- **不**在 pipeline 匹配裸 `ss://` FOFA（P2 opt-in pack）

### 3.4 GitHub artifacts（P0）

[`github_artifacts.rs`](../../crates/aipocket-discovery/src/github_artifacts.rs)：

- 增加 `SUBSCRIBE_URL=`、`CLASH_SUB_URL=`、`subscribe?token=` 等模式。
- 产出 `_credential` 或走与 pipeline 相同的 `NormalizedSubUrl`，**`credential_kind=proxy_sub`**。
- GitHub 命中**不经过** `pipeline::KEY_PATTERNS`；此处必须与 FOFA extract **共用** `normalize_subscription_url`。

---

## 4. 验证（Validate）

[`clash_subscription.rs`](../../crates/aipocket-prober/src/clash_subscription.rs)

### 4.1 请求

- `GET credential.apiurl`（不改库内 URL；可选仅用于请求的 `?flag=clash` 不写回 record）。
- `User-Agent: clash.meta`（固定字符串，写入常量）。
- 响应 body ≤ **2MB**（stream cap，含解压后）。
- 重定向：沿用 client `max_probe_redirects`；**跳转目标**若 RFC1918/link-local/元数据 IP → 中止并 `Rejected`，**仍保留** credential 记录。
- TLS：允许无效证书仅当 `PROXY_SUB_INSECURE_TLS=true`（默认 false）。

### 4.2 解析顺序

1. 尝试 **base64** 解码整体 body → 文本
2. Clash YAML `proxies:` / `proxy-providers`（providers **只记 url 存在，不二次 GET**）
3. 换行分隔的 `ss://` / `vmess://` / `trojan://` / … URI 列表
4. 失败 → `valid=false`，`validation_state` 见 §7

### 4.3 ProviderRegistry

- 新增 `ProtocolFamily::ClashSubscription`；`validator` **match 穷尽**，删除对 proxy 落入 `_ => /v1/models`。
- `resolve`：**先看 apiurl path**（`/client/subscribe`、`/link/`、`/sub/`），再看 `credential.product`。
- `PROXY_SUB_VALIDATE_ENABLED=false` → 跳过 GET，`validation_state=candidate` 或仅标记已提取。

### 4.4 enrich（`provider_evidence`）

聚合字段：`node_count`、`protocols`、`tags`、`regions`、`airport_tier`、`subscription_userinfo`、`profile_title`（可选）。

**禁止写入 record：** 完整 YAML、节点 `password`/`server`、完整 `tagged_nodes` 列表（可留最多 5 条**仅 name** 的样例且不含 secret）。

标签规则：[`proxy-sub-tags.md`](../../crates/aipocket-prober/docs/proxy-sub-tags.md)。

---

## 5. 列表 / reveal / 导出

### 5.1 `mask_record`（P0）

[`repository.rs`](../../crates/aipocket-db/src/repository.rs)：

```text
if credential_kind == proxy_sub:
  mask apikey（同 mask_apikey）
  mask apiurl 的 token= 值与 /link/{token} 段 → ****
  raw_context → "" 或 "[redacted]"
  response_snippet → "" 或 "[redacted]"
```

### 5.2 reveal（P0）

`POST /api/key/reveal`：

- 请求体优先 **`result_id`**（兼容现有 `apikey`+`apiurl` 仅 AI key）。
- 响应：`{ "subscription_url": "<完整 apiurl>" }` 或统一 `apikey` 字段承载（API 文档写明）。
- 审计：`action=reveal`，`detail={ masked, product, credential_kind }`，无明文 URL。

### 5.3 导出（P0）

| format | proxy_sub |
|--------|-----------|
| `subscription-url` | 每行一条完整 URL（txt） |
| `json` / `csv` | 列 `subscription_url` 完整明文 + 审计 |
| `sub2api` | **过滤掉** `credential_kind=proxy_sub` |

### 5.4 API 门禁（P0）

`key_balance` / `key_chat` / `key_models`：若 `credential_kind=proxy_sub`（或 `category=proxy_sub`）→ **400** `unsupported credential kind`。

### 5.5 前端（P1）

筛选 `proxy_sub`；复制订阅链（reveal）；隐藏 Chat/余额。

---

## 6. Scanner 旁路（P0）

[`scanner.rs`](../../crates/aipocket-services/src/scanner.rs) finalize 阶段：

| 步骤 | proxy_sub |
|------|-----------|
| `BalanceService::query_for_result` | **跳过** |
| `Analyzer` GPT recheck | **跳过** |
| `high_value_record` | **None**（已有计划） |

`gpt_from_hits`：不将 `proxy_sub` 候选送 GPT。

---

## 7. Finalize 与 `results.kind`

现状：`finalize_results` 丢弃 `!valid`。

**proxy_sub 例外：**

| 验证结果 | `valid` | `validation_state` | 进入 `results` |
|----------|---------|-------------------|----------------|
| 解析成功 | true | `final_verified` | `kind=valid` |
| GET 失败/空订阅 | false | `rejected` 或 `transient` | `kind=unavailable`（**仍含完整 apiurl**） |
| 蜜罐/跨 host（AI 规则） | — | — | proxy_sub **不适用** AI 蜜罐规则 |

实现：在 `finalize_results` 或 scanner 收口处，对 `credential_kind=proxy_sub` 且 `!valid` 的记录写入 `unavailable`，**不要** `continue` 丢弃。

列表 API `unavailable` 已存在；用户可在「全部密钥」看到死链订阅并 reveal。

---

## 8. 安全

- 仅处理 hit/artifact 内出现的 URL；不爆破。
- L0 GET 出站；`security.md` 说明 last-used 副作用。
- 列表不打码 `raw_context` = 泄密（§5.1 必做）。
- 授权研究 framing 同 sk- 验证。

---

## 9. Files（P0）

| 路径 | 变更 |
|------|------|
| `aipocket-core/src/models.rs` | `Credential.credential_kind` |
| `aipocket-core/src/subscription_url.rs` | 新建 normalize |
| `aipocket-core/src/config.rs` | `PROXY_SUB_*` |
| `aipocket-discovery/src/packs.rs` | `PROXY_PACKS` |
| `aipocket-discovery/src/queries.rs` | `compose_proxy_queries` |
| `aipocket-discovery/src/legacy_queries.rs` | proxy `query_priority` |
| `aipocket-discovery/src/github_artifacts.rs` | 订阅 URL 提取 |
| `aipocket-services/src/scan_assembly.rs` | 装配 + budget 追加 |
| `aipocket-services/src/pipeline.rs` | extract + 豁免 |
| `aipocket-services/src/scanner.rs` | balance/GPT 跳过；finalize 保留失败 URL |
| `aipocket-prober/src/clash_subscription.rs` | 新建 |
| `aipocket-prober/src/provider.rs`, `validator.rs` | ClashSubscription |
| `aipocket-db/src/repository.rs` | `mask_record` |
| `aipocket-api/src/routes/keys.rs` | reveal/export/400 |
| `.env.example` | env 默认 |
| Docs | `validation.md`, `security.md`, `scan-pipeline.md`, `packs.md`, `api.md` |

---

## 10. Tasks

### P0 — 可编码闭环

- [ ] ADR-005 → `accepted`
- [ ] `Credential.credential_kind` + `normalize_subscription_url`
- [ ] `PROXY_PACKS` + `proxy_v2board` / `proxy_sspanel` + `assemble_sources` 追加预算
- [ ] `pipeline` extract + 豁免 + GitHub artifacts
- [ ] `ClashSubscription` validator（base64/YAML/URI + Clash UA）
- [ ] `mask_record` 全字段；reveal by `result_id`
- [ ] finalize 保留失败 URL；scanner 跳过 balance/GPT
- [ ] export `subscription-url`；sub2api 排除；API 400
- [ ] 测试矩阵 §11；文档 §9

**完成标准：** fixture URL → DB 明文 apiurl → list 无 token 明文（含 raw_context）→ reveal 返回**逐字节相同** URL → enrich tags。

### P1

- [x] 更多 pack；扫描 UI `PROXY_SUB_ENABLED`；前端筛选
- [x] FOFA body 可选（`PROXY_SUB_FOFA_BODY`）

### P2

- [x] 待取证面板（设置页 + 扫描页实验 pack 勾选）
- [x] `clash-body` 导出（单独审计 `export_clash_body`）
- [x] 实验性 pack（`PROXY_SUB_EXTRA_PACKS`）

---

## 11. 测试矩阵

| 用例 | 断言 |
|------|------|
| 32hex token extract | 过 prefilter + finalize |
| 相对路径 + host | canonical https URL |
| GitHub `SUBSCRIBE_URL=` | artifacts 路径入库 |
| list API | apiurl/token/raw_context/snippet 无明文 |
| reveal by result_id | 与入库 apiurl 一致 |
| validate base64 URI 列表 | enrich 成功 |
| validate 403 | `kind=unavailable`，apiurl 仍在 |
| sub2api export | 无 proxy_sub |
| balance/chat | 400 |
| scanner | 不对 proxy_sub 调 balance |
| assemble `PROXY_SUB_ENABLED=false` | FOFA 查询无 proxy 项 |

---

## 12. Grok 二轮对照

| 项 | 落点 |
|----|------|
| credential_kind 在 Credential | §1.1 |
| FOFA 无 body | §2.3 |
| GitHub P0 | §3.4 |
| raw_context 泄密 | §5.1 |
| reveal 对账 | §5.2 result_id |
| finalize 丢失败链 | §7 |
| balance/GPT | §6 |
| budget 追加 / all 不含 proxy | §2.1 |
| base64 解析 | §4.2 |
| Validator 无回落 | §4.3 |
| 完整订阅链接 | §0 |

---

## 相关文档

- [ADR-005](../decisions/005-clash-subscription-in-pipeline.md)
- [proxy-sub-tags.md](../../crates/aipocket-prober/docs/proxy-sub-tags.md)
