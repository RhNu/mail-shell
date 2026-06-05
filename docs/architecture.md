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
4. The server persists the raw MIME file, parses message fields and attachments, writes SQLite indexes, and exposes the result through `/api/*`.
5. The client reads `/api/messages`, `/api/messages/:id`, `/api/tags`, and attachment download endpoints.

## Storage Model

Blob data stays out of SQLite:

- SQLite stores searchable metadata and relationships.
- Raw MIME files are written to the server data directory.
- Attachments are written to the server data directory.

Expected logical tables:

- `messages`
- `attachments`
- `message_tags`

## Classification Model

Classification is not a free-form folder tree. It is a system-tag model with attached data:

- each tag has `kind`
- each tag has `value`
- each tag has a display `label`
- each tag has `source = system`

The first two tag kinds are:

- `recipient_address`
- `recipient_domain`

This keeps filtering simple while preserving structured data for future expansion.

## Serving Model

- The server owns the `/api` namespace.
- The client uses hash routing, so deep-link fallback handling is unnecessary.
- The server serves the compiled `client/dist` directory as static assets.

