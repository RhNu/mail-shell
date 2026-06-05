# Tech Stack

## Chosen Technologies

- `worker`: TypeScript, Wrangler, Cloudflare Email Workers
- `server`: Rust, Axum, SQLx, SQLite, tower-http, tracing
- `client`: SolidJS, Vite, Tailwind CSS
- `ci`: GitHub Actions
- `container`: Docker multi-stage build
- `access`: Cloudflare Access

## Why These Choices

### Worker

TypeScript keeps the Cloudflare side small and direct. It avoids a Rust-to-WASM toolchain just for email forwarding.

### Server

Rust + Axum keeps the hot path fast and predictable. SQLite matches the single-node storage model and keeps deployment simple.

### Client

SolidJS keeps the frontend lightweight. Tailwind CSS gives fast styling with little infrastructure. Hash routing avoids server-side SPA fallback logic.

### Storage

SQLite stores indexes and relationships. The filesystem stores raw MIME and attachments, which avoids ballooning the database with large blobs.

### Classification

System tags are the smallest model that still carries data. This is enough to represent recipient-based filtering without committing to a larger rule engine.

