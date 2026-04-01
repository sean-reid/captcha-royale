# CAPTCHA Royale — Development Guide

## Quick Commands

```bash
# Install dependencies
pnpm install

# Build WASM engine
cd packages/captcha-engine && wasm-pack build --target web --out-dir pkg

# Run Rust tests
cd packages/captcha-engine && cargo test

# Dev server (frontend)
cd apps/web && pnpm dev

# Dev server (worker)
cd apps/worker && pnpm dev

# Type-check all
cd apps/web && npx tsc --noEmit
cd apps/worker && npx tsc --noEmit

# Build frontend
cd apps/web && npx vite build
```

## Architecture

- **Monorepo**: Turborepo + pnpm workspaces
- **packages/captcha-engine**: Rust WASM crate — procedural CAPTCHA generation + validation
- **apps/web**: React + Vite SPA — game UI, WASM integration
- **apps/worker**: Cloudflare Workers — auth, matchmaking, match rooms (Durable Objects), D1/KV

## Key Design Decisions

- **Seed-based generation**: Server sends seed + difficulty; both client & server WASM generate identical CAPTCHAs
- **Server-side validation**: WASM runs in Durable Objects to validate answers (client is never trusted)
- **Desktop only**: No mobile/touch support. Window must be >= 1024px wide.
- **WASM binary**: `wasm-opt` is disabled in dev; enable in CI with manual optimization pass

## WASM Integration

The WASM module is at `packages/captcha-engine/pkg/captcha_engine`. It's aliased in Vite as `captcha-engine`. The JS binding functions accept/return JSON strings for complex types.

## Database

D1 (SQLite at edge). Schema is in `apps/worker/migrations/0001_init.sql`. Apply with `wrangler d1 migrations apply`.
