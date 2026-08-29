<div align="center">
  <img src="docs/images/lion-avatar-round.png" alt="AIPocket lion" width="220">
  <br>
  <h3><strong>Tonight We Hunt!</strong></h3>
  <p>
    <img src="https://img.shields.io/badge/platform-Docker%20%7C%20Linux-6f42c1" alt="Platform: Docker and Linux">
    <img src="https://img.shields.io/badge/Rust-1.88-000000?logo=rust&amp;logoColor=white" alt="Rust 1.88">
    <img src="https://img.shields.io/badge/Axum-0.8-000000" alt="Axum 0.8">
    <img src="https://img.shields.io/badge/React-19-20232a?logo=react&amp;logoColor=61DAFB" alt="React 19">
    <img src="https://img.shields.io/badge/PostgreSQL-16-4169E1?logo=postgresql&amp;logoColor=white" alt="PostgreSQL 16">
    <img src="https://img.shields.io/badge/Redis-7-DC382D?logo=redis&amp;logoColor=white" alt="Redis 7">
    <img src="https://img.shields.io/badge/coverage-90.1%25-brightgreen" alt="Line coverage: 90.1%">
    <img src="https://img.shields.io/badge/license-AGPL--3.0--or-later-5c940d" alt="License: AGPL-3.0-or-later">
  </p>
  <p>
    <a href="#界面预览">界面预览</a> ·
    <a href="#快速开始">快速开始</a> ·
    <a href="docs/INDEX.md">文档</a> ·
    <a href="docs/reference/cli.md">CLI</a> ·
    <a href="docs/reference/api.md">Web API</a>
  </p>
</div>

<div align="center">
  <h1>aipocket</h1>
</div>

> 一起打野！基于 FOFA、Shodan 与 GitHub Artifact Hunter，自动发现 AI 基础设施暴露面与泄露凭证，并完成归因、验证、余额查询和持久化。

手册、架构与贡献指南在 **[docs/INDEX.md](docs/INDEX.md)**，不要把本 README 当成完整手册。

---

## 免责声明

本项目仅用于**已获授权的安全研究与泄露凭证排查**。请勿对未授权系统扫描、勿滥用泄露密钥。使用者自行承担一切后果。安全门控见 [docs/guides/security.md](docs/guides/security.md)。

---

## 界面预览

![全部密钥与余额状态](docs/images/all-keys.jpg)

[查看其余界面截图 →](docs/screenshots.md)

---

## QQ 交流群

群号：`1049528428`

<p align="center">
  <img src="docs/images/qq-group.jpg" alt="sbclaude × AI 开发交流群二维码，群号 1049528428" width="360">
</p>

---

## 快速开始

```bash
cp .env.example .env
# 设置 WEB_PASSWORD、WEB_JWT_SECRET，至少配一个 FOFA_KEYS 或 SHODAN_KEYS

docker compose up -d --build
# 前端: http://localhost:3080    API: http://localhost:8000
```

本地开发、生产更新、数据目录约束：[快速开始](docs/guides/getting-started.md) · [部署](docs/guides/deploy.md)。

禁止 `docker compose down -v` 或删除 `/data/aipocket`。

```bash
aipocket scan --source all --mode incremental
aipocket serve --host 0.0.0.0 --port 8000
aipocket watch
```

更多命令：[CLI](docs/reference/cli.md)。配置真源：[`.env.example`](.env.example) · [配置说明](docs/guides/configuration.md)。

---

## 文档

| 读者 | 入口 |
|------|------|
| 使用者 / 运维 | [docs/INDEX.md](docs/INDEX.md) |
| 贡献者 | [贡献指南](docs/guides/contributing.md) |
| Agent | [AGENTS.md](AGENTS.md) |

架构与 crate 图：[docs/architecture.md](docs/architecture.md)。

---

## License

AGPL-3.0-or-later

---

## 🌟 Special Thanks

<p align="center">
  <a href="https://linux.do">
    <img src="docs/images/linuxdo.png" alt="LINUX DO" width="420" />
  </a>
</p>
<p align="center"><b>For all things AI, head to LINUX DO! Wishing the community ever greater success~</b></p>
