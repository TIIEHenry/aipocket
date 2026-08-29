---
title: "Web UI"
type: guide
status: current
updated: 2026-08-20
summary: "前端页面与对应能力；截图见 screenshots.md"
---

# Web UI

登录后默认进入扫描历史。路由在 `frontend/src/App.tsx`，侧栏标签在 `frontend/src/lib/navigation.ts`。

| 路径 | 页面 | 做什么 |
|------|------|--------|
| `/login` | 登录 | `POST /api/auth/login` 换 JWT |
| `/history` | 扫描历史 | 列出 runs、状态、跳进结果 |
| `/runs/:runId` | 扫描结果 | 某次 run 的 valid / suspicious |
| `/keys` | 全部密钥 | 跨 run 列表、状态迁移、余额 |
| `/high-value` | 高价值 Key | 跨 run 去重后的高价值集合 |
| `/scan` | 执行扫描 | 启动/停止、SSE 日志 |
| `/github` | GitHub 狩猎 | 制品源相关视图 |
| `/manual` | 自定义狩猎 | 手工 origin，不走 FOFA/Shodan |
| `/cve` | CVE 库 | Tavily 同步与手工添加 |
| `/honeypot` | 蜜罐站点 | 跳过后续 probe/validate 的主机 |
| `/settings` | 设置 | 读写 `.env`；检测 FOFA/Shodan/GitHub |

## 执行扫描页

对应 `POST /api/scan/start`：

- 数据源：`all` / 单源 / `manual`
- 模式：incremental（默认）或 full
- GitHub pack：扫描页默认勾选 DeepSeek / GLM / Kimi / Cursor；空 = 全部；多选收窄 `github_pack_ids`
- 自定义狩猎可勾选 FOFA/Shodan 主机名富化（`manual_enrich`）
- 日志：`EventSource` `/api/scan/logs/stream?token=`

同一时刻只能有一个 running。lease 冲突时看 Redis `aipocket:scan:lock`。

设置页保存会写 `.env`；密钥框里的 `****` 不会覆盖真值。FOFA 增量条数 / 翻页可在设置页改 `FOFA_QUERY_BUDGET`、`FOFA_MAX_PAGES`，保存后热更新，下一轮扫描生效。改完 GitHub token 后若 UI 仍显示 disabled，重启后端。

更多画面：[截图](../screenshots.md)。HTTP 细节：[Web API](../reference/api.md)。
