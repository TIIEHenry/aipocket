# AGENTS.md

Guidelines for AI agents working on this codebase.

## New session

Read in order:

1. This file
2. [docs/DOCUMENTATION.md](docs/DOCUMENTATION.md) — 文档门禁
3. [dev/progress/status.md](dev/progress/status.md) — 当前进度

Then open the relevant page from [docs/INDEX.md](docs/INDEX.md). Architecture: [docs/architecture.md](docs/architecture.md). Config names: `.env.example`.

## Project

**AIPocket** scans for exposed AI infrastructure via FOFA + Shodan (+ GitHub artifact source), extracts and validates leaked API key/URL pairs, checks balances, and flags high-value findings. Authorized research only.

Monorepo: `crates/` (Rust 2024, Axum + Clap + SQLx + redis-rs + Reqwest) and `frontend/` (React 19 + Vite + Tailwind v4 + shadcn/ui, pnpm). Infra: PostgreSQL 16 + Redis 7.

## Backend

Dependency direction: **api → services → discovery / prober / clients / db / core**. Do not invert.

- Validate: `cargo test --workspace`
- Lint: `cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings`
- Coverage: `cargo llvm-cov --workspace --fail-under-lines 86`
- All HTTP I/O is async through a shared `reqwest::Client`.
- Config env names/defaults must stay compatible with `.env.example`; never add a required env without a compatible default.
- Schema changes must be additive and idempotent. Never drop/rebuild production tables or change Redis key formats.
- New HTTP routes belong in `aipocket-api`; business logic belongs in `aipocket-services`, not handlers.

## Frontend

- Source: `frontend/src/`
- `cd frontend && pnpm build` / `pnpm lint`
- shadcn/ui only (`components/ui/`). TanStack Query. `react-router-dom` v6.
- All backend calls go through `lib/api.ts`.

## Common tasks

| Task | Doc |
|------|-----|
| Add a platform prober | [docs/guides/add-prober.md](docs/guides/add-prober.md) |
| Add an API endpoint | [docs/guides/add-endpoint.md](docs/guides/add-endpoint.md) |
| Add a frontend page | [docs/guides/frontend.md](docs/guides/frontend.md) |
| Add an env var | `Settings` default + `.env.example` + [configuration](docs/guides/configuration.md) if the consequence is non-obvious |
| Docs structure / commit gate | [docs/DOCUMENTATION.md](docs/DOCUMENTATION.md) |

After documentation structure changes: `python3 scripts/check-docs-health.py`.

## Do Not

- Introduce synchronous HTTP calls in Rust backend code
- Store runtime state, credentials, or scan results in git-tracked files
- Commit `.env`, `docs/local/`, `.deploy-credentials.txt`, or real API keys
- Modify production PG/Redis volumes, URLs, database names, or ports for migrations
- Add destructive migrations, flush Redis, or require `.env` edits for deployment
- Skip `cargo test --workspace`, fmt, clippy, and frontend build validation
- Duplicate config tables, CLI flags, or HTTP paths in Markdown (link to the source of truth)
