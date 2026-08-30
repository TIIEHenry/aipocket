---
title: "安全边界"
type: guide
status: current
updated: 2026-08-30
summary: "授权范围、探测风险门控、以及会消耗目标额度的 Web 操作"
---

# 安全边界

本项目只用于**已获授权**的安全研究与泄露凭证排查。对未授权系统扫描、滥用泄露密钥由使用者自行承担后果。产品定位见根 [README.md](../../README.md)。

## 探测门控（fail closed）

默认 `INTRUSIVE_CHECKS=false`、`PROBE_MAX_RISK=0`：只做 L0 未认证读取。

打开 L1+（弱口令、IDOR、SSRF、SQLi、RCE）必须同时满足：

1. `INTRUSIVE_CHECKS=true`
2. `PROBE_MAX_RISK` 提到对应等级
3. `AUTHORIZED_PROBE_SCOPE` 为逗号分隔的 **精确 origin**（无 path/query）；为空则即使 1、2 为真也保持 L0
4. 对应 class 开关（如 `PROBE_SSRF_ENABLED`）为 true

未知 `PROBE_VULN_CLASSES` 名在启动时拒绝。不要为「扫得更全」而对全网打开主动探测。实现见 `aipocket-prober` 的 `ProbeContext::allows`。

## GitHub

v1 只处理响应中明确为公开的仓库。缺少 token 或 `DATABASE_URL` 时 fail closed，而不是静默用匿名配额硬扫。

## Web 上会花目标额度的操作

| 接口 / UI | 风险 |
|-----------|------|
| `POST /api/key/chat` | 用目标 key 发推理请求，**必须显式传 `model`** |
| `POST /api/key/balance`、批量 balance | 访问提供商余额接口，可能计入配额 |
| reveal | 列表默认打码；`POST /api/key/reveal` 与 high-value reveal 返回明文 |

导出、截图、日志不要把明文密钥提交进 git。reveal / chat / export / restart 会写审计，detail 只有打码与元数据，见 [ADR-004](../../dev/decisions/004-readiness-and-audit.md)。

## 鉴权

单一全局密码换 JWT。登录有失败节流。生产关闭 `WEB_CORS_ORIGINS=*`。见 [Web 鉴权](../systems/auth.md)。
