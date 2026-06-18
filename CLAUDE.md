# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

OpenAchieve (OA) is a marketing/auth web app: a React 18 + Vite frontend with a glassmorphism design, backed by a Rust (Actix-Web) API and PostgreSQL. The product it markets is a fictional coding-agent CLI with tiered subscriptions.

## Commands

Full stack (frontend + backend + Postgres) via Docker:
```bash
docker compose up --build -d      # start all services
docker compose logs -f            # follow logs
```
- Frontend: http://localhost:3000 (Docker) — Vite dev server uses 5173
- Backend API: http://localhost:8080

Frontend (local, with HMR):
```bash
npm install
npm run dev        # vite dev server (port 5173)
npm run build      # production build to dist/
npm run lint       # eslint
npm run preview    # serve the built dist/
```

Backend (local, needs a running Postgres matching DATABASE_URL):
```bash
cd backend
cargo run          # builds + runs; auto-creates schema and seeds test user
cargo build --release
```

There is no test suite in this repo.

## Architecture

### Frontend — single-file SPA with hash routing
The entire UI lives in `src/App.jsx` (~800 lines). There is **no router library**. "Pages" are components conditionally rendered in `App()` based on `window.location.hash` (`#install`, `#pricing`, `#dynamic`, `#auth`, `#dashboard`; empty hash = home). A `hashchange` listener drives `currentHash` state. When adding a page, add its component, an `is...Page` flag, and a render branch in `App()`, plus a nav entry in `Header`.

- Auth state is held in `loggedInUser` and persisted to `localStorage` as `oa_token` (JWT) and `oa_user` (JSON). Logout clears both.
- The backend base URL is **hardcoded** as `API_BASE = 'http://localhost:8080'` inside `AuthPage` (`src/App.jsx`). Change it there if the backend moves.
- `cn()` (clsx + tailwind-merge) is the className helper. Styling is Tailwind CSS v4 (via `@tailwindcss/vite`, no `tailwind.config.js`). Animations use GSAP (`gsap.context` per page, reverted on unmount) and Framer Motion (the `layoutId="nav-bubble"` nav indicator).
- The Dashboard renders **mock subscription data** (`mockPlan`, `mockCredits`, `mockExpiry`); see the `[TODO: Backend Integration]` comments for the intended `GET /api/me` / `POST /api/topup` wiring.

### Backend — Actix-Web, modular
`backend/src/` is split: `main.rs` (server bootstrap, CORS, routes, `AppState`), `auth.rs` (handlers + JWT/bcrypt), `db.rs` (schema init + queries), `email.rs` (Resend API).

- On startup `main.rs` connects to Postgres with retry (`connect_with_retry`), then runs `db::init_db` (creates the `users` table if absent) and `db::seed_test_user`. **Schema is managed in code via `CREATE TABLE IF NOT EXISTS`, not migration files** — change the table in `db::init_db`.
- Shared state (`AppState`: db client, jwt_secret, resend config) is passed via `web::Data`.
- Routes: `POST /api/register`, `POST /api/verify`, `POST /api/login`, `GET /api/health`.
- Auth flow: register hashes password (bcrypt), stores a 6-digit `verification_code`, and emails it via Resend. **If `RESEND_API_KEY` is empty the code is printed to stdout** instead of emailed — read it from `docker compose logs` during local dev. Verify checks the code and sets `verified = TRUE`. Login checks password + `verified`, then issues a JWT valid for 7 days. There is currently no endpoint that validates the JWT on subsequent requests.
- CORS `allowed_origin`s are an explicit allowlist in `main.rs` (localhost/127.0.0.1 on 5173 and 3000). New frontend origins must be added there.

### Config / env
Backend env is read in `main.rs` (with fallback defaults): `DATABASE_URL`, `JWT_SECRET`, `RESEND_API_KEY`, `RESEND_FROM`. Local values live in `backend/.env` (loaded via dotenvy); Docker values are set in `docker-compose.yml`. The seeded test user (`backend/src/db.rs`) is `1250585873@qq.com` / `12138Wsx.`.
