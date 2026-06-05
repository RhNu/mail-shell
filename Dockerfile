FROM node:22-bookworm-slim AS client-builder
WORKDIR /workspace

RUN corepack enable

COPY package.json pnpm-workspace.yaml pnpm-lock.yaml ./
COPY client/package.json ./client/package.json
COPY worker/package.json ./worker/package.json
RUN pnpm install --frozen-lockfile

COPY client ./client
RUN pnpm --filter @mail-shell/client build

FROM rust:1-bookworm AS server-builder
WORKDIR /workspace

COPY Cargo.toml rust-toolchain.toml ./
COPY server/Cargo.toml ./server/Cargo.toml
COPY server/src ./server/src
COPY server/migrations ./server/migrations
RUN cargo build --release -p mail-shell-server

FROM debian:bookworm-slim
WORKDIR /app

RUN apt-get update \
  && apt-get install -y --no-install-recommends ca-certificates curl \
  && rm -rf /var/lib/apt/lists/*

COPY --from=server-builder /workspace/target/release/mail-shell-server /usr/local/bin/mail-shell-server
COPY --from=client-builder /workspace/client/dist ./client/dist

ENV MAIL_SHELL_HOST=0.0.0.0
ENV MAIL_SHELL_PORT=3000
ENV MAIL_SHELL_DATA_DIR=/data
EXPOSE 3000

VOLUME ["/data"]

CMD ["mail-shell-server"]
