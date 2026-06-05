# Deployment and CI

## Deployment Shape

- `Cloudflare Worker`: deployed from `worker/`
- `Server image`: built from the repository root and pushed to GHCR

## GitHub Actions

### `build-image.yml`

Responsibilities:

- install the workspace
- run `pnpm check`
- build the client
- build the server+client image
- push the image to GHCR on `main` and version tags

### `deploy-worker.yml`

Responsibilities:

- install the workspace
- sync Worker secrets
- deploy the Worker with Wrangler

## Required GitHub Secrets

- `CLOUDFLARE_API_TOKEN`
- `CLOUDFLARE_ACCOUNT_ID`
- `INBOUND_URL`
- `CF_ACCESS_CLIENT_ID`
- `CF_ACCESS_CLIENT_SECRET`

`INBOUND_URL` should be treated as a Worker secret rather than a Wrangler `vars` entry. Cloudflare documents `vars` as plaintext configuration, while secrets remain hidden in Wrangler output and in the Cloudflare dashboard after creation.

## Cloudflare Access Boundary

`Wrangler` deploys code only. Cloudflare Access resources should be provisioned outside the repo, preferably through Terraform or the Cloudflare API.

Recommended Access shape on a single hostname:

- protect the site hostname with Access
- create a more specific Access application path for `/api/inbound`
- allow the Worker through that path with a service token policy
- keep user-facing access policies on the broader site paths

## Container Runtime Expectations

The server container expects:

- `MAIL_SHELL_HOST`
- `MAIL_SHELL_PORT`
- `MAIL_SHELL_DATA_DIR`

The image should mount durable storage at the data directory for SQLite, raw MIME files, and attachments.
