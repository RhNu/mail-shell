# mail-shell

Monorepo for a small mail-ingest stack:

- `worker/`: Cloudflare Email Worker in TypeScript
- `server/`: Rust Axum API and static asset host
- `client/`: SolidJS + Tailwind CSS web client

## Workspace

- `pnpm workspace` manages the TypeScript projects.
- `Cargo workspace` manages the Rust server.

## Commands

```bash
pnpm install
pnpm build
cargo check --workspace
docker build -t mail-shell .
docker compose up
```

## Runtime shape

- `Worker` forwards raw MIME plus envelope metadata to `POST /api/inbound`.
- `Server` stores SQLite indexes and filesystem blobs, and serves the built client.
- Classification is modeled as system tags with data payloads such as recipient address and recipient domain.

## Docker Compose

The repository includes a root `compose.yaml` for running the server image locally:

```bash
docker compose up
```
