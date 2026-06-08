# Architecture Design

## Overview

`mail-shell` is a small mail-ingest system split into three deployable concerns:

- `worker/`: a Cloudflare Email Worker that receives mail events and forwards them to the server.
- `server/`: a Rust Axum service that ingests mail, owns storage, and serves the web application.
- `client/`: a SolidJS application statically hosted by the server.

The runtime intentionally stays small:

- one Cloudflare Worker deployment
- one container image for server + client
- one SQLite database plus filesystem blob storage

## Request and Data Flow

1. Cloudflare routes an incoming email to the Worker.
2. The Worker serializes the raw MIME payload and minimal envelope metadata.
3. The Worker sends `POST /api/inbound` to the server using Cloudflare Access service-token headers.
4. The server persists the raw MIME file, parses the message once, writes searchable indexes plus a versioned attachment-free parsed snapshot to SQLite, writes attachment blobs to disk, and exposes the result through `/api/*`.
5. The client reads `/api/messages`, `/api/messages/:id`, `/api/tags`, and attachment download endpoints.

## Storage Model

Blob data stays out of SQLite:

- SQLite stores searchable metadata and relationships.
- SQLite also stores a versioned parsed-mail snapshot for each message. The snapshot preserves ordered headers and non-attachment MIME structure; attachment nodes contain metadata and attachment ids but not attachment bytes.
- Raw MIME files are written to the server data directory for archive/download only.
- Attachments are written to the server data directory.

Message detail and header APIs read the persisted SQLite snapshot. They do not read or re-parse the raw `.eml` file. The raw file is only read by the raw-download endpoint.

Expected logical tables:

- `messages`
- `attachments`
- `message_tags`

The current schema is intentionally destructive from earlier development versions. Existing data directories must be cleared before deploying this schema.

## Classification Model

Classification is not a free-form folder tree. It is a system-tag model with attached data:

- each tag has `kind`
- each tag has `value`
- each tag has a display `label`
- each tag has `source = system`

The initial tag kinds are:

- `recipient_address`
- `recipient_domain`
- `sender_domain`

This keeps filtering simple while preserving structured data for future expansion.

## Serving Model

- The server owns the `/api` namespace.
- The client uses hash routing, so deep-link fallback handling is unnecessary.
- The server serves the compiled `client/dist` directory as static assets.

## Notification Model

On successful mail ingest, the server can push a notification through a pluggable notifier:

- `Notifier` trait (`services/notifier.rs`) abstracts the push backend.
- `NoopNotifier` is the default (notifications disabled).
- `BarkNotifier` (`services/bark.rs`) sends push notifications via the Bark HTTP API (iOS).
- The notifier is selected at startup via `MAIL_SHELL_NOTIFIER` (env: `disabled` / `bark`).
- Notification is fire-and-forget: ingest succeeds even if the push fails. Errors are logged at warn level.
- Additional notifier backends can be added by implementing the `Notifier` trait and wiring them in `main.rs`.
