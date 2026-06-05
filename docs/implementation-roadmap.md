# Implementation Roadmap

## Phase 1: Repository Bootstrap

- Initialize Git and workspace metadata.
- Add workspace manifests for `pnpm` and Cargo.
- Create the Worker, Server, and Client skeletons.
- Add Docker and GitHub Actions.
- Write the architecture and deployment documents.

## Phase 2: Inbound Mail Pipeline

- Implement `POST /api/inbound`.
- Persist raw MIME files under the data directory.
- Parse envelope, headers, body parts, and attachments.
- Insert `messages`, `attachments`, and system tags into SQLite.

## Phase 3: Read APIs

- Add paginated message listing.
- Add message detail and attachment download endpoints.
- Add tag listing and filtering APIs.

## Phase 4: Client Features

- Add inbox list and filters.
- Add message detail view.
- Add attachment download affordances.
- Surface recipient-address and recipient-domain tag filters.

