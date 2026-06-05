# AGENTS.md

- Follow Conventional Commits: `<type>(<scope>): <description>`.
- Use Context7 for library, framework, SDK, CLI, and cloud-service documentation before changing related configuration.
- Keep changes aligned with the architecture documents in `docs/`.
- Keep the TypeScript quality gates intact: run root `pnpm check` for shared frontend/worker verification, and preserve `client`'s SolidJS-specific lint gate when changing frontend tooling.
