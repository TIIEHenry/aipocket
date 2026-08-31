---
title: "机场订阅节点标签规则"
type: reference
status: draft
updated: 2026-08-31
summary: "L0 enrich 用；不替代 apiurl 存储；英文词边界"
---

# 机场订阅节点标签规则

**完整订阅链接**存在 `credential.apiurl`；本文件只定义验证 enrich 的标签规则。见 [方案](../../../dev/plans/clash-subscription-discovery.md) §0。

实现：`aipocket-prober::clash_subscription`。匹配 `proxies[].name` / `remark`；**不**把节点 password 写入 `record`。

## 匹配规则

- 中文：子串匹配。
- 英文区域词：**词边界**（`\bUS\b` 不匹配 `Australia`）。
- `回国`、`回国专线`：不打任何区域标签。

## 线路档位

| 标签 | 关键词 |
|------|--------|
| `residential` | 家宽、住宅、宽带、住宅IP、家宽IP |
| `iepl` | IEPL |
| `iplc` | IPLC |
| `dedicated` | 专线、内网专线 |
| `relay` | 中转、转发（不含单独「落地」「入口」） |
| `bgp` | BGP |
| `native` | 原生IP、原生（不含单独英文 `native`） |

## 用途

| 标签 | 关键词 |
|------|--------|
| `streaming` | 流媒体、Netflix、迪士尼、HBO |
| `gaming` | 游戏、低延迟 |
| `premium` | 精品、VIP、旗舰（不含单独 `Pro`） |

## 区域

| code | 关键词 |
|------|--------|
| `hk` | 香港、\bHK\b |
| `tw` | 台湾、\bTW\b |
| `jp` | 日本、东京、\bJP\b |
| `sg` | 新加坡、\bSG\b |
| `us` | 美国、硅谷、\bUS\b |
| `kr` | 韩国、\bKR\b |
| `eu` | 欧洲、英国、德国、法兰克福 |

## airport_tier

| 值 | 条件 |
|----|------|
| `premium_residential` | (`iepl` 或 `iplc`) 且 `residential` |
| `premium` | `iepl` / `iplc` / `dedicated`，无 `residential` |
| `transit` | 仅 `relay` / `bgp` |
| `standard` | 其余 |

## 面板类型

来自 `credential.product`：`proxy_v2board`、`proxy_sspanel` 等（与 pack id 一致）。

## 相关文档

- [方案](../../../dev/plans/clash-subscription-discovery.md)
- [ADR-005](../../../dev/decisions/005-clash-subscription-in-pipeline.md)
