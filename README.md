# mail-shell

Monorepo for a small mail-ingest stack with push notifications.

| Component | Directory | Stack | Purpose |
|---|---|---|---|
| Worker | `worker/` | TypeScript, Cloudflare Email Workers | Receives email events, forwards raw MIME + envelope metadata to server |
| Server | `server/` | Rust (Axum, SQLx, SQLite) | Ingests mail, owns storage, serves REST API and static client |
| Client | `client/` | SolidJS, Vite, Tailwind CSS | Web frontend, statically hosted by server |

## Architecture

```
Email → Cloudflare → Worker → POST /api/inbound → Server (SQLite + filesystem)
                                                          ↓
                                                      Notifier (Bark / disabled)
                                                          ↓
                                                     Client reads /api/*
```

- Worker forwards raw MIME and envelope metadata via multipart POST.
- Server persists raw `.eml` files, parses message fields and attachments, writes SQLite indexes.
- Classification is modeled as system tags (kind/value/label), not free-form folders.
- On successful ingest, the server can push a notification through the configured notifier.
- The client uses hash routing; the server serves `client/dist` as static assets.

Full architecture details: [`docs/architecture.md`](docs/architecture.md).

## Prerequisites

- Node 22 + pnpm 11
- Rust stable (edition 2024) — toolchain pinned in `rust-toolchain.toml`
- Docker (for container builds)

## Development

```bash
pnpm install                  # install workspace dependencies
pnpm build                    # build client
pnpm check                    # format + lint + typecheck (TS + Rust)
cargo test --workspace        # run server tests
pnpm test:client              # run client tests
pnpm gen:client-api           # regenerate OpenAPI types from server spec
```

Individual commands:

```bash
pnpm format                   # apply Oxc formatting
pnpm lint                     # oxlint + client SolidJS lint
pnpm typecheck                # client tsc + worker wrangler dry-run + cargo check
pnpm lint:client              # client-only lint
pnpm lint:worker              # worker-only lint
```

Quality gate details: [`docs/code-quality.md`](docs/code-quality.md).

## Server Configuration

Environment variables (consumed by the server binary):

| Variable | Default | Description |
|---|---|---|
| `MAIL_SHELL_HOST` | `127.0.0.1` | Bind host |
| `MAIL_SHELL_PORT` | `3000` | Bind port |
| `MAIL_SHELL_DATA_DIR` | `data` | Root directory for SQLite DB, raw MIME, attachments |
| `RUST_LOG` | `mail_shell_server=info,tower_http=info` | Tracing filter level |
| `MAIL_SHELL_NOTIFIER` | `disabled` | Notifier backend: `disabled` or `bark` |
| `MAIL_SHELL_BARK_SERVER_URL` | `https://api.day.app` | Bark server URL (supports self-hosted) |
| `MAIL_SHELL_BARK_KEY` | — | Bark device key (required when notifier=bark) |
| `MAIL_SHELL_BARK_GROUP` | — | Notification group name |
| `MAIL_SHELL_BARK_SOUND` | — | Notification sound |
| `MAIL_SHELL_BARK_LEVEL` | — | Notification level: `active`, `timeSensitive`, `passive` |

Notification is fire-and-forget: ingest succeeds even if the push notification fails.

## Client Configuration

| Variable | Default | Description |
|---|---|---|
| `VITE_API_BASE_URL` | `window.location.origin` | API base URL (Vite env) |

## Worker Secrets

The Cloudflare Worker requires three secrets, synced via `deploy-worker.yml`:

| Secret | Description |
|---|---|
| `INBOUND_URL` | Server's `POST /api/inbound` endpoint URL |
| `CF_ACCESS_CLIENT_ID` | Cloudflare Access service token client ID |
| `CF_ACCESS_CLIENT_SECRET` | Cloudflare Access service token client secret |

These must be set as GitHub repository secrets. The deploy workflow pipes them into Wrangler secret storage.

## Cloudflare Access Setup

The Worker authenticates to the server through Cloudflare Access service tokens. Access resources are provisioned outside the repo (Terraform or Cloudflare API).

Recommended Access shape on a single hostname:

1. Protect the site hostname with Access.
2. Create a more specific Access application for `/api/inbound`.
3. Allow the Worker through that path with a service-token policy.
4. Keep user-facing access policies on broader site paths.

## Docker Deployment

### Build

```bash
docker build -t mail-shell .
```

The multi-stage Dockerfile builds:
1. Client assets (Node 22 → `pnpm build`)
2. Server binary (Rust → `cargo build --release`)
3. Runtime image (Debian bookworm-slim + `mail-shell-server` + `client/dist`)

### Run

```bash
docker compose up
```

The included `compose.yaml` mounts `./data:/data` for persistent SQLite, raw MIME, and attachment storage. Customize by adding environment variables:

```yaml
services:
  mail-shell:
    image: ghcr.io/rhnu/mail-shell:latest
    environment:
      MAIL_SHELL_HOST: 0.0.0.0
      MAIL_SHELL_PORT: 3000
      MAIL_SHELL_DATA_DIR: /data
      MAIL_SHELL_NOTIFIER: bark
      MAIL_SHELL_BARK_KEY: your-device-key
    volumes:
      - ./data:/data
```

### GHCR

CI pushes the image to `ghcr.io/rhnu/mail-shell` on `main` and version tags via `build-image.yml`.

## GitHub Actions

| Workflow | Trigger | Purpose |
|---|---|---|
| `build-image.yml` | PRs, `main` push, version tags | Quality gate + build + push image to GHCR |
| `deploy-worker.yml` | `main` push (worker path filter), manual | Sync secrets + deploy Worker via Wrangler |

Required GitHub secrets:

- `CLOUDFLARE_API_TOKEN`
- `CLOUDFLARE_ACCOUNT_ID`
- `INBOUND_URL`
- `CF_ACCESS_CLIENT_ID`
- `CF_ACCESS_CLIENT_SECRET`

Full CI details: [`docs/deployment-and-ci.md`](docs/deployment-and-ci.md).

## API Endpoints

| Method | Path | Description |
|---|---|---|
| GET | `/api/healthz` | Health check |
| POST | `/api/inbound` | Ingest raw MIME + metadata (multipart) |
| GET | `/api/messages` | Paginated message list, optional tag filter |
| GET | `/api/messages/{id}` | Full message detail + attachments |
| GET | `/api/attachments/{id}` | Binary attachment download |
| GET | `/api/tags` | All tags with message counts |
| GET | `/api-docs/openapi.json` | OpenAPI spec |

## Documentation

- [`docs/architecture.md`](docs/architecture.md) — system overview, request flow, storage and classification model
- [`docs/tech-stack.md`](docs/tech-stack.md) — technology choices and rationale
- [`docs/deployment-and-ci.md`](docs/deployment-and-ci.md) — CI workflows, required secrets, Access boundary
- [`docs/code-quality.md`](docs/code-quality.md) — lint, format, and typecheck gate layout
- [`docs/client-pwa.md`](docs/client-pwa.md) — PWA update and connectivity UX design