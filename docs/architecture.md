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
5. Before persistence, enabled rules classify the message in priority order. Rule effects and the
   message graph are committed together, so the client never observes a half-classified message.
6. The client reads the message, label, saved-view, facet, and attachment APIs.

## Storage Model

Blob data stays out of SQLite:

- SQLite stores searchable metadata and relationships.
- SQLite also stores a versioned parsed-mail snapshot for each message. The snapshot preserves ordered headers and non-attachment MIME structure; attachment nodes contain metadata and attachment ids but not attachment bytes.
- Raw MIME files are written to the server data directory for archive/download only.
- Attachments are written to the server data directory.
- `messages.mailbox` tracks inbox/archive placement. Read, starred, snoozed, and trashed
  state are independent. Trashed mail keeps its previous placement so restoring it is lossless.
- SQLite FTS5 indexes subjects, participants, and message bodies. Structured participants are
  stored in `message_addresses`; `+` addresses are normalized as exact addresses and are not
  implicitly grouped.

Message detail and header APIs read the persisted SQLite snapshot. They do not read or re-parse the raw `.eml` file. The raw file is only read by the raw-download endpoint.

Core logical tables:

- `messages`
- `attachments`
- `message_tags`
- `message_addresses`
- `message_fts`
- `labels` and `message_labels`
- `rules`
- `saved_views`

Migrations run synchronously before the HTTP listener starts. The v3 derived-index upgrade makes a
one-time database backup and rebuilds address and search data before serving traffic. Later
classification migrations are additive.

## Classification Model

Classification separates durable user choices from derived system dimensions:

- mailbox and message state provide fixed system views
- user labels are durable and may remain empty
- sender, recipient, and domain facets are derived from `message_addresses` and only appear when
  they currently contain messages

Labels are user-managed entities and do not disappear when their message count reaches zero. Saved
views store structured message-list queries without copying messages. Ordered inbound rules match
envelope addresses, parsed addresses, subjects, or named headers, then apply labels and message
state. Conditions use exact values as stored: local-part `+` semantics are deliberately not built
into classification and can be expressed explicitly by a user rule if desired.

The older system-tag tables and endpoint remain readable for upgrade compatibility, but new ingest
does not populate them and the client does not use them for navigation.

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
