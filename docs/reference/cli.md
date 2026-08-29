---
title: "CLI"
type: reference
status: current
updated: 2026-08-18
summary: "aipocket 子命令使用场景；参数以 clap --help 为准"
---

# CLI

装配在 `crates/aipocket`。参数真源：

```bash
cargo run -p aipocket -- --help
cargo run -p aipocket -- scan --help
```

| 命令 | 场景 |
|------|------|
| `aipocket serve` | 开 Web API（默认 `0.0.0.0:8000`） |
| `aipocket scan` | 跑一轮 discovery → validate → balance。`--source` 默认 `all`；`--mode incremental\|full`；`--resume-run run_…` 需 PG spill |
| `aipocket watch` | 按 `SCHEDULER_INTERVAL` 周期扫描 |
| `aipocket balance` | 对库里已有 valid key 再查余额 |
| `aipocket cve-sync` | 用 Tavily 拉 CVE 相关结果 |
| `aipocket queries` | 打印当前 pack 的 FOFA/Shodan/GitHub 查询规模 |
| `aipocket config` | 打印已解析 Settings（密钥已打码） |
| `aipocket shodan-info` | 各 Shodan key 账户信息（key 打码） |
| `aipocket maintenance` | 维护子命令（见 `--help`） |

配置始终来自 `.env` / 环境变量，不在 CLI 里再传一遍密钥。
