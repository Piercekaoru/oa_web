# OpenAchieve Web

OpenAchieve (OA) is a modern web application featuring a stunning glassmorphism design system, smooth animations, and a high-performance Rust backend.

## Tech Stack

### Frontend
- **React 18** (via Vite)
- **Tailwind CSS** for atomic, responsive styling
- **GSAP & Framer Motion** for high-performance scroll and layout animations
- **Lucide React** for crisp SVG iconography

### Backend
- **Rust (1.88+)** for maximum performance and memory safety
- **Actix-Web** for the HTTP server framework
- **Tokio-Postgres** for database communication
- **jsonwebtoken (JWT)** & **bcrypt** for secure authentication

### Infrastructure
- **Docker & Docker Compose** for reproducible builds and zero-config deployment
- **PostgreSQL** for relational data storage

## Features

- **Landing Page**: Immersive scroll animations, glassmorphism UI, and dynamic background videos.
- **Authentication**: Email/Password registration with JWT-based sessions. Includes pending support for email verification (via Resend).
- **Dashboard (UI Preview)**: A user-specific portal displaying Subscription Plans (Free/Plus/Pro/Max) and dynamic credit balances.
- **Pricing**: Multi-tier subscription model featuring `Max` tier with exclusive dynamic fusion routing.

## How to Run

The easiest way to run the entire stack (Frontend + Backend + Database) is using Docker Compose.

```bash
# Start all services (PostgreSQL, Rust Backend, React Frontend)
docker compose up --build -d

# View logs
docker compose logs -f
```

- **Frontend**: http://localhost:3000
- **Backend API**: http://localhost:8080

## Current API Endpoints

- `POST /api/register`: Register a new user account.
- `POST /api/verify`: Verify an account using the 6-digit code.
- `POST /api/login`: Authenticate and receive a JWT.

*(Note: The Dashboard data is currently mocked on the frontend. The `GET /api/me` and `/api/topup` endpoints are scheduled for the next development phase.)*

## Development Notes

### Frontend Development
If you prefer to run the frontend locally outside of Docker (for Vite HMR):
```bash
npm install
npm run dev
```

### Backend Development
If you prefer to run the backend locally outside of Docker (requires a running Postgres instance):
```bash
cd backend
cargo run
```
