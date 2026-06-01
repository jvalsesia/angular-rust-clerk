# Angular & Rust Axum Boilerplate (with Clerk Auth)

This repository provides a modern, full-stack monorepo boilerplate integrating a zoneless **Angular 21** frontend with an asynchronous **Rust Axum 0.8** backend. Authentication and route guarding are powered by **Clerk**.

## Project Architecture & Tech Stack

### Frontend (`/frontend`)
*   **Core**: Angular 21 with experimental zoneless change detection (`provideExperimentalZonelessChangeDetection()`) and Signal-based state management.
*   **UI Components**: Standalone components styled with a premium dark-mode glassmorphism theme and Angular Material.
*   **Routing**: Angular Router configured with Route Guards.
*   **Testing**: Unit testing powered by **Vitest** (TestBed configuration).

### Backend (`/backend`)
*   **Core**: Axum 0.8 HTTP web framework targeted to the Rust 2024 Edition.
*   **Runtime**: Tokio multi-threaded asynchronous execution pool.
*   **Middleware**: Tower-HTTP CORS policy layer gating requests.
*   **Logging**: Tracing subscriber structured log engine.
*   **Testing**: Integrated unit testing and oneshot integration testing.

---

## Workspace Layout

```
.
├── backend/                  # Rust Axum backend workspace
│   ├── src/
│   │   ├── config.rs         # Environment configuration parser
│   │   ├── lib.rs            # Core application library and routes
│   │   └── main.rs           # Binary entry point
│   ├── tests/
│   │   └── cors_tests.rs     # CORS pre-flight integration tests
│   └── Cargo.toml            # Backend dependencies
│
├── frontend/                 # Angular 21 application workspace
│   ├── src/
│   │   ├── app/
│   │   │   ├── components/   # Standalone landing, login, register components
│   │   │   └── services/     # Clerk authentication loading service
│   │   └── styles.css        # Custom dark-mode style sheets
│   └── package.json          # Node dependencies and test scripts
│
└── README.md                 # Project entry documentation
```

---

## Getting Started

### Prerequisites
*   [Node.js](https://nodejs.org/) v20 or higher.
*   [Rust & Cargo](https://rustup.rs/) (edition 2024 supported).

### Environment Setup
Both the frontend and backend require configuration templates.

1. **Frontend Setup**:
   ```bash
   cp frontend/.env.example frontend/.env
   ```
   *Edit `frontend/.env` to configure your `CLERK_PUBLISHABLE_KEY`.*

2. **Backend Setup**:
   ```bash
   cp backend/.env.example backend/.env
   ```
   *Edit `backend/.env` to configure the `PORT`, `ALLOWED_ORIGINS`, and your Clerk secret keys.*

---

## Development Execution

### Running the Frontend
1. Navigate to the frontend directory:
   ```bash
   cd frontend
   ```
2. Install dependencies:
   ```bash
   npm install
   ```
3. Run the development server (runs on `http://localhost:4200`):
   ```bash
   npm run dev
   ```
4. Run the frontend unit tests:
   ```bash
   npm run test
   ```

### Running the Backend
1. Navigate to the backend directory:
   ```bash
   cd backend
   ```
2. Run the development server (binds to `http://localhost:3000` by default):
   ```bash
   cargo run
   ```
3. Run unit and integration tests:
   ```bash
   cargo test
   ```
4. Run code lints:
   ```bash
   cargo clippy --all-targets --all-features
   ```
