# Code Quality Gates

## Goals

This repository uses lightweight, local-first quality gates for the TypeScript surface so `worker/` and the future `client/` code stay readable, consistent, and harder to regress.

The gates are intentionally split by concern:

- root `oxlint` for fast cross-package TypeScript and import checks
- root `oxfmt` for deterministic formatting
- `client`-only `eslint-plugin-solid` for SolidJS reactivity and JSX correctness rules

This keeps the shared gate simple while still enabling framework-specific checks where they add real value.

## Tooling Layout

### Root Oxc Gate

The repository root owns:

- [.oxlintrc.json](/d:/Source/_Rust/mail-shell/.oxlintrc.json)
- [.oxfmtrc.jsonc](/d:/Source/_Rust/mail-shell/.oxfmtrc.jsonc)
- [.oxlintignore](/d:/Source/_Rust/mail-shell/.oxlintignore)

`oxlint` runs across both `client/` and `worker/` with a stricter profile:

- `correctness`
- `suspicious`
- `pedantic`
- `typescript`
- `import`
- `unicorn`

The intent is to catch correctness and maintainability issues early without introducing a second general-purpose linter for the whole workspace.

### Client Solid Gate

The SolidJS-specific gate lives in:

- [client/eslint.config.mjs](/d:/Source/_Rust/mail-shell/client/eslint.config.mjs)

This uses `eslint-plugin-solid` with its TypeScript flat config so the client gets:

- JSX identifier safety
- Solid reactivity guidance
- React-to-Solid anti-pattern detection
- Solid-specific DOM and prop checks

`worker/` does not need this extra layer because it is not a SolidJS package.

## Commands

Run these from the repository root:

- `pnpm format`: apply Oxc formatting
- `pnpm format:check`: verify formatting without writing
- `pnpm lint`: run shared `oxlint` plus the SolidJS lint gate
- `pnpm lint:client`: run client-focused lint checks
- `pnpm lint:worker`: run worker-focused lint checks
- `pnpm typecheck`: run client TypeScript, worker validation, and Rust `cargo check`
- `pnpm check`: full TypeScript quality gate for local use and CI

## CI Expectations

GitHub Actions should treat `pnpm check` as the primary TypeScript quality gate before image build or worker deploy steps continue.

That keeps local developer workflows and CI aligned around one command path instead of maintaining separate ad hoc verification logic.
